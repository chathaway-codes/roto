//! Zero-allocation `#![no_std]` HTTP/1.1 wire codec, URL path and query parameter
//! Protobuf binding helpers, and [`HttpService`] / [`AsyncHttpService`] dispatch traits.

use core::fmt;
use core::iter::FusedIterator;

use roto_runtime::{BackwardWriter, DecodeError, EncodeError, FieldIter, WireType};

use crate::framing::take_writer;
use crate::h2::CLIENT_CONNECTION_PREFACE;
use crate::status::{percent_decode, try_percent_decode_in_place, GrpcStatus};

/// Canonical `Content-Type` header value for binary Protocol Buffers HTTP payloads.
pub const CONTENT_TYPE_PROTOBUF: &str = "application/x-protobuf";

/// Canonical `Content-Type` header value for plain-text error responses.
pub const CONTENT_TYPE_TEXT_PLAIN: &str = "text/plain; charset=utf-8";

/// Returns the standard HTTP/1.1 reason phrase for `status_code`.
#[inline]
pub const fn http_reason_phrase(status_code: u16) -> &'static str {
    match status_code {
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        408 => "Request Timeout",
        409 => "Conflict",
        411 => "Length Required",
        413 => "Payload Too Large",
        415 => "Unsupported Media Type",
        429 => "Too Many Requests",
        499 => "Client Closed Request",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        _ => "Unknown",
    }
}

/// Returns `true` if `b` is a valid HTTP token character (`tchar`) per RFC 9110 §5.6.2.
#[inline]
const fn is_http_tchar(b: u8) -> bool {
    matches!(
        b,
        b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.'
            | b'^' | b'_' | b'`' | b'|' | b'~'
            | b'0'..=b'9'
            | b'A'..=b'Z'
            | b'a'..=b'z'
    )
}

/// Zero-copy HTTP request method corresponding to `google.api.HttpRule.pattern`.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HttpMethod<'a> {
    Get,
    Put,
    Post,
    Delete,
    Patch,
    Custom(&'a str),
}

impl<'a> HttpMethod<'a> {
    /// Parses an HTTP method from raw wire bytes, validating that all bytes are valid
    /// RFC 9110 HTTP token characters (`tchar`).
    #[inline]
    pub fn from_bytes(bytes: &'a [u8]) -> Option<Self> {
        if bytes.is_empty() || !bytes.iter().all(|&b| is_http_tchar(b)) {
            return None;
        }
        let s = core::str::from_utf8(bytes).ok()?;
        Some(Self::from_str(s))
    }

    /// Classifies a borrowed method string into a standard variant or [`HttpMethod::Custom`].
    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &'a str) -> Self {
        match s {
            "GET" => Self::Get,
            "PUT" => Self::Put,
            "POST" => Self::Post,
            "DELETE" => Self::Delete,
            "PATCH" => Self::Patch,
            other => Self::Custom(other),
        }
    }

    /// Returns the uppercase string representation of this method.
    #[inline]
    pub const fn as_str(&self) -> &'a str {
        match *self {
            Self::Get => "GET",
            Self::Put => "PUT",
            Self::Post => "POST",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Custom(s) => s,
        }
    }

    /// Returns `true` if this method matches `expected` (ASCII case-insensitively).
    #[inline]
    pub fn matches(&self, expected: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(expected)
    }
}

impl fmt::Display for HttpMethod<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Zero-copy view of an incoming HTTP request.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpRequest<'a> {
    pub method: HttpMethod<'a>,
    pub path: &'a str,
    pub query: &'a str,
    pub content_type: Option<&'a str>,
    pub body: &'a [u8],
}

impl<'a> HttpRequest<'a> {
    /// Creates a new [`HttpRequest`] from pre-split `path` and `query` components.
    #[inline]
    pub const fn new(
        method: HttpMethod<'a>,
        path: &'a str,
        query: &'a str,
        content_type: Option<&'a str>,
        body: &'a [u8],
    ) -> Self {
        Self {
            method,
            path,
            query,
            content_type,
            body,
        }
    }

    /// Creates a new [`HttpRequest`] by splitting `uri` at the first `'?'` into `path` and `query`.
    #[inline]
    pub fn from_uri(
        method: HttpMethod<'a>,
        uri: &'a str,
        content_type: Option<&'a str>,
        body: &'a [u8],
    ) -> Self {
        let (path, query) = split_uri_path_and_query(uri);
        Self {
            method,
            path,
            query,
            content_type,
            body,
        }
    }
}

/// Zero-copy view of an outgoing HTTP response whose `body` slice is borrowed from
/// the caller's response buffer.
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HttpResponse<'a> {
    pub status_code: u16,
    pub content_type: &'static str,
    pub body: &'a [u8],
}

impl<'a> HttpResponse<'a> {
    /// Creates a new [`HttpResponse`].
    #[inline]
    pub const fn new(status_code: u16, content_type: &'static str, body: &'a [u8]) -> Self {
        Self {
            status_code,
            content_type,
            body,
        }
    }

    /// Creates a `200 OK` [`HttpResponse`] with [`CONTENT_TYPE_PROTOBUF`].
    #[inline]
    pub const fn ok_protobuf(body: &'a [u8]) -> Self {
        Self::new(200, CONTENT_TYPE_PROTOBUF, body)
    }
}

/// Splits a request URI on the first `'?'` into `(path, query)`.
#[inline]
pub fn split_uri_path_and_query(uri: &str) -> (&str, &str) {
    match uri.split_once('?') {
        Some((path, query)) => (path, query),
        None => (uri, ""),
    }
}

/// Splits a trailing `:verb` custom verb from the final segment of `path` per
/// the `google.api.http` path template grammar (`LITERAL:verb`).
///
/// Returns `(base_path, Some(verb))` if and only if the final path segment contains `':'`
/// with both a non-empty segment prefix before `':'` and a non-empty verb after `':'`.
/// Otherwise returns `(path, None)`.
#[inline]
pub fn split_custom_verb(path: &str) -> (&str, Option<&str>) {
    let last_slash = path.rfind('/');
    let seg_start = match last_slash {
        Some(idx) => idx + 1,
        None => 0,
    };
    let last_seg = &path[seg_start..];
    if let Some(colon_rel) = last_seg.rfind(':') {
        if colon_rel > 0 && colon_rel + 1 < last_seg.len() {
            let colon_abs = seg_start + colon_rel;
            return (&path[..colon_abs], Some(&path[colon_abs + 1..]));
        }
    }
    (path, None)
}

/// Percent-decodes a URL path segment or query string `input`, returning a borrowed `&'a str`
/// directly from `input` when no `'%'` escapes are present, or decoding into `scratch`.
pub fn percent_decode_str<'a>(
    input: &'a str,
    scratch: &'a mut [u8],
) -> Result<&'a str, GrpcStatus<'static>> {
    if !input.as_bytes().contains(&b'%') {
        return Ok(input);
    }
    percent_decode(input.as_bytes(), scratch).map_err(|err| match err {
        DecodeError::UnexpectedEof => {
            GrpcStatus::resource_exhausted("URL decode scratch buffer overflow")
        }
        _ => GrpcStatus::invalid_argument("invalid percent-encoded UTF-8 in URL parameter"),
    })
}

