use google_protobuf_roto::google::protobuf::FileDescriptorProtoView;
use grpc_reflection_roto::grpc::reflection::{v1, v1alpha};
use grpc_reflection_roto::services::grpc::reflection::{v1 as v1_grpc, v1alpha as v1alpha_grpc};
use roto_grpc::{
    block_on_pair, serve_async_connection, AsyncClientTransport, AsyncGrpcServer, GrpcClient,
    GrpcCode, GrpcServer, GrpcStatus, H2LoopbackChannel, MemoryDuplex, RecvStream, Route,
    SendStream,
};

#[test]
fn v1_and_v1alpha_unstripped_file_descriptors_decode_with_source_code_info() {
    assert_eq!(grpc_reflection_roto::FILE_DESCRIPTORS.len(), 2);

    let fd_v1 = FileDescriptorProtoView::from_bytes(v1::FILE_DESCRIPTOR_BYTES)
        .expect("decode v1 FileDescriptorProto");
    assert_eq!(fd_v1.name(), Ok("grpc/reflection/v1/reflection.proto"));
    assert_eq!(fd_v1.package(), Ok("grpc.reflection.v1"));
    assert_eq!(fd_v1.syntax(), Ok("proto3"));
    assert!(
        fd_v1.has_source_code_info(),
        "v1 FileDescriptorProto must preserve unstripped source_code_info"
    );
    let v1_services: Vec<&str> = fd_v1
        .service()
        .map(|s| s.expect("decode service").name().expect("service name"))
        .collect();
    assert_eq!(v1_services, vec!["ServerReflection"]);
    let v1_messages: Vec<&str> = fd_v1
        .message_type()
        .map(|m| m.expect("decode message").name().expect("message name"))
        .collect();
    assert_eq!(
        v1_messages,
        vec![
            "ServerReflectionRequest",
            "ExtensionRequest",
            "ServerReflectionResponse",
            "FileDescriptorResponse",
            "ExtensionNumberResponse",
            "ListServiceResponse",
            "ServiceResponse",
            "ErrorResponse",
        ]
    );

    let fd_v1alpha = FileDescriptorProtoView::from_bytes(v1alpha::FILE_DESCRIPTOR_BYTES)
        .expect("decode v1alpha FileDescriptorProto");
    assert_eq!(
        fd_v1alpha.name(),
        Ok("grpc/reflection/v1alpha/reflection.proto")
    );
    assert_eq!(fd_v1alpha.package(), Ok("grpc.reflection.v1alpha"));
    assert_eq!(fd_v1alpha.syntax(), Ok("proto3"));
    assert!(
        fd_v1alpha.has_source_code_info(),
        "v1alpha FileDescriptorProto must preserve unstripped source_code_info"
    );

    assert_eq!(
        v1_grpc::server_reflection::FILE_DESCRIPTOR_BYTES,
        v1::FILE_DESCRIPTOR_BYTES
    );
    assert_eq!(
        v1_grpc::server_reflection::FILE_DESCRIPTOR_SET,
        &[v1::FILE_DESCRIPTOR_BYTES]
    );
    assert_eq!(
        v1alpha_grpc::server_reflection::FILE_DESCRIPTOR_BYTES,
        v1alpha::FILE_DESCRIPTOR_BYTES
    );
    assert_eq!(
        v1alpha_grpc::server_reflection::FILE_DESCRIPTOR_SET,
        &[v1alpha::FILE_DESCRIPTOR_BYTES]
    );
}

