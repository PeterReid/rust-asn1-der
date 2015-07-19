
#[derive(Debug, Copy, Clone)]
pub enum Error {
    EOF,
    OverlongLength,
    InvalidLengthEncoding,
    UnrecognizedType,
    NotImplemented,
    IncorrectLength,
    Malformed,
    MalformedObjectIdentifier,
    ObjectIdentifierTooLarge,
    InvalidUTF8,
    InvalidPrintableString,
}
