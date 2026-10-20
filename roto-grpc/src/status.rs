use core::fmt;

use roto_runtime::{BackwardWriter, DecodeError, EncodeError};

/// Canonical gRPC status codes as defined by the gRPC over HTTP/2 specification.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum GrpcCode {
    /// Not an error; returned on success.
    Ok = 0,
    /// The operation was cancelled (typically by the caller).
    Cancelled = 1,
    /// Unknown error.
    Unknown = 2,
    /// Client specified an invalid argument.
    InvalidArgument = 3,
    /// Deadline expired before operation could complete.
    DeadlineExceeded = 4,
    /// Some requested entity (e.g., file or directory) was not found.
    NotFound = 5,
    /// Some entity that we attempted to create already exists.
    AlreadyExists = 6,
    /// The caller does not have permission to execute the specified operation.
    PermissionDenied = 7,
    /// Some resource has been exhausted (e.g., per-user quota, or out of space).
    ResourceExhausted = 8,
    /// Operation was rejected because the system is not in a state required for the operation's execution.
    FailedPrecondition = 9,
    /// The operation was aborted, typically due to a concurrency issue.
    Aborted = 10,
    /// Operation was attempted past the valid range.
    OutOfRange = 11,
    /// Operation is not implemented or not supported/enabled in this service.
    Unimplemented = 12,
    /// Internal errors.
    Internal = 13,
    /// The service is currently unavailable.
    Unavailable = 14,
    /// Unrecoverable data loss or corruption.
    DataLoss = 15,
    /// The request does not have valid authentication credentials for the operation.
    Unauthenticated = 16,
}

impl GrpcCode {
    /// Converts a raw `u8` value (`0..=16`) into a [`GrpcCode`].
    #[inline]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Ok),
            1 => Some(Self::Cancelled),
            2 => Some(Self::Unknown),
            3 => Some(Self::InvalidArgument),
            4 => Some(Self::DeadlineExceeded),
            5 => Some(Self::NotFound),
            6 => Some(Self::AlreadyExists),
            7 => Some(Self::PermissionDenied),
            8 => Some(Self::ResourceExhausted),
            9 => Some(Self::FailedPrecondition),
            10 => Some(Self::Aborted),
            11 => Some(Self::OutOfRange),
            12 => Some(Self::Unimplemented),
            13 => Some(Self::Internal),
            14 => Some(Self::Unavailable),
            15 => Some(Self::DataLoss),
            16 => Some(Self::Unauthenticated),
            _ => None,
        }
    }

    /// Returns the numeric `u8` representation (`0..=16`) of this status code.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// Returns the canonical ASCII decimal string (`"0"`..`"16"`) for the `grpc-status` HTTP/2 trailer.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "0",
            Self::Cancelled => "1",
            Self::Unknown => "2",
            Self::InvalidArgument => "3",
            Self::DeadlineExceeded => "4",
            Self::NotFound => "5",
            Self::AlreadyExists => "6",
            Self::PermissionDenied => "7",
            Self::ResourceExhausted => "8",
            Self::FailedPrecondition => "9",
            Self::Aborted => "10",
            Self::OutOfRange => "11",
            Self::Unimplemented => "12",
            Self::Internal => "13",
            Self::Unavailable => "14",
            Self::DataLoss => "15",
            Self::Unauthenticated => "16",
        }
    }

    /// Returns the canonical UPPER_SNAKE_CASE name (`"OK"`..`"UNAUTHENTICATED"`) of this status code.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Cancelled => "CANCELLED",
            Self::Unknown => "UNKNOWN",
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::DeadlineExceeded => "DEADLINE_EXCEEDED",
            Self::NotFound => "NOT_FOUND",
            Self::AlreadyExists => "ALREADY_EXISTS",
            Self::PermissionDenied => "PERMISSION_DENIED",
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::FailedPrecondition => "FAILED_PRECONDITION",
            Self::Aborted => "ABORTED",
            Self::OutOfRange => "OUT_OF_RANGE",
            Self::Unimplemented => "UNIMPLEMENTED",
            Self::Internal => "INTERNAL",
            Self::Unavailable => "UNAVAILABLE",
            Self::DataLoss => "DATA_LOSS",
            Self::Unauthenticated => "UNAUTHENTICATED",
        }
    }

    /// Parses a `grpc-status` trailer value from raw ASCII decimal bytes (`b"0"`..`b"16"`).
    ///
    /// Strictly rejects empty slices, leading zeros (such as `b"00"` or `b"01"`), signs,
    /// whitespace, and values greater than `16`.
    #[inline]
    pub const fn from_ascii_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [b'0'] => Some(Self::Ok),
            [b'1'] => Some(Self::Cancelled),
            [b'2'] => Some(Self::Unknown),
            [b'3'] => Some(Self::InvalidArgument),
            [b'4'] => Some(Self::DeadlineExceeded),
            [b'5'] => Some(Self::NotFound),
            [b'6'] => Some(Self::AlreadyExists),
            [b'7'] => Some(Self::PermissionDenied),
            [b'8'] => Some(Self::ResourceExhausted),
            [b'9'] => Some(Self::FailedPrecondition),
            [b'1', b'0'] => Some(Self::Aborted),
            [b'1', b'1'] => Some(Self::OutOfRange),
            [b'1', b'2'] => Some(Self::Unimplemented),
            [b'1', b'3'] => Some(Self::Internal),
            [b'1', b'4'] => Some(Self::Unavailable),
            [b'1', b'5'] => Some(Self::DataLoss),
            [b'1', b'6'] => Some(Self::Unauthenticated),
            _ => None,
        }
    }

    /// Parses a `grpc-status` trailer value from a string (`"0"`..`"16"`).
    #[inline]
    pub const fn from_header_str(s: &str) -> Option<Self> {
        Self::from_ascii_bytes(s.as_bytes())
    }
}