#[test]
fn v1_and_v1alpha_request_and_response_oneof_variants_round_trip() {
    use v1::server_reflection_request::MessageRequest as V1Req;
    use v1::server_reflection_response::MessageResponse as V1Resp;
    use v1alpha::server_reflection_request::MessageRequest as V1AlphaReq;
    use v1alpha::server_reflection_response::MessageResponse as V1AlphaResp;

    // 1. v1 request variants: file_by_filename, file_containing_symbol, file_containing_extension,
    //    all_extension_numbers_of_type, list_services
    let mut buf = [0u8; 512];
    let mut req_b = v1::ServerReflectionRequestBuilder::new(&mut buf);
    req_b
        .write_file_containing_extension(|ext| {
            ext.write_extension_number(1042)
                .write_containing_type("telemetry.v1.SensorReading");
        })
        .write_host("edge.local");
    let req_bytes = req_b.finish().unwrap();
    let req_view = v1::ServerReflectionRequestView::from_bytes(req_bytes).unwrap();
    assert_eq!(req_view.host(), Ok("edge.local"));
    match req_view.message_request().unwrap() {
        Some(V1Req::FileContainingExtension(ext)) => {
            assert_eq!(ext.containing_type(), Ok("telemetry.v1.SensorReading"));
            assert_eq!(ext.extension_number(), Ok(1042));
        }
        other => panic!("unexpected v1 request oneof: {other:?}"),
    }

    // 2. v1 response variants: file_descriptor_response, all_extension_numbers_response,
    //    list_services_response, error_response + original_request
    let mut resp_buf = [0u8; 16384];
    let mut resp_b = v1::ServerReflectionResponseBuilder::new(&mut resp_buf);
    resp_b
        .write_file_descriptor_response(|fdr| {
            fdr.write_file_descriptor_proto(v1::FILE_DESCRIPTOR_BYTES);
        })
        .write_original_request(|orig| {
            orig.write_file_by_filename("grpc/reflection/v1/reflection.proto")
                .write_host("edge.local");
        })
        .write_valid_host("edge.local");
    let resp_bytes = resp_b.finish().unwrap();
    let resp_view = v1::ServerReflectionResponseView::from_bytes(resp_bytes).unwrap();
    assert_eq!(resp_view.valid_host(), Ok("edge.local"));
    let orig = resp_view
        .original_request()
        .unwrap()
        .expect("original_request present");
    assert!(matches!(
        orig.message_request().unwrap(),
        Some(V1Req::FileByFilename("grpc/reflection/v1/reflection.proto"))
    ));
    match resp_view.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr
                .file_descriptor_proto()
                .map(|b| b.expect("decode bytes"))
                .collect();
            assert_eq!(protos, vec![v1::FILE_DESCRIPTOR_BYTES]);
        }
        other => panic!("unexpected v1 response oneof: {other:?}"),
    }

    // 3. v1alpha ExtensionNumberResponse & ErrorResponse
    let mut alpha_req_buf = [0u8; 256];
    let mut alpha_req_b = v1alpha::ServerReflectionRequestBuilder::new(&mut alpha_req_buf);
    alpha_req_b
        .write_all_extension_numbers_of_type("telemetry.v1.SensorReading")
        .write_host("alpha.local");
    let alpha_req_view =
        v1alpha::ServerReflectionRequestView::from_bytes(alpha_req_b.finish().unwrap()).unwrap();
    assert!(matches!(
        alpha_req_view.message_request().unwrap(),
        Some(V1AlphaReq::AllExtensionNumbersOfType(
            "telemetry.v1.SensorReading"
        ))
    ));

    let mut alpha_resp_buf = [0u8; 256];
    let mut alpha_resp_b = v1alpha::ServerReflectionResponseBuilder::new(&mut alpha_resp_buf);
    alpha_resp_b
        .write_all_extension_numbers_response(|enr| {
            enr.write_packed_extension_number(&[100, 101, 205])
                .write_base_type_name("telemetry.v1.SensorReading");
        })
        .write_valid_host("alpha.local");
    let alpha_resp_view =
        v1alpha::ServerReflectionResponseView::from_bytes(alpha_resp_b.finish().unwrap()).unwrap();
    match alpha_resp_view.message_response().unwrap() {
        Some(V1AlphaResp::AllExtensionNumbersResponse(enr)) => {
            assert_eq!(enr.base_type_name(), Ok("telemetry.v1.SensorReading"));
            let nums: Vec<i32> = enr
                .extension_number()
                .map(|n| n.expect("decode ext number"))
                .collect();
            assert_eq!(nums, vec![100, 101, 205]);
        }
        other => panic!("unexpected v1alpha response oneof: {other:?}"),
    }

    let mut err_buf = [0u8; 256];
    let mut err_b = v1alpha::ServerReflectionResponseBuilder::new(&mut err_buf);
    err_b.write_error_response(|err| {
        err.write_error_message("symbol not found")
            .write_error_code(GrpcCode::NotFound as i32);
    });
    let err_view =
        v1alpha::ServerReflectionResponseView::from_bytes(err_b.finish().unwrap()).unwrap();
    match err_view.message_response().unwrap() {
        Some(V1AlphaResp::ErrorResponse(err)) => {
            assert_eq!(err.error_code(), Ok(GrpcCode::NotFound as i32));
            assert_eq!(err.error_message(), Ok("symbol not found"));
        }
        other => panic!("unexpected v1alpha error response: {other:?}"),
    }
}