/// Percent-decodes raw bytes from `input`, returning a borrowed slice directly from `input`
/// when no `'%'` escapes are present, or decoding into `scratch` without requiring UTF-8.
pub fn percent_decode_bytes<'a>(
    input: &'a [u8],
    scratch: &'a mut [u8],
) -> Result<&'a [u8], GrpcStatus<'static>> {
    if !input.contains(&b'%') {
        return Ok(input);
    }
    let mut read_pos = 0;
    let mut write_pos = 0;
    while read_pos < input.len() {
        let b = input[read_pos];
        let decoded = if b == b'%' {
            if read_pos + 2 >= input.len() {
                return Err(GrpcStatus::invalid_argument(
                    "truncated percent-escape in URL parameter",
                ));
            }
            let hi = decode_hex(input[read_pos + 1]).ok_or_else(|| {
                GrpcStatus::invalid_argument("invalid hex digit in URL percent-escape")
            })?;
            let lo = decode_hex(input[read_pos + 2]).ok_or_else(|| {
                GrpcStatus::invalid_argument("invalid hex digit in URL percent-escape")
            })?;
            read_pos += 3;
            (hi << 4) | lo
        } else {
            read_pos += 1;
            b
        };
        if write_pos >= scratch.len() {
            return Err(GrpcStatus::resource_exhausted(
                "URL decode scratch buffer overflow",
            ));
        }
        scratch[write_pos] = decoded;
        write_pos += 1;
    }
    Ok(&scratch[..write_pos])
}

#[inline]
const fn decode_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

/// Zero-allocation cursor for matching URL paths against `google.api.http` path templates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathCursor<'a> {
    rest: &'a str,
    done: bool,
}

impl<'a> PathCursor<'a> {
    /// Creates a new [`PathCursor`] over `path` (which must begin with `'/'`).
    #[inline]
    pub fn new(path: &'a str) -> Option<Self> {
        let rest = path.strip_prefix('/')?;
        Some(Self {
            rest,
            done: rest.is_empty(),
        })
    }

    /// Creates a [`PathCursor`] over `path` only if `path` does not end with a `:verb` suffix.
    #[inline]
    pub fn without_custom_verb(path: &'a str) -> Option<Self> {
        let (base, verb) = split_custom_verb(path);
        if verb.is_some() {
            return None;
        }
        Self::new(base)
    }

    /// Creates a [`PathCursor`] over `path` after stripping and matching `:expected_verb`.
    #[inline]
    pub fn with_custom_verb(path: &'a str, expected_verb: &str) -> Option<Self> {
        let (base, verb) = split_custom_verb(path);
        if verb != Some(expected_verb) {
            return None;
        }
        Self::new(base)
    }

    /// Returns the unconsumed tail of the path.
    #[inline]
    pub const fn remaining(&self) -> &'a str {
        self.rest
    }

    /// Returns `true` if all path segments have been consumed.
    #[inline]
    pub const fn is_end(&self) -> bool {
        self.done
    }

    /// Consumes and returns the next non-empty path segment.
    ///
    /// Returns `None` if the cursor is at the end or encounters an empty segment (`"//"` or trailing `'/'`).
    #[inline]
    pub fn next_segment(&mut self) -> Option<&'a str> {
        if self.done {
            return None;
        }
        if let Some((seg, tail)) = self.rest.split_once('/') {
            if seg.is_empty() {
                return None;
            }
            self.rest = tail;
            Some(seg)
        } else {
            let seg = self.rest;
            if seg.is_empty() {
                return None;
            }
            self.rest = "";
            self.done = true;
            Some(seg)
        }
    }

    /// Consumes the next segment and returns `true` iff it equals `expected`.
    #[inline]
    pub fn literal(&mut self, expected: &str) -> bool {
        self.next_segment() == Some(expected)
    }

    /// Captures a single non-empty segment (`*`).
    #[inline]
    pub fn capture_single(&mut self) -> Option<&'a str> {
        self.next_segment()
    }

    /// Captures exactly `count` consecutive non-empty segments (`count >= 1`) and returns
    /// the borrowed slice spanning all of them (including `'/'` separators).
    pub fn capture_segments(&mut self, count: usize) -> Option<&'a str> {
        if count == 0 {
            return Some("");
        }
        let start = self.rest;
        for _ in 0..count {
            self.next_segment()?;
        }
        Some(self.slice_since(start))
    }

    /// Captures all remaining segments (`**` at the end of a template), requiring at least
    /// one non-empty segment and rejecting empty segments (`"//"` or trailing `'/'`).
    #[inline]
    pub fn capture_remaining(&mut self) -> Option<&'a str> {
        self.capture_until_remaining_segments(0)
    }

    /// Captures one or more segments for a `**` wildcard while leaving `trailing_segments`
    /// segments unconsumed for subsequent suffix matching.
    pub fn capture_until_remaining_segments(
        &mut self,
        trailing_segments: usize,
    ) -> Option<&'a str> {
        if self.done || self.rest.is_empty() {
            return None;
        }
        let mut total = 0usize;
        for seg in self.rest.split('/') {
            if seg.is_empty() {
                return None;
            }
            total += 1;
        }
        if total < 1 + trailing_segments {
            return None;
        }
        let capture_count = total - trailing_segments;
        self.capture_segments(capture_count)
    }

    /// Returns the slice of `start_rest` (previously obtained via [`Self::remaining`])
    /// consumed up to the current cursor position.
    #[inline]
    pub fn slice_since(&self, start_rest: &'a str) -> &'a str {
        if self.done {
            start_rest
        } else {
            let consumed_len = start_rest.len() - self.rest.len() - 1;
            &start_rest[..consumed_len]
        }
    }
}

/// Zero-allocation iterator over URL query string `(key, value)` pairs (`k1=v1&k2=v2`).
#[must_use]
#[derive(Clone, Copy, Debug)]
pub struct QueryIter<'a> {
    rest: &'a str,
}

impl<'a> QueryIter<'a> {
    /// Creates a new [`QueryIter`] over `query` (stripping a single leading `'?'` if present).
    #[inline]
    pub fn new(query: &'a str) -> Self {
        let rest = query.strip_prefix('?').unwrap_or(query);
        Self { rest }
    }
}

impl<'a> Iterator for QueryIter<'a> {
    type Item = (&'a str, &'a str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while !self.rest.is_empty() {
            let (pair, tail) = match self.rest.split_once('&') {
                Some((p, t)) => (p, t),
                None => (self.rest, ""),
            };
            self.rest = tail;
            if pair.is_empty() {
                continue;
            }
            let (k, v) = match pair.split_once('=') {
                Some((k, v)) => (k, v),
                None => (pair, ""),
            };
            if !k.is_empty() {
                return Some((k, v));
            }
        }
        None
    }
}

impl FusedIterator for QueryIter<'_> {}

/// Returns a [`QueryIter`] yielding `(key, value)` pairs from `query`.
#[inline]
pub fn query_iter(query: &str) -> QueryIter<'_> {
    QueryIter::new(query)
}

/// Parses an ASCII boolean parameter (`"true"` / `"1"` -> `true`, `"false"` / `"0"` -> `false`).
#[inline]
pub fn parse_bool(s: &str) -> Result<bool, GrpcStatus<'static>> {
    match s {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(GrpcStatus::invalid_argument("invalid boolean parameter")),
    }
}

/// Parses an ASCII `u32` parameter.
#[inline]
pub fn parse_u32(s: &str) -> Result<u32, GrpcStatus<'static>> {
    s.parse::<u32>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid uint32 parameter"))
}

/// Parses an ASCII `i32` parameter.
#[inline]
pub fn parse_i32(s: &str) -> Result<i32, GrpcStatus<'static>> {
    s.parse::<i32>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid int32 parameter"))
}

/// Parses an ASCII `u64` parameter.
#[inline]
pub fn parse_u64(s: &str) -> Result<u64, GrpcStatus<'static>> {
    s.parse::<u64>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid uint64 parameter"))
}

/// Parses an ASCII `i64` parameter.
#[inline]
pub fn parse_i64(s: &str) -> Result<i64, GrpcStatus<'static>> {
    s.parse::<i64>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid int64 parameter"))
}

/// Parses an ASCII `f32` parameter.
#[inline]
pub fn parse_f32(s: &str) -> Result<f32, GrpcStatus<'static>> {
    s.parse::<f32>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid float parameter"))
}

