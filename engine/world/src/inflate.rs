//! Bounded zlib/DEFLATE decoding for Anvil chunk payloads.
const LIMIT: usize = 16 * 1024 * 1024;
struct Bits<'a> { bytes: &'a [u8], bit: usize }
impl Bits<'_> {
    fn read(&mut self, n: usize) -> Result<u32, String> {
        if self.bit.checked_add(n).filter(|&end| end <= self.bytes.len() * 8).is_none() { return Err("truncated deflate stream".into()); }
        let mut v = 0;
        for i in 0..n { v |= (((self.bytes[self.bit / 8] >> (self.bit % 8)) & 1) as u32) << i; self.bit += 1; }
        Ok(v)
    }
}
struct Huffman { codes: Vec<(u32, u8, u16)> }
impl Huffman {
    fn new(lengths: &[u8]) -> Result<Self, String> {
        let mut count = [0u32; 16];
        for &n in lengths { if n > 15 { return Err("invalid Huffman length".into()); } if n != 0 { count[n as usize] += 1; } }
        let mut next = [0u32; 16]; let mut code = 0;
        for n in 1..=15 { code = (code + count[n-1]) << 1; if code + count[n] > 1 << n { return Err("oversubscribed Huffman tree".into()); } next[n] = code; }
        let mut codes = Vec::new();
        for (symbol, &n) in lengths.iter().enumerate() { if n != 0 { let c = next[n as usize]; next[n as usize] += 1; codes.push((c, n, symbol as u16)); } }
        Ok(Self { codes })
    }
    fn symbol(&self, bits: &mut Bits<'_>) -> Result<usize, String> {
        let mut code = 0;
        for len in 1..=15 { code = (code << 1) | bits.read(1)?; if let Some(&(_,_,s)) = self.codes.iter().find(|&&(c,n,_)| n == len && c == code) { return Ok(s as usize); } }
        Err("invalid Huffman code".into())
    }
}
fn trees(bits: &mut Bits<'_>, kind: u32) -> Result<(Huffman,Huffman), String> {
    if kind == 1 {
        let mut l = vec![8; 288]; l[144..256].fill(9); l[256..280].fill(7);
        return Ok((Huffman::new(&l)?, Huffman::new(&[5; 32])?));
    }
    let nl = bits.read(5)? as usize + 257; let nd = bits.read(5)? as usize + 1; let nc = bits.read(4)? as usize + 4;
    let order = [16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];
    let mut cl = [0u8; 19]; for &i in &order[..nc] { cl[i] = bits.read(3)? as u8; }
    let tree = Huffman::new(&cl)?; let mut lengths = Vec::new();
    while lengths.len() < nl + nd {
        let s = tree.symbol(bits)?;
        match s {
            0..=15 => lengths.push(s as u8),
            16..=18 => { let (n, value) = match s { 16 => (bits.read(2)? as usize + 3, *lengths.last().ok_or("repeat with no previous length")?), 17 => (bits.read(3)? as usize + 3, 0), _ => (bits.read(7)? as usize + 11, 0) }; if lengths.len() + n > nl + nd { return Err("Huffman repeat exceeds tree".into()); } lengths.extend(std::iter::repeat_n(value,n)); },
            _ => return Err("invalid code length".into()),
        }
    }
    if lengths[256] == 0 { return Err("missing end-of-block code".into()); }
    Ok((Huffman::new(&lengths[..nl])?, Huffman::new(&lengths[nl..])?))
}
/// Decode a zlib stream, rejecting malformed streams and output over 16 MiB.
pub fn zlib(input: &[u8]) -> Result<Vec<u8>, String> {
    if input.len() < 6 { return Err("truncated zlib stream".into()); }
    let cmf = input[0]; let flg = input[1];
    if cmf & 15 != 8 || cmf >> 4 > 7 || ((cmf as u16)*256 + flg as u16) % 31 != 0 || flg & 32 != 0 { return Err("unsupported zlib header".into()); }
    let out = deflate(&input[2..input.len()-4])?;
    let mut a = 1u32; let mut c = 0u32;
    for &v in &out { a = (a + v as u32) % 65521; c = (c + a) % 65521; }
    if (c << 16 | a) != u32::from_be_bytes(input[input.len()-4..].try_into().unwrap()) { return Err("zlib checksum mismatch".into()); }
    Ok(out)
}