struct MockReflectionService;

impl v1_grpc::ServerReflection for MockReflectionService {
    fn server_reflection_info<'a, 'b>(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        rx: &mut impl RecvStream<v1::ServerReflectionRequestView<'a>>,
        tx: &mut impl SendStream<v1::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        use v1::server_reflection_request::MessageRequest;
        while let Some(req) = rx
            .next()
            .map_err(|_| GrpcStatus::internal("decode v1 request"))?
        {
            let host = req.host().unwrap_or("");
            let req_kind = req
                .message_request()
                .map_err(|_| GrpcStatus::internal("decode message_request"))?;
            tx.send(|resp| {
                match req_kind {
                    Some(MessageRequest::ListServices(_)) => {
                        resp.write_list_services_response(|ls| {
                            ls.write_service(|s| {
                                s.write_name(v1alpha_grpc::server_reflection::SERVICE_NAME);
                            })
                            .write_service(|s| {
                                s.write_name(v1_grpc::server_reflection::SERVICE_NAME);
                            });
                        });
                    }
                    Some(MessageRequest::FileByFilename("grpc/reflection/v1/reflection.proto")) => {
                        resp.write_file_descriptor_response(|fdr| {
                            fdr.write_file_descriptor_proto(v1::FILE_DESCRIPTOR_BYTES);
                        });
                    }
                    _ => {
                        resp.write_error_response(|err| {
                            err.write_error_message("not found")
                                .write_error_code(GrpcCode::NotFound as i32);
                        });
                    }
                }
                resp.write_valid_host(host);
            })?;
        }
        Ok(())
    }
}

impl v1alpha_grpc::AsyncServerReflection for MockReflectionService {
    async fn server_reflection_info<'a, 'r, 'b>(
        &'a self,
        _meta: &'a mut roto_grpc::RpcMetadata<'_>,
        rx: &'a mut impl RecvStream<v1alpha::ServerReflectionRequestView<'r>>,
        tx: &'a mut impl SendStream<v1alpha::ServerReflectionResponseBuilder<'b>>,
    ) -> Result<(), GrpcStatus<'static>> {
        while let Some(req) = rx
            .next()
            .map_err(|_| GrpcStatus::internal("decode v1alpha request"))?
        {
            let host = req.host().unwrap_or("");
            tx.send(|resp| {
                resp.write_list_services_response(|ls| {
                    ls.write_service(|s| {
                        s.write_name(v1alpha_grpc::server_reflection::SERVICE_NAME);
                    });
                })
                .write_valid_host(host);
            })?;
        }
        Ok(())
    }
}

