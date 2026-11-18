//! Resident JNI bridge. JNI function indices are from the JNI specification.
use std::ffi::c_void;
mod resident;

type JNIEnv = *const *const c_void;
type JObject = *mut c_void;
type JLongArray = JObject;

pub fn qpc() -> (i64,i64) {
    let (mut counter,mut frequency) = (0,0);
    #[cfg(windows)]
    unsafe {
        if QueryPerformanceCounter(&mut counter) == 0 || QueryPerformanceFrequency(&mut frequency) == 0 { return (0,0); }
    }
    (counter,frequency)
}

// JNI specification, Native Method Interface: GetArrayLength=171,
// GetLongArrayRegion=204, NewLongArray=180, SetLongArrayRegion=212.
unsafe fn read(env: *mut JNIEnv, array: JLongArray) -> Result<Vec<u64>,String> {
    if array.is_null() { return Err("null array".into()); }
    let table = unsafe { *env };
    let len: unsafe extern "system" fn(*mut JNIEnv,JObject)->i32 = unsafe { std::mem::transmute(*table.add(171)) };
    let get: unsafe extern "system" fn(*mut JNIEnv,JLongArray,i32,i32,*mut i64) = unsafe { std::mem::transmute(*table.add(204)) };
    let n = unsafe { len(env,array) };
    if n < 0 { return Err("negative array length".into()); }
    let mut words = vec![0u64;n as usize];
    unsafe { get(env,array,0,n,words.as_mut_ptr().cast()) }; Ok(words)
}
unsafe fn array(env: *mut JNIEnv, words: &[u64]) -> JLongArray {
    let Ok(n) = i32::try_from(words.len()) else { return std::ptr::null_mut(); };
    let table = unsafe { *env };
    let new: unsafe extern "system" fn(*mut JNIEnv,i32)->JLongArray = unsafe { std::mem::transmute(*table.add(180)) };
    let set: unsafe extern "system" fn(*mut JNIEnv,JLongArray,i32,i32,*const i64) = unsafe { std::mem::transmute(*table.add(212)) };
    let out = unsafe { new(env,n) };
    if !out.is_null() { unsafe { set(env,out,0,n,words.as_ptr().cast()) }; } out
}
fn mask(words: &[u64], known: &[u64], provenance: &[u64]) -> Result<Vec<bool>,String> {
    if words.len() != known.len() || words.len() != provenance.len() { return Err("observation mask length".into()); }
    known.iter().zip(provenance).map(|(&k,&p)| {
        if k > 1 || p > 3 || (k == 0) != (p == 0) { return Err("observation mask/provenance".into()); }
        Ok(k == 1)
    }).collect()
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_roundTrip(env: *mut JNIEnv,_class: JObject,bits: JLongArray,known: JLongArray,provenance: JLongArray) -> JLongArray {
    let result = (|| {
        let bits = unsafe { read(env,bits)? }; let known = unsafe { read(env,known)? }; let provenance = unsafe { read(env,provenance)? };
        mask(&bits,&known,&provenance)?;
        let state = law::PlayerState::from_bits(&bits).map_err(|e| format!("{} at {}",e.reason,e.position))?;
        let mut out = vec![resident::READY,bits.len() as u64]; out.extend(state.bits()); out.extend(known); out.extend(provenance); Ok::<_,String>(out)
    })().unwrap_or_else(|e| resident::refusal(0,e));
    unsafe { array(env,&result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_RequestGoal(env: *mut JNIEnv,_class: JObject,bits: JLongArray,known: JLongArray,provenance: JLongArray,meta: JLongArray,goal: JLongArray,issue: i64,version: i64) -> JLongArray {
    let entered = qpc().0 as u64;
    let mut result = std::panic::catch_unwind(|| (|| {
        let bits = unsafe { read(env,bits)? }; let known = unsafe { read(env,known)? }; let provenance = unsafe { read(env,provenance)? };
        let known = mask(&bits,&known,&provenance)?;
        let state = law::PlayerState::from_observed_bits(&bits,&known).map_err(|e| format!("{} at {}",e.reason,e.position))?;
        let meta = unsafe { read(env,meta)? }; let raw_goal = unsafe { read(env,goal)? };
        if raw_goal.len() != 3 { return Err("goal length".into()); }
        let mut goal = [0;3]; for a in 0..3 { goal[a] = raw_goal[a] as i32; if goal[a] as u64 != raw_goal[a] { return Err("goal range".into()); } }
        let mut ready = resident::request(resident::Request { state,meta,goal,issue: issue as u64,version: version as u64 });
        if ready[0] == resident::READY { ready.extend([entered,qpc().0 as u64]); } Ok::<_,String>(ready)
    })()).unwrap_or_else(|_| Err("request panic".into())).unwrap_or_else(|e| resident::refusal(version as u64,e));
    result[2] = entered; result[3] = qpc().0 as u64;
    unsafe { array(env,&result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_PublishSourceDelta(env: *mut JNIEnv,_class: JObject,delta: JLongArray) -> JLongArray {
    let result = unsafe { read(env,delta) }.map(resident::publish).unwrap_or_else(|e| resident::refusal(0,e));
    unsafe { array(env,&result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_ObserveEvent(env: *mut JNIEnv,_class: JObject,event: JLongArray,bits: JLongArray,known: JLongArray,provenance: JLongArray) -> JLongArray {
    let result = (|| -> Result<Vec<u64>,String> {
        let event = unsafe { read(env,event)? };
        match event.first() {
            Some(0) => Ok(resident::poll()),
            Some(1) => {
                let bits = unsafe { read(env,bits)? }; let known = unsafe { read(env,known)? }; let provenance = unsafe { read(env,provenance)? };
                mask(&bits,&known,&provenance)?;
                law::PlayerState::from_bits(&bits).map_err(|e| e.reason)?;
                Ok(resident::observe(&event,bits,known,provenance))
            }
            Some(2) => Ok(resident::observation()),
            _ => Err("event kind".into()),
        }
    })().unwrap_or_else(|e| resident::refusal(0,e));
    unsafe { array(env,&result) }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_Cancel(env: *mut JNIEnv,_class: JObject,version: i64) -> JLongArray {
    unsafe { array(env,&resident::cancel(version as u64)) }
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn QueryPerformanceCounter(value: *mut i64) -> i32;
    fn QueryPerformanceFrequency(value: *mut i64) -> i32;
}

/// The counter and frequency are returned together so consumers retain raw QPC
/// ticks and can compare samples from different processes on the same host.
#[cfg(windows)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn Java_pathtracing_NativeChannel_clock(
    env: *mut JNIEnv, _class: JObject,
) -> JLongArray {
    let (counter,frequency) = qpc();
    if frequency == 0 { return std::ptr::null_mut(); }
    // JNIEnv is a pointer to the JNI function table. NewLongArray is slot
    // 180 and SetLongArrayRegion slot 212, including the four reserved slots.
    let table = unsafe { *env };
    let new_array: unsafe extern "system" fn(*mut JNIEnv, i32) -> JLongArray =
        unsafe { std::mem::transmute(*table.add(180)) };
    let set_region: unsafe extern "system" fn(*mut JNIEnv, JLongArray, i32, i32, *const i64) =
        unsafe { std::mem::transmute(*table.add(212)) };
    let array = unsafe { new_array(env, 2) };
    if !array.is_null() {
        unsafe { set_region(env, array, 0, 2, [counter, frequency].as_ptr()) };
    }
    array
}
