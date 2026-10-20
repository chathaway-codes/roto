use std::process::Command;

use google_protobuf_roto::google::protobuf::{
    compiler::{
        code_generator_response::Feature, CodeGeneratorRequestBuilder, CodeGeneratorRequestView,
        CodeGeneratorResponseBuilder, CodeGeneratorResponseView,
    },
    field_descriptor_proto::{Label, Type},
    file_options::OptimizeMode,
    Edition, FileDescriptorSetBuilder, FileDescriptorSetView,
};
use roto::runtime::BackwardWriter;

#[test]
fn parse_protoc_descriptor_set_for_descriptor_proto() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let proto_root = format!("{manifest_dir}/proto");
    let descriptor_proto = format!("{proto_root}/google/protobuf/descriptor.proto");

    let output = Command::new("protoc")
        .arg(format!("--proto_path={proto_root}"))
        .arg("--include_source_info")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg(&descriptor_proto)
        .output()
        .expect("failed to execute protoc");

    assert!(
        output.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let fds = FileDescriptorSetView::from_bytes(&output.stdout).expect("valid FileDescriptorSet");
    let mut files = fds.file();
    let file = files
        .next()
        .expect("at least one file")
        .expect("valid FileDescriptorProto");
    assert!(files.next().is_none());

    assert_eq!(file.name(), Ok("google/protobuf/descriptor.proto"));
    assert_eq!(file.package(), Ok("google.protobuf"));
    assert!(file.has_options());
    let fopt = file.options().unwrap().unwrap();
    assert_eq!(fopt.java_package(), Ok("com.google.protobuf"));
    assert_eq!(fopt.java_outer_classname(), Ok("DescriptorProtos"));
    assert_eq!(fopt.optimize_for(), Ok(OptimizeMode::Speed));
    assert_eq!(fopt.cc_enable_arenas(), Ok(true));
    assert_eq!(fopt.objc_class_prefix(), Ok("GPB"));
    assert_eq!(fopt.csharp_namespace(), Ok("Google.Protobuf.Reflection"));

    // Verify SourceCodeInfo locations (path, span, comments)
    assert!(file.has_source_code_info());
    let sci = file.source_code_info().unwrap().unwrap();
    let mut loc_count = 0usize;
    let mut commented_locs = 0usize;
    for loc_res in sci.location() {
        let loc = loc_res.expect("valid SourceCodeInfo.Location");
        loc_count += 1;
        for p in loc.path() {
            assert!(p.is_ok());
        }
        let span: Vec<i32> = loc.span().map(|s| s.expect("valid span int32")).collect();
        assert!(span.len() == 3 || span.len() == 4);
        if loc.has_leading_comments() && !loc.leading_comments().unwrap().is_empty() {
            commented_locs += 1;
        }
    }
    assert!(loc_count > 100);
    assert!(commented_locs > 20);

    // Verify top-level messages include FileDescriptorSet, FileDescriptorProto, DescriptorProto, FieldDescriptorProto, SourceCodeInfo
    let mut found_fds = false;
    let mut found_fdp = false;
    let mut found_dp = false;
    let mut found_field_dp = false;
    let mut found_location_packed = false;

    for msg_res in file.message_type() {
        let msg = msg_res.expect("valid DescriptorProto");
        match msg.name().unwrap() {
            "FileDescriptorSet" => {
                found_fds = true;
                let mut fields = msg.field();
                let f1 = fields.next().unwrap().unwrap();
                assert_eq!(f1.name(), Ok("file"));
                assert_eq!(f1.number(), Ok(1));
                assert_eq!(f1.label(), Ok(Label::LabelRepeated));
                assert_eq!(f1.r#type(), Ok(Type::TypeMessage));
                assert_eq!(f1.type_name(), Ok(".google.protobuf.FileDescriptorProto"));
            }
            "FileDescriptorProto" => {
                found_fdp = true;
                assert!(msg.field().count() >= 12);
            }
            "DescriptorProto" => {
                found_dp = true;
                // Check nested ExtensionRange and ReservedRange
                let mut nested_names = Vec::new();
                for nested in msg.nested_type() {
                    nested_names.push(nested.unwrap().name().unwrap());
                }
                assert!(nested_names.contains(&"ExtensionRange"));
                assert!(nested_names.contains(&"ReservedRange"));
            }
            "FieldDescriptorProto" => {
                found_field_dp = true;
                let mut enum_names = Vec::new();
                for en in msg.enum_type() {
                    let en_view = en.unwrap();
                    enum_names.push(en_view.name().unwrap());
                    if en_view.name().unwrap() == "Type" {
                        assert_eq!(en_view.value().count(), 18);
                    } else if en_view.name().unwrap() == "Label" {
                        assert_eq!(en_view.value().count(), 3);
                    }
                }
                assert!(enum_names.contains(&"Type"));
                assert!(enum_names.contains(&"Label"));
            }
            "SourceCodeInfo" => {
                for nested_res in msg.nested_type() {
                    let nested = nested_res.unwrap();
                    if nested.name().unwrap() == "Location" {
                        let mut path_packed = false;
                        let mut span_packed = false;
                        for f_res in nested.field() {
                            let f = f_res.unwrap();
                            if f.name().unwrap() == "path" {
                                path_packed = f.options().unwrap().unwrap().packed().unwrap();
                            } else if f.name().unwrap() == "span" {
                                span_packed = f.options().unwrap().unwrap().packed().unwrap();
                            }
                        }
                        found_location_packed = path_packed && span_packed;
                    }
                }
            }
            _ => {}
        }
    }

    assert!(found_fds && found_fdp && found_dp && found_field_dp && found_location_packed);

    // Check top-level Edition and SymbolVisibility enums
    let mut found_edition = false;
    let mut found_visibility = false;
    for en_res in file.enum_type() {
        let en = en_res.unwrap();
        match en.name().unwrap() {
            "Edition" => {
                found_edition = true;
                for val_res in en.value() {
                    let val = val_res.unwrap();
                    assert!(Edition::from_i32(val.number().unwrap()).is_some());
                }
            }
            "SymbolVisibility" => {
                found_visibility = true;
            }
            _ => {}
        }
    }
    assert!(found_edition && found_visibility);
}

#[test]
fn parse_and_reencode_sensor_proto_via_protoc() {
    let sensor_proto_src = r#"syntax = "proto2";
package demo;

message Location {
    required int32 lat_e7 = 1;
    required int32 lon_e7 = 2;
}

message SensorReading {
    required uint64 timestamp_ms = 1;
    optional string device_id = 2 [default = "unknown"];
    optional Location location = 3;
    repeated uint32 samples = 4;
}
"#;

    let tmp_dir = std::env::temp_dir().join(format!("roto_issue_006_{}", std::process::id()));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let sensor_proto_path = tmp_dir.join("sensor.proto");
    std::fs::write(&sensor_proto_path, sensor_proto_src).unwrap();

    let protoc_out = Command::new("protoc")
        .arg(format!("--proto_path={}", tmp_dir.display()))
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("sensor.proto")
        .output()
        .expect("failed to execute protoc");
    let _ = std::fs::remove_dir_all(&tmp_dir);

    assert!(
        protoc_out.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&protoc_out.stderr)
    );

    let protoc_fds = FileDescriptorSetView::from_bytes(&protoc_out.stdout).unwrap();
    let protoc_file = protoc_fds.file().next().unwrap().unwrap();
    assert_eq!(protoc_file.name(), Ok("sensor.proto"));
    assert_eq!(protoc_file.package(), Ok("demo"));
    assert_eq!(protoc_file.message_type().count(), 2);

    // Serialize a complete FileDescriptorSet representing README.md's sensor.proto
    // using our BackwardWriter + Builders, then decode it via FileDescriptorSetView.
    let mut buf = [0u8; 2048];
    let mut writer = BackwardWriter::new(&mut buf);
    {
        let mut fds = FileDescriptorSetBuilder::new(&mut writer);
        fds.write_file(|file| {
            file.write_syntax("proto2")?;
            file.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(Type::TypeUint32)?;
                    f.write_label(Label::LabelRepeated)?;
                    f.write_number(4)?;
                    f.write_name("samples")
                })?;
                msg.write_field(|f| {
                    f.write_type_name(".demo.Location")?;
                    f.write_type(Type::TypeMessage)?;
                    f.write_label(Label::LabelOptional)?;
                    f.write_number(3)?;
                    f.write_name("location")
                })?;
                msg.write_field(|f| {
                    f.write_default_value("unknown")?;
                    f.write_type(Type::TypeString)?;
                    f.write_label(Label::LabelOptional)?;
                    f.write_number(2)?;
                    f.write_name("device_id")
                })?;
                msg.write_field(|f| {
                    f.write_type(Type::TypeUint64)?;
                    f.write_label(Label::LabelRequired)?;
                    f.write_number(1)?;
                    f.write_name("timestamp_ms")
                })?;
                msg.write_name("SensorReading")
            })?;
            file.write_message_type(|msg| {
                msg.write_field(|f| {
                    f.write_type(Type::TypeInt32)?;
                    f.write_label(Label::LabelRequired)?;
                    f.write_number(2)?;
                    f.write_name("lon_e7")
                })?;
                msg.write_field(|f| {
                    f.write_type(Type::TypeInt32)?;
                    f.write_label(Label::LabelRequired)?;
                    f.write_number(1)?;
                    f.write_name("lat_e7")
                })?;
                msg.write_name("Location")
            })?;
            file.write_package("demo")?;
            file.write_name("sensor.proto")
        })
        .unwrap();
    }
    let encoded = writer.finish();

    let fds = FileDescriptorSetView::from_bytes(encoded).unwrap();
    let file = fds.file().next().unwrap().unwrap();
    assert_eq!(file.name(), Ok("sensor.proto"));
    assert_eq!(file.package(), Ok("demo"));
    assert_eq!(file.syntax(), Ok("proto2"));

    let mut msgs = file.message_type();
    let loc = msgs.next().unwrap().unwrap();
    assert_eq!(loc.name(), Ok("Location"));
    assert_eq!(loc.field().count(), 2);

    let reading = msgs.next().unwrap().unwrap();
    assert_eq!(reading.name(), Ok("SensorReading"));
    let fields: Vec<_> = reading.field().map(|f| f.unwrap()).collect();
    assert_eq!(fields.len(), 4);
    assert_eq!(fields[0].name(), Ok("timestamp_ms"));
    assert_eq!(fields[1].name(), Ok("device_id"));
    assert_eq!(fields[1].default_value(), Ok("unknown"));
    assert_eq!(fields[2].name(), Ok("location"));
    assert_eq!(fields[2].type_name(), Ok(".demo.Location"));
    assert_eq!(fields[3].name(), Ok("samples"));
    assert_eq!(fields[3].label(), Ok(Label::LabelRepeated));
}

