//! Inline storage is a capacity choice, never a semantic limit: excess spills.
use std::{mem::MaybeUninit,ops::{Deref,DerefMut}};
pub struct List<T,const N:usize> {inline:[MaybeUninit<T>;N],len:usize,spill:Option<Vec<T>>}
impl<T,const N:usize> List<T,N> {
    pub fn new()->Self {Self {inline:std::array::from_fn(|_|MaybeUninit::uninit()),len:0,spill:None}}
    pub fn push(&mut self,value:T) {
        if let Some(v)=&mut self.spill {v.push(value);return;}
        if self.len<N {self.inline[self.len].write(value);self.len+=1;return;}
        let mut v=Vec::with_capacity((N*2).max(1));
        for i in 0..self.len {v.push(unsafe {self.inline[i].assume_init_read()});}
        self.len=0;v.push(value);self.spill=Some(v);
    }
}
impl<T,const N:usize> Default for List<T,N> {fn default()->Self {Self::new()}}
impl<T,const N:usize> Deref for List<T,N> {type Target=[T];fn deref(&self)->&[T] {match &self.spill {Some(v)=>v,None=>unsafe {std::slice::from_raw_parts(self.inline.as_ptr().cast(),self.len)}}}}
impl<T,const N:usize> DerefMut for List<T,N> {fn deref_mut(&mut self)->&mut[T] {match &mut self.spill {Some(v)=>v,None=>unsafe {std::slice::from_raw_parts_mut(self.inline.as_mut_ptr().cast(),self.len)}}}}
impl<T,const N:usize> Drop for List<T,N> {fn drop(&mut self) {if self.spill.is_none() {for i in 0..self.len {unsafe {self.inline[i].assume_init_drop()};}}}}
impl<T:Clone,const N:usize> Clone for List<T,N> {fn clone(&self)->Self {self.iter().cloned().collect()}}
impl<T:std::fmt::Debug,const N:usize> std::fmt::Debug for List<T,N> {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {self.deref().fmt(f)}}
impl<T:PartialEq,const N:usize> PartialEq<Vec<T>> for List<T,N> {fn eq(&self,v:&Vec<T>)->bool {self.deref()==v}}
impl<T:PartialEq,const N:usize> PartialEq for List<T,N> {fn eq(&self,v:&Self)->bool {self.deref()==v.deref()}}
impl<T,const N:usize> Extend<T> for List<T,N> {fn extend<I:IntoIterator<Item=T>>(&mut self,i:I) {for value in i {self.push(value);}}}
impl<T,const N:usize> FromIterator<T> for List<T,N> {fn from_iter<I:IntoIterator<Item=T>>(i:I)->Self {let mut out=Self::new();out.extend(i);out}}
impl<T,const N:usize> From<Vec<T>> for List<T,N> {fn from(v:Vec<T>)->Self {v.into_iter().collect()}}
impl<'a,T,const N:usize> IntoIterator for &'a List<T,N> {type Item=&'a T;type IntoIter=std::slice::Iter<'a,T>;fn into_iter(self)->Self::IntoIter {self.iter()}}
impl<'a,T,const N:usize> IntoIterator for &'a mut List<T,N> {type Item=&'a mut T;type IntoIter=std::slice::IterMut<'a,T>;fn into_iter(self)->Self::IntoIter {self.iter_mut()}}
pub struct IntoIter<T,const N:usize> {list:List<T,N>,index:usize,spill:Option<std::vec::IntoIter<T>>}
impl<T,const N:usize> IntoIterator for List<T,N> {type Item=T;type IntoIter=IntoIter<T,N>;fn into_iter(mut self)->Self::IntoIter {let spill=self.spill.take().map(Vec::into_iter);IntoIter {list:self,index:0,spill}}}
impl<T,const N:usize> Iterator for IntoIter<T,N> {type Item=T;fn next(&mut self)->Option<T> {if let Some(v)=&mut self.spill {return v.next();}if self.index==self.list.len {return None;}let i=self.index;self.index+=1;Some(unsafe {self.list.inline[i].assume_init_read()})}}
impl<T,const N:usize> Drop for IntoIter<T,N> {fn drop(&mut self) {
    // A T destructor may unwind. Disarm List before invoking it, so field
    // cleanup cannot revisit values already moved out or dropped.
    let end=std::mem::replace(&mut self.list.len,0);
    if self.spill.is_none() {for i in self.index..end {unsafe {self.list.inline[i].assume_init_drop()};}}
}}
#[macro_export] macro_rules! list {($($v:expr),* $(,)?)=>{{let mut out=$crate::inline::List::new();$(out.push($v);)*out}};}
#[cfg(test)]mod tests {use super::*;#[test]fn ownership_on_inline_spill_and_partial_iteration(){let n=std::rc::Rc::new(std::cell::Cell::new(0));struct Item(std::rc::Rc<std::cell::Cell<usize>>);impl Drop for Item {fn drop(&mut self){self.0.set(self.0.get()+1)}}for size in [0,1,4,9] {let before=n.get();let a:List<_,4>=(0..size).map(|_|Item(n.clone())).collect();let mut iter=a.into_iter();drop(iter.next());drop(iter);assert_eq!(n.get()-before,size);}}}
impl<T:PartialEq,const N:usize> List<T,N> {pub fn dedup(&mut self) {let old=std::mem::take(self);for value in old {if self.last()!=Some(&value) {self.push(value);}}}}

#[cfg(test)]mod unwind_tests {
    use super::*;
    #[test]fn partially_consumed_iterator_never_drops_a_value_twice_on_unwind() {
        let counts=std::rc::Rc::new(std::cell::Cell::new([0;3]));
        struct Item {id:usize,counts:std::rc::Rc<std::cell::Cell<[u8;3]>>}
        impl Drop for Item {fn drop(&mut self) {let mut n=self.counts.get();n[self.id]+=1;self.counts.set(n);if self.id==2 && n[2]==1 {panic!("last destructor unwinds");}}}
        let list:List<_,4>=(0..3).map(|id|Item {id,counts:counts.clone()}).collect();
        let mut iter=list.into_iter();drop(iter.next());
        assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||drop(iter))).is_err());
        assert_eq!(counts.get(),[1;3]);
        assert_eq!(std::rc::Rc::strong_count(&counts),1,"no retained or duplicated owner");
    }
}
