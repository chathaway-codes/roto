use core::fmt;

use roto_runtime::{BackwardWriter, EncodeError};

use crate::framing::{write_grpc_frame, DEFAULT_MAX_MESSAGE_SIZE, GRPC_HEADER_LEN};
use crate::hpack::HpackEncoder;
use crate::status::{GrpcCode, GrpcStatus};

// ============================================================================
// RFC 9113 Constants & Frame Definitions
// ============================================================================

/// The 24-octet HTTP/2 client connection preface (`"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n"`, RFC 9113 §3.4).
pub const CLIENT_CONNECTION_PREFACE: &[u8; 24] = b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n";

/// Size in octets of the fixed HTTP/2 frame header (RFC 9113 §4.1).
pub const FRAME_HEADER_LEN: usize = 9;

/// Default initial flow-control window size in octets (`65,535`, RFC 9113 §6.5.2).
pub const DEFAULT_INITIAL_WINDOW_SIZE: u32 = 65_535;

/// Default and minimum allowed value for `SETTINGS_MAX_FRAME_SIZE` (`16,384`, RFC 9113 §6.5.2).
pub const DEFAULT_MAX_FRAME_SIZE: u32 = 16_384;

/// Maximum allowed value for `SETTINGS_MAX_FRAME_SIZE` (`16,777,215` / `2^24 - 1`, RFC 9113 §6.5.2).
pub const MAX_MAX_FRAME_SIZE: u32 = 16_777_215;

/// Maximum legal flow-control window size (`2^31 - 1` / `0x7FFF_FFFF`, RFC 9113 §6.9.1).
pub const MAX_FLOW_CONTROL_WINDOW: u32 = 0x7FFF_FFFF;

/// Maximum 31-bit stream identifier (`2^31 - 1` / `0x7FFF_FFFF`, RFC 9113 §5.1.1).
pub const MAX_STREAM_ID: u32 = 0x7FFF_FFFF;

/// Default maximum concurrent streams in a fixed-capacity `#![no_std]` [`H2Connection`].
pub const DEFAULT_MAX_STREAMS: usize = 8;

/// Default per-stream and header reassembly capacity in bytes (`512`).
pub const DEFAULT_REASM_CAP: usize = 512;

// RFC 9113 §6.5.2 Setting Identifiers
pub const SETTINGS_HEADER_TABLE_SIZE: u16 = 0x1;
pub const SETTINGS_ENABLE_PUSH: u16 = 0x2;
pub const SETTINGS_MAX_CONCURRENT_STREAMS: u16 = 0x3;
pub const SETTINGS_INITIAL_WINDOW_SIZE: u16 = 0x4;
pub const SETTINGS_MAX_FRAME_SIZE: u16 = 0x5;
pub const SETTINGS_MAX_HEADER_LIST_SIZE: u16 = 0x6;

// RFC 9113 §6 Frame Flags
/// `END_STREAM` flag (`0x1`) for `DATA` and `HEADERS` frames.
pub const FLAG_END_STREAM: u8 = 0x1;
/// `ACK` flag (`0x1`) for `SETTINGS` and `PING` frames.
pub const FLAG_ACK: u8 = 0x1;
/// `END_HEADERS` flag (`0x4`) for `HEADERS`, `PUSH_PROMISE`, and `CONTINUATION` frames.
pub const FLAG_END_HEADERS: u8 = 0x4;
/// `PADDED` flag (`0x8`) for `DATA`, `HEADERS`, and `PUSH_PROMISE` frames.
pub const FLAG_PADDED: u8 = 0x8;
/// `PRIORITY` flag (`0x20`) for `HEADERS` frames.
pub const FLAG_PRIORITY: u8 = 0x20;

/// HTTP/2 frame type identifiers (RFC 9113 §6).
#[must_use]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameType {
    Data = 0x0,
    Headers = 0x1,
    Priority = 0x2,
    RstStream = 0x3,
    Settings = 0x4,
    PushPromise = 0x5,
    Ping = 0x6,
    GoAway = 0x7,
    WindowUpdate = 0x8,
    Continuation = 0x9,
}

impl FrameType {
    pub const DATA: u8 = 0x0;
    pub const HEADERS: u8 = 0x1;
    pub const PRIORITY: u8 = 0x2;
    pub const RST_STREAM: u8 = 0x3;
    pub const SETTINGS: u8 = 0x4;
    pub const PUSH_PROMISE: u8 = 0x5;
    pub const PING: u8 = 0x6;
    pub const GOAWAY: u8 = 0x7;
    pub const WINDOW_UPDATE: u8 = 0x8;
    pub const CONTINUATION: u8 = 0x9;

    /// Converts a raw `u8` frame type byte into a known [`FrameType`], or `None` for unknown extension types.
    #[inline]
    pub const fn from_u8(val: u8) -> Option<Self> {
        match val {
            0x0 => Some(Self::Data),
            0x1 => Some(Self::Headers),
            0x2 => Some(Self::Priority),
            0x3 => Some(Self::RstStream),
            0x4 => Some(Self::Settings),
            0x5 => Some(Self::PushPromise),
            0x6 => Some(Self::Ping),
            0x7 => Some(Self::GoAway),
            0x8 => Some(Self::WindowUpdate),
            0x9 => Some(Self::Continuation),
            _ => None,
        }
    }

    /// Returns the raw `u8` wire representation of this frame type.
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// HTTP/2 32-bit error codes (RFC 9113 §7).
#[must_use]
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum H2ErrorCode {
    NoError = 0x0,
    ProtocolError = 0x1,
    InternalError = 0x2,
    FlowControlError = 0x3,
    SettingsTimeout = 0x4,
    StreamClosed = 0x5,
    FrameSizeError = 0x6,
    RefusedStream = 0x7,
    Cancel = 0x8,
    CompressionError = 0x9,
    ConnectError = 0xa,
    EnhanceYourCalm = 0xb,
    InadequateSecurity = 0xc,
    Http11Required = 0xd,
}

impl H2ErrorCode {
    /// Returns the 32-bit wire value of this error code.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u32
    }

    /// Converts a raw `u32` error code into a known [`H2ErrorCode`], or `None` if unknown.
    #[inline]
    pub const fn try_from_u32(code: u32) -> Option<Self> {
        match code {
            0x0 => Some(Self::NoError),
            0x1 => Some(Self::ProtocolError),
            0x2 => Some(Self::InternalError),
            0x3 => Some(Self::FlowControlError),
            0x4 => Some(Self::SettingsTimeout),
            0x5 => Some(Self::StreamClosed),
            0x6 => Some(Self::FrameSizeError),
            0x7 => Some(Self::RefusedStream),
            0x8 => Some(Self::Cancel),
            0x9 => Some(Self::CompressionError),
            0xa => Some(Self::ConnectError),
            0xb => Some(Self::EnhanceYourCalm),
            0xc => Some(Self::InadequateSecurity),
            0xd => Some(Self::Http11Required),
            _ => None,
        }
    }

    /// Converts a raw `u32` error code into an [`H2ErrorCode`], mapping unknown codes to
    /// [`H2ErrorCode::InternalError`] per RFC 9113 §7.
    #[inline]
    pub const fn from_u32(code: u32) -> Self {
        match Self::try_from_u32(code) {
            Some(known) => known,
            None => Self::InternalError,
        }
    }

    /// Maps an HTTP/2 error code to the corresponding [`GrpcCode`] per the gRPC over HTTP/2 specification.
    #[inline]
    pub const fn to_grpc_code(self) -> GrpcCode {
        match self {
            Self::NoError
            | Self::ProtocolError
            | Self::InternalError
            | Self::FlowControlError
            | Self::SettingsTimeout
            | Self::FrameSizeError
            | Self::CompressionError => GrpcCode::Internal,
            Self::RefusedStream => GrpcCode::Unavailable,
            Self::Cancel => GrpcCode::Cancelled,
            Self::EnhanceYourCalm => GrpcCode::ResourceExhausted,
            Self::InadequateSecurity => GrpcCode::PermissionDenied,
            Self::StreamClosed | Self::ConnectError | Self::Http11Required => GrpcCode::Unknown,
        }
    }
}

/// An HTTP/2 connection or stream error (RFC 9113 §5.4).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct H2Error {
    /// Stream identifier (`0` for a connection-level error, `> 0` for a stream-level error).
    pub stream_id: u32,
    /// The RFC 9113 §7 error code.
    pub code: H2ErrorCode,
    /// Human-readable static description of the error.
    pub reason: &'static str,
}

impl H2Error {
    /// Creates a connection-level HTTP/2 error (`stream_id == 0`).
    #[inline]
    pub const fn connection(code: H2ErrorCode, reason: &'static str) -> Self {
        Self {
            stream_id: 0,
            code,
            reason,
        }
    }

    /// Creates a stream-level HTTP/2 error for `stream_id`.
    #[inline]
    pub const fn stream(stream_id: u32, code: H2ErrorCode, reason: &'static str) -> Self {
        Self {
            stream_id,
            code,
            reason,
        }
    }

    /// Maps a [`GrpcStatus`] decoding failure on `stream_id` to an [`H2Error`].
    #[inline]
    pub const fn from_grpc_status(stream_id: u32, status: GrpcStatus<'static>) -> Self {
        let code = match status.code {
            GrpcCode::ResourceExhausted => H2ErrorCode::EnhanceYourCalm,
            _ => H2ErrorCode::InternalError,
        };
        Self::stream(stream_id, code, status.message)
    }

    /// Returns `true` if this error is a connection error (`stream_id == 0`).
    #[inline]
    pub const fn is_connection_error(&self) -> bool {
        self.stream_id == 0
    }

    /// Returns `true` if this error is a stream error (`stream_id != 0`).
    #[inline]
    pub const fn is_stream_error(&self) -> bool {
        self.stream_id != 0
    }

    /// Converts this HTTP/2 error into a [`GrpcStatus`].
    #[inline]
    pub const fn to_grpc_status(&self) -> GrpcStatus<'static> {
        GrpcStatus::new(self.code.to_grpc_code(), self.reason)
    }
}

impl fmt::Display for H2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.stream_id == 0 {
            write!(
                f,
                "HTTP/2 connection error ({:?}): {}",
                self.code, self.reason
            )
        } else {
            write!(
                f,
                "HTTP/2 stream {} error ({:?}): {}",
                self.stream_id, self.code, self.reason
            )
        }
    }
}

impl core::error::Error for H2Error {}

impl From<EncodeError> for H2Error {
    #[inline]
    fn from(_: EncodeError) -> Self {
        Self::connection(H2ErrorCode::InternalError, "output buffer overflow")
    }
}

impl From<H2Error> for GrpcStatus<'static> {
    #[inline]
    fn from(err: H2Error) -> Self {
        err.to_grpc_status()
    }
}

// ============================================================================
// 9-Byte HTTP/2 Frame Header Codec (RFC 9113 §4.1)
// ============================================================================

/// Parsed 9-octet HTTP/2 frame header (RFC 9113 §4.1).
///
/// ```text
/// +-----------------------------------------------+
/// |                 Length (24)                   |
/// +---------------+---------------+---------------+
/// |   Type (8)    |   Flags (8)   |
/// +-+-------------+---------------+-------------------------------+
/// |R|                 Stream Identifier (31)                      |
/// +=+=============================================================+
/// ```
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameHeader {
    /// 24-bit payload length in octets (`0..=16_777_215`).
    pub length: u32,
    /// 8-bit frame type code.
    pub frame_type: u8,
    /// 8-bit frame flags.
    pub flags: u8,
    /// 31-bit stream identifier (with the reserved high bit masked to `0`).
    pub stream_id: u32,
}

impl FrameHeader {
    /// Creates a new [`FrameHeader`], masking `length` to 24 bits and `stream_id` to 31 bits.
    #[inline]
    pub const fn new(length: u32, frame_type: u8, flags: u8, stream_id: u32) -> Self {
        Self {
            length: length & MAX_MAX_FRAME_SIZE,
            frame_type,
            flags,
            stream_id: stream_id & MAX_STREAM_ID,
        }
    }

    /// Decodes a 9-byte array into a [`FrameHeader`], masking out the reserved high bit `R` of `stream_id`.
    #[inline]
    pub const fn decode(bytes: [u8; FRAME_HEADER_LEN]) -> Self {
        let length = ((bytes[0] as u32) << 16) | ((bytes[1] as u32) << 8) | (bytes[2] as u32);
        let frame_type = bytes[3];
        let flags = bytes[4];
        let stream_id =
            u32::from_be_bytes([bytes[5], bytes[6], bytes[7], bytes[8]]) & MAX_STREAM_ID;
        Self {
            length,
            frame_type,
            flags,
            stream_id,
        }
    }

