use error::Error;

pub struct ObjectIdentifier<'a> {
    content: &'a [u8],
}

impl<'a> ObjectIdentifier<'a> {
    pub fn new(content: &'a [u8]) -> Result<ObjectIdentifier<'a>, Error> {
        let mut iter = content.iter();
        
        // The initial byte encodes the first two digits as x*40 + y, where x<3 and y<40
        match iter.next() {
            Some(first) => {
                if *first >= 3 * 40 {
                    return Err(Error::MalformedObjectIdentifier)
                }
            }
            None => { return Err(Error::MalformedObjectIdentifier) },
        }
        
        // We want to make sure that no digit represented in this OID will overflow a u32.
        // Allowing byte sequences for a single digit to be only up to 4 bytes long
        // accomplishes that. It actually only allows 7+7+7+8 = 29 bits per digit, but that is
        // larger than any reasonable digit.
        let mut current_length = 0;
        for x in iter {
            if *x & 0x80 == 0 {
                current_length = 0;
            } else {
                if current_length==0 && *x == 0x80 {
                    return Err(Error::MalformedObjectIdentifier); // This byte was not needed!
                }
                
                current_length += 1;
                if current_length > 4 {
                    return Err(Error::ObjectIdentifierTooLarge);
                }
            }
        }
        
        if current_length != 0 {
            // We are in the middle of a digit!
            return Err(Error::MalformedObjectIdentifier);
        }
        
        Ok(ObjectIdentifier{ content: content })
    }

    pub fn iter(&self) -> ObjectIdentifierIterator<'a> {
        ObjectIdentifierIterator{
            content: self.content,
            state: ObjectIdentifierIteratorState::First,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ObjectIdentifierIteratorState {
    First,
    Second,
    Later
}

pub struct ObjectIdentifierIterator<'a> {
    content: &'a [u8],
    state: ObjectIdentifierIteratorState,
}

impl<'a> Iterator for ObjectIdentifierIterator<'a> {
    type Item = u32;
    
    fn next(&mut self) -> Option<u32> {
        let first = if let Some(first) = self.content.first() { 
            *first 
        } else {
            return None;
        };
        
        match self.state {
            ObjectIdentifierIteratorState::First => {
                self.state = ObjectIdentifierIteratorState::Second;
                return Some((first / 40) as u32);
            },
            ObjectIdentifierIteratorState::Second => {
                self.state = ObjectIdentifierIteratorState::Later;
                self.content = &self.content[1..];
                return Some((first % 40) as u32);
            }
            ObjectIdentifierIteratorState::Later => {
                let mut accumulator = 0;
                
                for (idx, byte) in self.content.iter().enumerate() {
                    accumulator = (accumulator<<7) | ((*byte as u32) & 0x7f);
                    if (*byte & 0x80)==0 {
                        self.content = &self.content[idx+1..];
                        return Some(accumulator)
                    }
                }
                
                // This is malformed, since it did not end with a high-bit-off byte!
                // The ObjectIdentifier initializer should have caught that.
                unreachable!();
            }
        }
    }
}


#[test]
fn oids() {
    fn good_oid(bytes: &[u8], expected_digits: &[u32]) {
        let oid = ObjectIdentifier::new(&bytes).unwrap();
        let digits: Vec<u32> = oid.iter().collect();
        assert_eq!(digits, expected_digits.to_vec());
    }
    
    fn bad_oid(bytes: &[u8]) {
        assert!(ObjectIdentifier::new(&bytes).is_err());
    }

    good_oid(&[0x2B, 0x06, 0x01, 0x04, 0x01, 0x82, 0x37, 0x15, 0x14], 
             &[1,3,6,1,4,1,311,21,20]);
    good_oid(&[0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01, 0x01], 
             &[1,2,840,113549,1,1,1 ]);
             
    bad_oid(&[]);
    
    bad_oid(&[0xff]);
    bad_oid(&[3*40]);
    good_oid(&[2*40 + 39], 
             &[2,39]);
    
    bad_oid(&[0x00, 0x81]); // Ends with a high-bit-set byte
    
    good_oid(&[0x00, 0x85, 0x01], &[0,0,(5<<7) + 1]);
    bad_oid(&[0x00, 0x80, 0x01]); // The 0x80 is unnecessary
}

