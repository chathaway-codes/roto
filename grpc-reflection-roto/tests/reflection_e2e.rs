use google_protobuf_roto::google::protobuf::{
    field_descriptor_proto::{Label, Type},
    FileDescriptorProtoBuilder, FileDescriptorProtoView,
};
use grpc_reflection_roto::{
    grpc::reflection::{v1, v1alpha},
    services::grpc::reflection::{v1 as v1_grpc, v1alpha as v1alpha_grpc},
    ReflectionService,
};
use roto::runtime::BackwardWriter;
use roto_grpc::{
    block_on_pair, serve_async_connection, AsyncClientTransport, AsyncGrpcServer, GrpcClient,
    GrpcCode, GrpcServer, GrpcStatus, H2ErrorCode, H2LoopbackChannel, MemoryDuplex, Route,
};

fn build_geo_descriptor(buf: &mut [u8]) -> &[u8] {
    let mut b = FileDescriptorProtoBuilder::new(buf);
    b.write_enum_type(|en| {
        en.write_value(|v| {
            v.write_number(1).write_name("FRAME_ECEF");
        })
        .write_value(|v| {
            v.write_number(0).write_name("FRAME_WGS84");
        })
        .write_name("CoordinateFrame");
    })
    .write_message_type(|msg| {
        msg.write_field(|f| {
            f.write_type(Type::TypeInt32)
                .write_label(Label::LabelOptional)
                .write_number(2)
                .write_name("lon_e7");
        })
        .write_field(|f| {
            f.write_type(Type::TypeInt32)
                .write_label(Label::LabelOptional)
                .write_number(1)
                .write_name("lat_e7");
        })
        .write_name("LatLng");
    })
    .write_syntax("proto3")
    .write_package("common.geo")
    .write_name("common/geo.proto");
    b.finish().expect("encode geo descriptor")
}

fn build_sensor_descriptor(buf: &mut [u8]) -> &[u8] {
    let mut b = FileDescriptorProtoBuilder::new(buf);
    b.write_extension(|ext| {
        ext.write_type(Type::TypeInt32)
            .write_label(Label::LabelOptional)
            .write_extendee(".telemetry.v1.SensorReading")
            .write_number(101)
            .write_name("battery_mv_ext");
    })
    .write_extension(|ext| {
        ext.write_type(Type::TypeString)
            .write_label(Label::LabelOptional)
            .write_extendee(".telemetry.v1.SensorReading")
            .write_number(100)
            .write_name("firmware_tag_ext");
    })
    .write_service(|svc| {
        svc.write_method(|m| {
            m.write_server_streaming(true)
                .write_client_streaming(true)
                .write_output_type(".common.geo.LatLng")
                .write_input_type(".telemetry.v1.SensorReading")
                .write_name("StreamReadings");
        })
        .write_method(|m| {
            m.write_output_type(".common.geo.LatLng")
                .write_input_type(".telemetry.v1.SensorReading")
                .write_name("ReportLocation");
        })
        .write_name("SensorCollector");
    })
    .write_enum_type(|en| {
        en.write_value(|v| {
            v.write_number(1).write_name("POWER_SLEEP");
        })
        .write_value(|v| {
            v.write_number(0).write_name("POWER_ACTIVE");
        })
        .write_name("PowerMode");
    })
    .write_message_type(|msg| {
        msg.write_oneof_decl(|o| {
            o.write_name("payload");
        })
        .write_enum_type(|en| {
            en.write_value(|v| {
                v.write_number(1).write_name("STATUS_FAULT");
            })
            .write_value(|v| {
                v.write_number(0).write_name("STATUS_OK");
            })
            .write_name("Status");
        })
        .write_nested_type(|nested| {
            nested
                .write_extension(|ext| {
                    ext.write_type(Type::TypeInt32)
                        .write_label(Label::LabelOptional)
                        .write_extendee(".telemetry.v1.SensorReading")
                        .write_number(205)
                        .write_name("nested_cal_ext");
                })
                .write_field(|f| {
                    f.write_type(Type::TypeInt32)
                        .write_label(Label::LabelOptional)
                        .write_number(1)
                        .write_name("offset");
                })
                .write_name("Calibration");
        })
        .write_field(|f| {
            f.write_oneof_index(0)
                .write_type(Type::TypeBytes)
                .write_label(Label::LabelOptional)
                .write_number(4)
                .write_name("raw");
        })
        .write_field(|f| {
            f.write_oneof_index(0)
                .write_type(Type::TypeString)
                .write_label(Label::LabelOptional)
                .write_number(3)
                .write_name("text");
        })
        .write_field(|f| {
            f.write_type_name(".common.geo.LatLng")
                .write_type(Type::TypeMessage)
                .write_label(Label::LabelOptional)
                .write_number(2)
                .write_name("location");
        })
        .write_field(|f| {
            f.write_type(Type::TypeString)
                .write_label(Label::LabelOptional)
                .write_number(1)
                .write_name("device_id");
        })
        .write_name("SensorReading");
    })
    .write_dependency("common/geo.proto")
    .write_syntax("proto3")
    .write_package("telemetry.v1")
    .write_name("telemetry/v1/sensor.proto");
    b.finish().expect("encode sensor descriptor")
}

fn build_extra_ext_descriptor(buf: &mut [u8]) -> &[u8] {
    let mut b = FileDescriptorProtoBuilder::new(buf);
    b.write_extension(|ext| {
        ext.write_type(Type::TypeUint64)
            .write_label(Label::LabelOptional)
            .write_extendee(".telemetry.v1.SensorReading")
            .write_number(303)
            .write_name("hw_revision_ext");
    })
    .write_dependency("telemetry/v1/sensor.proto")
    .write_syntax("proto3")
    .write_package("telemetry.v1")
    .write_name("telemetry/v1/extra_ext.proto");
    b.finish().expect("encode extra_ext descriptor")
}

type LeakedDescriptors = (
    &'static [u8],
    &'static [u8],
    &'static [u8],
    &'static [&'static [u8]],
);

fn leaked_descriptors() -> LeakedDescriptors {
    use std::sync::OnceLock;
    static DESCS: OnceLock<LeakedDescriptors> = OnceLock::new();
    *DESCS.get_or_init(|| {
        let mut geo_buf = [0u8; 1024];
        let geo: &'static [u8] = Box::leak(
            build_geo_descriptor(&mut geo_buf)
                .to_vec()
                .into_boxed_slice(),
        );
        let mut sensor_buf = [0u8; 2048];
        let sensor: &'static [u8] = Box::leak(
            build_sensor_descriptor(&mut sensor_buf)
                .to_vec()
                .into_boxed_slice(),
        );
        let mut extra_buf = [0u8; 1024];
        let extra: &'static [u8] = Box::leak(
            build_extra_ext_descriptor(&mut extra_buf)
                .to_vec()
                .into_boxed_slice(),
        );
        let set: &'static [&'static [u8]] = Box::leak(vec![sensor, geo].into_boxed_slice());
        (geo, sensor, extra, set)
    })
}

struct StaticSensorService {
    descriptors: &'static [&'static [u8]],
}

impl Route for StaticSensorService {
    fn service_name(&self) -> &'static str {
        "telemetry.v1.SensorCollector"
    }

    fn file_descriptors(&self) -> &'static [&'static [u8]] {
        self.descriptors
    }

    fn dispatch(
        &self,
        _meta: &mut roto_grpc::RpcMetadata<'_>,
        _method_name: &str,
        _req_payload: &[u8],
        _writer: &mut BackwardWriter<'_>,
    ) -> Result<(), GrpcStatus<'static>> {
        Err(GrpcStatus::unimplemented("unimplemented"))
    }
}

