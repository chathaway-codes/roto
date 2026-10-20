use core::fmt;

use crate::wire::WireType;

/// Errors that can occur while decoding Protocol Buffers wire data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    UnexpectedEof,
    InvalidVarint,
    InvalidWireType(u8),
    InvalidUtf8,
    MissingRequiredField(&'static str),
    UnexpectedWireType { expected: WireType, actual: WireType },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEof => f.write_str("unexpected end of buffer"),
            Self::InvalidVarint => f.write_str("invalid or overflowing varint"),
            Self::InvalidWireType(wt) => write!(f, "invalid wire type: {wt}"),
            Self::InvalidUtf8 => f.write_str("invalid UTF-8 string"),
            Self::MissingRequiredField(field) => write!(f, "missing required field: {field}"),
            Self::UnexpectedWireType { expected, actual } => {
                write!(f, "unexpected wire type: expected {expected:?}, got {actual:?}")
            }
        }
    }
}

impl core::error::Error for DecodeError {}

/// Errors that can occur while encoding Protocol Buffers wire data.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeError {
    BufferOverflow,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferOverflow => f.write_str("encode buffer overflow"),
        }
    }
}

impl core::error::Error for EncodeError {}
