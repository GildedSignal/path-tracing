use world::inflate::zlib;
#[test]
fn stored_fixed_dynamic() {
 let stored = [0x78,0x01,0x01,0x05,0x00,0xfa,0xff,b'h',b'e',b'l',b'l',b'o',0x06,0x2c,0x02,0x15];
 let fixed = [0x78,0x9c,0xcb,0x48,0xcd,0xc9,0xc9,0x07,0x00,0x06,0x2c,0x02,0x15];
 assert_eq!(zlib(&stored).unwrap(), b"hello");
 assert_eq!(zlib(&fixed).unwrap(), b"hello");
 assert_eq!(zlib(include_bytes!("fixtures/dynamic.zlib")).unwrap(), b"Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100));
}
#[test]
fn rejects_corruption() {
 let mut bytes=include_bytes!("fixtures/dynamic.zlib").to_vec();
 *bytes.last_mut().unwrap() ^= 1;
 assert!(zlib(&bytes).is_err());
 assert!(zlib(&bytes[..10]).is_err());
}

// Hand-written dynamic tree: A=0, B=10, end=11. The code-length
// alphabet has 0=00, 1=01, 2=10, repeat-zero(18)=11.
#[test]
fn handmade_dynamic_huffman() {
    struct Bits { bytes: Vec<u8>, count: usize }
    impl Bits {
        fn put(&mut self,value:u32,n:usize) {
            for i in 0..n {
                if self.count%8==0 { self.bytes.push(0); }
                let at=self.bytes.len()-1;
                self.bytes[at] |= (((value>>i)&1) as u8) << (self.count%8);
                self.count+=1;
            }
        }
        fn code(&mut self,value:u32,n:usize) { self.put(value.reverse_bits()>>(32-n),n); }
    }
    let mut bits=Bits {bytes:Vec::new(),count:0};
    bits.put(1,1); bits.put(2,2); // final, dynamic
    bits.put(0,5); bits.put(0,5); bits.put(14,4); // 257 literals, 1 distance, 18 code lengths
    for length in [0,0,2,2,0,0,0,0,0,0,0,0,0,0,0,2,0,2] {bits.put(length,3);}
    bits.code(3,2); bits.put(65-11,7); // 65 zero lengths
    bits.code(1,2); bits.code(2,2); // A length 1, B length 2
    bits.code(3,2); bits.put(138-11,7);
    bits.code(3,2); bits.put(51-11,7); // 189 zero lengths
    bits.code(2,2); bits.code(1,2); // end length 2, distance 0 length 1
    bits.code(0,1); bits.code(2,2); bits.code(3,2); // A B end
    let mut stream=vec![0x78,0x01]; stream.extend(bits.bytes); stream.extend([0,0xc6,0,0x84]);
    assert_eq!(zlib(&stream).unwrap(),b"AB");
}
#[test]
fn gzip_metadata_checksum() {
    let bytes=include_bytes!("fixtures/world/level.dat");
    let result=world::inflate::gzip(bytes).unwrap();
    assert_eq!(&result[..3],&[10,0,0]);
    let mut corrupt=bytes.to_vec();
    let at=corrupt.len()-8; corrupt[at]^=1;
    assert!(world::inflate::gzip(&corrupt).is_err());
}