#[test]
fn v1_reflection_full_query_suite_and_error_recovery_over_loopback() {
    use v1::server_reflection_request::MessageRequest as V1Req;
    use v1::server_reflection_response::MessageResponse as V1Resp;

    let (geo_bytes, sensor_bytes, extra_bytes, sensor_set) = leaked_descriptors();
    let sensor_route = StaticSensorService {
        descriptors: sensor_set,
    };
    let app_routes: [&dyn Route; 1] = [&sensor_route];
    let extra_files = [extra_bytes];
    let reflection = ReflectionService::new(&app_routes).with_extra_descriptors(&extra_files);
    assert!(format!("{reflection:?}").contains("ReflectionService"));
    let refl_v1 = reflection.v1_server();
    let refl_v1alpha = reflection.v1alpha_server();
    let server_routes: [&dyn Route; 3] = [&sensor_route, &refl_v1, &refl_v1alpha];

    let mut call_buf = [0u8; 32768];
    let mut srv_msg_buf = [0u8; 32768];
    let mut srv_tx_buf = [0u8; 65536];
    let mut cli_tx_buf = [0u8; 8192];

    let server: GrpcServer<2, 16384> =
        GrpcServer::new(&server_routes, &mut srv_msg_buf, &mut srv_tx_buf);
    let client: GrpcClient<2, 16384> = GrpcClient::new("http", "reflect.local", &mut cli_tx_buf);
    let channel = H2LoopbackChannel::new(client, server, &mut call_buf);
    let mut v1_client = v1_grpc::ServerReflectionClient::new(channel);

    let mut stream = v1_client
        .server_reflection_info(|tx| {
            // 1. list_services
            tx.send(|req| {
                req.write_list_services("*").write_host("reflect.local");
            })?;
            // 2. file_by_filename (with transitive dependency closure)
            tx.send(|req| {
                req.write_file_by_filename("telemetry/v1/extra_ext.proto")
                    .write_host("reflect.local");
            })?;
            // 3. file_containing_symbol: nested message
            tx.send(|req| {
                req.write_file_containing_symbol(".telemetry.v1.SensorReading.Calibration")
                    .write_host("reflect.local");
            })?;
            // 4. file_containing_symbol: service method
            tx.send(|req| {
                req.write_file_containing_symbol("telemetry.v1.SensorCollector.StreamReadings")
                    .write_host("reflect.local");
            })?;
            // 5. file_containing_symbol: top-level enum value & nested enum value
            tx.send(|req| {
                req.write_file_containing_symbol("common.geo.FRAME_ECEF")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_symbol("telemetry.v1.SensorReading.Status.STATUS_FAULT")
                    .write_host("reflect.local");
            })?;
            // 6. file_containing_symbol: top-level & nested extensions
            tx.send(|req| {
                req.write_file_containing_symbol("telemetry.v1.hw_revision_ext")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_symbol(
                    "telemetry.v1.SensorReading.Calibration.nested_cal_ext",
                )
                .write_host("reflect.local");
            })?;
            // 7. file_containing_extension (top-level, nested, and extra_ext)
            tx.send(|req| {
                req.write_file_containing_extension(|ext| {
                    ext.write_extension_number(100)
                        .write_containing_type("telemetry.v1.SensorReading");
                })
                .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_extension(|ext| {
                    ext.write_extension_number(205)
                        .write_containing_type(".telemetry.v1.SensorReading");
                })
                .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_extension(|ext| {
                    ext.write_extension_number(303)
                        .write_containing_type("telemetry.v1.SensorReading");
                })
                .write_host("reflect.local");
            })?;
            // 8. all_extension_numbers_of_type (with extensions and with 0 extensions)
            tx.send(|req| {
                req.write_all_extension_numbers_of_type("telemetry.v1.SensorReading")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_all_extension_numbers_of_type("common.geo.LatLng")
                    .write_host("reflect.local");
            })?;
            // 9. In-band error responses (unknown file, unknown symbol, unknown extension, unknown type, missing oneof)
            tx.send(|req| {
                req.write_file_by_filename("does/not/exist.proto")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_symbol("telemetry.v1.MissingSymbol")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_file_containing_extension(|ext| {
                    ext.write_extension_number(999)
                        .write_containing_type("telemetry.v1.SensorReading");
                })
                .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_all_extension_numbers_of_type("telemetry.v1.MissingType")
                    .write_host("reflect.local");
            })?;
            tx.send(|req| {
                req.write_host("reflect.local");
            })?;
            // 10. Verify stream continues cleanly after in-band errors by querying reflection.proto itself
            tx.send(|req| {
                req.write_file_containing_symbol("grpc.reflection.v1.ServerReflection")
                    .write_host("reflect.local");
            })
        })
        .expect("v1 reflection bidi call");

    // 1. list_services
    let r1 = stream.next().unwrap().unwrap();
    assert_eq!(r1.valid_host(), Ok("reflect.local"));
    assert!(matches!(
        r1.original_request()
            .unwrap()
            .unwrap()
            .message_request()
            .unwrap(),
        Some(V1Req::ListServices("*"))
    ));
    match r1.message_response().unwrap() {
        Some(V1Resp::ListServicesResponse(ls)) => {
            let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
            assert_eq!(
                names,
                vec![
                    "telemetry.v1.SensorCollector",
                    "grpc.reflection.v1.ServerReflection",
                    "grpc.reflection.v1alpha.ServerReflection",
                ]
            );
        }
        other => panic!("expected ListServicesResponse, got {other:?}"),
    }

    // 2. file_by_filename("telemetry/v1/extra_ext.proto") -> [extra_ext, sensor, geo]
    let r2 = stream.next().unwrap().unwrap();
    match r2.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![extra_bytes, sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 3. file_containing_symbol(".telemetry.v1.SensorReading.Calibration") -> [sensor, geo]
    let r3 = stream.next().unwrap().unwrap();
    match r3.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 4. file_containing_symbol("telemetry.v1.SensorCollector.StreamReadings") -> [sensor, geo]
    let r4 = stream.next().unwrap().unwrap();
    match r4.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 5a. file_containing_symbol("common.geo.FRAME_ECEF") -> [geo]
    let r5a = stream.next().unwrap().unwrap();
    match r5a.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 5b. file_containing_symbol("telemetry.v1.SensorReading.Status.STATUS_FAULT") -> [sensor, geo]
    let r5b = stream.next().unwrap().unwrap();
    match r5b.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 6a. file_containing_symbol("telemetry.v1.hw_revision_ext") -> [extra, sensor, geo]
    let r6a = stream.next().unwrap().unwrap();
    match r6a.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![extra_bytes, sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 6b. file_containing_symbol("telemetry.v1.SensorReading.Calibration.nested_cal_ext") -> [sensor, geo]
    let r6b = stream.next().unwrap().unwrap();
    match r6b.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 7a, 7b, 7c: file_containing_extension (100, 205, 303)
    let r7a = stream.next().unwrap().unwrap();
    match r7a.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }
    let r7b = stream.next().unwrap().unwrap();
    match r7b.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }
    let r7c = stream.next().unwrap().unwrap();
    match r7c.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![extra_bytes, sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    // 8a. all_extension_numbers_of_type("telemetry.v1.SensorReading") -> [100, 101, 205, 303]
    let r8a = stream.next().unwrap().unwrap();
    match r8a.message_response().unwrap() {
        Some(V1Resp::AllExtensionNumbersResponse(enr)) => {
            assert_eq!(enr.base_type_name(), Ok("telemetry.v1.SensorReading"));
            let nums: Vec<i32> = enr.extension_number().map(|n| n.unwrap()).collect();
            assert_eq!(nums, vec![100, 101, 205, 303]);
        }
        other => panic!("expected AllExtensionNumbersResponse, got {other:?}"),
    }

    // 8b. all_extension_numbers_of_type("common.geo.LatLng") -> []
    let r8b = stream.next().unwrap().unwrap();
    match r8b.message_response().unwrap() {
        Some(V1Resp::AllExtensionNumbersResponse(enr)) => {
            assert_eq!(enr.base_type_name(), Ok("common.geo.LatLng"));
            let nums: Vec<i32> = enr.extension_number().map(|n| n.unwrap()).collect();
            assert!(nums.is_empty());
        }
        other => panic!("expected empty AllExtensionNumbersResponse, got {other:?}"),
    }

    // 9a..9d: 4 NOT_FOUND ErrorResponses
    for _ in 0..4 {
        let err_resp = stream.next().unwrap().unwrap();
        match err_resp.message_response().unwrap() {
            Some(V1Resp::ErrorResponse(err)) => {
                assert_eq!(err.error_code(), Ok(GrpcCode::NotFound as i32));
                assert!(!err.error_message().unwrap().is_empty());
            }
            other => panic!("expected NOT_FOUND ErrorResponse, got {other:?}"),
        }
    }

    // 9e: INVALID_ARGUMENT ErrorResponse for empty message_request
    let err_empty = stream.next().unwrap().unwrap();
    match err_empty.message_response().unwrap() {
        Some(V1Resp::ErrorResponse(err)) => {
            assert_eq!(err.error_code(), Ok(GrpcCode::InvalidArgument as i32));
        }
        other => panic!("expected INVALID_ARGUMENT ErrorResponse, got {other:?}"),
    }

    // 10. Post-error recovery: file_containing_symbol("grpc.reflection.v1.ServerReflection")
    let r10 = stream.next().unwrap().unwrap();
    match r10.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![v1::FILE_DESCRIPTOR_BYTES]);
            let decoded = FileDescriptorProtoView::from_bytes(protos[0]).unwrap();
            assert!(decoded.has_source_code_info());
        }
        other => panic!("expected FileDescriptorResponse, got {other:?}"),
    }

    assert!(stream.next().unwrap().is_none());
}