#[test]
fn generated_v1_and_v1alpha_bidi_streaming_stubs_round_trip_over_loopback_and_async_transport() {
    use v1::server_reflection_response::MessageResponse as V1Resp;
    use v1alpha::server_reflection_response::MessageResponse as V1AlphaResp;

    // 1. Sync v1 ServerReflectionServer + ServerReflectionClient over H2LoopbackChannel
    let v1_srv = v1_grpc::ServerReflectionServer::new(MockReflectionService);
    assert_eq!(v1_srv.file_descriptors(), &[v1::FILE_DESCRIPTOR_BYTES]);
    let routes: [&dyn Route; 1] = [&v1_srv];
    let mut call_buf = [0u8; 16384];
    let mut srv_msg_buf = [0u8; 16384];
    let mut srv_tx_buf = [0u8; 32768];
    let mut cli_tx_buf = [0u8; 4096];

    let server: GrpcServer<4, 16384> = GrpcServer::new(&routes, &mut srv_msg_buf, &mut srv_tx_buf);
    let client: GrpcClient<4, 16384> = GrpcClient::new("http", "loopback.v1", &mut cli_tx_buf);
    let channel = H2LoopbackChannel::new(client, server, &mut call_buf);
    let mut v1_client = v1_grpc::ServerReflectionClient::new(channel);
    let mut stream = v1_client
        .server_reflection_info(|tx| {
            tx.send(|req| {
                req.write_list_services("*").write_host("loopback.v1");
            })?;
            tx.send(|req| {
                req.write_file_by_filename("grpc/reflection/v1/reflection.proto")
                    .write_host("loopback.v1");
            })?;
            tx.send(|req| {
                req.write_file_by_filename("unknown.proto")
                    .write_host("loopback.v1");
            })
        })
        .expect("bidi call");

    let r1 = stream.next().expect("decode resp 1").expect("resp 1");
    assert_eq!(r1.valid_host(), Ok("loopback.v1"));
    match r1.message_response().unwrap() {
        Some(V1Resp::ListServicesResponse(ls)) => {
            let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
            assert_eq!(
                names,
                vec![
                    "grpc.reflection.v1.ServerReflection",
                    "grpc.reflection.v1alpha.ServerReflection"
                ]
            );
        }
        other => panic!("expected ListServicesResponse, got {other:?}"),
    }

    let r2 = stream.next().expect("decode resp 2").expect("resp 2");
    match r2.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![v1::FILE_DESCRIPTOR_BYTES]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    let r3 = stream.next().expect("decode resp 3").expect("resp 3");
    match r3.message_response().unwrap() {
        Some(V1Resp::ErrorResponse(err)) => {
            assert_eq!(err.error_code(), Ok(GrpcCode::NotFound as i32));
        }
        other => panic!("expected ErrorResponse, got {other:?}"),
    }
    assert!(stream.next().expect("eof").is_none());

    // 2. Async v1alpha ServerReflectionServer + ServerReflectionClient over MemoryDuplex
    let v1alpha_srv = v1alpha_grpc::ServerReflectionServer::new(MockReflectionService);
    let duplex: MemoryDuplex<2048> = MemoryDuplex::new();
    let (client_io, mut server_io) = duplex.split();
    let mut a_msg = [0u8; 512];
    let mut a_stx = [0u8; 2048];
    let mut a_ctx = [0u8; 2048];
    let mut a_call = [0u8; 512];
    let mut async_server: AsyncGrpcServer<_, 4, 512> =
        AsyncGrpcServer::new(&v1alpha_srv, &mut a_msg, &mut a_stx);
    let client: GrpcClient<4, 512> = GrpcClient::new("http", "alpha.local", &mut a_ctx);

    let (client_res, srv_res) = block_on_pair(
        async move {
            let mut transport: AsyncClientTransport<_, 4, 512, 512> =
                AsyncClientTransport::new_with_io_buf(client_io, client, &mut a_call);
            let mut stub = v1alpha_grpc::ServerReflectionClient::new(&mut transport);
            {
                let mut stream = stub
                    .server_reflection_info_async(|tx| {
                        tx.send(|req| {
                            req.write_list_services("*").write_host("alpha.local");
                        })
                    })
                    .await
                    .expect("v1alpha async bidi call");
                let resp = stream.next().expect("decode").expect("resp");
                assert_eq!(resp.valid_host(), Ok("alpha.local"));
                match resp.message_response().unwrap() {
                    Some(V1AlphaResp::ListServicesResponse(ls)) => {
                        let names: Vec<&str> =
                            ls.service().map(|s| s.unwrap().name().unwrap()).collect();
                        assert_eq!(names, vec!["grpc.reflection.v1alpha.ServerReflection"]);
                    }
                    other => panic!("unexpected v1alpha response: {other:?}"),
                }
                assert!(stream.next().expect("eof").is_none());
            }
            transport
                .send_goaway(roto_grpc::H2ErrorCode::NoError, b"done")
                .await
                .unwrap();
        },
        serve_async_connection(&mut async_server, &mut server_io),
    );
    let () = client_res;
    assert!(srv_res.is_ok());
}
