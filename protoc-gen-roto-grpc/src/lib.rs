#![forbid(unsafe_code)]

pub mod generator;

use std::io::{Read, Write};

use google_protobuf_roto_bootstrap::google::protobuf::compiler::{
    code_generator_response::Feature, CodeGeneratorRequestView, CodeGeneratorResponseBuilder,
};
pub use protoc_gen_roto::{GeneratedFile, PluginError};
use roto::runtime::EncodeError;

use crate::generator::GrpcGeneratorConfig;

/// Reads a `CodeGeneratorRequest` from `reader`, processes it, and writes the
/// serialized `CodeGeneratorResponse` to `writer`.
pub fn run_stdio<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<(), PluginError> {
    let mut stdin_buf = Vec::new();
    reader.read_to_end(&mut stdin_buf)?;

    let response_bytes = process_bytes(&stdin_buf)?;
    writer.write_all(&response_bytes)?;
    writer.flush()?;
    Ok(())
}

/// Parses `stdin_buf` as a `CodeGeneratorRequestView`, executes the gRPC service generator,
/// and serializes the `CodeGeneratorResponse` using `CodeGeneratorResponseBuilder`.
pub fn process_bytes(stdin_buf: &[u8]) -> Result<Vec<u8>, PluginError> {
    let request = CodeGeneratorRequestView::from_bytes(stdin_buf).map_err(PluginError::Framing)?;

    let mut capacity = stdin_buf.len().saturating_mul(2).max(4096);
    loop {
        let mut backing = vec![0u8; capacity];
        match handle_request(&request, &mut backing) {
            Ok(bytes) => return Ok(bytes.to_vec()),
            Err(EncodeError::BufferOverflow) => {
                capacity = capacity
                    .checked_mul(2)
                    .ok_or(PluginError::Encode(EncodeError::BufferOverflow))?;
            }
        }
    }
}

/// Processes a parsed `CodeGeneratorRequestView` and writes the `CodeGeneratorResponse`
/// into `buf` using `CodeGeneratorResponseBuilder`.
pub fn handle_request<'b>(
    request: &CodeGeneratorRequestView<'_>,
    buf: &'b mut [u8],
) -> Result<&'b [u8], EncodeError> {
    let mut resp = CodeGeneratorResponseBuilder::new(buf);

    match generate_request(request) {
        Ok(files) => {
            for file in files.iter().rev() {
                resp.write_file(|fb| {
                    fb.write_content(&file.content).write_name(&file.name);
                })
                .check()?;
            }
        }
        Err(err_msg) => {
            resp.write_error(&err_msg);
        }
    }

    resp.write_supported_features(Feature::FeatureProto3Optional as u64);
    resp.finish()
}

/// Validates all descriptors in `request` and invokes the `protoc-gen-roto-grpc` code generator.
pub fn generate_request(
    request: &CodeGeneratorRequestView<'_>,
) -> Result<Vec<GeneratedFile>, String> {
    generator::generate(request)
}

/// Validates a `CodeGeneratorRequestView` for `protoc-gen-roto-grpc`.
pub fn validate_request(request: &CodeGeneratorRequestView<'_>) -> Result<(), String> {
    let _config = GrpcGeneratorConfig::from_request(request)?;
    protoc_gen_roto::validate_request_descriptors(request)
}