#[test]
fn v1alpha_reflection_async_memory_duplex_e2e() {
    use v1alpha::server_reflection_response::MessageResponse as V1AlphaResp;

    let (geo_bytes, sensor_bytes, extra_bytes, sensor_set) = leaked_descriptors();

    let svc_names = ["telemetry.v1.SensorCollector"];
    let fd_sets = [sensor_set];
    let extra_files = [extra_bytes];
    let reflection = ReflectionService::from_descriptors(&svc_names, &fd_sets)
        .with_extra_descriptors(&extra_files)
        .with_include_reflection_services(false);
    let refl_v1alpha = reflection.into_v1alpha_server();

    let duplex: MemoryDuplex<8192> = MemoryDuplex::new();
    let (client_io, mut server_io) = duplex.split();
    let mut a_msg = [0u8; 16384];
    let mut a_stx = [0u8; 32768];
    let mut a_ctx = [0u8; 8192];
    let mut a_call = [0u8; 16384];
    let mut async_server: AsyncGrpcServer<_, 4, 8192> =
        AsyncGrpcServer::new(&refl_v1alpha, &mut a_msg, &mut a_stx);
    let client: GrpcClient<4, 16384> = GrpcClient::new("http", "alpha.local", &mut a_ctx);

    let ((), srv_res) = block_on_pair(
        async move {
            let mut transport: AsyncClientTransport<_, 4, 16384, 1024> =
                AsyncClientTransport::new_with_io_buf(client_io, client, &mut a_call);
            let mut stub = v1alpha_grpc::ServerReflectionClient::new(&mut transport);
            {
                let mut stream = stub
                    .server_reflection_info_async(|tx| {
                        tx.send(|req| {
                            req.write_list_services("*").write_host("alpha.local");
                        })?;
                        tx.send(|req| {
                            req.write_file_containing_symbol(
                                "telemetry.v1.SensorReading.device_id",
                            )
                            .write_host("alpha.local");
                        })?;
                        tx.send(|req| {
                            req.write_file_containing_symbol("telemetry.v1.SensorReading.payload")
                                .write_host("alpha.local");
                        })
                    })
                    .await
                    .expect("async v1alpha call");

                let r1 = stream.next().unwrap().unwrap();
                match r1.message_response().unwrap() {
                    Some(V1AlphaResp::ListServicesResponse(ls)) => {
                        let names: Vec<&str> =
                            ls.service().map(|s| s.unwrap().name().unwrap()).collect();
                        assert_eq!(names, vec!["telemetry.v1.SensorCollector"]);
                    }
                    other => panic!("expected ListServicesResponse, got {other:?}"),
                }

                for _ in 0..2 {
                    let r = stream.next().unwrap().unwrap();
                    match r.message_response().unwrap() {
                        Some(V1AlphaResp::FileDescriptorResponse(fdr)) => {
                            let protos: Vec<&[u8]> =
                                fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
                            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
                        }
                        other => panic!("expected FileDescriptorResponse, got {other:?}"),
                    }
                }
                assert!(stream.next().unwrap().is_none());
            }
            transport
                .send_goaway(H2ErrorCode::NoError, b"done")
                .await
                .unwrap();
        },
        serve_async_connection(&mut async_server, &mut server_io),
    );
    assert!(srv_res.is_ok());
}

#[cfg(feature = "std")]
#[test]
fn std_tcp_reflection_v1_and_v1alpha_e2e() {
    use roto_grpc::{accept_and_serve_std, connect_std_tcp};
    use std::net::{Shutdown, TcpListener};
    use v1alpha::server_reflection_response::MessageResponse as V1AlphaResp;

    let (geo_bytes, _sensor_bytes, extra_bytes, sensor_set) = leaked_descriptors();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind std TcpListener");
    let addr = listener.local_addr().unwrap();

    let server_thread = std::thread::spawn(move || {
        let sensor_route = StaticSensorService {
            descriptors: sensor_set,
        };
        let app_routes: [&dyn Route; 1] = [&sensor_route];
        let extra = [extra_bytes];
        let refl = ReflectionService::new(&app_routes).with_extra_descriptors(&extra);
        let v1_srv = refl.v1_server();
        let v1a_srv = refl.v1alpha_server();
        let routes: [&dyn Route; 3] = [&sensor_route, &v1_srv, &v1a_srv];
        let mut msg_buf = vec![0u8; 32768];
        let mut tx_buf = vec![0u8; 65536];
        let mut server: GrpcServer<2, 8192> = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);
        accept_and_serve_std(&listener, &mut server).expect("serve std connection");
    });

    let mut cli_tx = vec![0u8; 8192];
    let mut call_buf = vec![0u8; 16384];
    let mut transport =
        connect_std_tcp::<_, 2, 8192>(addr, "http", "localhost", &mut cli_tx, &mut call_buf)
            .expect("connect std tcp");

    {
        let mut v1_stub = v1_grpc::ServerReflectionClient::new(&mut transport);
        let mut s1 = v1_stub
            .server_reflection_info(|tx| {
                tx.send(|req| {
                    req.write_file_by_filename("common/geo.proto")
                        .write_host("localhost");
                })
            })
            .expect("std tcp v1 call");
        let resp = s1.next().unwrap().unwrap();
        match resp.message_response().unwrap() {
            Some(v1::server_reflection_response::MessageResponse::FileDescriptorResponse(fdr)) => {
                let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
                assert_eq!(protos, vec![geo_bytes]);
            }
            other => panic!("unexpected v1 response: {other:?}"),
        }
        assert!(s1.next().unwrap().is_none());
    }

    {
        let mut v1alpha_stub = v1alpha_grpc::ServerReflectionClient::new(&mut transport);
        let mut s2 = v1alpha_stub
            .server_reflection_info(|tx| {
                tx.send(|req| {
                    req.write_all_extension_numbers_of_type("telemetry.v1.SensorReading")
                        .write_host("localhost");
                })
            })
            .expect("std tcp v1alpha call");
        let resp = s2.next().unwrap().unwrap();
        match resp.message_response().unwrap() {
            Some(V1AlphaResp::AllExtensionNumbersResponse(enr)) => {
                let nums: Vec<i32> = enr.extension_number().map(|n| n.unwrap()).collect();
                assert_eq!(nums, vec![100, 101, 205, 303]);
            }
            other => panic!("unexpected v1alpha response: {other:?}"),
        }
        assert!(s2.next().unwrap().is_none());
    }

    transport
        .send_goaway(H2ErrorCode::NoError, b"done")
        .expect("send goaway");
    transport.io_mut().shutdown(Shutdown::Write).unwrap();
    drop(transport);
    server_thread.join().expect("server thread join");
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn tokio_tcp_reflection_v1_and_v1alpha_e2e() {
    use roto_grpc::{accept_and_serve_tokio, connect_tokio_tcp};
    use tokio::net::TcpListener;

    let (geo_bytes, sensor_bytes, extra_bytes, sensor_set) = leaked_descriptors();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_task = tokio::spawn(async move {
        let sensor_route = StaticSensorService {
            descriptors: sensor_set,
        };
        let app_routes: [&dyn Route; 1] = [&sensor_route];
        let extra = [extra_bytes];
        let refl = ReflectionService::new(&app_routes).with_extra_descriptors(&extra);
        let v1_srv = refl.into_v1_server();
        let v1a_srv = refl.v1alpha_server();
        let routes: [&dyn Route; 3] = [&sensor_route, &v1_srv, &v1a_srv];
        let mut msg_buf = vec![0u8; 16384];
        let mut tx_buf = vec![0u8; 32768];
        let mut server: GrpcServer<2, 8192> = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);
        accept_and_serve_tokio(&listener, &mut server)
            .await
            .expect("serve tokio connection");
    });

    let mut cli_tx = vec![0u8; 8192];
    let mut call_buf = vec![0u8; 16384];
    let mut transport =
        connect_tokio_tcp::<_, 2, 8192>(addr, "http", "tokio.local", &mut cli_tx, &mut call_buf)
            .await
            .expect("connect tokio tcp");

    {
        let mut v1_stub = v1_grpc::ServerReflectionClient::new(&mut transport);
        let mut stream = v1_stub
            .server_reflection_info_async(|tx| {
                tx.send(|req| {
                    req.write_list_services("*").write_host("tokio.local");
                })?;
                tx.send(|req| {
                    req.write_file_containing_symbol("telemetry.v1.SensorCollector.ReportLocation")
                        .write_host("tokio.local");
                })
            })
            .await
            .expect("tokio v1 bidi call");

        let r1 = stream.next().unwrap().unwrap();
        match r1.message_response().unwrap() {
            Some(v1::server_reflection_response::MessageResponse::ListServicesResponse(ls)) => {
                let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
                assert_eq!(
                    names,
                    vec![
                        "telemetry.v1.SensorCollector",
                        "grpc.reflection.v1.ServerReflection",
                        "grpc.reflection.v1alpha.ServerReflection",
                    ]
                );
            }
            other => panic!("unexpected response 1: {other:?}"),
        }

        let r2 = stream.next().unwrap().unwrap();
        match r2.message_response().unwrap() {
            Some(v1::server_reflection_response::MessageResponse::FileDescriptorResponse(fdr)) => {
                let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
                assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
            }
            other => panic!("unexpected response 2: {other:?}"),
        }
        assert!(stream.next().unwrap().is_none());
    }

    transport
        .send_goaway(H2ErrorCode::NoError, b"done")
        .await
        .unwrap();
    drop(transport);
    server_task.await.unwrap();
}