    /// Decodes a [`FrameHeader`] from the first 9 bytes of `slice`, or returns `None` if `slice.len() < 9`.
    #[inline]
    pub const fn decode_slice(slice: &[u8]) -> Option<Self> {
        if slice.len() < FRAME_HEADER_LEN {
            return None;
        }
        Some(Self::decode([
            slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7],
            slice[8],
        ]))
    }

    /// Encodes this [`FrameHeader`] into its 9-octet wire representation with the reserved bit set to `0`.
    #[inline]
    pub const fn encode(self) -> [u8; FRAME_HEADER_LEN] {
        let len = self.length & MAX_MAX_FRAME_SIZE;
        let sid = (self.stream_id & MAX_STREAM_ID).to_be_bytes();
        [
            ((len >> 16) & 0xFF) as u8,
            ((len >> 8) & 0xFF) as u8,
            (len & 0xFF) as u8,
            self.frame_type,
            self.flags,
            sid[0],
            sid[1],
            sid[2],
            sid[3],
        ]
    }

    /// Prepends this 9-octet [`FrameHeader`] onto `writer` in a single backward step.
    #[inline]
    pub fn write_backward(&self, writer: &mut BackwardWriter<'_>) {
        let _ = writer.write_bytes(&self.encode());
    }
}

/// Encodes a Protobuf message via `f`, prepends its 5-byte gRPC Length-Prefixed-Message (LPM) header,
/// and prepends the 9-byte HTTP/2 `DATA` [`FrameHeader`] onto `writer` in a single backward pass.
pub fn encode_grpc_data_frame<F>(
    writer: &mut BackwardWriter<'_>,
    stream_id: u32,
    end_stream: bool,
    compressed: bool,
    f: F,
) -> Result<(), EncodeError>
where
    F: FnOnce(&mut BackwardWriter<'_>),
{
    let end_pos = writer.cursor();
    write_grpc_frame(writer, compressed, f)?;
    let data_len = end_pos - writer.cursor();
    if data_len > MAX_MAX_FRAME_SIZE as usize {
        latch_overflow(writer);
        return Err(EncodeError::BufferOverflow);
    }
    let flags = if end_stream { FLAG_END_STREAM } else { 0 };
    let hdr = FrameHeader::new(data_len as u32, FrameType::DATA, flags, stream_id);
    hdr.write_backward(writer);
    writer.check()
}

/// Alias for [`encode_grpc_data_frame`].
#[inline]
pub fn write_grpc_data_frame<F>(
    writer: &mut BackwardWriter<'_>,
    stream_id: u32,
    end_stream: bool,
    compressed: bool,
    f: F,
) -> Result<(), EncodeError>
where
    F: FnOnce(&mut BackwardWriter<'_>),
{
    encode_grpc_data_frame(writer, stream_id, end_stream, compressed, f)
}

/// Writes a pre-encoded gRPC message `payload`, prepends its 5-byte gRPC LPM header, and prepends
/// the 9-byte HTTP/2 `DATA` [`FrameHeader`] onto `writer`.
#[inline]
pub fn write_grpc_data_frame_bytes(
    writer: &mut BackwardWriter<'_>,
    stream_id: u32,
    end_stream: bool,
    compressed: bool,
    payload: &[u8],
) -> Result<(), EncodeError> {
    encode_grpc_data_frame(writer, stream_id, end_stream, compressed, |w| {
        let _ = w.write_bytes(payload);
    })
}

fn latch_overflow(writer: &mut BackwardWriter<'_>) {
    let rem = writer.remaining_capacity();
    for _ in 0..=rem {
        let _ = writer.write_bytes(&[0]);
    }
}

// ============================================================================
// HTTP/2 Connection Settings (RFC 9113 §6.5)
// ============================================================================

/// HTTP/2 `SETTINGS` parameters for a connection endpoint (RFC 9113 §6.5.2).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct H2Settings {
    /// `SETTINGS_HEADER_TABLE_SIZE` (`0x1`). Defaults to `0` locally for stateless zero-alloc HPACK.
    pub header_table_size: u32,
    /// `SETTINGS_ENABLE_PUSH` (`0x2`). Defaults to `false`.
    pub enable_push: bool,
    /// `SETTINGS_MAX_CONCURRENT_STREAMS` (`0x3`).
    pub max_concurrent_streams: u32,
    /// `SETTINGS_INITIAL_WINDOW_SIZE` (`0x4`). Defaults to `65,535`.
    pub initial_window_size: u32,
    /// `SETTINGS_MAX_FRAME_SIZE` (`0x5`). Defaults to `16,384` (`16_384..=16_777_215`).
    pub max_frame_size: u32,
    /// `SETTINGS_MAX_HEADER_LIST_SIZE` (`0x6`).
    pub max_header_list_size: u32,
}

impl Default for H2Settings {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl H2Settings {
    /// Creates default [`H2Settings`] with `header_table_size = 0`, `enable_push = false`,
    /// `initial_window_size = 65_535`, and `max_frame_size = 16_384`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            header_table_size: 0,
            enable_push: false,
            max_concurrent_streams: DEFAULT_MAX_STREAMS as u32,
            initial_window_size: DEFAULT_INITIAL_WINDOW_SIZE,
            max_frame_size: DEFAULT_MAX_FRAME_SIZE,
            max_header_list_size: u32::MAX,
        }
    }

    /// Creates initial RFC 9113 peer settings prior to receiving the peer's `SETTINGS` frame.
    #[inline]
    pub const fn default_peer() -> Self {
        Self {
            header_table_size: 4096,
            enable_push: false,
            max_concurrent_streams: u32::MAX,
            initial_window_size: DEFAULT_INITIAL_WINDOW_SIZE,
            max_frame_size: DEFAULT_MAX_FRAME_SIZE,
            max_header_list_size: u32::MAX,
        }
    }

    /// Parses and applies a raw `SETTINGS` frame payload (6 bytes per parameter),
    /// returning the signed delta in `initial_window_size` (`new_initial - old_initial`).
    pub fn apply_payload(&mut self, payload: &[u8], peer_is_server: bool) -> Result<i64, H2Error> {
        if !payload.len().is_multiple_of(6) {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "SETTINGS frame payload length must be a multiple of 6",
            ));
        }
        let old_initial = self.initial_window_size as i64;
        let mut idx = 0;
        while idx < payload.len() {
            let id = u16::from_be_bytes([payload[idx], payload[idx + 1]]);
            let val = u32::from_be_bytes([
                payload[idx + 2],
                payload[idx + 3],
                payload[idx + 4],
                payload[idx + 5],
            ]);
            idx += 6;

            match id {
                SETTINGS_HEADER_TABLE_SIZE => {
                    self.header_table_size = val;
                }
                SETTINGS_ENABLE_PUSH => {
                    if val > 1 {
                        return Err(H2Error::connection(
                            H2ErrorCode::ProtocolError,
                            "SETTINGS_ENABLE_PUSH must be 0 or 1",
                        ));
                    }
                    if peer_is_server && val == 1 {
                        return Err(H2Error::connection(
                            H2ErrorCode::ProtocolError,
                            "server MUST NOT send SETTINGS_ENABLE_PUSH = 1",
                        ));
                    }
                    self.enable_push = val == 1;
                }
                SETTINGS_MAX_CONCURRENT_STREAMS => {
                    self.max_concurrent_streams = val;
                }
                SETTINGS_INITIAL_WINDOW_SIZE => {
                    if val > MAX_FLOW_CONTROL_WINDOW {
                        return Err(H2Error::connection(
                            H2ErrorCode::FlowControlError,
                            "SETTINGS_INITIAL_WINDOW_SIZE exceeds 2^31 - 1",
                        ));
                    }
                    self.initial_window_size = val;
                }
                SETTINGS_MAX_FRAME_SIZE => {
                    if !(DEFAULT_MAX_FRAME_SIZE..=MAX_MAX_FRAME_SIZE).contains(&val) {
                        return Err(H2Error::connection(
                            H2ErrorCode::ProtocolError,
                            "SETTINGS_MAX_FRAME_SIZE outside 16384..=16777215",
                        ));
                    }
                    self.max_frame_size = val;
                }
                SETTINGS_MAX_HEADER_LIST_SIZE => {
                    self.max_header_list_size = val;
                }
                _ => {
                    // RFC 9113 §6.5.2: Unknown settings MUST be ignored.
                }
            }
        }
        Ok((self.initial_window_size as i64) - old_initial)
    }

    /// Encodes a `SETTINGS` frame (`HEADER_TABLE_SIZE`, `ENABLE_PUSH`, `MAX_CONCURRENT_STREAMS`,
    /// `INITIAL_WINDOW_SIZE`, `MAX_FRAME_SIZE`) into `out` (`9 + 30 = 39` bytes).
    pub fn encode_frame(&self, out: &mut [u8]) -> Result<usize, H2Error> {
        const PAYLOAD_LEN: usize = 5 * 6;
        const TOTAL_LEN: usize = FRAME_HEADER_LEN + PAYLOAD_LEN;
        if out.len() < TOTAL_LEN {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "output buffer too small for SETTINGS frame",
            ));
        }
        let hdr = FrameHeader::new(PAYLOAD_LEN as u32, FrameType::SETTINGS, 0, 0);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        let params: [(u16, u32); 5] = [
            (SETTINGS_HEADER_TABLE_SIZE, self.header_table_size),
            (SETTINGS_ENABLE_PUSH, u32::from(self.enable_push)),
            (SETTINGS_MAX_CONCURRENT_STREAMS, self.max_concurrent_streams),
            (SETTINGS_INITIAL_WINDOW_SIZE, self.initial_window_size),
            (SETTINGS_MAX_FRAME_SIZE, self.max_frame_size),
        ];
        let mut pos = FRAME_HEADER_LEN;
        for (id, val) in params {
            out[pos..pos + 2].copy_from_slice(&id.to_be_bytes());
            out[pos + 2..pos + 6].copy_from_slice(&val.to_be_bytes());
            pos += 6;
        }
        Ok(TOTAL_LEN)
    }
}

// ============================================================================
// Endpoint Role, Stream Lifecycle State, and Events (RFC 9113 §5.1)
// ============================================================================

/// HTTP/2 connection endpoint role.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Client,
    Server,
}

/// Per-stream lifecycle state (RFC 9113 §5.1).
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamState {
    Idle,
    Open,
    HalfClosedRemote,
    HalfClosedLocal,
    Closed,
}

impl StreamState {
    /// Returns `true` if this stream is currently active (`Open`, `HalfClosedRemote`, or `HalfClosedLocal`).
    #[inline]
    pub const fn is_active(self) -> bool {
        matches!(
            self,
            Self::Open | Self::HalfClosedRemote | Self::HalfClosedLocal
        )
    }
}

/// High-level Sans-I/O event emitted by [`H2Connection::recv`].
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum H2Event<'a> {
    /// Peer `SETTINGS` frame received and applied (`SETTINGS` ACK is queued automatically).
    Settings(H2Settings),
    /// Peer `SETTINGS` ACK frame received.
    SettingsAck,
    /// Complete HPACK header block received from `HEADERS` (and any following `CONTINUATION` frames).
    Headers {
        stream_id: u32,
        header_block: &'a [u8],
        end_stream: bool,
    },
    /// Complete gRPC Length-Prefixed-Message (LPM) decoded from one or more `DATA` frames.
    ///
    /// On the single-frame fast path, `payload` borrows directly from the caller's input buffer
    /// with zero copying.
    GrpcMessage {
        stream_id: u32,
        compressed: bool,
        payload: &'a [u8],
        end_stream: bool,
    },
    /// An empty `DATA` frame with `END_STREAM` set was received on `stream_id`.
    EndStream { stream_id: u32 },
    /// `PING` frame received (`PING` ACK is queued automatically when `ack == false`).
    Ping { ack: bool, data: [u8; 8] },
    /// `WINDOW_UPDATE` frame received and applied.
    WindowUpdate { stream_id: u32, increment: u32 },
    /// `RST_STREAM` frame received; the stream has transitioned to [`StreamState::Closed`].
    RstStream {
        stream_id: u32,
        error_code: H2ErrorCode,
    },
    /// A stream-level error was detected on `stream_id`; `RST_STREAM` has been queued and the
    /// stream has transitioned to [`StreamState::Closed`].
    StreamError {
        stream_id: u32,
        error_code: H2ErrorCode,
    },
    /// `GOAWAY` frame received from the peer.
    GoAway {
        last_stream_id: u32,
        error_code: H2ErrorCode,
        debug_data: &'a [u8],
    },
}