impl TryFrom<u8> for GrpcCode {
    type Error = u8;

    #[inline]
    fn try_from(val: u8) -> Result<Self, Self::Error> {
        Self::from_u8(val).ok_or(val)
    }
}

impl fmt::Display for GrpcCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// Zero-allocation gRPC status carrying a [`GrpcCode`] and a borrowed status message.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GrpcStatus<'a> {
    pub code: GrpcCode,
    pub message: &'a str,
}

impl<'a> GrpcStatus<'a> {
    /// Creates a new [`GrpcStatus`] with the given `code` and `message`.
    #[inline]
    pub const fn new(code: GrpcCode, message: &'a str) -> Self {
        Self { code, message }
    }

    /// Creates a `GrpcCode::Ok` status with an empty message.
    #[inline]
    pub const fn ok() -> Self {
        Self::new(GrpcCode::Ok, "")
    }

    /// Creates a `GrpcCode::Cancelled` status.
    #[inline]
    pub const fn cancelled(msg: &'a str) -> Self {
        Self::new(GrpcCode::Cancelled, msg)
    }

    /// Creates a `GrpcCode::Unknown` status.
    #[inline]
    pub const fn unknown(msg: &'a str) -> Self {
        Self::new(GrpcCode::Unknown, msg)
    }

    /// Creates a `GrpcCode::InvalidArgument` status.
    #[inline]
    pub const fn invalid_argument(msg: &'a str) -> Self {
        Self::new(GrpcCode::InvalidArgument, msg)
    }

    /// Creates a `GrpcCode::DeadlineExceeded` status.
    #[inline]
    pub const fn deadline_exceeded(msg: &'a str) -> Self {
        Self::new(GrpcCode::DeadlineExceeded, msg)
    }

    /// Creates a `GrpcCode::NotFound` status.
    #[inline]
    pub const fn not_found(msg: &'a str) -> Self {
        Self::new(GrpcCode::NotFound, msg)
    }

    /// Creates a `GrpcCode::AlreadyExists` status.
    #[inline]
    pub const fn already_exists(msg: &'a str) -> Self {
        Self::new(GrpcCode::AlreadyExists, msg)
    }