/// Parses an ASCII `f64` parameter.
#[inline]
pub fn parse_f64(s: &str) -> Result<f64, GrpcStatus<'static>> {
    s.parse::<f64>()
        .map_err(|_| GrpcStatus::invalid_argument("invalid double parameter"))
}

/// Parses a Protobuf enum parameter by checking `by_name(s)` first and falling back to [`parse_i32`].
#[inline]
pub fn parse_enum<F>(s: &str, by_name: F) -> Result<i32, GrpcStatus<'static>>
where
    F: FnOnce(&str) -> Option<i32>,
{
    if let Some(val) = by_name(s) {
        Ok(val)
    } else {
        parse_i32(s).map_err(|_| GrpcStatus::invalid_argument("invalid enum parameter"))
    }
}

#[inline]
const fn map_req_encode_err(_err: EncodeError) -> GrpcStatus<'static> {
    GrpcStatus::resource_exhausted("request buffer overflow")
}

/// Prepends a raw varint field `(field_number, WireType::Varint)` onto `writer`.
#[inline]
pub fn write_varint_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: u64,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_varint(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Varint)
        .map_err(map_req_encode_err)
}

/// Prepends an `int32` or `enum` field (sign-extended to 64-bit varint per Protobuf wire format) onto `writer`.
#[inline]
pub fn write_int32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i32,
) -> Result<(), GrpcStatus<'static>> {
    write_varint_field(writer, field_number, val as i64 as u64)
}

/// Prepends an `int64` field onto `writer`.
#[inline]
pub fn write_int64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i64,
) -> Result<(), GrpcStatus<'static>> {
    write_varint_field(writer, field_number, val as u64)
}

/// Prepends a `uint32` field onto `writer`.
#[inline]
pub fn write_uint32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: u32,
) -> Result<(), GrpcStatus<'static>> {
    write_varint_field(writer, field_number, u64::from(val))
}

/// Prepends a `uint64` field onto `writer`.
#[inline]
pub fn write_uint64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: u64,
) -> Result<(), GrpcStatus<'static>> {
    write_varint_field(writer, field_number, val)
}

/// Prepends a ZigZag-encoded `sint32` field onto `writer`.
#[inline]
pub fn write_sint32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i32,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_zigzag32(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Varint)
        .map_err(map_req_encode_err)
}

/// Prepends a ZigZag-encoded `sint64` field onto `writer`.
#[inline]
pub fn write_sint64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i64,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_zigzag64(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Varint)
        .map_err(map_req_encode_err)
}

/// Prepends a `bool` field onto `writer`.
#[inline]
pub fn write_bool_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: bool,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_bool(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Varint)
        .map_err(map_req_encode_err)
}

/// Prepends a `fixed32` field onto `writer`.
#[inline]
pub fn write_fixed32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: u32,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_fixed32(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed32)
        .map_err(map_req_encode_err)
}

/// Prepends an `sfixed32` field onto `writer`.
#[inline]
pub fn write_sfixed32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i32,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_sfixed32(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed32)
        .map_err(map_req_encode_err)
}

/// Prepends a `fixed64` field onto `writer`.
#[inline]
pub fn write_fixed64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: u64,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_fixed64(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed64)
        .map_err(map_req_encode_err)
}

/// Prepends an `sfixed64` field onto `writer`.
#[inline]
pub fn write_sfixed64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: i64,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_sfixed64(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed64)
        .map_err(map_req_encode_err)
}

/// Prepends a `float` (`f32`) field onto `writer`.
#[inline]
pub fn write_f32_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: f32,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_f32(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed32)
        .map_err(map_req_encode_err)
}

/// Prepends a `double` (`f64`) field onto `writer`.
#[inline]
pub fn write_f64_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: f64,
) -> Result<(), GrpcStatus<'static>> {
    writer.write_f64(val).map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::Fixed64)
        .map_err(map_req_encode_err)
}

/// Prepends a length-delimited (`bytes` or submessage) field onto `writer`.
#[inline]
pub fn write_bytes_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: &[u8],
) -> Result<(), GrpcStatus<'static>> {
    writer
        .write_length_delimited(val)
        .map_err(map_req_encode_err)?;
    writer
        .write_tag(field_number, WireType::LengthDelimited)
        .map_err(map_req_encode_err)
}

/// Prepends a UTF-8 `string` field onto `writer`.
#[inline]
pub fn write_str_field(
    writer: &mut BackwardWriter<'_>,
    field_number: u32,
    val: &str,
) -> Result<(), GrpcStatus<'static>> {
    write_bytes_field(writer, field_number, val.as_bytes())
}

/// Prepends raw pre-encoded Protobuf wire bytes onto `writer`.
#[inline]
pub fn write_raw_bytes(
    writer: &mut BackwardWriter<'_>,
    raw: &[u8],
) -> Result<(), GrpcStatus<'static>> {
    writer.write_bytes(raw).map_err(map_req_encode_err)
}

/// Validates that `raw` is a well-formed sequence of Protobuf wire fields (ending on an exact
/// field boundary) and prepends it onto `writer`.
#[inline]
pub fn write_validated_message_bytes(
    writer: &mut BackwardWriter<'_>,
    raw: &[u8],
) -> Result<(), GrpcStatus<'static>> {
    for item in FieldIter::new(raw) {
        item.map_err(|_| GrpcStatus::invalid_argument("invalid protobuf request payload"))?;
    }
    write_raw_bytes(writer, raw)
}

/// Wraps the bytes written into `writer` since `prev_written_len` inside a chain of enclosing
/// length-delimited parent submessage tags (`parent_tags` ordered from outermost to innermost).
#[inline]
pub fn wrap_nested_path(
    writer: &mut BackwardWriter<'_>,
    prev_written_len: usize,
    parent_tags: &[u32],
) -> Result<(), GrpcStatus<'static>> {
    for &tag in parent_tags.iter().rev() {
        let len = writer.written_len().saturating_sub(prev_written_len);
        writer
            .write_varint(len as u64)
            .map_err(map_req_encode_err)?;
        writer
            .write_tag(tag, WireType::LengthDelimited)
            .map_err(map_req_encode_err)?;
    }
    Ok(())
}

/// Extracts the length-delimited payload of submessage field `field_number` from `encoded_req`
/// for merging nested URL path bindings with a request body.
///
/// Returns `Err(GrpcStatus::invalid_argument(...))` if `encoded_req` is not valid Protobuf wire
/// format or if `field_number` is encoded with a non-length-delimited wire type.
pub fn extract_request_submessage_field<'b>(
    encoded_req: &'b [u8],
    field_number: u32,
) -> Result<&'b [u8], GrpcStatus<'static>> {
    let mut found: &'b [u8] = &[];
    for item in FieldIter::new(encoded_req) {
        let field =
            item.map_err(|_| GrpcStatus::invalid_argument("invalid protobuf request payload"))?;
        if field.tag == field_number {
            if field.wire_type != WireType::LengthDelimited {
                return Err(GrpcStatus::invalid_argument(
                    "invalid protobuf request payload",
                ));
            }
            found = field.marker.slice(encoded_req);
        }
    }
    Ok(found)
}

/// Extracts the payload of top-level field `field_number` from `encoded_resp` for
/// `google.api.HttpRule.response_body` projection.
///
/// For length-delimited fields (submessages, `bytes`, `string`), returns the inner payload
/// slice (excluding tag and length prefix), or `&[]` if the field was not set.
/// For scalar fields, returns the raw encoded value slice, or `&[]` if not set.
pub fn extract_response_body_field<'b>(
    encoded_resp: &'b [u8],
    field_number: u32,
) -> Result<&'b [u8], GrpcStatus<'static>> {
    let mut found: &'b [u8] = &[];
    for item in FieldIter::new(encoded_resp) {
        let field = item.map_err(|_| GrpcStatus::internal("invalid protobuf response payload"))?;
        if field.tag == field_number {
            found = field.marker.slice(encoded_resp);
        }
    }
    Ok(found)
}

