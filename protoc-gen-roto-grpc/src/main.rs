#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use google_protobuf_roto_bootstrap::google::protobuf::compiler::CodeGeneratorRequestView;
use protoc_gen_roto_grpc::handle_request;
use roto::runtime::EncodeError;

fn main() -> ExitCode {
    let mut stdin_buf = Vec::new();
    if let Err(err) = io::stdin().read_to_end(&mut stdin_buf) {
        eprintln!("protoc-gen-roto-grpc: failed to read stdin: {err}");
        return ExitCode::from(1);
    }

    let request = match CodeGeneratorRequestView::from_bytes(&stdin_buf) {
        Ok(req) => req,
        Err(err) => {
            eprintln!(
                "protoc-gen-roto-grpc: failed to decode CodeGeneratorRequest from stdin: {err}"
            );
            return ExitCode::from(1);
        }
    };

    let mut capacity = stdin_buf.len().saturating_mul(2).max(4096);
    loop {
        let mut backing = vec![0u8; capacity];
        match handle_request(&request, &mut backing) {
            Ok(bytes) => {
                let mut stdout = io::stdout().lock();
                if let Err(err) = stdout.write_all(bytes).and_then(|()| stdout.flush()) {
                    eprintln!(
                        "protoc-gen-roto-grpc: failed to write CodeGeneratorResponse to stdout: {err}"
                    );
                    return ExitCode::from(1);
                }
                return ExitCode::SUCCESS;
            }
            Err(EncodeError::BufferOverflow) => {
                capacity = match capacity.checked_mul(2) {
                    Some(c) => c,
                    None => {
                        eprintln!("protoc-gen-roto-grpc: response buffer overflow");
                        return ExitCode::from(1);
                    }
                };
            }
        }
    }
}