#[test]
fn interactive_grpcurl_multi_turn_bidi_reflection_stream() {
    use roto_grpc::h2::{H2Connection, H2Event, Role};
    use roto_grpc::hpack::{GrpcResponseHeaders, GrpcTrailers, HpackDecoder, GRPC_CONTENT_TYPE};
    use v1::server_reflection_response::MessageResponse as V1Resp;

    let (geo_bytes, sensor_bytes, extra_bytes, sensor_set) = leaked_descriptors();
    let sensor_route = StaticSensorService {
        descriptors: sensor_set,
    };
    let app_routes: [&dyn Route; 1] = [&sensor_route];
    let extra = [extra_bytes];
    let refl = ReflectionService::new(&app_routes).with_extra_descriptors(&extra);
    let v1_srv = refl.v1_server();
    let v1a_srv = refl.v1alpha_server();
    let routes: [&dyn Route; 3] = [&sensor_route, &v1_srv, &v1a_srv];

    let mut msg_buf = [0u8; 8192];
    let mut stx_buf = [0u8; 16384];
    let mut server: GrpcServer<4, 4096> = GrpcServer::new(&routes, &mut msg_buf, &mut stx_buf);
    let mut raw_cli: H2Connection<4, 4096> = H2Connection::new(Role::Client);
    let mut wire = [0u8; 8192];

    // 1. Handshake (server SETTINGS, client preface + SETTINGS, SETTINGS ACKs)
    let m0 = server.poll_transmit(&mut wire);
    let mut pos = 0;
    while pos < m0 {
        let (c, _) = raw_cli.recv(&wire[pos..m0]).unwrap();
        pos += c;
    }
    let n0 = raw_cli.poll_transmit(&mut wire);
    server.recv(&wire[..n0]).unwrap();
    let m1 = server.poll_transmit(&mut wire);
    pos = 0;
    while pos < m1 {
        let (c, _) = raw_cli.recv(&wire[pos..m1]).unwrap();
        pos += c;
    }
    let n1 = raw_cli.poll_transmit(&mut wire);
    server.recv(&wire[..n1]).unwrap();
    assert_eq!(server.poll_transmit(&mut wire), 0);

    // 2. Client (simulating grpcurl) opens /grpc.reflection.v1.ServerReflection/ServerReflectionInfo
    //    with `content-type: application/grpc` and `end_stream = false`.
    let (stream_id, hdr_n) = raw_cli
        .send_grpc_request_headers(
            b"http",
            b"localhost",
            v1_grpc::server_reflection::PATH_SERVER_REFLECTION_INFO.as_bytes(),
            GRPC_CONTENT_TYPE,
            false,
            &mut wire,
        )
        .unwrap();
    assert_eq!(stream_id, 1);
    server.recv(&wire[..hdr_n]).unwrap();
    assert_eq!(server.poll_transmit(&mut wire), 0);

    // 3. Turn 1: Client sends 16-byte DATA frame containing `list_services: ""` (`[0x3a, 0x00]`)
    //    with `end_stream = false` and waits for the response BEFORE sending Turn 2.
    let data1_n = raw_cli
        .send_grpc_data_frame(stream_id, false, false, &mut wire, |w| {
            let mut b = v1::ServerReflectionRequestBuilder::from_writer(core::mem::replace(
                w,
                roto::runtime::BackwardWriter::new(&mut []),
            ));
            b.write_list_services("");
            *w = b.writer;
        })
        .unwrap()
        .len();
    let data1_start = wire.len() - data1_n;
    assert_eq!(
        data1_n, 16,
        "9-byte H2 header + 5-byte LPM + 2-byte list_services"
    );
    server.recv(&wire[data1_start..]).unwrap();

    let srv_turn1 = server.poll_transmit(&mut wire);
    assert!(
        srv_turn1 > 26,
        "server must immediately emit response HEADERS + ListServicesResponse DATA (not just 26-byte WINDOW_UPDATEs)"
    );

    let mut got_resp_headers = false;
    let mut turn1_resp_bytes: Option<Vec<u8>> = None;
    pos = 0;
    while pos < srv_turn1 {
        let (c, ev) = raw_cli.recv(&wire[pos..srv_turn1]).unwrap();
        pos += c;
        match ev {
            Some(H2Event::Headers {
                stream_id: 1,
                header_block,
                end_stream: false,
            }) => {
                let mut scratch = [0u8; 256];
                let mut dec = HpackDecoder::<0>::stateless();
                let hdrs =
                    GrpcResponseHeaders::decode(&mut dec, header_block, &mut scratch).unwrap();
                assert_eq!(hdrs.status, Some(&b"200"[..]));
                assert_eq!(
                    hdrs.content_type,
                    Some(GRPC_CONTENT_TYPE),
                    "response content-type must be application/grpc for standard gRPC clients"
                );
                got_resp_headers = true;
            }
            Some(H2Event::GrpcMessage {
                stream_id: 1,
                payload,
                end_stream: false,
                ..
            }) => {
                turn1_resp_bytes = Some(payload.to_vec());
            }
            _ => {}
        }
    }
    assert!(
        got_resp_headers,
        "must receive :status 200 HEADERS on Turn 1"
    );
    let turn1_payload = turn1_resp_bytes.expect("must receive ListServicesResponse on Turn 1");
    let turn1_view = v1::ServerReflectionResponseView::from_bytes(&turn1_payload).unwrap();
    match turn1_view.message_response().unwrap() {
        Some(V1Resp::ListServicesResponse(ls)) => {
            let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
            assert_eq!(
                names,
                vec![
                    "telemetry.v1.SensorCollector",
                    "grpc.reflection.v1.ServerReflection",
                    "grpc.reflection.v1alpha.ServerReflection",
                ]
            );
        }
        other => panic!("expected ListServicesResponse on Turn 1, got {other:?}"),
    }

    // 4. Turn 2: Client sends follow-up `file_containing_symbol` request on the SAME stream
    //    with `end_stream = false`.
    let data2_n = raw_cli
        .send_grpc_data_frame(stream_id, false, false, &mut wire, |w| {
            let mut b = v1::ServerReflectionRequestBuilder::from_writer(core::mem::replace(
                w,
                roto::runtime::BackwardWriter::new(&mut []),
            ));
            b.write_file_containing_symbol("telemetry.v1.SensorCollector");
            *w = b.writer;
        })
        .unwrap()
        .len();
    let data2_start = wire.len() - data2_n;
    server.recv(&wire[data2_start..]).unwrap();

    let srv_turn2 = server.poll_transmit(&mut wire);
    assert!(srv_turn2 > 26, "server must immediately emit Turn 2 DATA");
    let mut turn2_resp_bytes: Option<Vec<u8>> = None;
    pos = 0;
    while pos < srv_turn2 {
        let (c, ev) = raw_cli.recv(&wire[pos..srv_turn2]).unwrap();
        pos += c;
        match ev {
            Some(H2Event::Headers { stream_id: 1, .. }) => {
                panic!("server must not re-send HEADERS on Turn 2");
            }
            Some(H2Event::GrpcMessage {
                stream_id: 1,
                payload,
                end_stream: false,
                ..
            }) => {
                turn2_resp_bytes = Some(payload.to_vec());
            }
            _ => {}
        }
    }
    let turn2_payload = turn2_resp_bytes.expect("must receive FileDescriptorResponse on Turn 2");
    let turn2_view = v1::ServerReflectionResponseView::from_bytes(&turn2_payload).unwrap();
    match turn2_view.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![sensor_bytes, geo_bytes]);
        }
        other => panic!("expected FileDescriptorResponse on Turn 2, got {other:?}"),
    }

    // 5. Client half-closes stream with empty DATA (`end_stream = true`), and server responds
    //    with `grpc-status: 0` trailers (`end_stream = true`).
    let eof_n = raw_cli
        .send_data_frame(stream_id, &[], true, &mut wire)
        .unwrap();
    server.recv(&wire[..eof_n]).unwrap();
    let srv_eof = server.poll_transmit(&mut wire);
    assert!(srv_eof > 0, "server must emit trailing HEADERS on EOF");

    let mut got_ok_trailers = false;
    pos = 0;
    while pos < srv_eof {
        let (c, ev) = raw_cli.recv(&wire[pos..srv_eof]).unwrap();
        pos += c;
        if let Some(H2Event::Headers {
            stream_id: 1,
            header_block,
            end_stream: true,
        }) = ev
        {
            let mut scratch = [0u8; 256];
            let mut dec = HpackDecoder::<0>::stateless();
            let tr = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
            assert_eq!(tr.grpc_status, Some(GrpcCode::Ok));
            got_ok_trailers = true;
        }
    }
    assert!(
        got_ok_trailers,
        "must receive grpc-status: 0 trailers on EOF"
    );
}

