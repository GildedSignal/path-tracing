use std::collections::BTreeMap;

pub struct Object<'a>(BTreeMap<&'a str,&'a str>);
impl<'a> Object<'a> {
    pub fn parse(text: &'a str) -> Self {
        let text=text.trim(); assert!(text.starts_with('{') && text.ends_with('}'),"invalid JSON object: {text}");
        let bytes=text.as_bytes(); let mut p=1; let mut fields=BTreeMap::new();
        while p<bytes.len()-1 {
            while bytes[p].is_ascii_whitespace(){p+=1;}
            assert_eq!(bytes[p],b'"',"JSON field name");
            let start=p+1;p+=1;while bytes[p]!=b'"'{assert_ne!(bytes[p],b'\\',"escaped field name");p+=1;}
            let name=&text[start..p];p+=1;while bytes[p].is_ascii_whitespace(){p+=1;}
            assert_eq!(bytes[p],b':');p+=1;while bytes[p].is_ascii_whitespace(){p+=1;}
            let start=p;let mut depth=0;let mut quoted=false;let mut escaped=false;
            while p<bytes.len()-1 {
                let c=bytes[p];
                if quoted {if escaped{escaped=false;}else if c==b'\\'{escaped=true;}else if c==b'"'{quoted=false;}}
                else {match c{b'"'=>quoted=true,b'{'|b'['=>depth+=1,b'}'|b']'=>depth-=1,b',' if depth==0=>break,_=>{}}}
                p+=1;
            }
            assert!(!quoted && depth==0,"unclosed JSON value");
            let value=text[start..p].trim();assert!(!value.is_empty(),"empty JSON value");
            assert!(fields.insert(name,value).is_none(),"duplicate JSON key {name}");
            if p<bytes.len()-1 {p+=1;assert!(p<bytes.len()-1,"trailing comma");}
        }
        Self(fields)
    }
    pub fn get(&self,key:&str)->Option<&'a str>{self.0.get(key).copied()}
    pub fn required(&self,key:&str)->&'a str{self.get(key).unwrap_or_else(||panic!("missing {key}"))}
    pub fn string(&self,key:&str)->&'a str{
        let v=self.required(key);assert!(v.starts_with('"')&&v.ends_with('"')); &v[1..v.len()-1]
    }
    pub fn boolean(&self,key:&str)->bool{match self.required(key){"true"=>true,"false"=>false,v=>panic!("invalid bool {v}")}}
    pub fn number(&self,key:&str)->f64{self.required(key).parse().unwrap_or_else(|_|panic!("invalid number {key}"))}
    pub fn f64(&self,key:&str)->f64{
        self.get(&format!("{key}Bits")).map_or_else(||self.number(key),|v|f64::from_bits(hex(v,16)))
    }
    pub fn f32(&self,key:&str)->f32{
        self.get(&format!("{key}Bits")).map_or_else(||self.number(key) as f32,|v|f32::from_bits(hex(v,8) as u32))
    }
    pub fn object(&self,key:&str)->Self{Self::parse(self.required(key))}
}
pub fn hex(v:&str,len:usize)->u64{
    assert_eq!(v.len(),len+2);assert!(v.starts_with('"')&&v.ends_with('"'));
    u64::from_str_radix(&v[1..v.len()-1],16).expect("raw IEEE hex")
}
pub fn numbers(v:&str)->Vec<f64>{v.split([',','[',']']).filter_map(|s|{let s=s.trim();if s.is_empty(){None}else{Some(s.parse().expect("numeric array"))}}).collect()}

// Recorder speedModifiers tuples
pub fn array(text: &str) -> Vec<&str> {
    let text = text.trim();
    assert!(text.starts_with('[') && text.ends_with(']'), "invalid JSON array");
    let mut result = Vec::new();
    let mut start = 1;
    let mut depth = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (index, c) in text.char_indices().skip(1).take_while(|(i, _)| *i < text.len()-1) {
        if quoted {
            if escaped { escaped = false; }
            else if c == '\\' { escaped = true; }
            else if c == '"' { quoted = false; }
        } else {
            match c {
                '"' => quoted = true,
                '[' | '{' => depth += 1,
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    let value = text[start..index].trim();
                    assert!(!value.is_empty(), "empty JSON array element");
                    result.push(value);
                    start = index + 1;
                }
                _ => {}
            }
        }
    }
    assert!(!quoted && depth == 0, "unclosed JSON array value");
    let value = text[start..text.len()-1].trim();
    if !value.is_empty() { result.push(value); }
    else { assert!(result.is_empty(), "trailing JSON array comma"); }
    result
}
