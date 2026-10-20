#![forbid(unsafe_code)]

use std::io::{self, Read, Write};
use std::process::ExitCode;

use google_protobuf_roto::google::protobuf::compiler::CodeGeneratorRequestView;
use protoc_gen_roto::handle_request;
use roto::runtime::{BackwardWriter, EncodeError};

fn main() -> ExitCode {
    let mut stdin_buf = Vec::new();
    if let Err(err) = io::stdin().read_to_end(&mut stdin_buf) {
        eprintln!("protoc-gen-roto: failed to read stdin: {err}");
        return ExitCode::from(1);
    }

    let request = match CodeGeneratorRequestView::from_bytes(&stdin_buf) {
        Ok(req) => req,
        Err(err) => {
            eprintln!("protoc-gen-roto: failed to decode CodeGeneratorRequest from stdin: {err}");
            return ExitCode::from(1);
        }
    };

    let mut capacity = stdin_buf.len().saturating_mul(2).max(4096);
    loop {
        let mut backing = vec![0u8; capacity];
        let mut writer = BackwardWriter::new(&mut backing);
        match handle_request(&request, &mut writer) {
            Ok(()) => {
                let mut stdout = io::stdout().lock();
                if let Err(err) = stdout
                    .write_all(writer.finish())
                    .and_then(|()| stdout.flush())
                {
                    eprintln!(
                        "protoc-gen-roto: failed to write CodeGeneratorResponse to stdout: {err}"
                    );
                    return ExitCode::from(1);
                }
                return ExitCode::SUCCESS;
            }
            Err(EncodeError::BufferOverflow) => {
                capacity = match capacity.checked_mul(2) {
                    Some(c) => c,
                    None => {
                        eprintln!("protoc-gen-roto: response buffer overflow");
                        return ExitCode::from(1);
                    }
                };
            }
        }
    }
}