// ============================================================================
// Exact generated modules (`telemetry::v1` and `grpc::telemetry::v1::sensor_collector`),
// `MySensorHandler`, and `drive_connection` from `docs/grpc.md` (`proto/sensor.proto`)
// ============================================================================

#[allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]
pub mod telemetry {
    pub mod v1 {
        pub const FILE_DESCRIPTOR_BYTES_SENSOR: &[u8] = b"\x0a\x0c\x73\x65\x6e\x73\x6f\x72\x2e\x70\x72\x6f\x74\x6f\x12\x0c\x74\x65\x6c\x65\x6d\x65\x74\x72\x79\x2e\x76\x31\x22\x58\x0a\x08\x4c\x6f\x63\x61\x74\x69\x6f\x6e\x12\x1a\x0a\x06\x6c\x61\x74\x5f\x65\x37\x18\x01\x20\x01\x28\x05\x48\x00\x52\x05\x6c\x61\x74\x45\x37\x88\x01\x01\x12\x1a\x0a\x06\x6c\x6f\x6e\x5f\x65\x37\x18\x02\x20\x01\x28\x05\x48\x01\x52\x05\x6c\x6f\x6e\x45\x37\x88\x01\x01\x42\x09\x0a\x07\x5f\x6c\x61\x74\x5f\x65\x37\x42\x09\x0a\x07\x5f\x6c\x6f\x6e\x5f\x65\x37\x22\xbe\x01\x0a\x0d\x53\x65\x6e\x73\x6f\x72\x52\x65\x61\x64\x69\x6e\x67\x12\x20\x0a\x09\x64\x65\x76\x69\x63\x65\x5f\x69\x64\x18\x01\x20\x01\x28\x09\x48\x00\x52\x08\x64\x65\x76\x69\x63\x65\x49\x64\x88\x01\x01\x12\x26\x0a\x0c\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x5f\x6d\x73\x18\x02\x20\x01\x28\x04\x48\x01\x52\x0b\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x4d\x73\x88\x01\x01\x12\x37\x0a\x08\x6c\x6f\x63\x61\x74\x69\x6f\x6e\x18\x03\x20\x01\x28\x0b\x32\x16\x2e\x74\x65\x6c\x65\x6d\x65\x74\x72\x79\x2e\x76\x31\x2e\x4c\x6f\x63\x61\x74\x69\x6f\x6e\x48\x02\x52\x08\x6c\x6f\x63\x61\x74\x69\x6f\x6e\x88\x01\x01\x42\x0c\x0a\x0a\x5f\x64\x65\x76\x69\x63\x65\x5f\x69\x64\x42\x0f\x0a\x0d\x5f\x74\x69\x6d\x65\x73\x74\x61\x6d\x70\x5f\x6d\x73\x42\x0b\x0a\x09\x5f\x6c\x6f\x63\x61\x74\x69\x6f\x6e\x32\x58\x0a\x0f\x53\x65\x6e\x73\x6f\x72\x43\x6f\x6c\x6c\x65\x63\x74\x6f\x72\x12\x45\x0a\x0e\x52\x65\x70\x6f\x72\x74\x4c\x6f\x63\x61\x74\x69\x6f\x6e\x12\x1b\x2e\x74\x65\x6c\x65\x6d\x65\x74\x72\x79\x2e\x76\x31\x2e\x53\x65\x6e\x73\x6f\x72\x52\x65\x61\x64\x69\x6e\x67\x1a\x16\x2e\x74\x65\x6c\x65\x6d\x65\x74\x72\x79\x2e\x76\x31\x2e\x4c\x6f\x63\x61\x74\x69\x6f\x6e\x4a\xf4\x03\x0a\x06\x12\x04\x00\x00\x10\x01\x0a\x08\x0a\x01\x0c\x12\x03\x00\x00\x12\x0a\x08\x0a\x01\x02\x12\x03\x01\x00\x15\x0a\x0a\x0a\x02\x04\x00\x12\x04\x03\x00\x06\x01\x0a\x0a\x0a\x03\x04\x00\x01\x12\x03\x03\x08\x10\x0a\x0b\x0a\x04\x04\x00\x02\x00\x12\x03\x04\x02\x1c\x0a\x0c\x0a\x05\x04\x00\x02\x00\x04\x12\x03\x04\x02\x0a\x0a\x0c\x0a\x05\x04\x00\x02\x00\x05\x12\x03\x04\x0b\x10\x0a\x0c\x0a\x05\x04\x00\x02\x00\x01\x12\x03\x04\x11\x17\x0a\x0c\x0a\x05\x04\x00\x02\x00\x03\x12\x03\x04\x1a\x1b\x0a\x0b\x0a\x04\x04\x00\x02\x01\x12\x03\x05\x02\x1c\x0a\x0c\x0a\x05\x04\x00\x02\x01\x04\x12\x03\x05\x02\x0a\x0a\x0c\x0a\x05\x04\x00\x02\x01\x05\x12\x03\x05\x0b\x10\x0a\x0c\x0a\x05\x04\x00\x02\x01\x01\x12\x03\x05\x11\x17\x0a\x0c\x0a\x05\x04\x00\x02\x01\x03\x12\x03\x05\x1a\x1b\x0a\x0a\x0a\x02\x04\x01\x12\x04\x08\x00\x0c\x01\x0a\x0a\x0a\x03\x04\x01\x01\x12\x03\x08\x08\x15\x0a\x0b\x0a\x04\x04\x01\x02\x00\x12\x03\x09\x02\x20\x0a\x0c\x0a\x05\x04\x01\x02\x00\x04\x12\x03\x09\x02\x0a\x0a\x0c\x0a\x05\x04\x01\x02\x00\x05\x12\x03\x09\x0b\x11\x0a\x0c\x0a\x05\x04\x01\x02\x00\x01\x12\x03\x09\x12\x1b\x0a\x0c\x0a\x05\x04\x01\x02\x00\x03\x12\x03\x09\x1e\x1f\x0a\x0b\x0a\x04\x04\x01\x02\x01\x12\x03\x0a\x02\x23\x0a\x0c\x0a\x05\x04\x01\x02\x01\x04\x12\x03\x0a\x02\x0a\x0a\x0c\x0a\x05\x04\x01\x02\x01\x05\x12\x03\x0a\x0b\x11\x0a\x0c\x0a\x05\x04\x01\x02\x01\x01\x12\x03\x0a\x12\x1e\x0a\x0c\x0a\x05\x04\x01\x02\x01\x03\x12\x03\x0a\x21\x22\x0a\x0b\x0a\x04\x04\x01\x02\x02\x12\x03\x0b\x02\x21\x0a\x0c\x0a\x05\x04\x01\x02\x02\x04\x12\x03\x0b\x02\x0a\x0a\x0c\x0a\x05\x04\x01\x02\x02\x06\x12\x03\x0b\x0b\x13\x0a\x0c\x0a\x05\x04\x01\x02\x02\x01\x12\x03\x0b\x14\x1c\x0a\x0c\x0a\x05\x04\x01\x02\x02\x03\x12\x03\x0b\x1f\x20\x0a\x0a\x0a\x02\x06\x00\x12\x04\x0e\x00\x10\x01\x0a\x0a\x0a\x03\x06\x00\x01\x12\x03\x0e\x08\x17\x0a\x0b\x0a\x04\x06\x00\x02\x00\x12\x03\x0f\x02\x37\x0a\x0c\x0a\x05\x06\x00\x02\x00\x01\x12\x03\x0f\x06\x14\x0a\x0c\x0a\x05\x06\x00\x02\x00\x02\x12\x03\x0f\x15\x22\x0a\x0c\x0a\x05\x06\x00\x02\x00\x03\x12\x03\x0f\x2d\x35\x62\x06\x70\x72\x6f\x74\x6f\x33";
        pub const FILE_DESCRIPTOR_BYTES: &[u8] = FILE_DESCRIPTOR_BYTES_SENSOR;
        pub const FILE_DESCRIPTORS: &[&[u8]] = &[FILE_DESCRIPTOR_BYTES_SENSOR];

        #[derive(Clone, Copy, Debug)]
        pub struct LocationView<'a> {
            buf: &'a [u8],
            lat_e7: i32,
            lon_e7: i32,
            has_fields: [u32; 1],
        }

        impl<'a> LocationView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut lat_e7: i32 = 0i32;
                let mut lon_e7: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            lat_e7 = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            lon_e7 = field.val as i32;
                            has_fields[0] |= 2;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    lat_e7,
                    lon_e7,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_lat_e7(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn lat_e7(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.lat_e7)
            }

