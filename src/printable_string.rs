use error::Error;
use std::str;

const PRINTABLE_CHAR_MASK: [u32;8] = [
    0x00000000,
    0xa7fffb81,
    0x07fffffe,
    0x07fffffe,
    0x00000000,
    0x00000000,
    0x00000000,
    0x00000000,
];

fn is_printable_char(b: u8) -> bool {
    (PRINTABLE_CHAR_MASK[(b / 32) as usize] & (1<<(b % 32))) != 0
}

fn is_printable_string(bs: &[u8]) -> bool {
    bs.iter().map(|x| *x).all(is_printable_char)
}

pub fn to_printable_string(bs: &[u8]) -> Result<&str, Error> {
    if !is_printable_string(bs) {
        return Err(Error::InvalidPrintableString);
    }
    str::from_utf8(bs).map_err(|_| Error::InvalidUTF8)
}

#[cfg(test)]
mod test{
    use super::to_printable_string;

    fn should_be_printable(x: u8) -> bool {
           (x >= b'A' && x<= b'Z')
        || (x >= b'a' && x<= b'z')
        || (x >= b'0' && x<= b'9')
        || x == b' '
        || x == b'\''
        || x == b'('
        || x == b')'
        || x == b'+'
        || x == b','
        || x == b'-'
        || x == b'.'
        || x == b'/'
        || x == b':'
        || x == b'='
        || x == b'?'
    }

    #[test]
    fn printable_chars() {
        for i in 0..256u32 {
            let buf = [i as u8];
            if should_be_printable(i as u8) {
                let s = to_printable_string(&buf[..]).unwrap();
                let chars: Vec<char> = s.chars().collect();
                assert_eq!(chars, [i as u8 as char].to_vec());
            } else {
                assert!(to_printable_string(&buf[..]).is_err());
            }
        }
    }
}