    /// Creates a `GrpcCode::PermissionDenied` status.
    #[inline]
    pub const fn permission_denied(msg: &'a str) -> Self {
        Self::new(GrpcCode::PermissionDenied, msg)
    }

    /// Creates a `GrpcCode::ResourceExhausted` status.
    #[inline]
    pub const fn resource_exhausted(msg: &'a str) -> Self {
        Self::new(GrpcCode::ResourceExhausted, msg)
    }

    /// Creates a `GrpcCode::FailedPrecondition` status.
    #[inline]
    pub const fn failed_precondition(msg: &'a str) -> Self {
        Self::new(GrpcCode::FailedPrecondition, msg)
    }

    /// Creates a `GrpcCode::Aborted` status.
    #[inline]
    pub const fn aborted(msg: &'a str) -> Self {
        Self::new(GrpcCode::Aborted, msg)
    }

    /// Creates a `GrpcCode::OutOfRange` status.
    #[inline]
    pub const fn out_of_range(msg: &'a str) -> Self {
        Self::new(GrpcCode::OutOfRange, msg)
    }

    /// Creates a `GrpcCode::Unimplemented` status.
    #[inline]
    pub const fn unimplemented(msg: &'a str) -> Self {
        Self::new(GrpcCode::Unimplemented, msg)
    }

    /// Creates a `GrpcCode::Internal` status.
    #[inline]
    pub const fn internal(msg: &'a str) -> Self {
        Self::new(GrpcCode::Internal, msg)
    }

    /// Creates a `GrpcCode::Unavailable` status.
    #[inline]
    pub const fn unavailable(msg: &'a str) -> Self {
        Self::new(GrpcCode::Unavailable, msg)
    }

    /// Creates a `GrpcCode::DataLoss` status.
    #[inline]
    pub const fn data_loss(msg: &'a str) -> Self {
        Self::new(GrpcCode::DataLoss, msg)
    }

    /// Creates a `GrpcCode::Unauthenticated` status.
    #[inline]
    pub const fn unauthenticated(msg: &'a str) -> Self {
        Self::new(GrpcCode::Unauthenticated, msg)
    }

    /// Returns `true` if `self.code == GrpcCode::Ok`.
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(self.code, GrpcCode::Ok)
    }
}

impl fmt::Display for GrpcStatus<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(
                f,
                "grpc-status: {} ({})",
                self.code.name(),
                self.code.as_u8()
            )
        } else {
            write!(
                f,
                "grpc-status: {} ({}): {}",
                self.code.name(),
                self.code.as_u8(),
                self.message
            )
        }
    }
}

impl core::error::Error for GrpcStatus<'_> {}

impl From<DecodeError> for GrpcStatus<'static> {
    #[inline]
    fn from(_err: DecodeError) -> Self {
        Self::internal("protobuf decode error")
    }
}

impl From<EncodeError> for GrpcStatus<'static> {
    #[inline]
    fn from(_err: EncodeError) -> Self {
        Self::resource_exhausted("response buffer overflow")
    }
}

/// Returns `true` if `b` is an unescaped `grpc-message` byte per the gRPC over HTTP/2 specification
/// (`0x20..=0x7E` excluding `'%'` `0x25`).
#[inline]
pub const fn is_unreserved_grpc_message_byte(b: u8) -> bool {
    matches!(b, 0x20..=0x24 | 0x26..=0x7E)
}

#[inline]
const fn hex_upper(nibble: u8) -> u8 {
    match nibble & 0x0F {
        0..=9 => b'0' + (nibble & 0x0F),
        n => b'A' + (n - 10),
    }
}

#[inline]
const fn decode_hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Computes the exact byte length of `input` when percent-encoded for `grpc-message`.
#[inline]
pub const fn percent_encoded_len(input: &str) -> usize {
    let bytes = input.as_bytes();
    let mut i = 0;
    let mut total = 0;
    while i < bytes.len() {
        if is_unreserved_grpc_message_byte(bytes[i]) {
            total += 1;
        } else {
            total += 3;
        }
        i += 1;
    }
    total
}