/// Finishes `writer` and wraps the resulting Protobuf bytes (or projected `response_body_field`)
/// into a `200 OK` [`HttpResponse`].
#[inline]
pub fn finish_response<'b>(
    writer: BackwardWriter<'b>,
    response_body_field: Option<u32>,
) -> Result<HttpResponse<'b>, GrpcStatus<'static>> {
    let encoded = writer.finish().map_err(GrpcStatus::from)?;
    let body = match response_body_field {
        Some(tag) => extract_response_body_field(encoded, tag)?,
        None => encoded,
    };
    Ok(HttpResponse::ok_protobuf(body))
}

/// Finishes `writer` in-place via [`take_writer`] and wraps the resulting Protobuf bytes
/// (or projected `response_body_field`) into a `200 OK` [`HttpResponse`].
#[inline]
pub fn finish_writer_response<'b>(
    writer: &mut BackwardWriter<'b>,
    response_body_field: Option<u32>,
) -> Result<HttpResponse<'b>, GrpcStatus<'static>> {
    finish_response(take_writer(writer), response_body_field)
}

/// Synchronous `#![no_std]` zero-allocation HTTP service dispatch trait implemented by
/// generated `<Service>Server<T>` wrappers when `http` codegen is enabled.
///
/// Returns:
/// - `None` if `req` did not match any HTTP route on this service.
/// - `Some(Ok(resp))` if a route matched and the handler succeeded.
/// - `Some(Err(status))` if a route matched and parameter binding, decoding, or the handler returned an error.
pub trait HttpService {
    fn call_http<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>;
}

/// Asynchronous `#![no_std]` zero-allocation HTTP service dispatch trait implemented by
/// generated `<Service>Server<T>` wrappers when `http` codegen is enabled.
#[allow(async_fn_in_trait)]
pub trait AsyncHttpService {
    async fn call_http<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>;
}

/// Object-safe view of an [`HttpService`], allowing zero-allocation registration of
/// heterogeneous HTTP services as a `&[&dyn HttpRoute]` slice.
pub trait HttpRoute: Sync {
    fn dispatch_http<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>;
}

impl<T: HttpService + Sync> HttpRoute for T {
    #[inline]
    fn dispatch_http<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
        self.call_http(req, scratch, writer)
    }
}

/// Dispatches `req` across a slice of object-safe [`HttpRoute`] services, returning the first
/// matching route result or `None` if no service matched.
#[inline]
pub fn dispatch_http_routes<'a, 's, 'b>(
    routes: &[&dyn HttpRoute],
    req: &HttpRequest<'a>,
    scratch: &'s mut [u8],
    writer: &mut BackwardWriter<'b>,
) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
    for route in routes {
        if let Some(res) = route.dispatch_http(req, scratch, writer) {
            return Some(res);
        }
    }
    None
}

/// Zero-allocation asynchronous HTTP router trait, implemented for every [`AsyncHttpService`]
/// as well as 2-, 3-, and 4-tuples of [`AsyncHttpService`] implementations.
#[allow(async_fn_in_trait)]
pub trait AsyncHttpRouter {
    async fn dispatch_http_async<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>>;
}

impl<S: AsyncHttpService> AsyncHttpRouter for S {
    #[inline]
    async fn dispatch_http_async<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
        self.call_http(req, scratch, writer).await
    }
}

impl<A: AsyncHttpService, B: AsyncHttpService> AsyncHttpRouter for (A, B) {
    #[inline]
    async fn dispatch_http_async<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = self.0.call_http(req, scratch, writer).await {
            return Some(res);
        }
        self.1.call_http(req, scratch, writer).await
    }
}

impl<A: AsyncHttpService, B: AsyncHttpService, C: AsyncHttpService> AsyncHttpRouter for (A, B, C) {
    #[inline]
    async fn dispatch_http_async<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = self.0.call_http(req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = self.1.call_http(req, scratch, writer).await {
            return Some(res);
        }
        self.2.call_http(req, scratch, writer).await
    }
}

impl<A: AsyncHttpService, B: AsyncHttpService, C: AsyncHttpService, D: AsyncHttpService>
    AsyncHttpRouter for (A, B, C, D)
{
    #[inline]
    async fn dispatch_http_async<'a, 's, 'b>(
        &self,
        req: &HttpRequest<'a>,
        scratch: &'s mut [u8],
        writer: &mut BackwardWriter<'b>,
    ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
        if let Some(res) = self.0.call_http(req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = self.1.call_http(req, scratch, writer).await {
            return Some(res);
        }
        if let Some(res) = self.2.call_http(req, scratch, writer).await {
            return Some(res);
        }
        self.3.call_http(req, scratch, writer).await
    }
}

/// Inspects the initial bytes on a connection to determine whether the client is speaking
/// HTTP/2 prior-knowledge (`PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n`) or HTTP/1.1.
///
/// Returns:
/// - `None` if `buf.len() < 4` and `buf` is a prefix of the HTTP/2 client connection preface.
/// - `Some(true)` if `buf` has at least 4 bytes (`"PRI "`) and matches the HTTP/2 preface up to `min(buf.len(), 24)`.
/// - `Some(false)` as soon as any byte diverges from the HTTP/2 client connection preface.
#[inline]
pub fn is_h2_client_preface(buf: &[u8]) -> Option<bool> {
    let check_len = if buf.len() < CLIENT_CONNECTION_PREFACE.len() {
        buf.len()
    } else {
        CLIENT_CONNECTION_PREFACE.len()
    };
    if buf[..check_len] != CLIENT_CONNECTION_PREFACE[..check_len] {
        return Some(false);
    }
    if buf.len() < 4 {
        None
    } else {
        Some(true)
    }
}

/// Zero-copy parsed HTTP/1.1 request produced by [`parse_h1_request`].
#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParsedH1Request<'a> {
    /// Parsed HTTP request view borrowing directly from the input buffer.
    pub request: HttpRequest<'a>,
    /// Whether the connection should be kept alive after sending the response.
    pub keep_alive: bool,
    /// Total number of bytes consumed from the input buffer (headers + `\r\n\r\n` + body).
    pub consumed: usize,
}

#[inline]
fn find_crlf(buf: &[u8]) -> Option<usize> {
    buf.windows(2).position(|w| w == b"\r\n")
}

#[inline]
fn find_header_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4).position(|w| w == b"\r\n\r\n")
}

#[inline]
fn trim_ows(mut bytes: &[u8]) -> &[u8] {
    while let Some(&first) = bytes.first() {
        if first == b' ' || first == b'\t' {
            bytes = &bytes[1..];
        } else {
            break;
        }
    }
    while let Some(&last) = bytes.last() {
        if last == b' ' || last == b'\t' {
            bytes = &bytes[..bytes.len() - 1];
        } else {
            break;
        }
    }
    bytes
}

#[inline]
fn parse_content_length_ascii(s: &str) -> Option<usize> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse::<usize>().ok()
}