            #[inline]
            pub const fn has_lon_e7(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn lon_e7(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.lon_e7)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for LocationView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct LocationBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> LocationBuilder<'b> {
            #[inline]
            pub fn new(buf: &'b mut [u8]) -> Self {
                Self {
                    writer: ::roto::runtime::BackwardWriter::new(buf),
                    set_fields: [0u64; 1],
                }
            }

            #[inline]
            pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self {
                    writer,
                    set_fields: [0u64; 1],
                }
            }

            #[inline]
            pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
                &self.writer
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
                self.writer.finish()
            }

            #[inline]
            pub fn write_lat_e7(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_lon_e7(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct SensorReadingView<'a> {
            buf: &'a [u8],
            device_id: ::roto::runtime::FieldMarker,
            timestamp_ms: u64,
            location: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> SensorReadingView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut device_id = ::roto::runtime::FieldMarker::none();
                let mut timestamp_ms: u64 = 0u64;
                let mut location = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => {
                            device_id = field.marker
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            timestamp_ms = field.val;
                            has_fields[0] |= 1;
                        }
                        (3, ::roto::runtime::WireType::LengthDelimited) => location = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    device_id,
                    timestamp_ms,
                    location,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub fn device_id(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.device_id.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.device_id)
            }

            #[inline]
            pub const fn timestamp_ms(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.timestamp_ms)
            }

            #[inline]
            pub fn location(
                &self,
            ) -> Result<Option<crate::telemetry::v1::LocationView<'a>>, ::roto::runtime::DecodeError>
            {
                if !self.location.is_present() {
                    return Ok(None);
                }
                crate::telemetry::v1::LocationView::from_bytes(self.location.slice(self.buf))
                    .map(Some)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for SensorReadingView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct SensorReadingBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> SensorReadingBuilder<'b> {
            #[inline]
            pub fn new(buf: &'b mut [u8]) -> Self {
                Self {
                    writer: ::roto::runtime::BackwardWriter::new(buf),
                    set_fields: [0u64; 1],
                }
            }

            #[inline]
            pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self {
                    writer,
                    set_fields: [0u64; 1],
                }
            }

            #[inline]
            pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
                &self.writer
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn finish(self) -> Result<&'b [u8], ::roto::runtime::EncodeError> {
                self.writer.finish()
            }

            #[inline]
            pub fn write_device_id(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self
                    .writer
                    .write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_timestamp_ms(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_location<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::telemetry::v1::LocationBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder =
                        crate::telemetry::v1::LocationBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self
                    .writer
                    .write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }
    }
}

#[allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]
pub mod grpc {
    pub mod telemetry {
        pub mod v1 {
            pub mod sensor_collector {
                pub const SERVICE_NAME: &str = "telemetry.v1.SensorCollector";
                pub const FILE_DESCRIPTOR_BYTES: &[u8] =
                    crate::telemetry::v1::FILE_DESCRIPTOR_BYTES_SENSOR;
                pub const FILE_DESCRIPTOR_SET: &[&[u8]] = &[FILE_DESCRIPTOR_BYTES];
                pub const PATH_REPORT_LOCATION: &str =
                    "/telemetry.v1.SensorCollector/ReportLocation";

                pub trait SensorCollector {
                    fn report_location<'a, 'b>(
                        &self,
                        meta: &mut ::roto_grpc::RpcMetadata<'_>,
                        req: crate::telemetry::v1::SensorReadingView<'a>,
                        resp: &mut crate::telemetry::v1::LocationBuilder<'b>,
                    ) -> Result<(), ::roto_grpc::GrpcStatus<'static>>;
                }

                #[derive(Clone, Copy, Debug)]
                pub struct SensorCollectorServer<T> {
                    pub inner: T,
                }

                impl<T> SensorCollectorServer<T> {
                    #[inline]
                    pub const fn new(inner: T) -> Self {
                        Self { inner }
                    }
                }

                impl<T: SensorCollector> ::roto_grpc::GrpcService for SensorCollectorServer<T> {
                    const SERVICE_NAME: &'static str = SERVICE_NAME;
                    const FILE_DESCRIPTOR_SET: &'static [&'static [u8]] = FILE_DESCRIPTOR_SET;

                    fn call_unary(
                        &self,
                        meta: &mut ::roto_grpc::RpcMetadata<'_>,
                        method_name: &str,
                        req_payload: &[u8],
                        writer: &mut ::roto::runtime::BackwardWriter<'_>,
                    ) -> Result<(), ::roto_grpc::GrpcStatus<'static>> {
                        match method_name {
                            "ReportLocation" => {
                                let req_view =
                                    crate::telemetry::v1::SensorReadingView::from_bytes(req_payload)
                                        .map_err(|_| {
                                            ::roto_grpc::GrpcStatus::internal(
                                                "invalid protobuf request payload",
                                            )
                                        })?;
                                let end_pos = writer.cursor();
                                let taken = ::roto_grpc::framing::take_writer(writer);
                                let mut resp =
                                    crate::telemetry::v1::LocationBuilder::from_writer(taken);
                                let res = self.inner.report_location(meta, req_view, &mut resp);
                                *writer = resp.writer;
                                res?;
                                ::roto_grpc::framing::prepend_grpc_header(writer, false, end_pos)
                                    .map_err(::roto_grpc::GrpcStatus::from)
                            }
                            _ => Err(::roto_grpc::GrpcStatus::unimplemented(
                                "unimplemented gRPC method",
                            )),
                        }
                    }
                }

                pub trait ReportLocationRequest<'a> {
                    fn into_grpc_request_bytes(
                        self,
                    ) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>>;
                }

                impl<'a> ReportLocationRequest<'a>
                    for &'a crate::telemetry::v1::SensorReadingBuilder<'_>
                {
                    #[inline]
                    fn into_grpc_request_bytes(
                        self,
                    ) -> Result<&'a [u8], ::roto_grpc::GrpcStatus<'static>> {
                        self.check().map_err(::roto_grpc::GrpcStatus::from)?;
                        Ok(self.writer().as_written_slice())
                    }
                }

                #[derive(Clone, Copy, Debug)]
                pub struct SensorCollectorClient<C> {
                    pub channel: C,
                }

                impl<C> SensorCollectorClient<C> {
                    #[inline]
                    pub const fn new(channel: C) -> Self {
                        Self { channel }
                    }
                }

                impl<C: ::roto_grpc::UnaryChannel> SensorCollectorClient<C> {
                    pub fn report_location<'a, 'r, R>(
                        &'a mut self,
                        req: R,
                    ) -> Result<crate::telemetry::v1::LocationView<'a>, ::roto_grpc::GrpcStatus<'a>>
                    where
                        R: ReportLocationRequest<'r>,
                    {
                        let req_bytes = req.into_grpc_request_bytes()?;
                        let resp_bytes =
                            self.channel.call_unary_bytes(PATH_REPORT_LOCATION, req_bytes)?;
                        crate::telemetry::v1::LocationView::from_bytes(resp_bytes).map_err(|_| {
                            ::roto_grpc::GrpcStatus::internal("invalid protobuf response payload")
                        })
                    }
                }
            }
            pub use sensor_collector::{
                SensorCollector, SensorCollectorClient, SensorCollectorServer,
            };
        }
    }
}

pub struct MySensorHandler;

impl crate::grpc::telemetry::v1::sensor_collector::SensorCollector for MySensorHandler {
    fn report_location<'a, 'b>(
        &self,
        _meta: &mut ::roto_grpc::RpcMetadata<'_>,
        req: crate::telemetry::v1::SensorReadingView<'a>,
        resp: &mut crate::telemetry::v1::LocationBuilder<'b>,
    ) -> Result<(), GrpcStatus<'static>> {
        let Some(loc) = req
            .location()
            .map_err(|_| GrpcStatus::internal("corrupt location submessage"))?
        else {
            return Err(GrpcStatus::invalid_argument("missing location"));
        };
        let lat = loc.lat_e7().map_err(|_| GrpcStatus::internal("corrupt lat"))?;
        let lon = loc.lon_e7().map_err(|_| GrpcStatus::internal("corrupt lon"))?;

        resp.write_lon_e7(lon + 1).write_lat_e7(lat + 1);
        Ok(())
    }
}

/// Exact `drive_connection` function from `docs/grpc.md` Step 4A.
fn drive_connection<const MAX_STREAMS: usize, const REASM_CAP: usize>(
    stream: &mut std::net::TcpStream,
    server: &mut GrpcServer<'_, MAX_STREAMS, REASM_CAP>,
    io_buf: &mut [u8],
) -> std::io::Result<()> {
    use std::io::{Read, Write};
    loop {
        // 1. Flush all queued outbound HTTP/2 frames (initial server SETTINGS,
        //    SETTINGS/PING ACKs, and encoded RPC response HEADERS + DATA + trailers).
        loop {
            let tx_n = server.poll_transmit(io_buf);
            if tx_n == 0 {
                break;
            }
            stream.write_all(&io_buf[..tx_n])?;
        }
        stream.flush()?;

        // 2. Read the next chunk of raw bytes from the client TCP stream.
        let rx_n = stream.read(io_buf)?;
        if rx_n == 0 {
            return Ok(()); // Client closed the connection cleanly (EOF).
        }

        // 3. Feed bytes into the Sans-I/O state machine.
        if server.recv(&io_buf[..rx_n]).is_err() {
            let tx_n = server.poll_transmit(io_buf);
            if tx_n > 0 {
                let _ = stream.write_all(&io_buf[..tx_n]);
            }
            return Ok(());
        }
    }
}