/// Decode one gzip member (level.dat and saved world metadata), including CRC and size.
pub fn gzip(input: &[u8]) -> Result<Vec<u8>, String> {
    if input.len() < 18 || input[..3] != [0x1f,0x8b,8] || input[3] & 0xe0 != 0 { return Err("invalid gzip header".into()); }
    let end = input.len()-8;
    let mut at = 10usize;
    if input[3] & 4 != 0 {
        let raw = input.get(at..at+2).ok_or("truncated gzip extra field")?;
        at += 2 + u16::from_le_bytes(raw.try_into().unwrap()) as usize;
    }
    for flag in [8,16] {
        if input[3] & flag != 0 {
            loop {
                if at >= end { return Err("truncated gzip string".into()); }
                let value = input[at]; at += 1;
                if value == 0 { break; }
            }
        }
    }
    if input[3] & 2 != 0 {
        let raw = input.get(at..at+2).ok_or("truncated gzip header checksum")?;
        if crc32(input.get(..at).ok_or("truncated gzip header")?) as u16 != u16::from_le_bytes(raw.try_into().unwrap()) { return Err("gzip header checksum mismatch".into()); }
        at += 2;
    }
    let out = deflate(input.get(at..end).ok_or("truncated gzip payload")?)?;
    let crc = u32::from_le_bytes(input[end..end+4].try_into().unwrap());
    let size = u32::from_le_bytes(input[end+4..].try_into().unwrap());
    if crc32(&out) != crc || out.len() as u32 != size { return Err("gzip checksum or size mismatch".into()); }
    Ok(out)
}
fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = !0u32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 { crc = (crc >> 1) ^ (0xedb88320u32 & 0u32.wrapping_sub(crc & 1)); }
    }
    !crc
}
fn deflate(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut b = Bits { bytes: input, bit: 0 }; let mut out = Vec::new();
    const LB: [usize;29] = [3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258];
    const LE: [usize;29] = [0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0];
    const DB: [usize;30] = [1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577];
    const DE: [usize;30] = [0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13];
    loop {
        let last = b.read(1)? != 0; let kind = b.read(2)?;
        if kind == 0 {
            b.bit = (b.bit + 7) & !7; let len = b.read(16)? as usize; let inverse = b.read(16)? as u16;
            if !(len as u16) != inverse { return Err("invalid stored block length".into()); }
            if out.len() + len > LIMIT { return Err("inflate output limit exceeded".into()); }
            for _ in 0..len { out.push(b.read(8)? as u8); }
        } else if kind == 1 || kind == 2 {
            let (lit, dist) = trees(&mut b,kind)?;
            loop {
                let s = lit.symbol(&mut b)?;
                if s < 256 { if out.len() == LIMIT { return Err("inflate output limit exceeded".into()); } out.push(s as u8); }
                else if s == 256 { break; }
                else if s <= 285 {
                    let i = s-257; let len = LB[i] + b.read(LE[i])? as usize; let d = dist.symbol(&mut b)?;
                    if d >= 30 { return Err("invalid distance code".into()); }
                    let offset = DB[d] + b.read(DE[d])? as usize;
                    if offset > out.len() || out.len() + len > LIMIT { return Err("invalid distance or output limit exceeded".into()); }
                    for _ in 0..len { out.push(out[out.len()-offset]); }
                } else { return Err("invalid literal code".into()); }
            }
        } else { return Err("reserved deflate block".into()); }
        if last { break; }
    }
    if (b.bit + 7)/8 != b.bytes.len() { return Err("trailing deflate data".into()); }
    Ok(out)
}