#[test]
fn parse_plugin_proto_and_build_code_generator_request() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let proto_root = format!("{manifest_dir}/proto");

    let output = Command::new("protoc")
        .arg(format!("--proto_path={proto_root}"))
        .arg("--include_imports")
        .arg("--include_source_info")
        .arg("--descriptor_set_out=/dev/stdout")
        .arg("google/protobuf/compiler/plugin.proto")
        .output()
        .expect("failed to execute protoc");

    assert!(
        output.status.success(),
        "protoc failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let fds = FileDescriptorSetView::from_bytes(&output.stdout).expect("valid FileDescriptorSet");
    let files: Vec<_> = fds
        .file()
        .map(|f| f.expect("valid FileDescriptorProto"))
        .collect();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0].name(), Ok("google/protobuf/descriptor.proto"));
    assert_eq!(files[1].name(), Ok("google/protobuf/compiler/plugin.proto"));
    assert_eq!(files[1].package(), Ok("google.protobuf.compiler"));

    // Verify plugin.proto messages (Version, CodeGeneratorRequest, CodeGeneratorResponse)
    let plugin_msgs: Vec<_> = files[1].message_type().map(|m| m.unwrap()).collect();
    assert_eq!(plugin_msgs.len(), 3);
    assert_eq!(plugin_msgs[0].name(), Ok("Version"));
    assert_eq!(plugin_msgs[1].name(), Ok("CodeGeneratorRequest"));
    assert_eq!(plugin_msgs[2].name(), Ok("CodeGeneratorResponse"));

    // Construct a CodeGeneratorRequest containing the FileDescriptorProtos
    let mut backing = vec![0u8; output.stdout.len() * 2 + 1024];
    let mut writer = BackwardWriter::new(&mut backing);
    {
        let mut req = CodeGeneratorRequestBuilder::new(&mut writer);
        req.write_compiler_version(|ver| {
            ver.write_suffix("")?;
            ver.write_patch(2)?;
            ver.write_minor(28)?;
            ver.write_major(4)
        })
        .unwrap();
        req.write_source_file_descriptors(|fdp| fdp.writer.write_bytes(files[1].as_bytes()))
            .unwrap();
        for fdp_view in files.iter().rev() {
            req.write_proto_file(|fdp| fdp.writer.write_bytes(fdp_view.as_bytes()))
                .unwrap();
        }
        req.write_parameter("no_std=true").unwrap();
        req.write_file_to_generate("google/protobuf/compiler/plugin.proto")
            .unwrap();
    }
    let req_bytes = writer.finish();

    let req_view =
        CodeGeneratorRequestView::from_bytes(req_bytes).expect("valid CodeGeneratorRequest");
    let files_to_gen: Vec<_> = req_view.file_to_generate().map(|f| f.unwrap()).collect();
    assert_eq!(files_to_gen, vec!["google/protobuf/compiler/plugin.proto"]);
    assert_eq!(req_view.parameter(), Ok("no_std=true"));

    let ver = req_view.compiler_version().unwrap().unwrap();
    assert_eq!(ver.major(), Ok(4));
    assert_eq!(ver.minor(), Ok(28));
    assert_eq!(ver.patch(), Ok(2));
    assert_eq!(ver.suffix(), Ok(""));

    let req_proto_files: Vec<_> = req_view.proto_file().map(|f| f.unwrap()).collect();
    assert_eq!(req_proto_files.len(), 2);
    assert_eq!(
        req_proto_files[0].name(),
        Ok("google/protobuf/descriptor.proto")
    );
    assert_eq!(
        req_proto_files[1].name(),
        Ok("google/protobuf/compiler/plugin.proto")
    );

    let req_source_files: Vec<_> = req_view
        .source_file_descriptors()
        .map(|f| f.unwrap())
        .collect();
    assert_eq!(req_source_files.len(), 1);
    assert_eq!(
        req_source_files[0].name(),
        Ok("google/protobuf/compiler/plugin.proto")
    );
    assert!(req_source_files[0].has_source_code_info());

    // Also build and decode a matching CodeGeneratorResponse
    let mut resp_backing = [0u8; 512];
    let mut resp_writer = BackwardWriter::new(&mut resp_backing);
    {
        let mut resp = CodeGeneratorResponseBuilder::new(&mut resp_writer);
        resp.write_file(|f| {
            f.write_content("// generated plugin.rs\n")?;
            f.write_name("google/protobuf/compiler/plugin.rs")
        })
        .unwrap();
        resp.write_maximum_edition(Edition::Edition2024.as_i32())
            .unwrap();
        resp.write_minimum_edition(Edition::EditionProto2.as_i32())
            .unwrap();
        resp.write_supported_features(
            (Feature::FeatureProto3Optional.as_i32() | Feature::FeatureSupportsEditions.as_i32())
                as u64,
        )
        .unwrap();
    }
    let resp_view = CodeGeneratorResponseView::from_bytes(resp_writer.finish()).unwrap();
    assert!(!resp_view.has_error());
    assert_eq!(resp_view.supported_features(), Ok(3));
    assert_eq!(
        resp_view.minimum_edition(),
        Ok(Edition::EditionProto2.as_i32())
    );
    assert_eq!(
        resp_view.maximum_edition(),
        Ok(Edition::Edition2024.as_i32())
    );
    let mut gen_files = resp_view.file();
    let gen_file = gen_files.next().unwrap().unwrap();
    assert!(gen_files.next().is_none());
    assert_eq!(gen_file.name(), Ok("google/protobuf/compiler/plugin.rs"));
    assert_eq!(gen_file.content(), Ok("// generated plugin.rs\n"));
}
