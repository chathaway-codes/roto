#![no_std]
#![forbid(unsafe_code)]

pub mod framing;
pub mod hpack;
pub mod status;

pub use framing::{
    decode_grpc_frame, encode_grpc_message, iter_grpc_frames, write_grpc_frame,
    write_grpc_frame_bytes, GrpcFrame, GrpcFrameIter, LpmDecoder, DEFAULT_MAX_MESSAGE_SIZE,
    GRPC_HEADER_LEN,
};
pub use hpack::{
    decode_header_block, decode_prefix_int, decode_string_literal,
    encode_dynamic_table_size_update, encode_grpc_request_headers, encode_grpc_response_headers,
    encode_grpc_trailers, encode_grpc_trailers_backward, encode_grpc_trailers_only, encode_indexed,
    encode_indexed_backward, encode_literal, encode_literal_backward, encode_literal_ext,
    encode_literal_indexed_name, encode_literal_indexed_name_backward,
    encode_literal_indexed_name_ext, encode_prefix_int, encode_prefix_int_backward,
    encode_string_literal, encode_string_literal_backward, huffman_decode, huffman_encode,
    huffman_encoded_len, is_grpc_content_type, static_table_get, DynamicTable, GrpcRequestHeaders,
    GrpcResponseHeaders, GrpcTrailers, HeaderField, HpackDecoder, HpackEncoder, HpackError,
    DEFAULT_DYNAMIC_TABLE_CAP, DYNAMIC_ENTRY_OVERHEAD, GRPC_CONTENT_TYPE, GRPC_CONTENT_TYPE_ROTO,
    HUFFMAN_BIT_LENS, HUFFMAN_CODES, INDEX_ACCEPT_ENCODING, INDEX_AUTHORITY,
    INDEX_CONTENT_ENCODING, INDEX_CONTENT_LENGTH, INDEX_CONTENT_TYPE, INDEX_METHOD_GET,
    INDEX_METHOD_POST, INDEX_PATH_INDEX_HTML, INDEX_PATH_ROOT, INDEX_SCHEME_HTTP,
    INDEX_SCHEME_HTTPS, INDEX_STATUS_200, INDEX_STATUS_204, INDEX_STATUS_206, INDEX_STATUS_304,
    INDEX_STATUS_400, INDEX_STATUS_404, INDEX_STATUS_500, INDEX_USER_AGENT, STATIC_TABLE,
    STATIC_TABLE_LEN,
};
pub use status::{
    is_unreserved_grpc_message_byte, percent_decode, percent_encode, percent_encode_backward,
    percent_encoded_len, try_percent_decode_in_place, GrpcCode, GrpcStatus,
};
