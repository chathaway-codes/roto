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