// ============================================================================
// Sans-I/O Multiplexed Stream Slot & Connection Engine
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct ContinuationState {
    stream_id: u32,
    end_stream: bool,
    discarding: bool,
}

#[derive(Clone, Copy, Debug)]
struct ActiveDataFrame {
    stream_id: u32,
    stream_idx: usize,
    end_stream: bool,
    need_pad_len: bool,
    raw_remaining: usize,
    data_remaining: usize,
    pad_remaining: usize,
}

#[must_use]
#[derive(Clone, Copy, Debug)]
struct StreamSlot<const REASM_CAP: usize> {
    stream_id: u32,
    state: StreamState,
    send_window: i64,
    recv_window: i64,
    headers_received: bool,
    reasm_buf: [u8; REASM_CAP],
    reasm_len: usize,
}

impl<const REASM_CAP: usize> StreamSlot<REASM_CAP> {
    const fn new() -> Self {
        Self {
            stream_id: 0,
            state: StreamState::Idle,
            send_window: DEFAULT_INITIAL_WINDOW_SIZE as i64,
            recv_window: DEFAULT_INITIAL_WINDOW_SIZE as i64,
            headers_received: false,
            reasm_buf: [0u8; REASM_CAP],
            reasm_len: 0,
        }
    }

    fn reset(&mut self, stream_id: u32, state: StreamState, send_win: i64, recv_win: i64) {
        self.stream_id = stream_id;
        self.state = state;
        self.send_window = send_win;
        self.recv_window = recv_win;
        self.headers_received = false;
        self.reasm_len = 0;
    }

    fn append_reasm(&mut self, bytes: &[u8]) -> Result<(), H2Error> {
        if bytes.is_empty() {
            return Ok(());
        }
        if self.reasm_len + bytes.len() > REASM_CAP {
            return Err(H2Error::stream(
                self.stream_id,
                H2ErrorCode::EnhanceYourCalm,
                "stream gRPC reassembly buffer overflow",
            ));
        }
        self.reasm_buf[self.reasm_len..self.reasm_len + bytes.len()].copy_from_slice(bytes);
        self.reasm_len += bytes.len();
        Ok(())
    }
}

enum StepOutcome<'b> {
    Continue,
    EventFromInput(H2Event<'b>),
    HeadersFromReasm {
        stream_id: u32,
        len: usize,
        end_stream: bool,
    },
    GrpcFromStreamReasm {
        stream_idx: usize,
        stream_id: u32,
        compressed: bool,
        start: usize,
        end: usize,
        end_stream: bool,
    },
    HeadersFromPartial {
        stream_id: u32,
        start: usize,
        end: usize,
        end_stream: bool,
    },
    GoAwayFromPartial {
        last_stream_id: u32,
        error_code: H2ErrorCode,
        start: usize,
        end: usize,
    },
}

const TX_QUEUE_CAP: usize = 512;

/// `#![no_std]` zero-allocation Sans-I/O HTTP/2 connection and multiplexed stream state machine (RFC 9113).
#[must_use]
pub struct H2Connection<
    const MAX_STREAMS: usize = DEFAULT_MAX_STREAMS,
    const REASM_CAP: usize = DEFAULT_REASM_CAP,
> {
    role: Role,
    local_settings: H2Settings,
    peer_settings: H2Settings,
    preface_received: usize,
    peer_initial_settings_received: bool,
    peer_acked_settings: bool,
    conn_send_window: i64,
    conn_recv_window: i64,
    last_peer_stream_id: u32,
    next_local_stream_id: u32,
    max_message_size: usize,
    auto_window_update: bool,
    goaway_received: Option<(u32, H2ErrorCode)>,
    goaway_sent: bool,
    continuation_state: Option<ContinuationState>,
    active_data: Option<ActiveDataFrame>,
    discard_remaining: u32,
    header_reasm_buf: [u8; REASM_CAP],
    header_reasm_len: usize,
    partial_hdr_buf: [u8; FRAME_HEADER_LEN],
    partial_hdr_len: usize,
    partial_payload_buf: [u8; REASM_CAP],
    partial_payload_len: usize,
    streams: [StreamSlot<REASM_CAP>; MAX_STREAMS],
    tx_buf: [u8; TX_QUEUE_CAP],
    tx_len: usize,
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> fmt::Debug
    for H2Connection<MAX_STREAMS, REASM_CAP>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("H2Connection")
            .field("role", &self.role)
            .field("local_settings", &self.local_settings)
            .field("peer_settings", &self.peer_settings)
            .field("conn_send_window", &self.conn_send_window)
            .field("conn_recv_window", &self.conn_recv_window)
            .field("last_peer_stream_id", &self.last_peer_stream_id)
            .field("next_local_stream_id", &self.next_local_stream_id)
            .field("tx_len", &self.tx_len)
            .finish()
    }
}

impl<const MAX_STREAMS: usize, const REASM_CAP: usize> H2Connection<MAX_STREAMS, REASM_CAP> {
    /// Creates a new [`H2Connection`] for the given [`Role`], queuing the outbound connection
    /// preface (`CLIENT_CONNECTION_PREFACE` + `SETTINGS` for [`Role::Client`], or `SETTINGS` for
    /// [`Role::Server`]).
    pub fn new(role: Role) -> Self {
        let mut local_settings = H2Settings::new();
        local_settings.max_concurrent_streams = MAX_STREAMS as u32;

        let mut conn = Self {
            role,
            local_settings,
            peer_settings: H2Settings::default_peer(),
            preface_received: if role == Role::Client {
                CLIENT_CONNECTION_PREFACE.len()
            } else {
                0
            },
            peer_initial_settings_received: false,
            peer_acked_settings: false,
            conn_send_window: DEFAULT_INITIAL_WINDOW_SIZE as i64,
            conn_recv_window: DEFAULT_INITIAL_WINDOW_SIZE as i64,
            last_peer_stream_id: 0,
            next_local_stream_id: if role == Role::Client { 1 } else { 2 },
            max_message_size: DEFAULT_MAX_MESSAGE_SIZE,
            auto_window_update: false,
            goaway_received: None,
            goaway_sent: false,
            continuation_state: None,
            active_data: None,
            discard_remaining: 0,
            header_reasm_buf: [0u8; REASM_CAP],
            header_reasm_len: 0,
            partial_hdr_buf: [0u8; FRAME_HEADER_LEN],
            partial_hdr_len: 0,
            partial_payload_buf: [0u8; REASM_CAP],
            partial_payload_len: 0,
            streams: [const { StreamSlot::new() }; MAX_STREAMS],
            tx_buf: [0u8; TX_QUEUE_CAP],
            tx_len: 0,
        };

        if role == Role::Client {
            let _ = conn.queue_raw_bytes(CLIENT_CONNECTION_PREFACE);
        }
        let _ = conn.queue_settings(&local_settings);
        conn
    }

    /// Creates a new server-side [`H2Connection`].
    #[inline]
    pub fn new_server() -> Self {
        Self::new(Role::Server)
    }

    /// Creates a new client-side [`H2Connection`].
    #[inline]
    pub fn new_client() -> Self {
        Self::new(Role::Client)
    }

    /// Returns the endpoint [`Role`] (`Client` or `Server`).
    #[inline]
    pub const fn role(&self) -> Role {
        self.role
    }

    /// Returns a reference to our local [`H2Settings`].
    #[inline]
    pub const fn local_settings(&self) -> &H2Settings {
        &self.local_settings
    }

    /// Returns a reference to the peer's [`H2Settings`].
    #[inline]
    pub const fn peer_settings(&self) -> &H2Settings {
        &self.peer_settings
    }

    /// Returns `true` once the peer has sent its initial `SETTINGS` (and the 24-byte client preface if server).
    #[inline]
    pub const fn is_handshake_complete(&self) -> bool {
        self.preface_received == CLIENT_CONNECTION_PREFACE.len()
            && self.peer_initial_settings_received
    }

    /// Returns `true` if the peer has acknowledged our initial `SETTINGS` frame.
    #[inline]
    pub const fn peer_acked_settings(&self) -> bool {
        self.peer_acked_settings
    }

    /// Returns the current connection-level send flow-control window.
    #[inline]
    pub const fn conn_send_window(&self) -> i64 {
        self.conn_send_window
    }

    /// Returns the current connection-level receive flow-control window.
    #[inline]
    pub const fn conn_recv_window(&self) -> i64 {
        self.conn_recv_window
    }

    /// Returns the maximum inbound gRPC message payload size in bytes.
    #[inline]
    pub const fn max_message_size(&self) -> usize {
        self.max_message_size
    }

    /// Configures the maximum inbound gRPC message payload size in bytes.
    #[inline]
    pub fn set_max_message_size(&mut self, max_message_size: usize) {
        self.max_message_size = max_message_size;
    }

    /// Returns whether `DATA` frames automatically queue `WINDOW_UPDATE` frames upon receipt.
    #[inline]
    pub const fn auto_window_update(&self) -> bool {
        self.auto_window_update
    }

    /// Configures whether receiving `DATA` frames automatically replenishes receive windows
    /// and queues `WINDOW_UPDATE` frames.
    #[inline]
    pub fn set_auto_window_update(&mut self, enabled: bool) {
        self.auto_window_update = enabled;
    }

    /// Returns the `(last_stream_id, error_code)` from a received `GOAWAY` frame, if any.
    #[inline]
    pub const fn goaway_received(&self) -> Option<(u32, H2ErrorCode)> {
        self.goaway_received
    }

    /// Returns the current [`StreamState`] for `stream_id`.
    pub fn stream_state(&self, stream_id: u32) -> StreamState {
        if stream_id == 0 {
            return StreamState::Idle;
        }
        if let Some(idx) = self.find_stream_idx(stream_id) {
            return self.streams[idx].state;
        }
        if self.was_stream_opened(stream_id) {
            StreamState::Closed
        } else {
            StreamState::Idle
        }
    }

    /// Returns the current send flow-control window for `stream_id`, if tracked.
    pub fn stream_send_window(&self, stream_id: u32) -> Option<i64> {
        self.find_stream_idx(stream_id)
            .map(|idx| self.streams[idx].send_window)
    }

    /// Returns the current receive flow-control window for `stream_id`, if tracked.
    pub fn stream_recv_window(&self, stream_id: u32) -> Option<i64> {
        self.find_stream_idx(stream_id)
            .map(|idx| self.streams[idx].recv_window)
    }

    /// Returns the number of currently active (`Open`, `HalfClosedRemote`, or `HalfClosedLocal`) streams.
    pub fn active_stream_count(&self) -> usize {
        let mut count = 0;
        let mut i = 0;
        while i < MAX_STREAMS {
            if self.streams[i].state.is_active() {
                count += 1;
            }
            i += 1;
        }
        count
    }

    /// Returns `true` if there are queued outbound control/preface bytes ready to be written via [`Self::poll_transmit`].
    #[inline]
    pub const fn has_pending_transmit(&self) -> bool {
        self.tx_len > 0
    }

    /// Returns the number of queued outbound control/preface bytes.
    #[inline]
    pub const fn pending_transmit_len(&self) -> usize {
        self.tx_len
    }

    /// Copies queued outbound preface and control frames (`SETTINGS`, `SETTINGS` ACK, `PING` ACK,
    /// `WINDOW_UPDATE`, `RST_STREAM`, `GOAWAY`) into `out` and returns the number of bytes written.
    pub fn poll_transmit(&mut self, out: &mut [u8]) -> usize {
        let n = self.tx_len.min(out.len());
        if n == 0 {
            return 0;
        }
        out[..n].copy_from_slice(&self.tx_buf[..n]);
        if n < self.tx_len {
            self.tx_buf.copy_within(n..self.tx_len, 0);
        }
        self.tx_len -= n;
        n
    }

    /// Alias for [`Self::poll_transmit`].
    #[inline]
    pub fn write_pending_control_frames(&mut self, out: &mut [u8]) -> usize {
        self.poll_transmit(out)
    }

    // ========================================================================
    // Outbound Control Frame Queuing
    // ========================================================================

