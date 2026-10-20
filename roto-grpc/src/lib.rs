#![no_std]
#![forbid(unsafe_code)]

pub mod framing;
pub mod status;

pub use framing::{
    decode_grpc_frame, encode_grpc_message, iter_grpc_frames, write_grpc_frame,
    write_grpc_frame_bytes, GrpcFrame, GrpcFrameIter, LpmDecoder, DEFAULT_MAX_MESSAGE_SIZE,
    GRPC_HEADER_LEN,
};
pub use status::{
    is_unreserved_grpc_message_byte, percent_decode, percent_encode, percent_encode_backward,
    percent_encoded_len, try_percent_decode_in_place, GrpcCode, GrpcStatus,
};