/// Sans-I/O zero-allocation HTTP/1.1 request parser.
///
/// Returns:
/// - `Ok(None)` if the complete header block (`\r\n\r\n`) or the full `Content-Length` body
///   has not yet arrived in `buf`.
/// - `Ok(Some(ParsedH1Request))` when a complete HTTP/1.1 (or HTTP/1.0) request is available.
/// - `Err(GrpcStatus)` with [`GrpcCode::InvalidArgument`] if the request line, headers, or
///   `Transfer-Encoding` are malformed or unsupported.
pub fn parse_h1_request<'a>(
    buf: &'a [u8],
) -> Result<Option<ParsedH1Request<'a>>, GrpcStatus<'static>> {
    if buf.is_empty() {
        return Ok(None);
    }

    let Some(header_end) = find_header_end(buf) else {
        return Ok(None);
    };
    let header_block = &buf[..header_end];
    let body_offset = header_end + 4;

    let (req_line, mut rest_headers) = match find_crlf(header_block) {
        Some(pos) => (&header_block[..pos], &header_block[pos + 2..]),
        None => (header_block, &[][..]),
    };

    // Parse request line: `<METHOD> <TARGET> <HTTP/1.x>`
    let Some(sp1) = req_line.iter().position(|&b| b == b' ') else {
        return Err(GrpcStatus::invalid_argument(
            "malformed HTTP/1.1 request line",
        ));
    };
    let method_bytes = &req_line[..sp1];
    let after_method = &req_line[sp1 + 1..];
    let Some(sp2) = after_method.iter().position(|&b| b == b' ') else {
        return Err(GrpcStatus::invalid_argument(
            "malformed HTTP/1.1 request line",
        ));
    };
    let target_bytes = &after_method[..sp2];
    let version_bytes = &after_method[sp2 + 1..];

    if target_bytes.is_empty()
        || target_bytes.iter().any(|&b| b <= 0x20 || b == 0x7F)
        || version_bytes.contains(&b' ')
    {
        return Err(GrpcStatus::invalid_argument(
            "malformed HTTP/1.1 request line",
        ));
    }

    let method = HttpMethod::from_bytes(method_bytes)
        .ok_or_else(|| GrpcStatus::invalid_argument("invalid HTTP method"))?;

    let target_str = try_percent_decode_in_place(target_bytes)
        .or_else(|| core::str::from_utf8(target_bytes).ok())
        .ok_or_else(|| GrpcStatus::invalid_argument("invalid UTF-8 in HTTP request target"))?;
    if !target_str.starts_with('/') {
        return Err(GrpcStatus::invalid_argument(
            "HTTP request target must start with '/'",
        ));
    }

    let default_keep_alive = match version_bytes {
        b"HTTP/1.1" => true,
        b"HTTP/1.0" => false,
        _ => {
            return Err(GrpcStatus::invalid_argument(
                "unsupported or invalid HTTP version",
            ))
        }
    };

    let mut content_length: Option<usize> = None;
    let mut content_type: Option<&'a str> = None;
    let mut conn_close = false;
    let mut conn_keep_alive = false;

    while !rest_headers.is_empty() {
        let (line, next_rest) = match find_crlf(rest_headers) {
            Some(pos) => (&rest_headers[..pos], &rest_headers[pos + 2..]),
            None => (rest_headers, &[][..]),
        };
        rest_headers = next_rest;

        let Some(colon_pos) = line.iter().position(|&b| b == b':') else {
            return Err(GrpcStatus::invalid_argument("malformed HTTP header line"));
        };
        let name_bytes = &line[..colon_pos];
        if name_bytes.is_empty() || !name_bytes.iter().all(|&b| is_http_tchar(b)) {
            return Err(GrpcStatus::invalid_argument("invalid HTTP header name"));
        }
        let val_bytes = trim_ows(&line[colon_pos + 1..]);
        if val_bytes
            .iter()
            .any(|&b| (b < 0x20 && b != b'\t') || b == 0x7F)
        {
            return Err(GrpcStatus::invalid_argument("invalid HTTP header value"));
        }
        let name_str = core::str::from_utf8(name_bytes)
            .map_err(|_| GrpcStatus::invalid_argument("invalid HTTP header name"))?;
        let val_str = core::str::from_utf8(val_bytes)
            .map_err(|_| GrpcStatus::invalid_argument("invalid UTF-8 in HTTP header value"))?;

        if name_str.eq_ignore_ascii_case("content-length") {
            let parsed_len = parse_content_length_ascii(val_str)
                .ok_or_else(|| GrpcStatus::invalid_argument("invalid Content-Length header"))?;
            if let Some(prev) = content_length {
                if prev != parsed_len {
                    return Err(GrpcStatus::invalid_argument(
                        "conflicting Content-Length headers",
                    ));
                }
            }
            content_length = Some(parsed_len);
        } else if name_str.eq_ignore_ascii_case("content-type") {
            content_type = Some(val_str);
        } else if name_str.eq_ignore_ascii_case("transfer-encoding") {
            if !val_str.is_empty() && !val_str.eq_ignore_ascii_case("identity") {
                return Err(GrpcStatus::invalid_argument(
                    "Transfer-Encoding is not supported",
                ));
            }
        } else if name_str.eq_ignore_ascii_case("connection") {
            for token in val_str.split(',') {
                let t = token.trim();
                if t.eq_ignore_ascii_case("close") {
                    conn_close = true;
                } else if t.eq_ignore_ascii_case("keep-alive") {
                    conn_keep_alive = true;
                }
            }
        }
    }

    let body_len = content_length.unwrap_or(0);
    let total_len = body_offset
        .checked_add(body_len)
        .ok_or_else(|| GrpcStatus::invalid_argument("Content-Length overflow"))?;

    if buf.len() < total_len {
        return Ok(None);
    }

    let keep_alive = if conn_close {
        false
    } else if conn_keep_alive {
        true
    } else {
        default_keep_alive
    };

    let body = &buf[body_offset..total_len];
    let request = HttpRequest::from_uri(method, target_str, content_type, body);

    Ok(Some(ParsedH1Request {
        request,
        keep_alive,
        consumed: total_len,
    }))
}

#[inline]
fn append_slice(out: &mut [u8], pos: &mut usize, bytes: &[u8]) -> Result<(), EncodeError> {
    if *pos + bytes.len() > out.len() {
        return Err(EncodeError::BufferOverflow);
    }
    out[*pos..*pos + bytes.len()].copy_from_slice(bytes);
    *pos += bytes.len();
    Ok(())
}

#[inline]
fn append_usize_decimal(
    out: &mut [u8],
    pos: &mut usize,
    mut val: usize,
) -> Result<(), EncodeError> {
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    if val == 0 {
        i -= 1;
        digits[i] = b'0';
    } else {
        while val > 0 {
            i -= 1;
            digits[i] = b'0' + (val % 10) as u8;
            val /= 10;
        }
    }
    append_slice(out, pos, &digits[i..])
}

/// Serializes an HTTP/1.1 response into `out`, returning the number of bytes written.
pub fn encode_h1_response(
    status_code: u16,
    reason: &str,
    content_type: &str,
    body: &[u8],
    keep_alive: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    let mut pos = 0;
    append_slice(out, &mut pos, b"HTTP/1.1 ")?;
    append_usize_decimal(out, &mut pos, usize::from(status_code))?;
    append_slice(out, &mut pos, b" ")?;
    append_slice(out, &mut pos, reason.as_bytes())?;
    append_slice(out, &mut pos, b"\r\nContent-Type: ")?;
    append_slice(out, &mut pos, content_type.as_bytes())?;
    append_slice(out, &mut pos, b"\r\nContent-Length: ")?;
    append_usize_decimal(out, &mut pos, body.len())?;
    if keep_alive {
        append_slice(out, &mut pos, b"\r\nConnection: keep-alive\r\n\r\n")?;
    } else {
        append_slice(out, &mut pos, b"\r\nConnection: close\r\n\r\n")?;
    }
    append_slice(out, &mut pos, body)?;
    Ok(pos)
}

/// Serializes an [`HttpResponse`] into `out` as an HTTP/1.1 response.
#[inline]
pub fn encode_h1_http_response(
    resp: &HttpResponse<'_>,
    keep_alive: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_h1_response(
        resp.status_code,
        http_reason_phrase(resp.status_code),
        resp.content_type,
        resp.body,
        keep_alive,
        out,
    )
}