/// Issue 1 regression test: verifies that `drive_connection` from `docs/grpc.md` over a real
/// `std::net::TcpListener` / `TcpStream` immediately dispatches and flushes responses for
/// interactive multi-turn bidirectional-streaming reflection requests (`grpc_cli` / `grpcurl`)
/// where `HEADERS` and `DATA` have `end_stream = false`, without hanging on `stream.read`.
#[test]
fn docs_grpc_drive_connection_interactive_grpc_cli_multi_turn_bidi_tcp() {
    use crate::grpc::telemetry::v1::sensor_collector::SensorCollectorServer;
    use roto_grpc::h2::{H2Connection, H2Event, Role};
    use roto_grpc::hpack::{GrpcResponseHeaders, GrpcTrailers, HpackDecoder, GRPC_CONTENT_TYPE};
    use std::io::{Read, Write};
    use std::net::{Shutdown, TcpListener, TcpStream};
    use std::time::Duration;
    use v1::server_reflection_response::MessageResponse as V1Resp;

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind TcpListener");
    let addr = listener.local_addr().unwrap();

    let server_thread = std::thread::spawn(move || {
        let svc = SensorCollectorServer::new(MySensorHandler);
        let app_routes: [&dyn Route; 1] = [&svc];
        let reflection = ReflectionService::new(&app_routes);
        let refl_v1 = reflection.v1_server();
        let refl_v1alpha = reflection.v1alpha_server();
        let routes: [&dyn Route; 3] = [&svc, &refl_v1, &refl_v1alpha];

        let mut msg_buf = [0u8; 8192];
        let mut tx_buf = [0u8; 16384];
        let mut io_buf = [0u8; 4096];
        let mut server: GrpcServer = GrpcServer::new(&routes, &mut msg_buf, &mut tx_buf);

        let (mut stream, _) = listener.accept().expect("accept client");
        stream.set_nodelay(true).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        server.reset();
        drive_connection(&mut stream, &mut server, &mut io_buf).expect("drive_connection");
    });

    let mut cli_stream = TcpStream::connect(addr).expect("connect to server");
    cli_stream.set_nodelay(true).unwrap();
    cli_stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();

    let mut raw_cli: H2Connection<4, 4096> = H2Connection::new(Role::Client);
    let mut wire = [0u8; 8192];
    let mut rx_buf = [0u8; 4096];

    // 1. Send client preface + initial SETTINGS
    let n0 = raw_cli.poll_transmit(&mut wire);
    cli_stream.write_all(&wire[..n0]).unwrap();
    cli_stream.flush().unwrap();

    // 2. Open `/grpc.reflection.v1.ServerReflection/ServerReflectionInfo` with `end_stream = false`
    let (stream_id, hdr_n) = raw_cli
        .send_grpc_request_headers(
            b"http",
            b"127.0.0.1:50051",
            v1_grpc::server_reflection::PATH_SERVER_REFLECTION_INFO.as_bytes(),
            GRPC_CONTENT_TYPE,
            false,
            &mut wire,
        )
        .unwrap();
    cli_stream.write_all(&wire[..hdr_n]).unwrap();

    // 3. Turn 1: Send 16-byte DATA frame (`list_services: ""`, `end_stream = false`) and wait for
    //    `drive_connection` to flush `:status 200` HEADERS + `ListServicesResponse` DATA over TCP.
    let data1_n = raw_cli
        .send_grpc_data_frame(stream_id, false, false, &mut wire, |w| {
            let mut b = v1::ServerReflectionRequestBuilder::from_writer(core::mem::replace(
                w,
                roto::runtime::BackwardWriter::new(&mut []),
            ));
            b.write_list_services("");
            *w = b.writer;
        })
        .unwrap()
        .len();
    assert_eq!(data1_n, 16);
    let data1_start = wire.len() - data1_n;
    cli_stream.write_all(&wire[data1_start..]).unwrap();
    cli_stream.flush().unwrap();

    let mut got_resp_headers = false;
    let mut turn1_payload: Option<Vec<u8>> = None;
    while turn1_payload.is_none() {
        let rx_n = cli_stream
            .read(&mut rx_buf)
            .expect("read Turn 1 response from TCP socket without hanging");
        assert!(rx_n > 0, "unexpected EOF waiting for Turn 1 response");
        let mut pos = 0;
        while pos < rx_n {
            let (c, ev) = raw_cli.recv(&rx_buf[pos..rx_n]).unwrap();
            pos += c;
            match ev {
                Some(H2Event::Headers {
                    stream_id: 1,
                    header_block,
                    end_stream: false,
                }) => {
                    let mut scratch = [0u8; 256];
                    let mut dec = HpackDecoder::<0>::stateless();
                    let hdrs =
                        GrpcResponseHeaders::decode(&mut dec, header_block, &mut scratch).unwrap();
                    assert_eq!(hdrs.status, Some(&b"200"[..]));
                    assert_eq!(hdrs.content_type, Some(GRPC_CONTENT_TYPE));
                    got_resp_headers = true;
                }
                Some(H2Event::GrpcMessage {
                    stream_id: 1,
                    payload,
                    end_stream: false,
                    ..
                }) => {
                    turn1_payload = Some(payload.to_vec());
                }
                _ => {}
            }
        }
        let ack_n = raw_cli.poll_transmit(&mut wire);
        if ack_n > 0 {
            cli_stream.write_all(&wire[..ack_n]).unwrap();
        }
    }
    assert!(got_resp_headers);
    let turn1_bytes = turn1_payload.unwrap();
    let turn1_view = v1::ServerReflectionResponseView::from_bytes(&turn1_bytes).unwrap();
    match turn1_view.message_response().unwrap() {
        Some(V1Resp::ListServicesResponse(ls)) => {
            let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
            assert_eq!(
                names,
                vec![
                    "telemetry.v1.SensorCollector",
                    "grpc.reflection.v1.ServerReflection",
                    "grpc.reflection.v1alpha.ServerReflection",
                ]
            );
        }
        other => panic!("expected ListServicesResponse on Turn 1, got {other:?}"),
    }

    // 4. Turn 2: Send `file_containing_symbol: "telemetry.v1.SensorCollector"` (`end_stream = false`)
    //    on the same stream and read `FileDescriptorResponse` from the TCP socket.
    let data2_n = raw_cli
        .send_grpc_data_frame(stream_id, false, false, &mut wire, |w| {
            let mut b = v1::ServerReflectionRequestBuilder::from_writer(core::mem::replace(
                w,
                roto::runtime::BackwardWriter::new(&mut []),
            ));
            b.write_file_containing_symbol("telemetry.v1.SensorCollector");
            *w = b.writer;
        })
        .unwrap()
        .len();
    let data2_start = wire.len() - data2_n;
    cli_stream.write_all(&wire[data2_start..]).unwrap();
    cli_stream.flush().unwrap();

    let mut turn2_payload: Option<Vec<u8>> = None;
    while turn2_payload.is_none() {
        let rx_n = cli_stream
            .read(&mut rx_buf)
            .expect("read Turn 2 response from TCP socket without hanging");
        assert!(rx_n > 0, "unexpected EOF waiting for Turn 2 response");
        let mut pos = 0;
        while pos < rx_n {
            let (c, ev) = raw_cli.recv(&rx_buf[pos..rx_n]).unwrap();
            pos += c;
            if let Some(H2Event::GrpcMessage {
                stream_id: 1,
                payload,
                end_stream: false,
                ..
            }) = ev
            {
                turn2_payload = Some(payload.to_vec());
            }
        }
    }
    let turn2_bytes = turn2_payload.unwrap();
    let turn2_view = v1::ServerReflectionResponseView::from_bytes(&turn2_bytes).unwrap();
    match turn2_view.message_response().unwrap() {
        Some(V1Resp::FileDescriptorResponse(fdr)) => {
            let protos: Vec<&[u8]> = fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
            assert_eq!(protos, vec![telemetry::v1::FILE_DESCRIPTOR_BYTES]);
        }
        other => panic!("expected FileDescriptorResponse on Turn 2, got {other:?}"),
    }

    // 5. Turn 3: Send empty DATA with `end_stream = true` and read `grpc-status: 0` trailers.
    let eof_n = raw_cli
        .send_data_frame(stream_id, &[], true, &mut wire)
        .unwrap();
    cli_stream.write_all(&wire[..eof_n]).unwrap();
    cli_stream.flush().unwrap();

    let mut got_ok_trailers = false;
    while !got_ok_trailers {
        let rx_n = cli_stream
            .read(&mut rx_buf)
            .expect("read Turn 3 trailers from TCP socket");
        assert!(rx_n > 0, "unexpected EOF waiting for trailers");
        let mut pos = 0;
        while pos < rx_n {
            let (c, ev) = raw_cli.recv(&rx_buf[pos..rx_n]).unwrap();
            pos += c;
            if let Some(H2Event::Headers {
                stream_id: 1,
                header_block,
                end_stream: true,
            }) = ev
            {
                let mut scratch = [0u8; 256];
                let mut dec = HpackDecoder::<0>::stateless();
                let tr = GrpcTrailers::decode(&mut dec, header_block, &mut scratch).unwrap();
                assert_eq!(tr.grpc_status, Some(GrpcCode::Ok));
                got_ok_trailers = true;
            }
        }
    }

    cli_stream.shutdown(Shutdown::Both).unwrap();
    drop(cli_stream);
    server_thread.join().expect("join server thread");
}

