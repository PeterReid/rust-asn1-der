
pub struct Integer<'a>(&'a [u8]);

impl<'a> Integer<'a> {
    pub fn new(bytes: &'a [u8]) -> Integer<'a> {
        Integer(bytes)
    }

    pub fn as_u8(&self) -> Option<u8> {
        if self.0.len() > 1 {
            return None;
        }
        
        self.0.get(0).map(|x| *x)
    }
    
    pub fn as_u32(&self) -> Option<u32> {
        if self.0.len() > 4 {
            return None;
        }
        
        Some( self.0.iter().fold(0u32, |accum, b| (accum<<8) | (*b as u32)) )
    }
    
    pub fn as_u64(&self) -> Option<u64> {
        if self.0.len() > 8 {
            return None;
        }
        
        Some( self.0.iter().fold(0u64, |accum, b| (accum<<8) | (*b as u64)) )
    }
    
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0
    }
}

#[test]
fn integer() {
    let xs = [0x02, 0x01, 0x03];
    let mut p = Parser::new(&xs[..]);
    
    match p.next() {
        Ok(Asn1Value::Integer(x)) => {
            assert_eq!(x.as_u8(), Some(3));
        },
        _ => {
            assert!(false);
        }
    }
}
