//! A borrowed, validated NBT view. Arrays and unneeded tags are never expanded.
use crate::LoadError;

#[derive(Clone, Copy)]
pub(crate) struct Tag<'a> { kind: u8, bytes: &'a [u8] }
struct Cursor<'a> { bytes: &'a [u8], at: usize }
impl<'a> Cursor<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], LoadError> {
        let end = self.at.checked_add(n).ok_or_else(|| bad("length overflow"))?;
        let data = self.bytes.get(self.at..end).ok_or_else(|| bad("truncated tag"))?;
        self.at = end;
        Ok(data)
    }
    fn byte(&mut self) -> Result<u8, LoadError> { Ok(self.take(1)?[0]) }
    fn string_bytes(&mut self) -> Result<&'a [u8], LoadError> {
        let n = u16::from_be_bytes(self.take(2)?.try_into().unwrap()) as usize;
        self.take(n)
    }
    fn count(&mut self) -> Result<usize, LoadError> {
        let n = i32::from_be_bytes(self.take(4)?.try_into().unwrap());
        usize::try_from(n).map_err(|_| bad("negative array/list length"))
    }
    fn tag(&mut self, kind: u8, depth: usize) -> Result<Tag<'a>, LoadError> {
        if depth > 64 { return Err(bad("nesting exceeds 64")); }
        let start = self.at;
        match kind {
            1 => { self.take(1)?; }
            2 => { self.take(2)?; }
            3 | 5 => { self.take(4)?; }
            4 | 6 => { self.take(8)?; }
            7 | 11 | 12 => {
                let n = self.count()?;
                let width = match kind { 7 => 1, 11 => 4, _ => 8 };
                self.take(n.checked_mul(width).ok_or_else(|| bad("array overflow"))?)?;
            }
            8 => { self.string_bytes()?; }
            9 => {
                let child = self.byte()?;
                let n = self.count()?;
                if child > 12 || (child == 0 && n != 0) { return Err(bad("invalid list type")); }
                if n > self.bytes.len() - self.at { return Err(bad("truncated list")); }
                for _ in 0..n { self.tag(child, depth + 1)?; }
            }
            10 => loop {
                let child = self.byte()?;
                if child == 0 { break; }
                self.string_bytes()?;
                self.tag(child, depth + 1)?;
            },
            _ => return Err(bad("unknown tag type")),
        }
        Ok(Tag { kind, bytes: &self.bytes[start..self.at] })
    }
}
fn bad(message: &str) -> LoadError { LoadError::Invalid(format!("NBT: {message}")) }

pub(crate) fn root(bytes: &[u8]) -> Result<Tag<'_>, LoadError> {
    let mut c = Cursor { bytes, at: 0 };
    if c.byte()? != 10 { return Err(bad("root is not a compound")); }
    c.string_bytes()?;
    let tag = c.tag(10, 0)?;
    if c.at != bytes.len() { return Err(bad("trailing bytes")); }
    Ok(tag)
}
impl<'a> Tag<'a> {
    pub fn fields(self) -> Result<Vec<(String, Tag<'a>)>, LoadError> {
        if self.kind != 10 { return Err(bad("expected compound")); }
        let mut c = Cursor { bytes: self.bytes, at: 0 };
        let mut fields = Vec::new();
        loop {
            let kind = c.byte()?;
            if kind == 0 { break; }
            let name = mutf8(c.string_bytes()?)?;
            if fields.iter().any(|(key, _)| key == &name) { return Err(bad("duplicate compound field")); }
            fields.push((name, c.tag(kind, 0)?));
        }
        Ok(fields)
    }
    pub fn get(self, name: &str) -> Result<Option<Tag<'a>>, LoadError> {
        if self.kind != 10 { return Err(bad("expected compound")); }
        let mut c = Cursor { bytes: self.bytes, at: 0 };
        let mut found = None;
        loop {
            let kind = c.byte()?;
            if kind == 0 { return Ok(found); }
            let key = c.string_bytes()?;
            let tag = c.tag(kind, 0)?;
            if key == name.as_bytes() {
                if found.is_some() { return Err(bad("duplicate requested field")); }
                found = Some(tag);
            }
        }
    }
    pub fn required(self, name: &str) -> Result<Tag<'a>, LoadError> {
        self.get(name)?.ok_or_else(|| bad(&format!("missing {name}")))
    }
    pub fn int(self) -> Result<i32, LoadError> {
        if self.kind != 3 { return Err(bad("expected int")); }
        Ok(i32::from_be_bytes(self.bytes.try_into().unwrap()))
    }
    pub fn byte(self) -> Result<i8, LoadError> {
        if self.kind != 1 { return Err(bad("expected byte")); }
        Ok(self.bytes[0] as i8)
    }
    pub fn string(self) -> Result<String, LoadError> {
        if self.kind != 8 { return Err(bad("expected string")); }
        mutf8(&self.bytes[2..])
    }
    pub fn list(self, expected_kind: u8) -> Result<Vec<Tag<'a>>, LoadError> {
        if self.kind != 9 { return Err(bad("expected list")); }
        let mut c = Cursor { bytes: self.bytes, at: 0 };
        let kind = c.byte()?;
        let n = c.count()?;
        if kind != expected_kind && !(kind == 0 && n == 0) { return Err(bad("unexpected list element type")); }
        (0..n).map(|_| c.tag(kind, 0)).collect()
    }
    pub fn longs(self) -> Result<&'a [u8], LoadError> {
        if self.kind != 12 { return Err(bad("expected long array")); }
        Ok(&self.bytes[4..])
    }
}

// Java DataInput.readUTF: NUL uses C0 80, supplementary characters use surrogate pairs.
fn mutf8(bytes: &[u8]) -> Result<String, LoadError> {
    let mut units = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let a = bytes[at]; at += 1;
        let (mut value, rest) = match a {
            1..=127 => (a as u16, 0),
            0xc0..=0xdf => ((a & 31) as u16, 1),
            0xe0..=0xef => ((a & 15) as u16, 2),
            _ => return Err(bad("invalid modified UTF-8")),
        };
        for _ in 0..rest {
            let &b = bytes.get(at).ok_or_else(|| bad("truncated UTF-8"))?;
            if b & 0xc0 != 0x80 { return Err(bad("invalid UTF-8 continuation")); }
            value = (value << 6) | (b & 63) as u16; at += 1;
        }
        if (rest == 1 && value != 0 && value < 128) || (rest == 2 && value < 2048) { return Err(bad("overlong UTF-8")); }
        units.push(value);
    }
    String::from_utf16(&units).map_err(|_| bad("unpaired UTF-16 surrogate"))
}