/// Issue 2 regression test: verifies `grpc_cli ls 127.0.0.1:50051 telemetry.v1.SensorCollector`
/// (`FindFileContainingSymbol`) against the `docs/grpc.md` `SensorCollectorServer` +
/// `ReflectionService` over `drive_connection`, and reproduces why an empty `FILE_DESCRIPTOR_SET`
/// previously caused `NOT_FOUND from server for FindFileContainingSymbol(telemetry.v1.SensorCollector)`.
#[test]
fn docs_grpc_sensor_collector_find_file_containing_symbol_and_missing_descriptors_repro() {
    use crate::grpc::telemetry::v1::sensor_collector::{
        SensorCollectorClient, SensorCollectorServer,
    };
    use crate::telemetry::v1::SensorReadingBuilder;
    use v1::server_reflection_response::MessageResponse as V1Resp;

    // Part A: With `FILE_DESCRIPTOR_SET` populated (default when using `grpc-reflection-roto` or `--descriptors`),
    // `FindFileContainingSymbol` resolves the service, RPC method, request message, and response message,
    // and unary `ReportLocation` RPC works on the same server.
    let svc = SensorCollectorServer::new(MySensorHandler);
    let app_routes: [&dyn Route; 1] = [&svc];
    let reflection = ReflectionService::new(&app_routes);
    let refl_v1 = reflection.v1_server();
    let refl_v1alpha = reflection.v1alpha_server();
    let routes: [&dyn Route; 3] = [&svc, &refl_v1, &refl_v1alpha];

    let mut call_buf = [0u8; 16384];
    let mut srv_msg_buf = [0u8; 16384];
    let mut srv_tx_buf = [0u8; 32768];
    let mut cli_tx_buf = [0u8; 8192];

    let server: GrpcServer<2, 8192> = GrpcServer::new(&routes, &mut srv_msg_buf, &mut srv_tx_buf);
    let client: GrpcClient<2, 8192> = GrpcClient::new("http", "127.0.0.1:50051", &mut cli_tx_buf);
    let mut channel = H2LoopbackChannel::new(client, server, &mut call_buf);

    {
        let mut refl_client = v1_grpc::ServerReflectionClient::new(&mut channel);
        let symbols = [
            "telemetry.v1.SensorCollector",
            "telemetry.v1.SensorCollector.ReportLocation",
            "telemetry.v1.SensorReading",
            "telemetry.v1.Location",
        ];
        let mut stream = refl_client
            .server_reflection_info(|tx| {
                for sym in symbols {
                    tx.send(|req| {
                        req.write_file_containing_symbol(sym)
                            .write_host("127.0.0.1:50051");
                    })?;
                }
                Ok(())
            })
            .expect("reflection stream");

        for sym in symbols {
            let resp = stream
                .next()
                .unwrap()
                .unwrap_or_else(|| panic!("missing response for {sym}"));
            match resp.message_response().unwrap() {
                Some(V1Resp::FileDescriptorResponse(fdr)) => {
                    let protos: Vec<&[u8]> =
                        fdr.file_descriptor_proto().map(|b| b.unwrap()).collect();
                    assert_eq!(protos, vec![telemetry::v1::FILE_DESCRIPTOR_BYTES]);
                    let fd = FileDescriptorProtoView::from_bytes(protos[0]).unwrap();
                    assert_eq!(fd.name(), Ok("sensor.proto"));
                    assert_eq!(fd.package(), Ok("telemetry.v1"));
                    assert_eq!(fd.syntax(), Ok("proto3"));
                    assert!(fd.has_source_code_info());
                    let svc_desc = fd.service().next().unwrap().unwrap();
                    assert_eq!(svc_desc.name(), Ok("SensorCollector"));
                    let method_desc = svc_desc.method().next().unwrap().unwrap();
                    assert_eq!(method_desc.name(), Ok("ReportLocation"));
                    assert_eq!(method_desc.input_type(), Ok(".telemetry.v1.SensorReading"));
                    assert_eq!(method_desc.output_type(), Ok(".telemetry.v1.Location"));
                }
                other => panic!("expected FileDescriptorResponse for {sym}, got {other:?}"),
            }
        }
        assert!(stream.next().unwrap().is_none());
    }

    // Also invoke the unary `ReportLocation` RPC from `docs/grpc.md` Step 4B on the same server
    {
        let mut sensor_stub = SensorCollectorClient::new(&mut channel);
        let mut req_buf = [0u8; 128];
        let mut req = SensorReadingBuilder::new(&mut req_buf);
        req.write_location(|lb| {
            lb.write_lon_e7(2000).write_lat_e7(1000);
        })
        .write_device_id("rp2040-node-1")
        .write_timestamp_ms(42);

        let loc_view = sensor_stub.report_location(&req).expect("unary RPC succeeded");
        assert_eq!(loc_view.lat_e7(), Ok(1001));
        assert_eq!(loc_view.lon_e7(), Ok(2001));
    }

    // Part B: Reproduce why omitting descriptors (`FILE_DESCRIPTOR_SET = &[]`) caused:
    //   - `grpc_cli ls 127.0.0.1:50051` (`list_services`) to list `telemetry.v1.SensorCollector`
    //   - `grpc_cli ls 127.0.0.1:50051 telemetry.v1.SensorCollector` (`FindFileContainingSymbol`)
    //     to fail with `NOT_FOUND`
    let stripped_sensor_route = StaticSensorService { descriptors: &[] };
    let stripped_app_routes: [&dyn Route; 1] = [&stripped_sensor_route];
    let stripped_refl = ReflectionService::new(&stripped_app_routes);
    let stripped_v1 = stripped_refl.v1_server();
    let stripped_routes: [&dyn Route; 2] = [&stripped_sensor_route, &stripped_v1];

    let mut srv_msg_buf2 = [0u8; 8192];
    let mut srv_tx_buf2 = [0u8; 16384];
    let mut cli_tx_buf2 = [0u8; 4096];
    let mut call_buf2 = [0u8; 8192];
    let server2: GrpcServer<2, 4096> =
        GrpcServer::new(&stripped_routes, &mut srv_msg_buf2, &mut srv_tx_buf2);
    let client2: GrpcClient<2, 4096> =
        GrpcClient::new("http", "127.0.0.1:50051", &mut cli_tx_buf2);
    let mut channel2 = H2LoopbackChannel::new(client2, server2, &mut call_buf2);
    let mut refl_client2 = v1_grpc::ServerReflectionClient::new(&mut channel2);

    let mut stream2 = refl_client2
        .server_reflection_info(|tx| {
            tx.send(|req| {
                req.write_list_services("*");
            })?;
            tx.send(|req| {
                req.write_file_containing_symbol("telemetry.v1.SensorCollector");
            })
        })
        .unwrap();

    // 1. `list_services` still reports `telemetry.v1.SensorCollector` from `Route::service_name()`
    let r_list = stream2.next().unwrap().unwrap();
    match r_list.message_response().unwrap() {
        Some(V1Resp::ListServicesResponse(ls)) => {
            let names: Vec<&str> = ls.service().map(|s| s.unwrap().name().unwrap()).collect();
            assert!(names.contains(&"telemetry.v1.SensorCollector"));
        }
        other => panic!("expected ListServicesResponse, got {other:?}"),
    }

    // 2. `file_containing_symbol("telemetry.v1.SensorCollector")` returns `NOT_FOUND` when descriptors are stripped
    let r_sym = stream2.next().unwrap().unwrap();
    match r_sym.message_response().unwrap() {
        Some(V1Resp::ErrorResponse(err)) => {
            assert_eq!(err.error_code(), Ok(GrpcCode::NotFound as i32));
        }
        other => panic!("expected NOT_FOUND ErrorResponse when FILE_DESCRIPTOR_SET is empty, got {other:?}"),
    }
}