/// Serializes a [`GrpcStatus`] error as an HTTP/1.1 response into `out` using [`GrpcCode::http_status`]
/// and [`GrpcCode::http_reason`].
#[inline]
pub fn encode_h1_error(
    status: &GrpcStatus<'_>,
    keep_alive: bool,
    out: &mut [u8],
) -> Result<usize, EncodeError> {
    encode_h1_response(
        status.http_status(),
        status.http_reason(),
        CONTENT_TYPE_TEXT_PLAIN,
        status.message.as_bytes(),
        keep_alive,
        out,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::status::GrpcCode;
    use roto_runtime::{
        decode_bool_at, decode_f32_at, decode_f64_at, decode_fixed32_at, decode_fixed64_at,
        decode_sfixed32_at, decode_sfixed64_at, decode_sint32_at, decode_sint64_at, decode_str_at,
        decode_varint_at,
    };

    #[test]
    fn http_method_parsing_and_matching() {
        assert_eq!(HttpMethod::from_bytes(b"GET"), Some(HttpMethod::Get));
        assert_eq!(HttpMethod::from_bytes(b"PUT"), Some(HttpMethod::Put));
        assert_eq!(HttpMethod::from_bytes(b"POST"), Some(HttpMethod::Post));
        assert_eq!(HttpMethod::from_bytes(b"DELETE"), Some(HttpMethod::Delete));
        assert_eq!(HttpMethod::from_bytes(b"PATCH"), Some(HttpMethod::Patch));
        assert_eq!(
            HttpMethod::from_bytes(b"OPTIONS"),
            Some(HttpMethod::Custom("OPTIONS"))
        );
        assert_eq!(
            HttpMethod::from_bytes(b"PROPFIND"),
            Some(HttpMethod::Custom("PROPFIND"))
        );

        assert_eq!(HttpMethod::from_bytes(b""), None);
        assert_eq!(HttpMethod::from_bytes(b"GET /"), None);
        assert_eq!(HttpMethod::from_bytes(b"GET\r\n"), None);
        assert_eq!(HttpMethod::from_bytes(&[0xFF]), None);

        assert!(HttpMethod::Get.matches("GET"));
        assert!(HttpMethod::Get.matches("get"));
        assert!(!HttpMethod::Get.matches("POST"));
        assert!(HttpMethod::Custom("LOCK").matches("lock"));
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
    }

    #[test]
    fn http_request_from_uri_and_custom_verb_splitting() {
        let req = HttpRequest::from_uri(
            HttpMethod::Get,
            "/v1/users/42:activate?notify=true&retry=2",
            Some(CONTENT_TYPE_PROTOBUF),
            b"payload",
        );
        assert_eq!(req.method, HttpMethod::Get);
        assert_eq!(req.path, "/v1/users/42:activate");
        assert_eq!(req.query, "notify=true&retry=2");
        assert_eq!(req.content_type, Some(CONTENT_TYPE_PROTOBUF));
        assert_eq!(req.body, b"payload");

        assert_eq!(
            split_custom_verb("/v1/users/42:activate"),
            ("/v1/users/42", Some("activate"))
        );
        assert_eq!(
            split_custom_verb("/v1/messages:batchGet"),
            ("/v1/messages", Some("batchGet"))
        );
        assert_eq!(split_custom_verb("/v1/users/42"), ("/v1/users/42", None));
        assert_eq!(
            split_custom_verb("/v1/foo:bar/baz"),
            ("/v1/foo:bar/baz", None)
        );
        assert_eq!(
            split_custom_verb("/v1/:empty_prefix"),
            ("/v1/:empty_prefix", None)
        );
        assert_eq!(
            split_custom_verb("/v1/empty_verb:"),
            ("/v1/empty_verb:", None)
        );
    }

    #[test]
    fn path_cursor_matching_and_captures() {
        // Basic literal + single segment capture
        let mut cursor = PathCursor::without_custom_verb("/v1/shelves/s1/books/b2").unwrap();
        assert!(cursor.literal("v1"));
        assert!(cursor.literal("shelves"));
        assert_eq!(cursor.capture_single(), Some("s1"));
        assert!(cursor.literal("books"));
        assert_eq!(cursor.capture_single(), Some("b2"));
        assert!(cursor.is_end());
        assert_eq!(cursor.next_segment(), None);

        // Multi-segment sub-template capture `{name=shelves/*/books/*}`
        let mut cursor = PathCursor::without_custom_verb("/v1/shelves/s1/books/b2").unwrap();
        assert!(cursor.literal("v1"));
        let start = cursor.remaining();
        assert!(cursor.literal("shelves"));
        assert!(cursor.capture_single().is_some());
        assert!(cursor.literal("books"));
        assert!(cursor.capture_single().is_some());
        let name = cursor.slice_since(start);
        assert_eq!(name, "shelves/s1/books/b2");
        assert!(cursor.is_end());

        // `**` capture at end and with trailing literal segments
        let mut cursor = PathCursor::without_custom_verb("/v1/files/a/b/c/download").unwrap();
        assert!(cursor.literal("v1"));
        assert!(cursor.literal("files"));
        assert_eq!(cursor.capture_until_remaining_segments(1), Some("a/b/c"));
        assert!(cursor.literal("download"));
        assert!(cursor.is_end());

        // Custom verb matching
        let mut cursor = PathCursor::with_custom_verb("/v1/books/42:archive", "archive").unwrap();
        assert!(cursor.literal("v1"));
        assert!(cursor.literal("books"));
        assert_eq!(cursor.capture_single(), Some("42"));
        assert!(cursor.is_end());
        assert!(PathCursor::with_custom_verb("/v1/books/42:archive", "cancel").is_none());
        assert!(PathCursor::without_custom_verb("/v1/books/42:archive").is_none());

        // Reject trailing slash and empty segments
        let mut bad = PathCursor::new("/v1//books").unwrap();
        assert!(bad.literal("v1"));
        assert_eq!(bad.next_segment(), None);

        let mut trailing = PathCursor::new("/v1/").unwrap();
        assert!(trailing.literal("v1"));
        assert!(!trailing.is_end());
        assert_eq!(trailing.next_segment(), None);
    }

    #[test]
    fn percent_decoding_and_query_iter() {
        let mut scratch = [0u8; 64];
        assert_eq!(
            percent_decode_str("plain_ascii", &mut scratch),
            Ok("plain_ascii")
        );
        assert_eq!(
            percent_decode_str("hello%20world%2F42", &mut scratch),
            Ok("hello world/42")
        );
        let err = percent_decode_str("%ZZ", &mut scratch).unwrap_err();
        assert_eq!(err.code, GrpcCode::InvalidArgument);

        let mut tiny = [0u8; 2];
        let overflow_err = percent_decode_str("a%20b", &mut tiny).unwrap_err();
        assert_eq!(overflow_err.code, GrpcCode::ResourceExhausted);

        let raw_bytes = percent_decode_bytes(b"a%00%ff", &mut scratch).unwrap();
        assert_eq!(raw_bytes, &[b'a', 0x00, 0xFF]);

        let mut q = query_iter("?a=1&&b=hello%20world&flag&empty=&= nokey &c=3");
        assert_eq!(q.next(), Some(("a", "1")));
        assert_eq!(q.next(), Some(("b", "hello%20world")));
        assert_eq!(q.next(), Some(("flag", "")));
        assert_eq!(q.next(), Some(("empty", "")));
        assert_eq!(q.next(), Some(("c", "3")));
        assert_eq!(q.next(), None);
    }

    #[test]
    fn ascii_scalar_and_enum_parsers() {
        assert_eq!(parse_bool("true"), Ok(true));
        assert_eq!(parse_bool("1"), Ok(true));
        assert_eq!(parse_bool("false"), Ok(false));
        assert_eq!(parse_bool("0"), Ok(false));
        assert_eq!(
            parse_bool("yes").unwrap_err().code,
            GrpcCode::InvalidArgument
        );

        assert_eq!(parse_u32("42"), Ok(42));
        assert_eq!(parse_u32("-1").unwrap_err().code, GrpcCode::InvalidArgument);
        assert_eq!(parse_i32("-42"), Ok(-42));
        assert_eq!(parse_u64("18446744073709551615"), Ok(u64::MAX));
        assert_eq!(parse_i64("-9223372036854775808"), Ok(i64::MIN));
        assert_eq!(parse_f32("3.5"), Ok(3.5f32));
        assert_eq!(parse_f64("-125.25"), Ok(-125.25f64));

        let by_name = |name: &str| match name {
            " UNKNOWN " => None,
            "ACTIVE" => Some(1),
            "ARCHIVED" => Some(2),
            _ => None,
        };
        assert_eq!(parse_enum("ACTIVE", by_name), Ok(1));
        assert_eq!(parse_enum("2", by_name), Ok(2));
        assert_eq!(parse_enum("-1", by_name), Ok(-1));
        assert_eq!(
            parse_enum("BOGUS", by_name).unwrap_err().code,
            GrpcCode::InvalidArgument
        );
    }

    #[test]
    fn backward_writer_field_helpers_and_nested_wrapping() {
        let mut buf = [0u8; 256];
        let mut writer = BackwardWriter::new(&mut buf);

        // Write a nested path `{outer.inner.leaf = "hello"}` where outer=tag 10, inner=tag 2, leaf=tag 1
        let mark = writer.written_len();
        write_str_field(&mut writer, 1, "hello").unwrap();
        wrap_nested_path(&mut writer, mark, &[10, 2]).unwrap();

        // Write top-level scalar fields
        write_f64_field(&mut writer, 9, -8.5).unwrap();
        write_f32_field(&mut writer, 8, 2.25).unwrap();
        write_sfixed64_field(&mut writer, 7, -700).unwrap();
        write_fixed64_field(&mut writer, 6, 600).unwrap();
        write_sfixed32_field(&mut writer, 5, -500).unwrap();
        write_fixed32_field(&mut writer, 4, 400).unwrap();
        write_sint64_field(&mut writer, 3, -300).unwrap();
        write_sint32_field(&mut writer, 2, -200).unwrap();
        write_bool_field(&mut writer, 1, true).unwrap();
        write_int32_field(&mut writer, 11, -1).unwrap();

        let encoded = writer.finish().unwrap();
        let mut iter = FieldIter::new(encoded);

        let f11 = iter.next().unwrap().unwrap();
        assert_eq!(f11.tag, 11);
        assert_eq!(decode_varint_at(encoded, f11.marker).unwrap() as i32, -1);

        let f1 = iter.next().unwrap().unwrap();
        assert_eq!(f1.tag, 1);
        assert_eq!(decode_bool_at(encoded, f1.marker), Ok(true));

        let f2 = iter.next().unwrap().unwrap();
        assert_eq!(f2.tag, 2);
        assert_eq!(decode_sint32_at(encoded, f2.marker), Ok(-200));

        let f3 = iter.next().unwrap().unwrap();
        assert_eq!(f3.tag, 3);
        assert_eq!(decode_sint64_at(encoded, f3.marker), Ok(-300));

        let f4 = iter.next().unwrap().unwrap();
        assert_eq!(f4.tag, 4);
        assert_eq!(decode_fixed32_at(encoded, f4.marker), Ok(400));

        let f5 = iter.next().unwrap().unwrap();
        assert_eq!(f5.tag, 5);
        assert_eq!(decode_sfixed32_at(encoded, f5.marker), Ok(-500));

        let f6 = iter.next().unwrap().unwrap();
        assert_eq!(f6.tag, 6);
        assert_eq!(decode_fixed64_at(encoded, f6.marker), Ok(600));

        let f7 = iter.next().unwrap().unwrap();
        assert_eq!(f7.tag, 7);
        assert_eq!(decode_sfixed64_at(encoded, f7.marker), Ok(-700));

        let f8 = iter.next().unwrap().unwrap();
        assert_eq!(f8.tag, 8);
        assert_eq!(decode_f32_at(encoded, f8.marker), Ok(2.25));

        let f9 = iter.next().unwrap().unwrap();
        assert_eq!(f9.tag, 9);
        assert_eq!(decode_f64_at(encoded, f9.marker), Ok(-8.5));

        let f10 = iter.next().unwrap().unwrap();
        assert_eq!(f10.tag, 10);
        let outer_slice = f10.marker.slice(encoded);
        let inner_field = FieldIter::new(outer_slice).next().unwrap().unwrap();
        assert_eq!(inner_field.tag, 2);
        let inner_slice = inner_field.marker.slice(outer_slice);
        let leaf_field = FieldIter::new(inner_slice).next().unwrap().unwrap();
        assert_eq!(leaf_field.tag, 1);
        assert_eq!(decode_str_at(inner_slice, leaf_field.marker), Ok("hello"));

        // Verify response_body extraction and request submessage extraction
        let projected = extract_response_body_field(encoded, 10).unwrap();
        assert_eq!(projected, outer_slice);
        let req_sub = extract_request_submessage_field(encoded, 10).unwrap();
        assert_eq!(req_sub, outer_slice);

        // Request submessage extraction rejects non-LengthDelimited wire types and malformed wire bytes with InvalidArgument
        let wrong_wt_err = extract_request_submessage_field(encoded, 1).unwrap_err();
        assert_eq!(wrong_wt_err.code, GrpcCode::InvalidArgument);
        let malformed_err = extract_request_submessage_field(&[0x0a, 0x05, b'a'], 1).unwrap_err();
        assert_eq!(malformed_err.code, GrpcCode::InvalidArgument);

        // write_validated_message_bytes rejects truncated wire bytes with InvalidArgument
        let mut vbuf = [0u8; 32];
        let mut vw = BackwardWriter::new(&mut vbuf);
        assert!(write_validated_message_bytes(&mut vw, outer_slice).is_ok());
        let bad_vw = write_validated_message_bytes(&mut vw, &[0x0a, 0x05, b'a']).unwrap_err();
        assert_eq!(bad_vw.code, GrpcCode::InvalidArgument);
    }

    struct EchoHttpService;

    impl HttpService for EchoHttpService {
        fn call_http<'a, 's, 'b>(
            &self,
            req: &HttpRequest<'a>,
            scratch: &'s mut [u8],
            writer: &mut BackwardWriter<'b>,
        ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
            if !req.method.matches("GET") {
                return None;
            }
            let mut cursor = PathCursor::without_custom_verb(req.path)?;
            if !cursor.literal("v1") || !cursor.literal("echo") {
                return None;
            }
            let msg = cursor.capture_single()?;
            if !cursor.is_end() {
                return None;
            }
            let decoded = match percent_decode_str(msg, scratch) {
                Ok(s) => s,
                Err(e) => return Some(Err(e)),
            };
            if let Err(e) = write_str_field(writer, 1, decoded) {
                return Some(Err(e));
            }
            Some(finish_writer_response(writer, None))
        }
    }

    #[test]
    fn http_service_and_route_dispatch() {
        let svc = EchoHttpService;
        let routes: &[&dyn HttpRoute] = &[&svc];
        let mut scratch = [0u8; 64];
        let mut out_buf = [0u8; 64];

        // Unmatched route returns None
        let mut writer = BackwardWriter::new(&mut out_buf);
        let unmatched = HttpRequest::from_uri(HttpMethod::Post, "/v1/other", None, &[]);
        assert!(dispatch_http_routes(routes, &unmatched, &mut scratch, &mut writer).is_none());

        // Matched route encodes protobuf field 1 = "hi!"
        let matched = HttpRequest::from_uri(HttpMethod::Get, "/v1/echo/hi%21", None, &[]);
        let resp = dispatch_http_routes(routes, &matched, &mut scratch, &mut writer)
            .unwrap()
            .unwrap();
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.content_type, CONTENT_TYPE_PROTOBUF);
        assert_eq!(resp.body, b"\x0a\x03hi!");
    }

    struct AsyncPingHttpService;

    impl AsyncHttpService for AsyncPingHttpService {
        async fn call_http<'a, 's, 'b>(
            &self,
            req: &HttpRequest<'a>,
            _scratch: &'s mut [u8],
            writer: &mut BackwardWriter<'b>,
        ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
            if !req.method.matches("POST") || req.path != "/v1/ping" {
                return None;
            }
            if let Err(e) = write_raw_bytes(writer, req.body) {
                return Some(Err(e));
            }
            Some(finish_writer_response(writer, None))
        }
    }

    impl<T: HttpService> AsyncHttpService for &T {
        async fn call_http<'a, 's, 'b>(
            &self,
            req: &HttpRequest<'a>,
            scratch: &'s mut [u8],
            writer: &mut BackwardWriter<'b>,
        ) -> Option<Result<HttpResponse<'b>, GrpcStatus<'static>>> {
            (*self).call_http(req, scratch, writer)
        }
    }

    #[test]
    fn async_http_router_tuple_dispatch() {
        let echo = EchoHttpService;
        let router = (&echo, AsyncPingHttpService);
        let mut scratch = [0u8; 64];
        let mut out_buf = [0u8; 64];

        crate::transport::block_on_pair(
            async {
                let mut writer = BackwardWriter::new(&mut out_buf);
                let req = HttpRequest::from_uri(HttpMethod::Post, "/v1/ping", None, b"\x08\x07");
                let resp = router
                    .dispatch_http_async(&req, &mut scratch, &mut writer)
                    .await
                    .unwrap()
                    .unwrap();
                assert_eq!(resp.status_code, 200);
                assert_eq!(resp.body, b"\x08\x07");
            },
            async {},
        );
    }

    #[test]
    fn h2_preface_detection() {
        assert_eq!(is_h2_client_preface(b""), None);
        assert_eq!(is_h2_client_preface(b"P"), None);
        assert_eq!(is_h2_client_preface(b"PR"), None);
        assert_eq!(is_h2_client_preface(b"PRI"), None);
        assert_eq!(is_h2_client_preface(b"PRI "), Some(true));
        assert_eq!(is_h2_client_preface(b"PRI * "), Some(true));
        assert_eq!(is_h2_client_preface(CLIENT_CONNECTION_PREFACE), Some(true));
        assert_eq!(
            is_h2_client_preface(b"PRI * HTTP/2.0\r\n\r\nSM\r\n\r\n\x00\x00\x00\x04"),
            Some(true)
        );

        assert_eq!(is_h2_client_preface(b"G"), Some(false));
        assert_eq!(is_h2_client_preface(b"GET / HTTP/1.1\r\n"), Some(false));
        assert_eq!(is_h2_client_preface(b"PRINT / HTTP/1.1\r\n"), Some(false));
        assert_eq!(is_h2_client_preface(b"PRI / HTTP/1.1\r\n"), Some(false));
    }

    #[test]
    fn h1_request_parser_and_response_encoder() {
        let raw_get = b"GET /v1/items/7?filter=active HTTP/1.1\r\nHost: localhost\r\n\r\n";
        for split in 0..raw_get.len() {
            assert_eq!(parse_h1_request(&raw_get[..split]), Ok(None));
        }
        let parsed = parse_h1_request(raw_get).unwrap().unwrap();
        assert_eq!(parsed.consumed, raw_get.len());
        assert!(parsed.keep_alive);
        assert_eq!(parsed.request.method, HttpMethod::Get);
        assert_eq!(parsed.request.path, "/v1/items/7");
        assert_eq!(parsed.request.query, "filter=active");
        assert_eq!(parsed.request.content_type, None);
        assert_eq!(parsed.request.body, b"");

        // POST with Content-Length, Content-Type, Connection: close, and pipelined trailing bytes
        let raw_post = b"POST /v1/items:create HTTP/1.1\r\n\
Content-Type: application/x-protobuf\r\n\
Content-Length: 5\r\n\
Connection: close\r\n\
\r\n\
\x08\x2a\x12\x01xGET /next HTTP/1.1\r\n\r\n";

        // Partial body returns Ok(None)
        let header_only_len = raw_post.len() - b"GET /next HTTP/1.1\r\n\r\n".len() - 2;
        assert_eq!(parse_h1_request(&raw_post[..header_only_len]), Ok(None));

        let parsed_post = parse_h1_request(raw_post).unwrap().unwrap();
        assert!(!parsed_post.keep_alive);
        assert_eq!(parsed_post.request.method, HttpMethod::Post);
        assert_eq!(parsed_post.request.path, "/v1/items:create");
        assert_eq!(
            parsed_post.request.content_type,
            Some(CONTENT_TYPE_PROTOBUF)
        );
        assert_eq!(parsed_post.request.body, b"\x08\x2a\x12\x01x");
        assert_eq!(
            &raw_post[parsed_post.consumed..],
            b"GET /next HTTP/1.1\r\n\r\n"
        );

        // HTTP/1.0 default keep-alive is false unless Connection: keep-alive is set
        let raw_10 = b"GET /v1/ping HTTP/1.0\r\n\r\n";
        let parsed_10 = parse_h1_request(raw_10).unwrap().unwrap();
        assert!(!parsed_10.keep_alive);

        let raw_10_ka = b"GET /v1/ping HTTP/1.0\r\nConnection: keep-alive\r\n\r\n";
        let parsed_10_ka = parse_h1_request(raw_10_ka).unwrap().unwrap();
        assert!(parsed_10_ka.keep_alive);

        // Reject chunked Transfer-Encoding and malformed inputs
        let bad_cases: &[&[u8]] = &[
            b"INVALID_LINE\r\n\r\n",
            b"GET / HTTP/2.0\r\n\r\n",
            b"GET relative/path HTTP/1.1\r\n\r\n",
            b"GET / HTTP/1.1\r\nBadHeaderWithoutColon\r\n\r\n",
            b"GET / HTTP/1.1\r\nContent-Length: abc\r\n\r\n",
            b"GET / HTTP/1.1\r\nContent-Length: 4\r\nContent-Length: 5\r\n\r\n12345",
            b"POST / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n",
        ];
        for &bad in bad_cases {
            let err = parse_h1_request(bad).unwrap_err();
            assert_eq!(err.code, GrpcCode::InvalidArgument);
        }

        // Response encoding
        let mut out = [0u8; 256];
        let http_resp = HttpResponse::ok_protobuf(b"\x08\x01");
        let n = encode_h1_http_response(&http_resp, true, &mut out).unwrap();
        assert_eq!(
            &out[..n],
            b"HTTP/1.1 200 OK\r\nContent-Type: application/x-protobuf\r\nContent-Length: 2\r\nConnection: keep-alive\r\n\r\n\x08\x01"
        );

        let err_status = GrpcStatus::not_found("book not found");
        let n_err = encode_h1_error(&err_status, false, &mut out).unwrap();
        assert_eq!(
            &out[..n_err],
            b"HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 14\r\nConnection: close\r\n\r\nbook not found"
        );

        let mut tiny_out = [0u8; 8];
        assert_eq!(
            encode_h1_error(&err_status, false, &mut tiny_out),
            Err(EncodeError::BufferOverflow)
        );
    }
}