/// Percent-encodes `input` into `out` according to the gRPC `grpc-message` specification,
/// returning the written UTF-8 `&str` prefix or [`EncodeError::BufferOverflow`] if `out` is too small.
///
/// Printable ASCII bytes (`0x20..=0x7E` except `'%'` `0x25`) are written verbatim; all other
/// bytes (`< 0x20`, `> 0x7E`, and `'%'` `0x25`) are escaped as `%XX` using uppercase hexadecimal.
pub fn percent_encode<'a>(input: &str, out: &'a mut [u8]) -> Result<&'a str, EncodeError> {
    let mut write_pos = 0;
    for &b in input.as_bytes() {
        if is_unreserved_grpc_message_byte(b) {
            if write_pos >= out.len() {
                return Err(EncodeError::BufferOverflow);
            }
            out[write_pos] = b;
            write_pos += 1;
        } else {
            if write_pos + 3 > out.len() {
                return Err(EncodeError::BufferOverflow);
            }
            out[write_pos] = b'%';
            out[write_pos + 1] = hex_upper(b >> 4);
            out[write_pos + 2] = hex_upper(b & 0x0F);
            write_pos += 3;
        }
    }
    // All emitted bytes are valid ASCII (0x20..=0x7E), so UTF-8 validation always succeeds.
    core::str::from_utf8(&out[..write_pos]).map_err(|_| EncodeError::BufferOverflow)
}

/// Writes `input` as a percent-encoded `grpc-message` directly into `writer` in a single
/// backward pass.
///
/// If the writer lacks capacity, [`EncodeError::BufferOverflow`] is latched onto `writer`.
pub fn percent_encode_backward(input: &str, writer: &mut BackwardWriter<'_>) {
    for &b in input.as_bytes().iter().rev() {
        if is_unreserved_grpc_message_byte(b) {
            if writer.write_bytes(&[b]).is_err() {
                return;
            }
        } else {
            let esc = [b'%', hex_upper(b >> 4), hex_upper(b & 0x0F)];
            if writer.write_bytes(&esc).is_err() {
                return;
            }
        }
    }
}

/// Fast-path helper that borrows `input` directly as a `&str` if it contains no `%` escapes
/// and is valid UTF-8.
#[inline]
pub const fn try_percent_decode_in_place(input: &[u8]) -> Option<&str> {
    let mut i = 0;
    while i < input.len() {
        if input[i] == b'%' {
            return None;
        }
        i += 1;
    }
    match core::str::from_utf8(input) {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

/// Decodes a percent-encoded `grpc-message` header byte slice `input` into `out`,
/// returning the decoded UTF-8 `&str` prefix.
///
/// Accepts both uppercase and lowercase hexadecimal digits in `%XX` escapes.
/// Returns [`DecodeError::InvalidUtf8`] on truncated or non-hex `%` escapes or if the
/// decoded byte sequence is not valid UTF-8, and [`DecodeError::UnexpectedEof`] if `out`
/// does not have enough capacity.
pub fn percent_decode<'a>(input: &[u8], out: &'a mut [u8]) -> Result<&'a str, DecodeError> {
    let mut read_pos = 0;
    let mut write_pos = 0;
    while read_pos < input.len() {
        let b = input[read_pos];
        let decoded = if b == b'%' {
            if read_pos + 2 >= input.len() {
                return Err(DecodeError::InvalidUtf8);
            }
            let hi = decode_hex_nibble(input[read_pos + 1]).ok_or(DecodeError::InvalidUtf8)?;
            let lo = decode_hex_nibble(input[read_pos + 2]).ok_or(DecodeError::InvalidUtf8)?;
            read_pos += 3;
            (hi << 4) | lo
        } else {
            read_pos += 1;
            b
        };
        if write_pos >= out.len() {
            return Err(DecodeError::UnexpectedEof);
        }
        out[write_pos] = decoded;
        write_pos += 1;
    }
    core::str::from_utf8(&out[..write_pos]).map_err(|_| DecodeError::InvalidUtf8)
}
