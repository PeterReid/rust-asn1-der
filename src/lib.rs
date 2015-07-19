#![allow(dead_code)]
#![allow(unused_variables)]

pub mod integer;
pub mod object_identifier;
pub mod error;
pub mod printable_string;

use integer::Integer;
use object_identifier::ObjectIdentifier;
use error::Error;
use printable_string::to_printable_string;

use std::usize;
use std::str;

fn usize_bytes() -> usize {
    // TODO: once usize::BYTES is stabilized, we can use that
    let mut surviving = usize::MAX;
    let mut count = 0;
    while surviving != 0 {
        surviving = surviving >> 8;
        count += 1;
    }
    count
}


pub enum Asn1Value<'a> {
    Null,
    Boolean(bool),
    Integer(Integer<'a>),
    ObjectIdentifier(ObjectIdentifier<'a>),
    OctetString(&'a [u8]),
    PrintableString(&'a str),
    Utf8String(&'a str),
}

pub struct Parser<'a> {
    input: &'a [u8]
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8]) -> Parser<'a> {
        Parser{
            input: input
        }
    }

    fn read_length(&mut self) -> Result<usize, Error> {
        let initial = try!(self.consume_one());
        
        if initial < 0x80 {
            return Ok(initial as usize);
        }
        
        let length_byte_count = (initial & 0x7f) as usize;
        
        if length_byte_count > usize_bytes()  {
            return Err(Error::OverlongLength);
        }
        
        let length_bytes = try!(self.consume(length_byte_count));
        let mut length_bytes_iter = length_bytes.iter();
        
        let mut length_accumulator = if let Some(length_msb) = length_bytes_iter.next() {
            if *length_msb == 0 {
                // The most significant byte being 0 means this uses needlessly many bytes.
                return Err(Error::InvalidLengthEncoding);
            }
            *length_msb as usize
        } else {
            return Err(Error::InvalidLengthEncoding);
        };
        
        for length_byte in length_bytes_iter {
            length_accumulator = (length_accumulator<<8) | (*length_byte as usize);
        }
        
        if length_accumulator < 128 {
            return Err(Error::InvalidLengthEncoding); // should have used the one-byte form
        }
        
        Ok(length_accumulator)
    }

    fn consume_one(&mut self) -> Result<u8, Error> {
        if self.input.is_empty() {
            return Err(Error::EOF);
        }
        
        let result = self.input[0];
        
        self.input = &self.input[1..];
        
        Ok(result)
    }

    fn consume(&mut self, count: usize) -> Result<&[u8], Error> {
        if self.input.len() < count {
            return Err(Error::EOF);
        }
        
        let (result, remainder) = self.input.split_at(count);
        
        self.input = remainder;
        
        Ok(result)
    }

    fn read_boolean(&mut self, length: usize) -> Result<Asn1Value, Error> {
        if length != 1 {
            return Err(Error::IncorrectLength);
        }
        
        match try!(self.consume_one()) {
            0x00 => Ok(Asn1Value::Boolean(false)),
            0xff => Ok(Asn1Value::Boolean(true)),
            _ => Err(Error::Malformed),
        }
    }

    fn read_integer(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Ok(Asn1Value::Integer( Integer::new(try!(self.consume(length)))) )
    }

    fn read_bit_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Err(Error::NotImplemented)
    }

    fn read_octet_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Ok(Asn1Value::OctetString( try!(self.consume(length)) ))
    }

    fn read_null(&mut self, length: usize) -> Result<Asn1Value, Error> {
        if length != 0 {
            return Err(Error::IncorrectLength);
        }
        
        Ok(Asn1Value::Null)
    }

    fn read_object_identifier(&mut self, length: usize) -> Result<Asn1Value, Error> {
        let oid_bytes = try!(self.consume(length));
        Ok(Asn1Value::ObjectIdentifier( try!(ObjectIdentifier::new(oid_bytes)) ))
    }

    fn read_utf8_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        let utf8_bytes = try!(self.consume(length));
        let utf8_str = try!(str::from_utf8(utf8_bytes).map_err(|_| Error::InvalidUTF8));
        Ok(Asn1Value::Utf8String( utf8_str ))
    }

    fn read_printable_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        let bs = try!(self.consume(length));
        
        Ok(Asn1Value::PrintableString( try!(to_printable_string(bs)) ))
    }

    fn read_ia5_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Err(Error::NotImplemented)
    }

    fn read_bmp_string(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Err(Error::NotImplemented)
    }

    fn read_sequence(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Err(Error::NotImplemented)
    }

    fn read_set(&mut self, length: usize) -> Result<Asn1Value, Error> {
        Err(Error::NotImplemented)
    }
    
    pub fn next(&mut self) -> Result<Asn1Value, Error> {
        let value_type = try!(self.consume_one());
        let length = try!(self.read_length());
        
        match value_type {
            0x01 => self.read_boolean(length),
            0x02 => self.read_integer(length),
            0x03 => self.read_bit_string(length),
            0x04 => self.read_octet_string(length),
            0x05 => self.read_null(length),
            0x06 => self.read_object_identifier(length),
            0x0C => self.read_utf8_string(length),
            0x13 => self.read_printable_string(length),
            0x16 => self.read_ia5_string(length),
            0x1E => self.read_bmp_string(length),
            0x30 => self.read_sequence(length),
            0x31 => self.read_set(length),
            _ => Err(Error::UnrecognizedType)
        }
    }
}