    fn queue_raw_bytes(&mut self, bytes: &[u8]) -> Result<(), H2Error> {
        if self.tx_len + bytes.len() > TX_QUEUE_CAP {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "outbound control frame queue overflow",
            ));
        }
        self.tx_buf[self.tx_len..self.tx_len + bytes.len()].copy_from_slice(bytes);
        self.tx_len += bytes.len();
        Ok(())
    }

    /// Queues an outbound `SETTINGS` frame.
    pub fn queue_settings(&mut self, settings: &H2Settings) -> Result<(), H2Error> {
        let mut tmp = [0u8; FRAME_HEADER_LEN + 30];
        let n = settings.encode_frame(&mut tmp)?;
        self.queue_raw_bytes(&tmp[..n])
    }

    /// Queues an outbound 9-byte `SETTINGS` ACK frame.
    pub fn queue_settings_ack(&mut self) -> Result<(), H2Error> {
        let hdr = FrameHeader::new(0, FrameType::SETTINGS, FLAG_ACK, 0);
        self.queue_raw_bytes(&hdr.encode())
    }

    /// Queues an outbound `PING` or `PING` ACK frame with the 8-byte `data` payload.
    pub fn queue_ping(&mut self, ack: bool, data: [u8; 8]) -> Result<(), H2Error> {
        let flags = if ack { FLAG_ACK } else { 0 };
        let hdr = FrameHeader::new(8, FrameType::PING, flags, 0);
        let mut frame = [0u8; FRAME_HEADER_LEN + 8];
        frame[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        frame[FRAME_HEADER_LEN..].copy_from_slice(&data);
        self.queue_raw_bytes(&frame)
    }

    /// Queues an outbound `WINDOW_UPDATE` frame on `stream_id` (`0` for connection, `> 0` for stream).
    pub fn queue_window_update(&mut self, stream_id: u32, increment: u32) -> Result<(), H2Error> {
        if increment == 0 || increment > MAX_FLOW_CONTROL_WINDOW {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "invalid WINDOW_UPDATE increment",
            ));
        }
        let hdr = FrameHeader::new(4, FrameType::WINDOW_UPDATE, 0, stream_id);
        let mut frame = [0u8; FRAME_HEADER_LEN + 4];
        frame[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        frame[FRAME_HEADER_LEN..]
            .copy_from_slice(&(increment & MAX_FLOW_CONTROL_WINDOW).to_be_bytes());
        self.queue_raw_bytes(&frame)
    }

    /// Replenishes the local receive flow-control window by `increment` bytes and queues
    /// `WINDOW_UPDATE` frame(s) for the connection (`stream_id == 0`) or for both the connection
    /// and the specified stream (`stream_id > 0`).
    pub fn replenish_recv_window(&mut self, stream_id: u32, increment: u32) -> Result<(), H2Error> {
        if increment == 0 || increment > MAX_FLOW_CONTROL_WINDOW {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "invalid WINDOW_UPDATE increment",
            ));
        }
        let inc_i64 = increment as i64;
        if self.conn_recv_window + inc_i64 > MAX_FLOW_CONTROL_WINDOW as i64 {
            return Err(H2Error::connection(
                H2ErrorCode::FlowControlError,
                "local connection receive window overflow",
            ));
        }
        if stream_id != 0 {
            if let Some(idx) = self.find_stream_idx(stream_id) {
                if self.streams[idx].recv_window + inc_i64 > MAX_FLOW_CONTROL_WINDOW as i64 {
                    return Err(H2Error::stream(
                        stream_id,
                        H2ErrorCode::FlowControlError,
                        "local stream receive window overflow",
                    ));
                }
                self.streams[idx].recv_window += inc_i64;
            }
            self.conn_recv_window += inc_i64;
            self.queue_window_update(0, increment)?;
            self.queue_window_update(stream_id, increment)?;
        } else {
            self.conn_recv_window += inc_i64;
            self.queue_window_update(0, increment)?;
        }
        Ok(())
    }

    /// Queues an outbound `RST_STREAM` frame for `stream_id` and transitions the stream to [`StreamState::Closed`].
    pub fn queue_rst_stream(
        &mut self,
        stream_id: u32,
        error_code: H2ErrorCode,
    ) -> Result<(), H2Error> {
        if stream_id == 0 {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "cannot send RST_STREAM on stream 0",
            ));
        }
        if let Some(ad) = self.active_data {
            if ad.stream_id == stream_id {
                let rem = if ad.need_pad_len {
                    ad.raw_remaining
                } else {
                    ad.data_remaining + ad.pad_remaining
                };
                self.discard_remaining = self.discard_remaining.saturating_add(rem as u32);
                self.active_data = None;
            }
        }
        if let Some(idx) = self.find_stream_idx(stream_id) {
            self.streams[idx].state = StreamState::Closed;
            self.streams[idx].reasm_len = 0;
        }
        let hdr = FrameHeader::new(4, FrameType::RST_STREAM, 0, stream_id);
        let mut frame = [0u8; FRAME_HEADER_LEN + 4];
        frame[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        frame[FRAME_HEADER_LEN..].copy_from_slice(&error_code.as_u32().to_be_bytes());
        self.queue_raw_bytes(&frame)
    }

    /// Queues an outbound `GOAWAY` frame on stream `0`.
    pub fn queue_goaway(
        &mut self,
        last_stream_id: u32,
        error_code: H2ErrorCode,
        debug_data: &[u8],
    ) -> Result<(), H2Error> {
        let payload_len = 8 + debug_data.len();
        if self.tx_len + FRAME_HEADER_LEN + payload_len > TX_QUEUE_CAP {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "outbound control frame queue overflow",
            ));
        }
        let hdr = FrameHeader::new(payload_len as u32, FrameType::GOAWAY, 0, 0);
        let _ = self.queue_raw_bytes(&hdr.encode());
        let _ = self.queue_raw_bytes(&(last_stream_id & MAX_STREAM_ID).to_be_bytes());
        let _ = self.queue_raw_bytes(&error_code.as_u32().to_be_bytes());
        let _ = self.queue_raw_bytes(debug_data);
        self.goaway_sent = true;
        Ok(())
    }

    fn record_error(&mut self, err: H2Error) -> H2Error {
        if err.is_connection_error() {
            if !self.goaway_sent {
                let _ = self.queue_goaway(self.last_peer_stream_id, err.code, &[]);
            }
        } else {
            let _ = self.queue_rst_stream(err.stream_id, err.code);
        }
        err
    }

    // ========================================================================
    // Outbound Stream Creation, HEADERS, and DATA Framing
    // ========================================================================

    /// Opens a new local stream (with odd stream ID `1, 3, 5, ...` for [`Role::Client`])
    /// and transitions it to [`StreamState::Open`].
    pub fn open_stream(&mut self) -> Result<u32, H2Error> {
        if self.goaway_received.is_some() {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "cannot open stream after receiving GOAWAY",
            ));
        }
        if self.active_stream_count()
            >= (self.peer_settings.max_concurrent_streams as usize).min(MAX_STREAMS)
        {
            return Err(H2Error::connection(
                H2ErrorCode::RefusedStream,
                "maximum concurrent streams reached",
            ));
        }
        let stream_id = self.next_local_stream_id;
        if stream_id > MAX_STREAM_ID {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "stream ID space exhausted",
            ));
        }
        let slot_idx = self.alloc_stream_slot(stream_id)?;
        self.next_local_stream_id = stream_id.saturating_add(2);
        self.streams[slot_idx].reset(
            stream_id,
            StreamState::Open,
            self.peer_settings.initial_window_size as i64,
            self.local_settings.initial_window_size as i64,
        );
        Ok(stream_id)
    }

    fn validate_outbound_stream(&self, stream_id: u32) -> Result<usize, H2Error> {
        let Some(idx) = self.find_stream_idx(stream_id) else {
            return Err(H2Error::stream(
                stream_id,
                H2ErrorCode::StreamClosed,
                "stream not found for outbound frame",
            ));
        };
        if !matches!(
            self.streams[idx].state,
            StreamState::Open | StreamState::HalfClosedRemote
        ) {
            return Err(H2Error::stream(
                stream_id,
                H2ErrorCode::StreamClosed,
                "stream closed for outbound frame",
            ));
        }
        Ok(idx)
    }

    fn encode_stream_headers_in_place<F>(
        &mut self,
        stream_id: u32,
        end_stream: bool,
        out: &mut [u8],
        encode_block: F,
    ) -> Result<usize, H2Error>
    where
        F: FnOnce(&mut [u8]) -> Result<usize, EncodeError>,
    {
        if out.len() < FRAME_HEADER_LEN {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "output buffer too small for HEADERS frame",
            ));
        }
        let idx = self.validate_outbound_stream(stream_id)?;
        let block_len = encode_block(&mut out[FRAME_HEADER_LEN..])?;
        if block_len > self.peer_settings.max_frame_size as usize {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "outbound HEADERS exceed peer max_frame_size",
            ));
        }
        let flags = FLAG_END_HEADERS | if end_stream { FLAG_END_STREAM } else { 0 };
        let hdr = FrameHeader::new(block_len as u32, FrameType::HEADERS, flags, stream_id);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        if end_stream {
            self.transition_local_end_stream(idx);
        }
        Ok(FRAME_HEADER_LEN + block_len)
    }

    /// Encodes a `HEADERS` frame (automatically splitting into `CONTINUATION` frames if
    /// `header_block.len() > peer_settings.max_frame_size`) into `out` and updates stream state.
    pub fn send_headers_frame(
        &mut self,
        stream_id: u32,
        header_block: &[u8],
        end_stream: bool,
        out: &mut [u8],
    ) -> Result<usize, H2Error> {
        let idx = self.validate_outbound_stream(stream_id)?;
        let max_chunk = (self.peer_settings.max_frame_size as usize).max(1);
        let mut pos = 0usize;
        if header_block.len() <= max_chunk {
            let total = FRAME_HEADER_LEN + header_block.len();
            if out.len() < total {
                return Err(H2Error::connection(
                    H2ErrorCode::InternalError,
                    "output buffer too small for HEADERS frame",
                ));
            }
            let flags = FLAG_END_HEADERS | if end_stream { FLAG_END_STREAM } else { 0 };
            let hdr = FrameHeader::new(
                header_block.len() as u32,
                FrameType::HEADERS,
                flags,
                stream_id,
            );
            out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
            out[FRAME_HEADER_LEN..total].copy_from_slice(header_block);
            pos = total;
        } else {
            let mut offset = 0usize;
            let mut first = true;
            while offset < header_block.len() {
                let chunk_len = (header_block.len() - offset).min(max_chunk);
                let is_last = offset + chunk_len == header_block.len();
                if out.len() - pos < FRAME_HEADER_LEN + chunk_len {
                    return Err(H2Error::connection(
                        H2ErrorCode::InternalError,
                        "output buffer too small for fragmented HEADERS/CONTINUATION",
                    ));
                }
                let (frame_type, flags) = if first {
                    first = false;
                    let f = if end_stream { FLAG_END_STREAM } else { 0 }
                        | if is_last { FLAG_END_HEADERS } else { 0 };
                    (FrameType::HEADERS, f)
                } else {
                    let f = if is_last { FLAG_END_HEADERS } else { 0 };
                    (FrameType::CONTINUATION, f)
                };
                let hdr = FrameHeader::new(chunk_len as u32, frame_type, flags, stream_id);
                out[pos..pos + FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
                pos += FRAME_HEADER_LEN;
                out[pos..pos + chunk_len]
                    .copy_from_slice(&header_block[offset..offset + chunk_len]);
                pos += chunk_len;
                offset += chunk_len;
            }
        }

        if end_stream {
            self.transition_local_end_stream(idx);
        }
        Ok(pos)
    }

    /// Opens a client stream and encodes outbound gRPC request `HEADERS` (`:method: POST`,
    /// `:scheme`, `:path`, `:authority`, `content-type`, `te: trailers`) into `out`.
    pub fn send_grpc_request_headers(
        &mut self,
        scheme: &[u8],
        authority: &[u8],
        path: &[u8],
        content_type: &[u8],
        end_stream: bool,
        out: &mut [u8],
    ) -> Result<(u32, usize), H2Error> {
        if out.len() < FRAME_HEADER_LEN {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "output buffer too small for HEADERS frame",
            ));
        }
        let block_len = HpackEncoder::new().encode_grpc_request_headers(
            scheme,
            authority,
            path,
            content_type,
            &mut out[FRAME_HEADER_LEN..],
        )?;
        if block_len > self.peer_settings.max_frame_size as usize {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "outbound HEADERS exceed peer max_frame_size",
            ));
        }
        let stream_id = self.open_stream()?;
        let flags = FLAG_END_HEADERS | if end_stream { FLAG_END_STREAM } else { 0 };
        let hdr = FrameHeader::new(block_len as u32, FrameType::HEADERS, flags, stream_id);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        if end_stream {
            if let Some(idx) = self.find_stream_idx(stream_id) {
                self.transition_local_end_stream(idx);
            }
        }
        Ok((stream_id, FRAME_HEADER_LEN + block_len))
    }

    /// Encodes outbound gRPC response `HEADERS` (`:status: 200`, `content-type`) on `stream_id` into `out`.
    pub fn send_grpc_response_headers(
        &mut self,
        stream_id: u32,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, H2Error> {
        self.encode_stream_headers_in_place(stream_id, false, out, |dst| {
            HpackEncoder::new().encode_grpc_response_headers(content_type, dst)
        })
    }

    /// Encodes outbound gRPC response trailers (`grpc-status`, `grpc-message`, `END_STREAM = 1`)
    /// on `stream_id` into `out`.
    pub fn send_grpc_trailers(
        &mut self,
        stream_id: u32,
        status: GrpcStatus<'_>,
        out: &mut [u8],
    ) -> Result<usize, H2Error> {
        self.encode_stream_headers_in_place(stream_id, true, out, |dst| {
            HpackEncoder::new().encode_grpc_trailers(status, dst)
        })
    }

    /// Encodes an outbound gRPC Trailers-Only response (`:status: 200`, `content-type`,
    /// `grpc-status`, `grpc-message`, `END_STREAM = 1`) on `stream_id` into `out`.
    pub fn send_grpc_trailers_only(
        &mut self,
        stream_id: u32,
        status: GrpcStatus<'_>,
        content_type: &[u8],
        out: &mut [u8],
    ) -> Result<usize, H2Error> {
        self.encode_stream_headers_in_place(stream_id, true, out, |dst| {
            HpackEncoder::new().encode_grpc_trailers_only(status, content_type, dst)
        })
    }

    /// Encodes an outbound `DATA` frame on `stream_id` into `out`, deducting `payload.len()`
    /// from both connection and stream send flow-control windows.
    pub fn send_data_frame(
        &mut self,
        stream_id: u32,
        payload: &[u8],
        end_stream: bool,
        out: &mut [u8],
    ) -> Result<usize, H2Error> {
        let idx = self.validate_outbound_stream(stream_id)?;
        if payload.len() > self.peer_settings.max_frame_size as usize {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "outbound DATA exceeds peer max_frame_size",
            ));
        }
        let len_i64 = payload.len() as i64;
        if len_i64 > self.conn_send_window || len_i64 > self.streams[idx].send_window {
            return Err(H2Error::stream(
                stream_id,
                H2ErrorCode::FlowControlError,
                "insufficient send flow-control window for DATA",
            ));
        }
        let total = FRAME_HEADER_LEN + payload.len();
        if out.len() < total {
            return Err(H2Error::connection(
                H2ErrorCode::InternalError,
                "output buffer too small for DATA frame",
            ));
        }
        self.conn_send_window -= len_i64;
        self.streams[idx].send_window -= len_i64;
        let flags = if end_stream { FLAG_END_STREAM } else { 0 };
        let hdr = FrameHeader::new(payload.len() as u32, FrameType::DATA, flags, stream_id);
        out[..FRAME_HEADER_LEN].copy_from_slice(&hdr.encode());
        out[FRAME_HEADER_LEN..total].copy_from_slice(payload);
        if end_stream {
            self.transition_local_end_stream(idx);
        }
        Ok(total)
    }

    /// Serializes a Roto message via `f`, prepends its 5-byte gRPC LPM header, and prepends
    /// the 9-byte HTTP/2 `DATA` frame header into `buf` in a single backward pass, updating
    /// connection and stream flow-control windows and lifecycle state.
    pub fn send_grpc_data_frame<'a, F>(
        &mut self,
        stream_id: u32,
        compressed: bool,
        end_stream: bool,
        buf: &'a mut [u8],
        f: F,
    ) -> Result<&'a [u8], H2Error>
    where
        F: FnOnce(&mut BackwardWriter<'_>),
    {
        let idx = self.validate_outbound_stream(stream_id)?;
        let mut writer = BackwardWriter::new(buf);
        encode_grpc_data_frame(&mut writer, stream_id, end_stream, compressed, f)?;
        let total_len = writer.written_len();
        let data_len = total_len - FRAME_HEADER_LEN;
        if data_len > self.peer_settings.max_frame_size as usize {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "outbound gRPC DATA frame exceeds peer max_frame_size",
            ));
        }
        let len_i64 = data_len as i64;
        if len_i64 > self.conn_send_window || len_i64 > self.streams[idx].send_window {
            return Err(H2Error::stream(
                stream_id,
                H2ErrorCode::FlowControlError,
                "insufficient send flow-control window for gRPC DATA",
            ));
        }
        self.conn_send_window -= len_i64;
        self.streams[idx].send_window -= len_i64;
        if end_stream {
            self.transition_local_end_stream(idx);
        }
        Ok(writer.finish()?)
    }

    // ========================================================================
    // Inbound Frame & Preface Processing (Zero-Copy Fast Path + Reassembly)
    // ========================================================================

    /// Feeds inbound wire bytes `buf` into the HTTP/2 connection engine and returns
    /// `(bytes_consumed, Option<H2Event<'a>>)`.
    ///
    /// When an inbound `DATA` frame contains a complete gRPC message in `buf` with no prior
    /// partial message bytes on that stream, [`H2Event::GrpcMessage::payload`] borrows directly
    /// from `buf` with zero copying.
    pub fn recv<'a>(&'a mut self, buf: &'a [u8]) -> Result<(usize, Option<H2Event<'a>>), H2Error> {
        let mut consumed = 0usize;

        // 1. If server role, validate the 24-byte client connection preface first.
        if self.role == Role::Server && self.preface_received < CLIENT_CONNECTION_PREFACE.len() {
            let need = CLIENT_CONNECTION_PREFACE.len() - self.preface_received;
            let take = buf.len().min(need);
            if take == 0 {
                return Ok((0, None));
            }
            if buf[..take]
                != CLIENT_CONNECTION_PREFACE[self.preface_received..self.preface_received + take]
            {
                return Err(self.record_error(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "invalid HTTP/2 client connection preface",
                )));
            }
            self.preface_received += take;
            consumed += take;
            if self.preface_received < CLIENT_CONNECTION_PREFACE.len() {
                return Ok((consumed, None));
            }
        }

        // 2. Process streaming DATA payloads and HTTP/2 frames until an H2Event is ready or `buf` is exhausted.
        let outcome: StepOutcome<'a> = loop {
            let rem = &buf[consumed..];

            // 2a-pre. If a previous stream-errored DATA frame still has unconsumed payload bytes, discard them.
            if self.discard_remaining > 0 {
                let skip = rem.len().min(self.discard_remaining as usize);
                if skip == 0 {
                    return Ok((consumed, None));
                }
                consumed += skip;
                self.discard_remaining -= skip as u32;
                if self.discard_remaining > 0 {
                    return Ok((consumed, None));
                }
                continue;
            }

            // 2a. If an inbound DATA frame payload is actively streaming, consume from `rem`.
            if self.active_data.is_some() {
                let (step_bytes, res) = self.step_active_data(rem);
                consumed += step_bytes;
                let maybe_outcome = match res {
                    Ok(o) => o,
                    Err(err) => return self.handle_recv_error(err, buf.len(), &mut consumed),
                };
                match maybe_outcome {
                    Some(StepOutcome::Continue) => continue,
                    Some(out) => break out,
                    None => return Ok((consumed, None)),
                }
            }

            // 2b. Handle any partially buffered frame header or non-DATA payload from a fragmented read.
            if self.partial_hdr_len > 0 {
                if self.partial_hdr_len < FRAME_HEADER_LEN {
                    let need = FRAME_HEADER_LEN - self.partial_hdr_len;
                    let take = rem.len().min(need);
                    if take == 0 {
                        return Ok((consumed, None));
                    }
                    self.partial_hdr_buf[self.partial_hdr_len..self.partial_hdr_len + take]
                        .copy_from_slice(&rem[..take]);
                    self.partial_hdr_len += take;
                    consumed += take;
                    if self.partial_hdr_len < FRAME_HEADER_LEN {
                        return Ok((consumed, None));
                    }
                }

                let hdr = FrameHeader::decode(self.partial_hdr_buf);
                if let Err(err) = self.validate_frame_header(hdr) {
                    self.partial_hdr_len = 0;
                    self.partial_payload_len = 0;
                    return Err(self.record_error(err));
                }

                // DATA frames stream directly via `active_data` once their 9-byte header is complete.
                if hdr.frame_type == FrameType::DATA {
                    self.partial_hdr_len = 0;
                    self.partial_payload_len = 0;
                    let outcome = match self.begin_data_frame(hdr) {
                        Ok(o) => o,
                        Err(err) => return self.handle_recv_error(err, buf.len(), &mut consumed),
                    };
                    if matches!(outcome, StepOutcome::Continue) {
                        continue;
                    }
                    break outcome;
                }

                let frame_len = hdr.length as usize;
                if frame_len > REASM_CAP {
                    self.partial_hdr_len = 0;
                    self.partial_payload_len = 0;
                    return Err(self.record_error(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "fragmented frame exceeds reassembly buffer capacity",
                    )));
                }
                let rem_after_hdr = &buf[consumed..];
                let need_payload = frame_len - self.partial_payload_len;
                let take_payload = rem_after_hdr.len().min(need_payload);
                self.partial_payload_buf
                    [self.partial_payload_len..self.partial_payload_len + take_payload]
                    .copy_from_slice(&rem_after_hdr[..take_payload]);
                self.partial_payload_len += take_payload;
                consumed += take_payload;
                if self.partial_payload_len < frame_len {
                    return Ok((consumed, None));
                }

                self.partial_hdr_len = 0;
                self.partial_payload_len = 0;
                let outcome = match self.process_partial_frame(hdr, frame_len) {
                    Ok(o) => o,
                    Err(err) => return self.handle_recv_error(err, buf.len(), &mut consumed),
                };
                if matches!(outcome, StepOutcome::Continue) {
                    continue;
                }
                break outcome;
            }

            if rem.is_empty() {
                return Ok((consumed, None));
            }

            if rem.len() < FRAME_HEADER_LEN {
                self.partial_hdr_buf[..rem.len()].copy_from_slice(rem);
                self.partial_hdr_len = rem.len();
                self.partial_payload_len = 0;
                consumed += rem.len();
                return Ok((consumed, None));
            }

            let hdr = FrameHeader::decode([
                rem[0], rem[1], rem[2], rem[3], rem[4], rem[5], rem[6], rem[7], rem[8],
            ]);
            if let Err(err) = self.validate_frame_header(hdr) {
                return Err(self.record_error(err));
            }

            // Stream DATA frames via `active_data` (both contiguous zero-copy fast path and fragmented).
            if hdr.frame_type == FrameType::DATA {
                consumed += FRAME_HEADER_LEN;
                let outcome = match self.begin_data_frame(hdr) {
                    Ok(o) => o,
                    Err(err) => return self.handle_recv_error(err, buf.len(), &mut consumed),
                };
                if matches!(outcome, StepOutcome::Continue) {
                    continue;
                }
                break outcome;
            }

            let frame_len = hdr.length as usize;
            let frame_total = FRAME_HEADER_LEN + frame_len;
            if rem.len() < frame_total {
                if frame_len > REASM_CAP {
                    return Err(self.record_error(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "fragmented frame exceeds reassembly buffer capacity",
                    )));
                }
                self.partial_hdr_buf.copy_from_slice(&rem[..FRAME_HEADER_LEN]);
                self.partial_hdr_len = FRAME_HEADER_LEN;
                let avail_payload = rem.len() - FRAME_HEADER_LEN;
                self.partial_payload_buf[..avail_payload]
                    .copy_from_slice(&rem[FRAME_HEADER_LEN..]);
                self.partial_payload_len = avail_payload;
                consumed += rem.len();
                return Ok((consumed, None));
            }

            // Complete non-DATA frame is contiguous in `rem`.
            let payload_slice = &rem[FRAME_HEADER_LEN..frame_total];
            consumed += frame_total;
            let outcome = match self.process_input_frame(hdr, payload_slice) {
                Ok(res) => res,
                Err(err) => return self.handle_recv_error(err, buf.len(), &mut consumed),
            };
            if matches!(outcome, StepOutcome::Continue) {
                continue;
            }
            break outcome;
        };

        let event = match outcome {
            StepOutcome::Continue => None,
            StepOutcome::EventFromInput(ev) => Some(ev),
            StepOutcome::HeadersFromReasm {
                stream_id,
                len,
                end_stream,
            } => Some(H2Event::Headers {
                stream_id,
                header_block: &self.header_reasm_buf[..len],
                end_stream,
            }),
            StepOutcome::GrpcFromStreamReasm {
                stream_idx,
                stream_id,
                compressed,
                start,
                end,
                end_stream,
            } => Some(H2Event::GrpcMessage {
                stream_id,
                compressed,
                payload: &self.streams[stream_idx].reasm_buf[start..end],
                end_stream,
            }),
            StepOutcome::HeadersFromPartial {
                stream_id,
                start,
                end,
                end_stream,
            } => Some(H2Event::Headers {
                stream_id,
                header_block: &self.partial_payload_buf[start..end],
                end_stream,
            }),
            StepOutcome::GoAwayFromPartial {
                last_stream_id,
                error_code,
                start,
                end,
            } => Some(H2Event::GoAway {
                last_stream_id,
                error_code,
                debug_data: &self.partial_payload_buf[start..end],
            }),
        };
        Ok((consumed, event))
    }

    fn handle_recv_error<'a>(
        &mut self,
        err: H2Error,
        buf_len: usize,
        consumed: &mut usize,
    ) -> Result<(usize, Option<H2Event<'a>>), H2Error> {
        let err = self.record_error(err);
        if err.is_connection_error() {
            return Err(err);
        }
        if self.discard_remaining > 0 {
            let skip = (buf_len - *consumed).min(self.discard_remaining as usize);
            *consumed += skip;
            self.discard_remaining -= skip as u32;
        }
        Ok((
            *consumed,
            Some(H2Event::StreamError {
                stream_id: err.stream_id,
                error_code: err.code,
            }),
        ))
    }

    fn validate_frame_header(&self, hdr: FrameHeader) -> Result<(), H2Error> {
        if hdr.length > self.local_settings.max_frame_size {
            return Err(H2Error::connection(
                H2ErrorCode::FrameSizeError,
                "frame payload length exceeds SETTINGS_MAX_FRAME_SIZE",
            ));
        }

        // First frame received from the peer MUST be a non-ACK SETTINGS frame on stream 0 (RFC 9113 §3.4).
        if !self.peer_initial_settings_received
            && (hdr.frame_type != FrameType::SETTINGS
                || (hdr.flags & FLAG_ACK) != 0
                || hdr.stream_id != 0)
        {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "first frame after connection preface must be SETTINGS",
            ));
        }

        // While a HEADERS..CONTINUATION block is in progress, only CONTINUATION on that same stream is allowed.
        if let Some(cont) = self.continuation_state {
            if hdr.frame_type != FrameType::CONTINUATION || hdr.stream_id != cont.stream_id {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "interleaved frame during HEADERS/CONTINUATION sequence",
                ));
            }
        } else if hdr.frame_type == FrameType::CONTINUATION {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "unexpected CONTINUATION frame without preceding HEADERS",
            ));
        }

        match hdr.frame_type {
            FrameType::DATA | FrameType::HEADERS | FrameType::CONTINUATION
                if hdr.stream_id == 0 =>
            {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "stream-level frame received on stream 0",
                ));
            }
            FrameType::PRIORITY => {
                if hdr.stream_id == 0 {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "PRIORITY frame received on stream 0",
                    ));
                }
                if hdr.length != 5 {
                    return Err(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "PRIORITY frame length must be 5",
                    ));
                }
            }
            FrameType::RST_STREAM => {
                if hdr.stream_id == 0 {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "RST_STREAM frame received on stream 0",
                    ));
                }
                if hdr.length != 4 {
                    return Err(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "RST_STREAM frame length must be 4",
                    ));
                }
            }
            FrameType::SETTINGS => {
                if hdr.stream_id != 0 {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "SETTINGS frame received on non-zero stream",
                    ));
                }
                if (hdr.flags & FLAG_ACK) != 0 {
                    if hdr.length != 0 {
                        return Err(H2Error::connection(
                            H2ErrorCode::FrameSizeError,
                            "SETTINGS ACK frame must have length 0",
                        ));
                    }
                } else if !hdr.length.is_multiple_of(6) {
                    return Err(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "SETTINGS payload length must be a multiple of 6",
                    ));
                }
            }
            FrameType::PUSH_PROMISE => {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "PUSH_PROMISE rejected when SETTINGS_ENABLE_PUSH is 0",
                ));
            }
            FrameType::PING => {
                if hdr.stream_id != 0 {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "PING frame received on non-zero stream",
                    ));
                }
                if hdr.length != 8 {
                    return Err(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "PING frame length must be 8",
                    ));
                }
            }
            FrameType::GOAWAY => {
                if hdr.stream_id != 0 {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "GOAWAY frame received on non-zero stream",
                    ));
                }
                if hdr.length < 8 {
                    return Err(H2Error::connection(
                        H2ErrorCode::FrameSizeError,
                        "GOAWAY frame length must be at least 8",
                    ));
                }
            }
            FrameType::WINDOW_UPDATE if hdr.length != 4 => {
                return Err(H2Error::connection(
                    H2ErrorCode::FrameSizeError,
                    "WINDOW_UPDATE frame length must be 4",
                ));
            }
            _ => {
                // Unknown extension frame type or valid header: allowed per RFC 9113 §5.5.
            }
        }
        Ok(())
    }

    fn process_input_frame<'b>(
        &mut self,
        hdr: FrameHeader,
        payload: &'b [u8],
    ) -> Result<StepOutcome<'b>, H2Error> {
        match hdr.frame_type {
            FrameType::SETTINGS => {
                let ev = self.handle_settings(hdr, payload)?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::PING => {
                let ev = self.handle_ping(hdr, payload)?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::PRIORITY => Ok(StepOutcome::Continue),
            FrameType::WINDOW_UPDATE => {
                let ev = self.handle_window_update(hdr, payload)?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::RST_STREAM => {
                let ev = self.handle_rst_stream(hdr, payload)?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::GOAWAY => {
                let (last_stream_id, error_code) = self.parse_goaway(payload);
                Ok(StepOutcome::EventFromInput(H2Event::GoAway {
                    last_stream_id,
                    error_code,
                    debug_data: &payload[8..],
                }))
            }
            FrameType::HEADERS => {
                let (start, end) = Self::extract_headers_fragment(hdr.flags, payload)?;
                let fragment = &payload[start..end];
                if let Some(end_stream) = self.begin_headers_block(hdr, fragment)? {
                    Ok(StepOutcome::EventFromInput(H2Event::Headers {
                        stream_id: hdr.stream_id,
                        header_block: fragment,
                        end_stream,
                    }))
                } else {
                    Ok(StepOutcome::Continue)
                }
            }
            FrameType::CONTINUATION => self.handle_continuation(hdr, payload),
            _ => Ok(StepOutcome::Continue),
        }
    }

    fn process_partial_frame(
        &mut self,
        hdr: FrameHeader,
        frame_len: usize,
    ) -> Result<StepOutcome<'static>, H2Error> {
        let mut local_copy = [0u8; 8];
        match hdr.frame_type {
            FrameType::SETTINGS => {
                let mut settings = self.peer_settings;
                if (hdr.flags & FLAG_ACK) != 0 {
                    self.peer_acked_settings = true;
                    Ok(StepOutcome::EventFromInput(H2Event::SettingsAck))
                } else {
                    let peer_is_server = self.role == Role::Client;
                    let delta = settings
                        .apply_payload(&self.partial_payload_buf[..frame_len], peer_is_server)?;
                    self.peer_settings = settings;
                    self.peer_initial_settings_received = true;
                    self.adjust_streams_send_window(delta)?;
                    self.queue_settings_ack()?;
                    Ok(StepOutcome::EventFromInput(H2Event::Settings(
                        self.peer_settings,
                    )))
                }
            }
            FrameType::PING => {
                local_copy.copy_from_slice(&self.partial_payload_buf[..8]);
                let ev = self.handle_ping(hdr, &local_copy)?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::PRIORITY => Ok(StepOutcome::Continue),
            FrameType::WINDOW_UPDATE => {
                local_copy[..4].copy_from_slice(&self.partial_payload_buf[..4]);
                let ev = self.handle_window_update(hdr, &local_copy[..4])?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::RST_STREAM => {
                local_copy[..4].copy_from_slice(&self.partial_payload_buf[..4]);
                let ev = self.handle_rst_stream(hdr, &local_copy[..4])?;
                Ok(StepOutcome::EventFromInput(ev))
            }
            FrameType::GOAWAY => {
                local_copy.copy_from_slice(&self.partial_payload_buf[..8]);
                let (last_stream_id, error_code) = self.parse_goaway(&local_copy);
                Ok(StepOutcome::GoAwayFromPartial {
                    last_stream_id,
                    error_code,
                    start: 8,
                    end: frame_len,
                })
            }
            FrameType::HEADERS => {
                let (start, end) = Self::extract_headers_fragment(
                    hdr.flags,
                    &self.partial_payload_buf[..frame_len],
                )?;
                let end_stream = (hdr.flags & FLAG_END_STREAM) != 0;
                let end_headers = (hdr.flags & FLAG_END_HEADERS) != 0;
                let stream_idx = match self.open_or_validate_headers_stream(hdr.stream_id, end_stream) {
                    Ok(idx) => idx,
                    Err(err) => {
                        if err.is_stream_error() && !end_headers {
                            self.header_reasm_len = 0;
                            self.continuation_state = Some(ContinuationState {
                                stream_id: hdr.stream_id,
                                end_stream,
                                discarding: true,
                            });
                        }
                        return Err(err);
                    }
                };
                if end_headers {
                    if end_stream {
                        self.transition_remote_end_stream(stream_idx);
                    }
                    Ok(StepOutcome::HeadersFromPartial {
                        stream_id: hdr.stream_id,
                        start,
                        end,
                        end_stream,
                    })
                } else {
                    let frag_len = end - start;
                    self.header_reasm_buf[..frag_len]
                        .copy_from_slice(&self.partial_payload_buf[start..end]);
                    self.header_reasm_len = frag_len;
                    self.continuation_state = Some(ContinuationState {
                        stream_id: hdr.stream_id,
                        end_stream,
                        discarding: false,
                    });
                    Ok(StepOutcome::Continue)
                }
            }
            FrameType::CONTINUATION => {
                let cont = self.continuation_state.ok_or_else(|| {
                    H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "unexpected CONTINUATION frame",
                    )
                })?;
                if cont.discarding {
                    if (hdr.flags & FLAG_END_HEADERS) != 0 {
                        self.continuation_state = None;
                    }
                    return Ok(StepOutcome::Continue);
                }
                if self.header_reasm_len + frame_len > REASM_CAP {
                    return Err(H2Error::connection(
                        H2ErrorCode::CompressionError,
                        "CONTINUATION header block exceeds reassembly capacity",
                    ));
                }
                self.header_reasm_buf[self.header_reasm_len..self.header_reasm_len + frame_len]
                    .copy_from_slice(&self.partial_payload_buf[..frame_len]);
                self.header_reasm_len += frame_len;
                self.finish_continuation_if_end_headers(hdr.flags, cont)
            }
            _ => Ok(StepOutcome::Continue),
        }
    }

    fn begin_headers_block(
        &mut self,
        hdr: FrameHeader,
        fragment: &[u8],
    ) -> Result<Option<bool>, H2Error> {
        let end_stream = (hdr.flags & FLAG_END_STREAM) != 0;
        let end_headers = (hdr.flags & FLAG_END_HEADERS) != 0;
        let stream_idx = match self.open_or_validate_headers_stream(hdr.stream_id, end_stream) {
            Ok(idx) => idx,
            Err(err) => {
                if err.is_stream_error() && !end_headers {
                    self.header_reasm_len = 0;
                    self.continuation_state = Some(ContinuationState {
                        stream_id: hdr.stream_id,
                        end_stream,
                        discarding: true,
                    });
                }
                return Err(err);
            }
        };
        if end_headers {
            if end_stream {
                self.transition_remote_end_stream(stream_idx);
            }
            Ok(Some(end_stream))
        } else {
            if fragment.len() > REASM_CAP {
                return Err(H2Error::connection(
                    H2ErrorCode::CompressionError,
                    "HEADERS fragment exceeds header reassembly capacity",
                ));
            }
            self.header_reasm_buf[..fragment.len()].copy_from_slice(fragment);
            self.header_reasm_len = fragment.len();
            self.continuation_state = Some(ContinuationState {
                stream_id: hdr.stream_id,
                end_stream,
                discarding: false,
            });
            Ok(None)
        }
    }

    fn finish_continuation_if_end_headers<'b>(
        &mut self,
        flags: u8,
        cont: ContinuationState,
    ) -> Result<StepOutcome<'b>, H2Error> {
        if (flags & FLAG_END_HEADERS) != 0 {
            self.continuation_state = None;
            let len = self.header_reasm_len;
            self.header_reasm_len = 0;
            if cont.end_stream {
                if let Some(idx) = self.find_stream_idx(cont.stream_id) {
                    self.transition_remote_end_stream(idx);
                }
            }
            Ok(StepOutcome::HeadersFromReasm {
                stream_id: cont.stream_id,
                len,
                end_stream: cont.end_stream,
            })
        } else {
            Ok(StepOutcome::Continue)
        }
    }

    fn handle_settings(
        &mut self,
        hdr: FrameHeader,
        payload: &[u8],
    ) -> Result<H2Event<'static>, H2Error> {
        if (hdr.flags & FLAG_ACK) != 0 {
            self.peer_acked_settings = true;
            return Ok(H2Event::SettingsAck);
        }
        let peer_is_server = self.role == Role::Client;
        let delta = self.peer_settings.apply_payload(payload, peer_is_server)?;
        self.peer_initial_settings_received = true;
        self.adjust_streams_send_window(delta)?;
        self.queue_settings_ack()?;
        Ok(H2Event::Settings(self.peer_settings))
    }

    fn adjust_streams_send_window(&mut self, delta: i64) -> Result<(), H2Error> {
        if delta == 0 {
            return Ok(());
        }
        let mut i = 0;
        while i < MAX_STREAMS {
            if self.streams[i].state.is_active() {
                let new_win = self.streams[i].send_window + delta;
                if new_win > MAX_FLOW_CONTROL_WINDOW as i64 {
                    return Err(H2Error::connection(
                        H2ErrorCode::FlowControlError,
                        "SETTINGS_INITIAL_WINDOW_SIZE change overflowed stream send window",
                    ));
                }
                self.streams[i].send_window = new_win;
            }
            i += 1;
        }
        Ok(())
    }

    fn handle_ping(
        &mut self,
        hdr: FrameHeader,
        payload: &[u8],
    ) -> Result<H2Event<'static>, H2Error> {
        let data = [
            payload[0], payload[1], payload[2], payload[3], payload[4], payload[5], payload[6],
            payload[7],
        ];
        let ack = (hdr.flags & FLAG_ACK) != 0;
        if !ack {
            self.queue_ping(true, data)?;
        }
        Ok(H2Event::Ping { ack, data })
    }

    fn handle_window_update(
        &mut self,
        hdr: FrameHeader,
        payload: &[u8],
    ) -> Result<H2Event<'static>, H2Error> {
        let increment =
            u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) & MAX_STREAM_ID;
        if increment == 0 {
            if hdr.stream_id == 0 {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "WINDOW_UPDATE with increment 0 on stream 0",
                ));
            }
            return Err(H2Error::stream(
                hdr.stream_id,
                H2ErrorCode::ProtocolError,
                "WINDOW_UPDATE with increment 0 on stream",
            ));
        }
        let inc_i64 = increment as i64;
        if hdr.stream_id == 0 {
            let new_win = self.conn_send_window + inc_i64;
            if new_win > MAX_FLOW_CONTROL_WINDOW as i64 {
                return Err(H2Error::connection(
                    H2ErrorCode::FlowControlError,
                    "connection send flow-control window overflow",
                ));
            }
            self.conn_send_window = new_win;
        } else {
            if !self.was_stream_opened(hdr.stream_id) {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "WINDOW_UPDATE received on idle stream",
                ));
            }
            if let Some(idx) = self.find_stream_idx(hdr.stream_id) {
                if self.streams[idx].state.is_active() {
                    let new_win = self.streams[idx].send_window + inc_i64;
                    if new_win > MAX_FLOW_CONTROL_WINDOW as i64 {
                        return Err(H2Error::stream(
                            hdr.stream_id,
                            H2ErrorCode::FlowControlError,
                            "stream send flow-control window overflow",
                        ));
                    }
                    self.streams[idx].send_window = new_win;
                }
            }
        }
        Ok(H2Event::WindowUpdate {
            stream_id: hdr.stream_id,
            increment,
        })
    }

    fn handle_rst_stream(
        &mut self,
        hdr: FrameHeader,
        payload: &[u8],
    ) -> Result<H2Event<'static>, H2Error> {
        if !self.was_stream_opened(hdr.stream_id) {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "RST_STREAM received on idle stream",
            ));
        }
        let raw_code = u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]);
        let error_code = H2ErrorCode::from_u32(raw_code);
        if let Some(idx) = self.find_stream_idx(hdr.stream_id) {
            self.streams[idx].state = StreamState::Closed;
            self.streams[idx].reasm_len = 0;
        }
        Ok(H2Event::RstStream {
            stream_id: hdr.stream_id,
            error_code,
        })
    }

    fn parse_goaway(&mut self, payload: &[u8]) -> (u32, H2ErrorCode) {
        let last_stream_id =
            u32::from_be_bytes([payload[0], payload[1], payload[2], payload[3]]) & MAX_STREAM_ID;
        let raw_code = u32::from_be_bytes([payload[4], payload[5], payload[6], payload[7]]);
        let error_code = H2ErrorCode::from_u32(raw_code);
        self.goaway_received = Some((last_stream_id, error_code));
        (last_stream_id, error_code)
    }

    fn extract_headers_fragment(flags: u8, payload: &[u8]) -> Result<(usize, usize), H2Error> {
        let mut start = 0usize;
        let mut end = payload.len();
        if (flags & FLAG_PADDED) != 0 {
            if payload.is_empty() {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "PADDED HEADERS frame has empty payload",
                ));
            }
            let pad_len = payload[0] as usize;
            start += 1;
            if start + pad_len > end {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "HEADERS frame padding length exceeds payload length",
                ));
            }
            end -= pad_len;
        }
        if (flags & FLAG_PRIORITY) != 0 {
            if end - start < 5 {
                return Err(H2Error::connection(
                    H2ErrorCode::FrameSizeError,
                    "HEADERS frame with PRIORITY flag shorter than 5 bytes",
                ));
            }
            start += 5;
        }
        Ok((start, end))
    }

    fn validate_existing_stream_for_headers(
        &mut self,
        idx: usize,
        end_stream: bool,
    ) -> Result<usize, H2Error> {
        match self.streams[idx].state {
            StreamState::Open | StreamState::HalfClosedLocal => {
                if self.streams[idx].headers_received && !end_stream {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "trailing HEADERS frame must have END_STREAM set",
                    ));
                }
                self.streams[idx].headers_received = true;
                Ok(idx)
            }
            StreamState::HalfClosedRemote | StreamState::Closed => Err(H2Error::stream(
                self.streams[idx].stream_id,
                H2ErrorCode::StreamClosed,
                "HEADERS received on remote-closed stream",
            )),
            StreamState::Idle => Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "HEADERS received on idle stream",
            )),
        }
    }

    fn open_or_validate_headers_stream(
        &mut self,
        stream_id: u32,
        end_stream: bool,
    ) -> Result<usize, H2Error> {
        match self.role {
            Role::Server => {
                if stream_id.is_multiple_of(2) {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "client-initiated stream ID must be odd",
                    ));
                }
                if let Some(idx) = self.find_stream_idx(stream_id) {
                    return self.validate_existing_stream_for_headers(idx, end_stream);
                }
                if stream_id <= self.last_peer_stream_id {
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "new client stream ID must increase monotonically",
                    ));
                }
                self.last_peer_stream_id = stream_id;
                if self.active_stream_count()
                    >= (self.local_settings.max_concurrent_streams as usize).min(MAX_STREAMS)
                {
                    return Err(H2Error::stream(
                        stream_id,
                        H2ErrorCode::RefusedStream,
                        "max concurrent streams exceeded",
                    ));
                }
                let idx = self.alloc_stream_slot(stream_id)?;
                self.streams[idx].reset(
                    stream_id,
                    StreamState::Open,
                    self.peer_settings.initial_window_size as i64,
                    self.local_settings.initial_window_size as i64,
                );
                self.streams[idx].headers_received = true;
                Ok(idx)
            }
            Role::Client => {
                let Some(idx) = self.find_stream_idx(stream_id) else {
                    if self.was_stream_opened(stream_id) {
                        return Err(H2Error::stream(
                            stream_id,
                            H2ErrorCode::StreamClosed,
                            "HEADERS received on closed client stream",
                        ));
                    }
                    return Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "HEADERS received on idle stream",
                    ));
                };
                self.validate_existing_stream_for_headers(idx, end_stream)
            }
        }
    }

    fn handle_continuation<'b>(
        &mut self,
        hdr: FrameHeader,
        payload: &'b [u8],
    ) -> Result<StepOutcome<'b>, H2Error> {
        let cont = self.continuation_state.ok_or_else(|| {
            H2Error::connection(
                H2ErrorCode::ProtocolError,
                "unexpected CONTINUATION frame",
            )
        })?;
        if cont.discarding {
            if (hdr.flags & FLAG_END_HEADERS) != 0 {
                self.continuation_state = None;
            }
            return Ok(StepOutcome::Continue);
        }
        if self.header_reasm_len + payload.len() > REASM_CAP {
            return Err(H2Error::connection(
                H2ErrorCode::CompressionError,
                "CONTINUATION header block exceeds reassembly capacity",
            ));
        }
        self.header_reasm_buf[self.header_reasm_len..self.header_reasm_len + payload.len()]
            .copy_from_slice(payload);
        self.header_reasm_len += payload.len();
        self.finish_continuation_if_end_headers(hdr.flags, cont)
    }

    // ========================================================================
    // Streaming DATA Frame Processing (Zero-Copy Fast Path + LPM Reassembly)
    // ========================================================================

    fn validate_and_account_data_stream(&mut self, hdr: FrameHeader) -> Result<usize, H2Error> {
        if !self.was_stream_opened(hdr.stream_id) {
            return Err(H2Error::connection(
                H2ErrorCode::ProtocolError,
                "DATA frame received on idle stream",
            ));
        }
        let idx_opt = self.find_stream_idx(hdr.stream_id);
        if let Some(idx) = idx_opt {
            if matches!(
                self.streams[idx].state,
                StreamState::Open | StreamState::HalfClosedLocal
            ) && !self.streams[idx].headers_received
            {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "DATA frame received before initial HEADERS",
                ));
            }
        }

        // Flow control accounting (counts entire DATA payload including padding per RFC 9113 §6.9.1)
        let raw_len = hdr.length as i64;
        if raw_len > self.conn_recv_window {
            return Err(H2Error::connection(
                H2ErrorCode::FlowControlError,
                "connection receive flow-control window exceeded",
            ));
        }
        self.conn_recv_window -= raw_len;

        let Some(idx) = idx_opt else {
            if self.auto_window_update && hdr.length > 0 {
                self.replenish_recv_window(0, hdr.length)?;
            }
            self.discard_remaining = hdr.length;
            return Err(H2Error::stream(
                hdr.stream_id,
                H2ErrorCode::StreamClosed,
                "DATA frame received on closed stream",
            ));
        };
        if !matches!(
            self.streams[idx].state,
            StreamState::Open | StreamState::HalfClosedLocal
        ) {
            if self.auto_window_update && hdr.length > 0 {
                self.replenish_recv_window(0, hdr.length)?;
            }
            self.discard_remaining = hdr.length;
            return Err(H2Error::stream(
                hdr.stream_id,
                H2ErrorCode::StreamClosed,
                "DATA frame received on remote-closed stream",
            ));
        }

        if raw_len > self.streams[idx].recv_window {
            if self.auto_window_update && hdr.length > 0 {
                self.replenish_recv_window(0, hdr.length)?;
            }
            self.discard_remaining = hdr.length;
            return Err(H2Error::stream(
                hdr.stream_id,
                H2ErrorCode::FlowControlError,
                "stream receive flow-control window exceeded",
            ));
        }
        self.streams[idx].recv_window -= raw_len;

        if self.auto_window_update && hdr.length > 0 {
            self.replenish_recv_window(hdr.stream_id, hdr.length)?;
        }
        Ok(idx)
    }

    fn begin_data_frame(&mut self, hdr: FrameHeader) -> Result<StepOutcome<'static>, H2Error> {
        let idx = self.validate_and_account_data_stream(hdr)?;
        let padded = (hdr.flags & FLAG_PADDED) != 0;
        let end_stream = (hdr.flags & FLAG_END_STREAM) != 0;
        let raw_len = hdr.length as usize;

        if raw_len == 0 {
            if padded {
                return Err(H2Error::connection(
                    H2ErrorCode::ProtocolError,
                    "PADDED DATA frame has empty payload",
                ));
            }
            if self.streams[idx].reasm_len > 0 {
                if end_stream {
                    self.streams[idx].reasm_len = 0;
                    return Err(H2Error::stream(
                        hdr.stream_id,
                        H2ErrorCode::ProtocolError,
                        "truncated gRPC frame at end of stream",
                    ));
                }
                return Ok(StepOutcome::Continue);
            }
            if end_stream {
                self.transition_remote_end_stream(idx);
                return Ok(StepOutcome::EventFromInput(H2Event::EndStream {
                    stream_id: hdr.stream_id,
                }));
            }
            return Ok(StepOutcome::Continue);
        }

        self.active_data = Some(ActiveDataFrame {
            stream_id: hdr.stream_id,
            stream_idx: idx,
            end_stream,
            need_pad_len: padded,
            raw_remaining: raw_len,
            data_remaining: if padded { 0 } else { raw_len },
            pad_remaining: 0,
        });
        Ok(StepOutcome::Continue)
    }

    fn parse_lpm_header(&self, stream_id: u32, hdr_bytes: &[u8]) -> Result<(bool, usize), H2Error> {
        let compressed_byte = hdr_bytes[0];
        if compressed_byte > 1 {
            return Err(H2Error::from_grpc_status(
                stream_id,
                GrpcStatus::internal("invalid gRPC compression flag"),
            ));
        }
        let msg_len =
            u32::from_be_bytes([hdr_bytes[1], hdr_bytes[2], hdr_bytes[3], hdr_bytes[4]]) as usize;
        if msg_len > self.max_message_size {
            return Err(H2Error::from_grpc_status(
                stream_id,
                GrpcStatus::resource_exhausted("gRPC message exceeds maximum size"),
            ));
        }
        Ok((compressed_byte == 1, msg_len))
    }

    fn abort_active_data(&mut self, ad: ActiveDataFrame, idx: usize) {
        self.discard_remaining = (ad.data_remaining + ad.pad_remaining) as u32;
        self.active_data = None;
        self.streams[idx].reasm_len = 0;
    }

    fn step_active_data<'b>(
        &mut self,
        mut rem: &'b [u8],
    ) -> (usize, Result<Option<StepOutcome<'b>>, H2Error>) {
        let Some(mut ad) = self.active_data else {
            return (0, Ok(Some(StepOutcome::Continue)));
        };
        let mut step_consumed = 0usize;
        let idx = ad.stream_idx;

        // 1. Parse the 1-byte pad length if `FLAG_PADDED` was set on this DATA frame.
        if ad.need_pad_len {
            if rem.is_empty() {
                return (0, Ok(None));
            }
            let pad_len = rem[0] as usize;
            if 1 + pad_len > ad.raw_remaining {
                self.active_data = None;
                return (
                    step_consumed,
                    Err(H2Error::connection(
                        H2ErrorCode::ProtocolError,
                        "DATA frame padding length exceeds payload length",
                    )),
                );
            }
            ad.need_pad_len = false;
            ad.data_remaining = ad.raw_remaining - 1 - pad_len;
            ad.pad_remaining = pad_len;
            step_consumed += 1;
            rem = &rem[1..];
            self.active_data = Some(ad);
        }

        // 2. Process unpadded DATA bytes.
        if ad.data_remaining > 0 {
            let avail_data = rem.len().min(ad.data_remaining);
            let mut data_chunk = &rem[..avail_data];

            // 2a. Zero-Copy Fast Path when no partial gRPC bytes are buffered on this stream.
            if self.streams[idx].reasm_len == 0 && data_chunk.len() >= GRPC_HEADER_LEN {
                let (compressed, msg_len) = match self.parse_lpm_header(ad.stream_id, data_chunk) {
                    Ok(v) => v,
                    Err(err) => {
                        self.abort_active_data(ad, idx);
                        return (step_consumed, Err(err));
                    }
                };
                let wire_msg_len = GRPC_HEADER_LEN + msg_len;
                let pad_needed_now = if ad.data_remaining == wire_msg_len {
                    ad.pad_remaining
                } else {
                    0
                };
                if data_chunk.len() >= wire_msg_len && rem.len() >= wire_msg_len + pad_needed_now {
                    let payload = &data_chunk[GRPC_HEADER_LEN..wire_msg_len];
                    ad.data_remaining -= wire_msg_len;
                    step_consumed += wire_msg_len;
                    if ad.data_remaining == 0 {
                        step_consumed += ad.pad_remaining;
                        ad.pad_remaining = 0;
                        self.active_data = None;
                        if ad.end_stream {
                            self.transition_remote_end_stream(idx);
                        }
                        return (
                            step_consumed,
                            Ok(Some(StepOutcome::EventFromInput(H2Event::GrpcMessage {
                                stream_id: ad.stream_id,
                                compressed,
                                payload,
                                end_stream: ad.end_stream,
                            }))),
                        );
                    }
                    self.active_data = Some(ad);
                    return (
                        step_consumed,
                        Ok(Some(StepOutcome::EventFromInput(H2Event::GrpcMessage {
                            stream_id: ad.stream_id,
                            compressed,
                            payload,
                            end_stream: false,
                        }))),
                    );
                }
            }

            // 2b. Reassembly Path: copy only up to the current message's boundary (`5` then `5 + msg_len`).
            if self.streams[idx].reasm_len < GRPC_HEADER_LEN {
                if data_chunk.is_empty() {
                    self.active_data = Some(ad);
                    return (step_consumed, Ok(None));
                }
                let need_hdr = GRPC_HEADER_LEN - self.streams[idx].reasm_len;
                let take_hdr = data_chunk.len().min(need_hdr);
                if let Err(err) = self.streams[idx].append_reasm(&data_chunk[..take_hdr]) {
                    self.abort_active_data(ad, idx);
                    return (step_consumed, Err(err));
                }
                ad.data_remaining -= take_hdr;
                step_consumed += take_hdr;
                rem = &rem[take_hdr..];
                data_chunk = &data_chunk[take_hdr..];
            }

            if self.streams[idx].reasm_len >= GRPC_HEADER_LEN {
                let hdr_copy = [
                    self.streams[idx].reasm_buf[0],
                    self.streams[idx].reasm_buf[1],
                    self.streams[idx].reasm_buf[2],
                    self.streams[idx].reasm_buf[3],
                    self.streams[idx].reasm_buf[4],
                ];
                let (compressed, msg_len) = match self.parse_lpm_header(ad.stream_id, &hdr_copy) {
                    Ok(v) => v,
                    Err(err) => {
                        self.abort_active_data(ad, idx);
                        return (step_consumed, Err(err));
                    }
                };
                let target_len = GRPC_HEADER_LEN + msg_len;
                if target_len > REASM_CAP {
                    self.abort_active_data(ad, idx);
                    return (
                        step_consumed,
                        Err(H2Error::stream(
                            ad.stream_id,
                            H2ErrorCode::EnhanceYourCalm,
                            "stream gRPC reassembly buffer overflow",
                        )),
                    );
                }
                let need_body = target_len - self.streams[idx].reasm_len;
                let take_body = data_chunk.len().min(need_body);
                if let Err(err) = self.streams[idx].append_reasm(&data_chunk[..take_body]) {
                    self.abort_active_data(ad, idx);
                    return (step_consumed, Err(err));
                }
                ad.data_remaining -= take_body;
                step_consumed += take_body;
                rem = &rem[take_body..];

                if self.streams[idx].reasm_len == target_len {
                    // Message is complete in `reasm_buf`. If more data remains in this DATA frame, emit immediately.
                    if ad.data_remaining > 0 {
                        self.active_data = Some(ad);
                        self.streams[idx].reasm_len = 0;
                        return (
                            step_consumed,
                            Ok(Some(StepOutcome::GrpcFromStreamReasm {
                                stream_idx: idx,
                                stream_id: ad.stream_id,
                                compressed,
                                start: GRPC_HEADER_LEN,
                                end: target_len,
                                end_stream: false,
                            })),
                        );
                    }
                    // Otherwise fall through to Step 3 to drain any trailing padding bytes first.
                }
            }
        }

        // 3. If `ad.data_remaining == 0`, drain any trailing padding bytes.
        if ad.data_remaining == 0 {
            let take_pad = rem.len().min(ad.pad_remaining);
            ad.pad_remaining -= take_pad;
            step_consumed += take_pad;
            if ad.pad_remaining > 0 {
                self.active_data = Some(ad);
                return (step_consumed, Ok(None));
            }

            // Entire DATA frame (including any padding) has now been consumed.
            self.active_data = None;
            let reasm_len = self.streams[idx].reasm_len;
            if reasm_len >= GRPC_HEADER_LEN {
                let hdr_copy = [
                    self.streams[idx].reasm_buf[0],
                    self.streams[idx].reasm_buf[1],
                    self.streams[idx].reasm_buf[2],
                    self.streams[idx].reasm_buf[3],
                    self.streams[idx].reasm_buf[4],
                ];
                let (compressed, msg_len) = match self.parse_lpm_header(ad.stream_id, &hdr_copy) {
                    Ok(v) => v,
                    Err(err) => {
                        self.streams[idx].reasm_len = 0;
                        return (step_consumed, Err(err));
                    }
                };
                let target_len = GRPC_HEADER_LEN + msg_len;
                if reasm_len == target_len {
                    self.streams[idx].reasm_len = 0;
                    if ad.end_stream {
                        self.transition_remote_end_stream(idx);
                    }
                    return (
                        step_consumed,
                        Ok(Some(StepOutcome::GrpcFromStreamReasm {
                            stream_idx: idx,
                            stream_id: ad.stream_id,
                            compressed,
                            start: GRPC_HEADER_LEN,
                            end: target_len,
                            end_stream: ad.end_stream,
                        })),
                    );
                }
            }

            if reasm_len > 0 {
                if ad.end_stream {
                    self.streams[idx].reasm_len = 0;
                    return (
                        step_consumed,
                        Err(H2Error::stream(
                            ad.stream_id,
                            H2ErrorCode::ProtocolError,
                            "truncated gRPC frame at end of stream",
                        )),
                    );
                }
                return (step_consumed, Ok(Some(StepOutcome::Continue)));
            }

            if ad.end_stream {
                self.transition_remote_end_stream(idx);
                return (
                    step_consumed,
                    Ok(Some(StepOutcome::EventFromInput(H2Event::EndStream {
                        stream_id: ad.stream_id,
                    }))),
                );
            }
            return (step_consumed, Ok(Some(StepOutcome::Continue)));
        }

        // `ad.data_remaining > 0` and `rem` is exhausted for this call.
        self.active_data = Some(ad);
        (step_consumed, Ok(None))
    }

    // ========================================================================
    // Internal Stream Slot Management
    // ========================================================================

    fn find_stream_idx(&self, stream_id: u32) -> Option<usize> {
        let mut i = 0;
        while i < MAX_STREAMS {
            if self.streams[i].stream_id == stream_id && self.streams[i].state != StreamState::Idle
            {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    fn was_stream_opened(&self, stream_id: u32) -> bool {
        if stream_id == 0 {
            return false;
        }
        if self.find_stream_idx(stream_id).is_some() {
            return true;
        }
        let is_client_initiated = (stream_id & 1) == 1;
        match self.role {
            Role::Server => is_client_initiated && stream_id <= self.last_peer_stream_id,
            Role::Client => is_client_initiated && stream_id < self.next_local_stream_id,
        }
    }

    fn alloc_stream_slot(&self, _stream_id: u32) -> Result<usize, H2Error> {
        // Prefer an Idle slot first
        let mut i = 0;
        while i < MAX_STREAMS {
            if self.streams[i].state == StreamState::Idle {
                return Ok(i);
            }
            i += 1;
        }
        // Otherwise reuse a Closed slot
        let mut i = 0;
        while i < MAX_STREAMS {
            if self.streams[i].state == StreamState::Closed {
                return Ok(i);
            }
            i += 1;
        }
        Err(H2Error::connection(
            H2ErrorCode::RefusedStream,
            "all stream slots are active",
        ))
    }

    fn transition_remote_end_stream(&mut self, idx: usize) {
        self.streams[idx].state = match self.streams[idx].state {
            StreamState::Open => StreamState::HalfClosedRemote,
            StreamState::HalfClosedLocal => StreamState::Closed,
            other => other,
        };
    }

    fn transition_local_end_stream(&mut self, idx: usize) {
        self.streams[idx].state = match self.streams[idx].state {
            StreamState::Open => StreamState::HalfClosedLocal,
            StreamState::HalfClosedRemote => StreamState::Closed,
            other => other,
        };
    }
}
