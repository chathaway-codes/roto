use google_protobuf_roto::google::protobuf::{
    compiler::{
        code_generator_response::{Feature, FileBuilder, FileView},
        CodeGeneratorRequestBuilder, CodeGeneratorRequestView, CodeGeneratorResponseBuilder,
        CodeGeneratorResponseView, VersionBuilder, VersionView,
    },
    extension_range_options::VerificationState,
    feature_set::{
        visibility_feature::DefaultSymbolVisibility, EnforceNamingStyle, EnumType, FieldPresence,
        JsonFormat, MessageEncoding, RepeatedFieldEncoding, Utf8Validation,
    },
    field_descriptor_proto::{Label, Type},
    field_options::{CType, JSType, OptionRetention, OptionTargetType},
    file_options::OptimizeMode,
    generated_code_info::annotation::Semantic,
    method_options::IdempotencyLevel,
    uninterpreted_option::{NamePartBuilder, NamePartView},
    DescriptorProtoView, Edition, EnumDescriptorProtoView, EnumValueDescriptorProtoView,
    FieldDescriptorProtoView, FieldOptionsBuilder, FieldOptionsView, FileDescriptorProtoView,
    FileDescriptorSetBuilder, FileDescriptorSetView, FileOptionsBuilder, FileOptionsView,
    GeneratedCodeInfoBuilder, GeneratedCodeInfoView, MessageOptionsView, MethodDescriptorProtoView,
    OneofDescriptorProtoView, ServiceDescriptorProtoView, SourceCodeInfoBuilder,
    SourceCodeInfoView, SymbolVisibility,
};
use roto::runtime::{BackwardWriter, DecodeError, VarintValue, WireType};

#[test]
fn enum_conversions_and_varint_value() {
    let editions = [
        (0, Edition::EditionUnknown),
        (1, Edition::Edition1TestOnly),
        (2, Edition::Edition2TestOnly),
        (900, Edition::EditionLegacy),
        (998, Edition::EditionProto2),
        (999, Edition::EditionProto3),
        (1000, Edition::Edition2023),
        (1001, Edition::Edition2024),
        (1002, Edition::Edition2026),
        (9999, Edition::EditionUnstable),
        (99997, Edition::Edition99997TestOnly),
        (99998, Edition::Edition99998TestOnly),
        (99999, Edition::Edition99999TestOnly),
        (0x7FFFFFFF, Edition::EditionMax),
    ];
    for (raw, expected) in editions {
        assert_eq!(Edition::from_i32(raw), Some(expected));
        assert_eq!(Edition::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(Edition::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(Edition::from_i32(42), None);
    assert_eq!(Edition::try_from(42), Err(42));
    assert_eq!(Edition::from_raw_varint(42), Edition::EditionUnknown);

    let vis = [
        (0, SymbolVisibility::VisibilityUnset),
        (1, SymbolVisibility::VisibilityLocal),
        (2, SymbolVisibility::VisibilityExport),
    ];
    for (raw, expected) in vis {
        assert_eq!(SymbolVisibility::from_i32(raw), Some(expected));
        assert_eq!(SymbolVisibility::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(SymbolVisibility::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(SymbolVisibility::from_i32(9), None);
    assert_eq!(
        SymbolVisibility::from_raw_varint(9),
        SymbolVisibility::VisibilityUnset
    );

    let types = [
        (1, Type::TypeDouble),
        (2, Type::TypeFloat),
        (3, Type::TypeInt64),
        (4, Type::TypeUint64),
        (5, Type::TypeInt32),
        (6, Type::TypeFixed64),
        (7, Type::TypeFixed32),
        (8, Type::TypeBool),
        (9, Type::TypeString),
        (10, Type::TypeGroup),
        (11, Type::TypeMessage),
        (12, Type::TypeBytes),
        (13, Type::TypeUint32),
        (14, Type::TypeEnum),
        (15, Type::TypeSfixed32),
        (16, Type::TypeSfixed64),
        (17, Type::TypeSint32),
        (18, Type::TypeSint64),
    ];
    for (raw, expected) in types {
        assert_eq!(Type::from_i32(raw), Some(expected));
        assert_eq!(Type::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(Type::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(Type::from_i32(0), None);
    assert_eq!(Type::from_raw_varint(99), Type::TypeDouble);

    let labels = [
        (1, Label::LabelOptional),
        (2, Label::LabelRequired),
        (3, Label::LabelRepeated),
    ];
    for (raw, expected) in labels {
        assert_eq!(Label::from_i32(raw), Some(expected));
        assert_eq!(Label::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(Label::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(Label::from_i32(0), None);
    assert_eq!(Label::from_raw_varint(99), Label::LabelOptional);

    for (raw, expected) in [
        (1, OptimizeMode::Speed),
        (2, OptimizeMode::CodeSize),
        (3, OptimizeMode::LiteRuntime),
    ] {
        assert_eq!(OptimizeMode::from_i32(raw), Some(expected));
        assert_eq!(OptimizeMode::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(OptimizeMode::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(OptimizeMode::from_i32(0), None);
    assert_eq!(OptimizeMode::from_raw_varint(99), OptimizeMode::Speed);

    for (raw, expected) in [
        (0, CType::String),
        (1, CType::Cord),
        (2, CType::StringPiece),
    ] {
        assert_eq!(CType::from_i32(raw), Some(expected));
        assert_eq!(CType::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(CType::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(CType::from_i32(9), None);
    assert_eq!(CType::from_raw_varint(9), CType::String);

    for (raw, expected) in [
        (0, JSType::JsNormal),
        (1, JSType::JsString),
        (2, JSType::JsNumber),
    ] {
        assert_eq!(JSType::from_i32(raw), Some(expected));
        assert_eq!(JSType::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(JSType::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(JSType::from_i32(9), None);
    assert_eq!(JSType::from_raw_varint(9), JSType::JsNormal);

    for (raw, expected) in [
        (0, OptionRetention::RetentionUnknown),
        (1, OptionRetention::RetentionRuntime),
        (2, OptionRetention::RetentionSource),
    ] {
        assert_eq!(OptionRetention::from_i32(raw), Some(expected));
        assert_eq!(OptionRetention::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(OptionRetention::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(OptionRetention::from_i32(9), None);
    assert_eq!(
        OptionRetention::from_raw_varint(9),
        OptionRetention::RetentionUnknown
    );

    for (raw, expected) in [
        (0, OptionTargetType::TargetTypeUnknown),
        (1, OptionTargetType::TargetTypeFile),
        (2, OptionTargetType::TargetTypeExtensionRange),
        (3, OptionTargetType::TargetTypeMessage),
        (4, OptionTargetType::TargetTypeField),
        (5, OptionTargetType::TargetTypeOneof),
        (6, OptionTargetType::TargetTypeEnum),
        (7, OptionTargetType::TargetTypeEnumEntry),
        (8, OptionTargetType::TargetTypeService),
        (9, OptionTargetType::TargetTypeMethod),
    ] {
        assert_eq!(OptionTargetType::from_i32(raw), Some(expected));
        assert_eq!(OptionTargetType::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(OptionTargetType::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(OptionTargetType::from_i32(99), None);
    assert_eq!(
        OptionTargetType::from_raw_varint(99),
        OptionTargetType::TargetTypeUnknown
    );

    for (raw, expected) in [
        (0, IdempotencyLevel::IdempotencyUnknown),
        (1, IdempotencyLevel::NoSideEffects),
        (2, IdempotencyLevel::Idempotent),
    ] {
        assert_eq!(IdempotencyLevel::from_i32(raw), Some(expected));
        assert_eq!(IdempotencyLevel::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(IdempotencyLevel::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(IdempotencyLevel::from_i32(9), None);
    assert_eq!(
        IdempotencyLevel::from_raw_varint(9),
        IdempotencyLevel::IdempotencyUnknown
    );

    for (raw, expected) in [
        (0, VerificationState::Declaration),
        (1, VerificationState::Unverified),
    ] {
        assert_eq!(VerificationState::from_i32(raw), Some(expected));
        assert_eq!(VerificationState::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(VerificationState::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(VerificationState::from_i32(9), None);
    assert_eq!(
        VerificationState::from_raw_varint(9),
        VerificationState::Declaration
    );

    for (raw, expected) in [
        (0, Semantic::None),
        (1, Semantic::Set),
        (2, Semantic::Alias),
    ] {
        assert_eq!(Semantic::from_i32(raw), Some(expected));
        assert_eq!(Semantic::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(Semantic::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(Semantic::from_i32(9), None);
    assert_eq!(Semantic::from_raw_varint(9), Semantic::None);

    for (raw, expected) in [
        (0, Feature::FeatureNone),
        (1, Feature::FeatureProto3Optional),
        (2, Feature::FeatureSupportsEditions),
    ] {
        assert_eq!(Feature::from_i32(raw), Some(expected));
        assert_eq!(Feature::try_from(raw), Ok(expected));
        assert_eq!(expected.as_i32(), raw);
        assert_eq!(Feature::from_raw_varint(raw as u64), expected);
    }
    assert_eq!(Feature::from_i32(99), None);
    assert_eq!(Feature::try_from(99), Err(99));
    assert_eq!(Feature::from_raw_varint(99), Feature::FeatureNone);
}

#[test]
fn empty_views_defaults_and_presence() {
    let fds = FileDescriptorSetView::from_bytes(&[]).unwrap();
    assert_eq!(fds.file().count(), 0);
    assert_eq!(fds.unknown_fields().count(), 0);

    let file = FileDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!file.has_name());
    assert_eq!(file.name(), Ok(""));
    assert!(!file.has_package());
    assert_eq!(file.package(), Ok(""));
    assert!(!file.has_options());
    assert!(file.options().unwrap().is_none());
    assert!(!file.has_source_code_info());
    assert!(file.source_code_info().unwrap().is_none());
    assert!(!file.has_syntax());
    assert_eq!(file.syntax(), Ok(""));
    assert!(!file.has_edition());
    assert_eq!(file.edition(), Ok(Edition::EditionUnknown));

    let msg = DescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!msg.has_name());
    assert_eq!(msg.name(), Ok(""));
    assert!(!msg.has_options());
    assert!(msg.options().unwrap().is_none());
    assert!(!msg.has_visibility());
    assert_eq!(msg.visibility(), Ok(SymbolVisibility::VisibilityUnset));

    let fld = FieldDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!fld.has_name());
    assert_eq!(fld.name(), Ok(""));
    assert!(!fld.has_extendee());
    assert_eq!(fld.extendee(), Ok(""));
    assert!(!fld.has_number());
    assert_eq!(fld.number(), Ok(0));
    assert!(!fld.has_label());
    assert_eq!(fld.label(), Ok(Label::LabelOptional));
    assert!(!fld.has_type());
    assert_eq!(fld.r#type(), Ok(Type::TypeDouble));
    assert!(!fld.has_type_name());
    assert_eq!(fld.type_name(), Ok(""));
    assert!(!fld.has_default_value());
    assert_eq!(fld.default_value(), Ok(""));
    assert!(!fld.has_options());
    assert!(fld.options().unwrap().is_none());
    assert!(!fld.has_oneof_index());
    assert_eq!(fld.oneof_index(), Ok(0));
    assert!(!fld.has_json_name());
    assert_eq!(fld.json_name(), Ok(""));
    assert!(!fld.has_proto3_optional());
    assert_eq!(fld.proto3_optional(), Ok(false));

    let oneof = OneofDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!oneof.has_name());
    assert_eq!(oneof.name(), Ok(""));
    assert!(!oneof.has_options());
    assert!(oneof.options().unwrap().is_none());

    let en = EnumDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!en.has_name());
    assert_eq!(en.name(), Ok(""));
    assert!(!en.has_options());
    assert!(en.options().unwrap().is_none());
    assert!(!en.has_visibility());
    assert_eq!(en.visibility(), Ok(SymbolVisibility::VisibilityUnset));

    let ev = EnumValueDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!ev.has_name());
    assert_eq!(ev.name(), Ok(""));
    assert!(!ev.has_number());
    assert_eq!(ev.number(), Ok(0));
    assert!(!ev.has_options());
    assert!(ev.options().unwrap().is_none());

    let svc = ServiceDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!svc.has_name());
    assert_eq!(svc.name(), Ok(""));
    assert!(!svc.has_options());
    assert!(svc.options().unwrap().is_none());

    let m = MethodDescriptorProtoView::from_bytes(&[]).unwrap();
    assert!(!m.has_name());
    assert_eq!(m.name(), Ok(""));
    assert!(!m.has_input_type());
    assert_eq!(m.input_type(), Ok(""));
    assert!(!m.has_output_type());
    assert_eq!(m.output_type(), Ok(""));
    assert!(!m.has_options());
    assert!(m.options().unwrap().is_none());
    assert!(!m.has_client_streaming());
    assert_eq!(m.client_streaming(), Ok(false));
    assert!(!m.has_server_streaming());
    assert_eq!(m.server_streaming(), Ok(false));

    let fopt = FileOptionsView::from_bytes(&[]).unwrap();
    assert!(!fopt.has_java_package());
    assert_eq!(fopt.java_package(), Ok(""));
    assert!(!fopt.has_optimize_for());
    assert_eq!(fopt.optimize_for(), Ok(OptimizeMode::Speed));
    assert!(!fopt.has_cc_enable_arenas());
    assert_eq!(fopt.cc_enable_arenas(), Ok(true));
    assert!(!fopt.has_deprecated());
    assert_eq!(fopt.deprecated(), Ok(false));

    let mopt = MessageOptionsView::from_bytes(&[]).unwrap();
    assert!(!mopt.has_map_entry());
    assert_eq!(mopt.map_entry(), Ok(false));
    assert_eq!(mopt.message_set_wire_format(), Ok(false));
    assert_eq!(mopt.no_standard_descriptor_accessor(), Ok(false));
    assert_eq!(mopt.deprecated(), Ok(false));

    let flopt = FieldOptionsView::from_bytes(&[]).unwrap();
    assert!(!flopt.has_ctype());
    assert_eq!(flopt.ctype(), Ok(CType::String));
    assert!(!flopt.has_packed());
    assert_eq!(flopt.packed(), Ok(false));
    assert!(!flopt.has_jstype());
    assert_eq!(flopt.jstype(), Ok(JSType::JsNormal));
    assert!(!flopt.has_retention());
    assert_eq!(flopt.retention(), Ok(OptionRetention::RetentionUnknown));

    let ver = VersionView::from_bytes(&[]).unwrap();
    assert!(ver.as_bytes().is_empty());
    assert!(!ver.has_major());
    assert_eq!(ver.major(), Ok(0));
    assert!(!ver.has_minor());
    assert_eq!(ver.minor(), Ok(0));
    assert!(!ver.has_patch());
    assert_eq!(ver.patch(), Ok(0));
    assert!(!ver.has_suffix());
    assert_eq!(ver.suffix(), Ok(""));
    assert_eq!(ver.unknown_fields().count(), 0);

    let req = CodeGeneratorRequestView::from_bytes(&[]).unwrap();
    assert!(req.as_bytes().is_empty());
    assert_eq!(req.file_to_generate().count(), 0);
    assert!(!req.has_parameter());
    assert_eq!(req.parameter(), Ok(""));
    assert_eq!(req.proto_file().count(), 0);
    assert_eq!(req.source_file_descriptors().count(), 0);
    assert!(!req.has_compiler_version());
    assert!(req.compiler_version().unwrap().is_none());
    assert_eq!(req.unknown_fields().count(), 0);

    let resp_file = FileView::from_bytes(&[]).unwrap();
    assert!(resp_file.as_bytes().is_empty());
    assert!(!resp_file.has_name());
    assert_eq!(resp_file.name(), Ok(""));
    assert!(!resp_file.has_insertion_point());
    assert_eq!(resp_file.insertion_point(), Ok(""));
    assert!(!resp_file.has_content());
    assert_eq!(resp_file.content(), Ok(""));
    assert!(!resp_file.has_generated_code_info());
    assert!(resp_file.generated_code_info().unwrap().is_none());
    assert_eq!(resp_file.unknown_fields().count(), 0);

    let resp = CodeGeneratorResponseView::from_bytes(&[]).unwrap();
    assert!(resp.as_bytes().is_empty());
    assert!(!resp.has_error());
    assert_eq!(resp.error(), Ok(""));
    assert!(!resp.has_supported_features());
    assert_eq!(resp.supported_features(), Ok(0));
    assert!(!resp.has_minimum_edition());
    assert_eq!(resp.minimum_edition(), Ok(0));
    assert!(!resp.has_maximum_edition());
    assert_eq!(resp.maximum_edition(), Ok(0));
    assert_eq!(resp.file().count(), 0);
    assert_eq!(resp.unknown_fields().count(), 0);
}

#[test]
fn full_descriptor_roundtrip_and_unknown_fields() {
    let mut backing = [0u8; 1024];
    let mut writer = BackwardWriter::new(&mut backing);
    writer.write_varint(12345).unwrap();
    writer.write_tag(99, WireType::Varint).unwrap();

    let mut fds = FileDescriptorSetBuilder::from_writer(writer);
    fds.write_file(|file| {
        file.writer_mut().write_varint(777).unwrap();
        file.writer_mut().write_tag(88, WireType::Varint).unwrap();

        file.write_edition(Edition::Edition2024)
            .write_syntax("editions")
            .write_source_code_info(|sci| {
                sci.write_location(|loc| {
                    loc.write_leading_comments("file comment")
                        .write_packed_span(&[0, 0, 10, 1])
                        .write_packed_path(&[12]);
                });
            })
            .write_options(|opt| {
                opt.write_cc_enable_arenas(false)
                    .write_optimize_for(OptimizeMode::CodeSize)
                    .write_java_package("com.example.demo");
            })
            .write_extension(|ext| {
                ext.write_type(Type::TypeString)
                    .write_label(Label::LabelOptional)
                    .write_number(1000)
                    .write_extendee(".google.protobuf.MessageOptions")
                    .write_name("custom_ext");
            })
            .write_service(|svc| {
                svc.write_options(|opt| {
                    opt.write_deprecated(true);
                })
                .write_method(|m| {
                    m.write_server_streaming(true)
                        .write_client_streaming(false)
                        .write_options(|opt| {
                            opt.write_idempotency_level(IdempotencyLevel::NoSideEffects)
                                .write_deprecated(true);
                        })
                        .write_output_type(".demo.SensorReading")
                        .write_input_type(".demo.Location")
                        .write_name("ReadStream");
                })
                .write_name("TelemetryService");
            })
            .write_enum_type(|en| {
                en.write_visibility(SymbolVisibility::VisibilityExport)
                    .write_reserved_name("OLD_STATUS")
                    .write_reserved_range(|rr| {
                        rr.write_end(20).write_start(10);
                    })
                    .write_options(|opt| {
                        opt.write_allow_alias(true);
                    })
                    .write_value(|val| {
                        val.write_options(|opt| {
                            opt.write_deprecated(true);
                        })
                        .write_number(0)
                        .write_name("UNKNOWN");
                    })
                    .write_name("DeviceState");
            })
            .write_message_type(|msg| {
                msg.write_visibility(SymbolVisibility::VisibilityLocal)
                    .write_reserved_name("legacy_field")
                    .write_reserved_range(|rr| {
                        rr.write_end(200).write_start(100);
                    })
                    .write_oneof_decl(|oo| {
                        oo.write_options(|opt| {
                            opt.write_features(|fs| {
                                fs.write_field_presence(FieldPresence::Explicit);
                            });
                        })
                        .write_name("payload");
                    })
                    .write_options(|opt| {
                        opt.write_map_entry(true);
                    })
                    .write_extension(|ext| {
                        ext.write_type(Type::TypeInt32)
                            .write_number(500)
                            .write_extendee(".demo.SensorReading")
                            .write_name("nested_ext");
                    })
                    .write_extension_range(|er| {
                        er.write_options(|opt| {
                            opt.write_verification(VerificationState::Declaration);
                        })
                        .write_end(1000)
                        .write_start(500);
                    })
                    .write_enum_type(|nes_en| {
                        nes_en.write_name("NestedEnum");
                    })
                    .write_nested_type(|nes_msg| {
                        nes_msg.write_name("NestedMsg");
                    })
                    .write_field(|fld| {
                        fld.write_proto3_optional(true)
                            .write_json_name("deviceId")
                            .write_oneof_index(0)
                            .write_options(|opt| {
                                opt.write_packed(true);
                            })
                            .write_default_value("unknown")
                            .write_type_name(".demo.DeviceState")
                            .write_type(Type::TypeEnum)
                            .write_label(Label::LabelOptional)
                            .write_number(2)
                            .write_name("device_id");
                    })
                    .write_name("SensorReading");
            })
            .write_option_dependency("custom_options.proto")
            .write_weak_dependency(1)
            .write_public_dependency(0)
            .write_dependency("common.proto")
            .write_package("demo")
            .write_name("sensor.proto");
    });
    assert!(fds.writer().written_len() > 0);
    let wire = fds.finish().unwrap();

    let fds = FileDescriptorSetView::from_bytes(wire).unwrap();
    assert_eq!(fds.unknown_fields().count(), 1);

    let mut files = fds.file();
    let file = files.next().unwrap().unwrap();
    assert!(files.next().is_none());
    assert_eq!(file.unknown_fields().count(), 1);

    assert!(file.has_name());
    assert_eq!(file.name(), Ok("sensor.proto"));
    assert!(file.has_package());
    assert_eq!(file.package(), Ok("demo"));
    assert!(file.has_syntax());
    assert_eq!(file.syntax(), Ok("editions"));
    assert!(file.has_edition());
    assert_eq!(file.edition(), Ok(Edition::Edition2024));
    assert!(file.has_options());
    let file_opt = file.options().unwrap().unwrap();
    assert_eq!(file_opt.java_package(), Ok("com.example.demo"));
    assert_eq!(file_opt.optimize_for(), Ok(OptimizeMode::CodeSize));
    assert_eq!(file_opt.cc_enable_arenas(), Ok(false));

    assert!(file.has_source_code_info());
    let sci = file.source_code_info().unwrap().unwrap();
    let loc = sci.location().next().unwrap().unwrap();
    assert_eq!(loc.leading_comments(), Ok("file comment"));

    let mut deps = file.dependency();
    assert_eq!(deps.next(), Some(Ok("common.proto")));
    assert!(deps.next().is_none());

    let mut pub_deps = file.public_dependency();
    assert_eq!(pub_deps.next(), Some(Ok(0)));
    assert!(pub_deps.next().is_none());

    let mut weak_deps = file.weak_dependency();
    assert_eq!(weak_deps.next(), Some(Ok(1)));
    assert!(weak_deps.next().is_none());

    let mut opt_deps = file.option_dependency();
    assert_eq!(opt_deps.next(), Some(Ok("custom_options.proto")));
    assert!(opt_deps.next().is_none());

    let mut msgs = file.message_type();
    let msg = msgs.next().unwrap().unwrap();
    assert!(msgs.next().is_none());
    assert_eq!(msg.name(), Ok("SensorReading"));
    assert_eq!(msg.visibility(), Ok(SymbolVisibility::VisibilityLocal));
    let msg_opt = msg.options().unwrap().unwrap();
    assert_eq!(msg_opt.map_entry(), Ok(true));

    let mut fields = msg.field();
    let fld = fields.next().unwrap().unwrap();
    assert!(fields.next().is_none());
    assert_eq!(fld.name(), Ok("device_id"));
    assert_eq!(fld.number(), Ok(2));
    assert_eq!(fld.label(), Ok(Label::LabelOptional));
    assert_eq!(fld.r#type(), Ok(Type::TypeEnum));
    assert_eq!(fld.type_name(), Ok(".demo.DeviceState"));
    assert_eq!(fld.default_value(), Ok("unknown"));
    assert_eq!(fld.oneof_index(), Ok(0));
    assert_eq!(fld.json_name(), Ok("deviceId"));
    assert_eq!(fld.proto3_optional(), Ok(true));
    let fld_opt = fld.options().unwrap().unwrap();
    assert_eq!(fld_opt.packed(), Ok(true));

    let mut nested_msgs = msg.nested_type();
    let nm = nested_msgs.next().unwrap().unwrap();
    assert_eq!(nm.name(), Ok("NestedMsg"));

    let mut nested_enums = msg.enum_type();
    let ne = nested_enums.next().unwrap().unwrap();
    assert_eq!(ne.name(), Ok("NestedEnum"));

    let mut ext_ranges = msg.extension_range();
    let er = ext_ranges.next().unwrap().unwrap();
    assert_eq!(er.start(), Ok(500));
    assert_eq!(er.end(), Ok(1000));
    let er_opt = er.options().unwrap().unwrap();
    assert_eq!(er_opt.verification(), Ok(VerificationState::Declaration));

    let mut msg_exts = msg.extension();
    let me = msg_exts.next().unwrap().unwrap();
    assert_eq!(me.name(), Ok("nested_ext"));
    assert_eq!(me.extendee(), Ok(".demo.SensorReading"));
    assert_eq!(me.number(), Ok(500));

    let mut oneofs = msg.oneof_decl();
    let oo = oneofs.next().unwrap().unwrap();
    assert_eq!(oo.name(), Ok("payload"));
    let oo_opt = oo.options().unwrap().unwrap();
    assert_eq!(
        oo_opt.features().unwrap().unwrap().field_presence(),
        Ok(FieldPresence::Explicit)
    );

    let mut res_ranges = msg.reserved_range();
    let rr = res_ranges.next().unwrap().unwrap();
    assert_eq!(rr.start(), Ok(100));
    assert_eq!(rr.end(), Ok(200));

    let mut res_names = msg.reserved_name();
    assert_eq!(res_names.next(), Some(Ok("legacy_field")));

    let mut enums = file.enum_type();
    let en = enums.next().unwrap().unwrap();
    assert_eq!(en.name(), Ok("DeviceState"));
    assert_eq!(en.visibility(), Ok(SymbolVisibility::VisibilityExport));
    assert_eq!(en.options().unwrap().unwrap().allow_alias(), Ok(true));
    let mut evs = en.value();
    let ev = evs.next().unwrap().unwrap();
    assert_eq!(ev.name(), Ok("UNKNOWN"));
    assert_eq!(ev.number(), Ok(0));
    assert_eq!(ev.options().unwrap().unwrap().deprecated(), Ok(true));
    let mut er_iter = en.reserved_range();
    let err = er_iter.next().unwrap().unwrap();
    assert_eq!(err.start(), Ok(10));
    assert_eq!(err.end(), Ok(20));
    let mut en_rn = en.reserved_name();
    assert_eq!(en_rn.next(), Some(Ok("OLD_STATUS")));

    let mut svcs = file.service();
    let svc = svcs.next().unwrap().unwrap();
    assert_eq!(svc.name(), Ok("TelemetryService"));
    assert_eq!(svc.options().unwrap().unwrap().deprecated(), Ok(true));
    let mut methods = svc.method();
    let m = methods.next().unwrap().unwrap();
    assert_eq!(m.name(), Ok("ReadStream"));
    assert_eq!(m.input_type(), Ok(".demo.Location"));
    assert_eq!(m.output_type(), Ok(".demo.SensorReading"));
    assert_eq!(m.client_streaming(), Ok(false));
    assert_eq!(m.server_streaming(), Ok(true));
    let m_opt = m.options().unwrap().unwrap();
    assert_eq!(m_opt.deprecated(), Ok(true));
    assert_eq!(
        m_opt.idempotency_level(),
        Ok(IdempotencyLevel::NoSideEffects)
    );

    let mut file_exts = file.extension();
    let fe = file_exts.next().unwrap().unwrap();
    assert_eq!(fe.name(), Ok("custom_ext"));
    assert_eq!(fe.extendee(), Ok(".google.protobuf.MessageOptions"));
    assert_eq!(fe.number(), Ok(1000));
}

#[test]
fn options_and_custom_extensions_roundtrip() {
    let mut buf = [0u8; 512];
    let mut fopt = FileOptionsBuilder::new(&mut buf);
    fopt.writer.write_varint(42).unwrap();
    fopt.writer.write_tag(50001, WireType::Varint).unwrap();

    fopt.write_uninterpreted_option(|uo| {
        uo.write_aggregate_value("{ a: 1 }")
            .write_string_value(b"raw\xFF")
            .write_double_value(3.25)
            .write_negative_int_value(-99)
            .write_positive_int_value(123)
            .write_identifier_value("IDENT")
            .write_name(|np| {
                np.write_is_extension(true)
                    .write_name_part("pkg.custom_opt");
            })
            .write_name(|np| {
                np.write_is_extension(false).write_name_part("foo");
            });
    })
    .write_features(|fs| {
        fs.write_default_symbol_visibility(DefaultSymbolVisibility::ExportTopLevel)
            .write_enforce_naming_style(EnforceNamingStyle::Style2024)
            .write_json_format(JsonFormat::Allow)
            .write_message_encoding(MessageEncoding::LengthPrefixed)
            .write_utf8_validation(Utf8Validation::Verify)
            .write_repeated_field_encoding(RepeatedFieldEncoding::Packed)
            .write_enum_type(EnumType::Closed)
            .write_field_presence(FieldPresence::Explicit);
    })
    .write_ruby_package("Demo::Ruby")
    .write_php_metadata_namespace("Demo\\Meta")
    .write_php_namespace("Demo\\Php")
    .write_php_class_prefix("DMO")
    .write_swift_prefix("DMO")
    .write_csharp_namespace("Demo.CSharp")
    .write_objc_class_prefix("DMO")
    .write_cc_enable_arenas(false)
    .write_java_string_check_utf8(true)
    .write_deprecated(true)
    .write_java_generate_equals_and_hash(true)
    .write_py_generic_services(true)
    .write_java_generic_services(true)
    .write_cc_generic_services(true)
    .write_go_package("example.com/demo")
    .write_java_multiple_files(true)
    .write_optimize_for(OptimizeMode::LiteRuntime)
    .write_java_outer_classname("DemoOuter")
    .write_java_package("com.example.demo");
    let wire = fopt.finish().unwrap();

    let view = FileOptionsView::from_bytes(wire).unwrap();
    assert_eq!(view.java_package(), Ok("com.example.demo"));
    assert_eq!(view.java_outer_classname(), Ok("DemoOuter"));
    assert_eq!(view.optimize_for(), Ok(OptimizeMode::LiteRuntime));
    assert_eq!(view.java_multiple_files(), Ok(true));
    assert_eq!(view.go_package(), Ok("example.com/demo"));
    assert_eq!(view.cc_generic_services(), Ok(true));
    assert_eq!(view.java_generic_services(), Ok(true));
    assert_eq!(view.py_generic_services(), Ok(true));
    assert_eq!(view.java_generate_equals_and_hash(), Ok(true));
    assert_eq!(view.deprecated(), Ok(true));
    assert_eq!(view.java_string_check_utf8(), Ok(true));
    assert_eq!(view.cc_enable_arenas(), Ok(false));
    assert_eq!(view.objc_class_prefix(), Ok("DMO"));
    assert_eq!(view.csharp_namespace(), Ok("Demo.CSharp"));
    assert_eq!(view.swift_prefix(), Ok("DMO"));
    assert_eq!(view.php_class_prefix(), Ok("DMO"));
    assert_eq!(view.php_namespace(), Ok("Demo\\Php"));
    assert_eq!(view.php_metadata_namespace(), Ok("Demo\\Meta"));
    assert_eq!(view.ruby_package(), Ok("Demo::Ruby"));

    let fs = view.features().unwrap().unwrap();
    assert_eq!(fs.field_presence(), Ok(FieldPresence::Explicit));
    assert_eq!(fs.enum_type(), Ok(EnumType::Closed));
    assert_eq!(
        fs.repeated_field_encoding(),
        Ok(RepeatedFieldEncoding::Packed)
    );
    assert_eq!(fs.utf8_validation(), Ok(Utf8Validation::Verify));
    assert_eq!(fs.message_encoding(), Ok(MessageEncoding::LengthPrefixed));
    assert_eq!(fs.json_format(), Ok(JsonFormat::Allow));
    assert_eq!(fs.enforce_naming_style(), Ok(EnforceNamingStyle::Style2024));
    assert_eq!(
        fs.default_symbol_visibility(),
        Ok(DefaultSymbolVisibility::ExportTopLevel)
    );

    let mut uos = view.uninterpreted_option();
    let uo = uos.next().unwrap().unwrap();
    assert!(uos.next().is_none());
    assert_eq!(uo.identifier_value(), Ok("IDENT"));
    assert_eq!(uo.positive_int_value(), Ok(123));
    assert_eq!(uo.negative_int_value(), Ok(-99));
    assert_eq!(uo.double_value(), Ok(3.25));
    assert_eq!(uo.string_value(), Ok(&b"raw\xFF"[..]));
    assert_eq!(uo.aggregate_value(), Ok("{ a: 1 }"));
    let mut names = uo.name();
    let np1 = names.next().unwrap().unwrap();
    assert_eq!(np1.name_part(), Ok("foo"));
    assert_eq!(np1.is_extension(), Ok(false));
    let np2 = names.next().unwrap().unwrap();
    assert_eq!(np2.name_part(), Ok("pkg.custom_opt"));
    assert_eq!(np2.is_extension(), Ok(true));
    assert!(names.next().is_none());

    assert_eq!(view.unknown_fields().count(), 1);
    let mut copy_buf = [0u8; 64];
    let mut b = FileOptionsBuilder::new(&mut copy_buf);
    b.copy_unknown_fields(&view);
    let copied = FileOptionsView::from_bytes(b.finish().unwrap()).unwrap();
    assert_eq!(copied.unknown_fields().count(), 1);

    let mut fbuf = [0u8; 256];
    let mut flopt = FieldOptionsBuilder::new(&mut fbuf);
    flopt.writer.write_length_delimited(b"ext_val").unwrap();
    flopt
        .writer
        .write_tag(50002, WireType::LengthDelimited)
        .unwrap();

    flopt
        .write_feature_support(|fsup| {
            fsup.write_edition_removed(Edition::Edition2026)
                .write_deprecation_warning("use new_field")
                .write_edition_deprecated(Edition::Edition2024)
                .write_edition_introduced(Edition::Edition2023);
        })
        .write_edition_defaults(|ed| {
            ed.write_value("42").write_edition(Edition::EditionProto2);
        })
        .write_packed_targets(&[
            OptionTargetType::TargetTypeField,
            OptionTargetType::TargetTypeMessage,
        ])
        .write_targets(OptionTargetType::TargetTypeFile)
        .write_retention(OptionRetention::RetentionSource)
        .write_debug_redact(true)
        .write_unverified_lazy(true)
        .write_weak(true)
        .write_jstype(JSType::JsString)
        .write_lazy(true)
        .write_deprecated(true)
        .write_packed(true)
        .write_ctype(CType::Cord);
    let fwire = flopt.finish().unwrap();
    let fview = FieldOptionsView::from_bytes(fwire).unwrap();
    assert_eq!(fview.ctype(), Ok(CType::Cord));
    assert_eq!(fview.packed(), Ok(true));
    assert_eq!(fview.deprecated(), Ok(true));
    assert_eq!(fview.lazy(), Ok(true));
    assert_eq!(fview.jstype(), Ok(JSType::JsString));
    assert_eq!(fview.weak(), Ok(true));
    assert_eq!(fview.unverified_lazy(), Ok(true));
    assert_eq!(fview.debug_redact(), Ok(true));
    assert_eq!(fview.retention(), Ok(OptionRetention::RetentionSource));

    let mut targets = fview.targets();
    assert_eq!(targets.next(), Some(Ok(OptionTargetType::TargetTypeFile)));
    assert_eq!(targets.next(), Some(Ok(OptionTargetType::TargetTypeField)));
    assert_eq!(
        targets.next(),
        Some(Ok(OptionTargetType::TargetTypeMessage))
    );
    assert!(targets.next().is_none());

    let mut ed_iter = fview.edition_defaults();
    let ed = ed_iter.next().unwrap().unwrap();
    assert!(ed_iter.next().is_none());
    assert_eq!(ed.edition(), Ok(Edition::EditionProto2));
    assert_eq!(ed.value(), Ok("42"));

    let fsup = fview.feature_support().unwrap().unwrap();
    assert_eq!(fsup.edition_introduced(), Ok(Edition::Edition2023));
    assert_eq!(fsup.edition_deprecated(), Ok(Edition::Edition2024));
    assert_eq!(fsup.deprecation_warning(), Ok("use new_field"));
    assert_eq!(fsup.edition_removed(), Ok(Edition::Edition2026));
    assert_eq!(fview.unknown_fields().count(), 1);
}

#[test]
fn name_part_missing_required_fields() {
    assert_eq!(
        NamePartView::from_bytes(&[]).unwrap_err(),
        DecodeError::MissingRequiredField("NamePart.name_part")
    );

    let mut buf = [0u8; 32];
    let mut b = NamePartBuilder::new(&mut buf);
    b.write_name_part("foo");
    assert_eq!(
        NamePartView::from_bytes(b.finish().unwrap()).unwrap_err(),
        DecodeError::MissingRequiredField("NamePart.is_extension")
    );

    let mut buf2 = [0u8; 32];
    let mut b2 = NamePartBuilder::new(&mut buf2);
    b2.write_is_extension(true);
    assert_eq!(
        NamePartView::from_bytes(b2.finish().unwrap()).unwrap_err(),
        DecodeError::MissingRequiredField("NamePart.name_part")
    );
}

#[test]
fn source_code_info_and_generated_code_info_roundtrip() {
    let mut buf = [0u8; 256];
    let mut sci = SourceCodeInfoBuilder::new(&mut buf);
    sci.write_location(|loc| {
        loc.write_leading_detached_comments("detached 2")
            .write_leading_detached_comments("detached 1")
            .write_trailing_comments("trailing")
            .write_leading_comments("leading")
            .write_span(20)
            .write_packed_span(&[1, 2, 10])
            .write_path(1)
            .write_packed_path(&[4, 0, 2, 0]);
    });
    let sci_view = SourceCodeInfoView::from_bytes(sci.finish().unwrap()).unwrap();
    let mut locs = sci_view.location();
    let loc = locs.next().unwrap().unwrap();
    assert!(locs.next().is_none());

    let mut path_iter = loc.path();
    for expected in [4, 0, 2, 0, 1] {
        assert_eq!(path_iter.next(), Some(Ok(expected)));
    }
    assert!(path_iter.next().is_none());

    let mut span_iter = loc.span();
    for expected in [1, 2, 10, 20] {
        assert_eq!(span_iter.next(), Some(Ok(expected)));
    }
    assert!(span_iter.next().is_none());

    assert!(loc.has_leading_comments());
    assert_eq!(loc.leading_comments(), Ok("leading"));
    assert!(loc.has_trailing_comments());
    assert_eq!(loc.trailing_comments(), Ok("trailing"));

    let mut detached = loc.leading_detached_comments();
    assert_eq!(detached.next(), Some(Ok("detached 1")));
    assert_eq!(detached.next(), Some(Ok("detached 2")));
    assert!(detached.next().is_none());

    let mut gbuf = [0u8; 256];
    let mut gci = GeneratedCodeInfoBuilder::new(&mut gbuf);
    gci.write_annotation(|ann| {
        ann.write_semantic(Semantic::Set)
            .write_end(150)
            .write_begin(100)
            .write_source_file("sensor.proto")
            .write_path(1)
            .write_packed_path(&[4, 0]);
    });
    let gci_view = GeneratedCodeInfoView::from_bytes(gci.finish().unwrap()).unwrap();
    let mut anns = gci_view.annotation();
    let ann = anns.next().unwrap().unwrap();
    assert!(anns.next().is_none());

    let mut ann_path = ann.path();
    assert_eq!(ann_path.next(), Some(Ok(4)));
    assert_eq!(ann_path.next(), Some(Ok(0)));
    assert_eq!(ann_path.next(), Some(Ok(1)));
    assert!(ann_path.next().is_none());
    assert_eq!(ann.source_file(), Ok("sensor.proto"));
    assert_eq!(ann.begin(), Ok(100));
    assert_eq!(ann.end(), Ok(150));
    assert_eq!(ann.semantic(), Ok(Semantic::Set));
}

#[test]
fn version_and_request_roundtrip_with_unknown_fields() {
    let mut backing = [0u8; 1024];
    let mut req = CodeGeneratorRequestBuilder::new(&mut backing);
    req.writer.write_varint(4242).unwrap();
    req.writer.write_tag(99, WireType::Varint).unwrap();

    req.write_compiler_version(|ver: &mut VersionBuilder<'_>| {
        ver.writer.write_varint(999).unwrap();
        ver.writer.write_tag(88, WireType::Varint).unwrap();

        ver.write_suffix("rc1")
            .write_patch(2)
            .write_minor(28)
            .write_major(4);
    })
    .write_source_file_descriptor(|fdp| {
        fdp.write_syntax("proto2")
            .write_package("demo")
            .write_name("sensor.proto");
    })
    .write_proto_file(|fdp| {
        fdp.write_message_type(|msg| {
            msg.write_field(|fld| {
                fld.write_type(Type::TypeInt32)
                    .write_label(Label::LabelRequired)
                    .write_number(1)
                    .write_name("lat_e7");
            })
            .write_name("Location");
        })
        .write_package("demo")
        .write_name("sensor.proto");
    })
    .write_proto_file(|fdp| {
        fdp.write_package("google.protobuf")
            .write_name("google/protobuf/descriptor.proto");
    })
    .write_parameter("no_std=true,edition=2023")
    .write_file_to_generate("sensor.proto");
    let wire = req.finish().unwrap();

    let req_view = CodeGeneratorRequestView::from_bytes(wire).unwrap();
    assert_eq!(req_view.as_bytes(), wire);
    assert_eq!(req_view.unknown_fields().count(), 1);

    let mut files_to_gen = req_view.file_to_generate();
    assert_eq!(files_to_gen.next(), Some(Ok("sensor.proto")));
    assert!(files_to_gen.next().is_none());

    assert!(req_view.has_parameter());
    assert_eq!(req_view.parameter(), Ok("no_std=true,edition=2023"));

    let mut proto_files = req_view.proto_file();
    let pf1 = proto_files.next().unwrap().unwrap();
    assert_eq!(pf1.name(), Ok("google/protobuf/descriptor.proto"));
    assert_eq!(pf1.package(), Ok("google.protobuf"));
    let pf2 = proto_files.next().unwrap().unwrap();
    assert_eq!(pf2.name(), Ok("sensor.proto"));
    assert_eq!(pf2.package(), Ok("demo"));
    let mut msgs = pf2.message_type();
    let loc_msg = msgs.next().unwrap().unwrap();
    assert_eq!(loc_msg.name(), Ok("Location"));
    assert!(proto_files.next().is_none());

    let mut src_files = req_view.source_file_descriptors();
    let sf1 = src_files.next().unwrap().unwrap();
    assert_eq!(sf1.name(), Ok("sensor.proto"));
    assert_eq!(sf1.syntax(), Ok("proto2"));
    assert!(src_files.next().is_none());

    assert!(req_view.has_compiler_version());
    let ver_view = req_view.compiler_version().unwrap().unwrap();
    assert!(ver_view.has_major());
    assert_eq!(ver_view.major(), Ok(4));
    assert!(ver_view.has_minor());
    assert_eq!(ver_view.minor(), Ok(28));
    assert!(ver_view.has_patch());
    assert_eq!(ver_view.patch(), Ok(2));
    assert!(ver_view.has_suffix());
    assert_eq!(ver_view.suffix(), Ok("rc1"));
    assert_eq!(ver_view.unknown_fields().count(), 1);

    let mut backing2 = [0u8; 256];
    let mut req2 = CodeGeneratorRequestBuilder::new(&mut backing2);
    req2.copy_unknown_fields(&req_view)
        .write_compiler_version(|ver2| {
            ver2.copy_unknown_fields(&ver_view).write_major(4);
        });
    let req_view2 = CodeGeneratorRequestView::from_bytes(req2.finish().unwrap()).unwrap();
    assert_eq!(req_view2.unknown_fields().count(), 1);
    let ver_view2 = req_view2.compiler_version().unwrap().unwrap();
    assert_eq!(ver_view2.major(), Ok(4));
    assert_eq!(ver_view2.unknown_fields().count(), 1);
}

#[test]
fn response_and_file_roundtrip_with_generated_code_info_and_unknown_fields() {
    let mut backing = [0u8; 1024];
    let mut resp = CodeGeneratorResponseBuilder::new(&mut backing);
    resp.writer.write_varint(7777).unwrap();
    resp.writer.write_tag(77, WireType::Varint).unwrap();

    resp.write_file(|file| {
        file.writer.write_varint(6666).unwrap();
        file.writer.write_tag(66, WireType::Varint).unwrap();

        file.write_generated_code_info(|gci| {
            gci.write_annotation(|ann| {
                ann.write_semantic(Semantic::Set)
                    .write_end(42)
                    .write_begin(10)
                    .write_source_file("sensor.proto")
                    .write_packed_path(&[4, 0, 2, 0]);
            });
        })
        .write_content("pub struct LocationView<'a> { buf: &'a [u8] }\n")
        .write_insertion_point("module_scope")
        .write_name("sensor.rs");
    })
    .write_maximum_edition(Edition::Edition2024.as_i32())
    .write_minimum_edition(Edition::EditionProto2.as_i32())
    .write_supported_features(
        (Feature::FeatureProto3Optional.as_i32() | Feature::FeatureSupportsEditions.as_i32())
            as u64,
    )
    .write_error("");
    let wire = resp.finish().unwrap();

    let resp_view = CodeGeneratorResponseView::from_bytes(wire).unwrap();
    assert_eq!(resp_view.as_bytes(), wire);
    assert_eq!(resp_view.unknown_fields().count(), 1);
    assert!(resp_view.has_error());
    assert_eq!(resp_view.error(), Ok(""));
    assert!(resp_view.has_supported_features());
    assert_eq!(resp_view.supported_features(), Ok(3));
    assert!(resp_view.has_minimum_edition());
    assert_eq!(
        resp_view.minimum_edition(),
        Ok(Edition::EditionProto2.as_i32())
    );
    assert!(resp_view.has_maximum_edition());
    assert_eq!(
        resp_view.maximum_edition(),
        Ok(Edition::Edition2024.as_i32())
    );

    let mut files = resp_view.file();
    let f_view = files.next().unwrap().unwrap();
    assert!(files.next().is_none());
    assert_eq!(f_view.unknown_fields().count(), 1);
    assert!(f_view.has_name());
    assert_eq!(f_view.name(), Ok("sensor.rs"));
    assert!(f_view.has_insertion_point());
    assert_eq!(f_view.insertion_point(), Ok("module_scope"));
    assert!(f_view.has_content());
    assert_eq!(
        f_view.content(),
        Ok("pub struct LocationView<'a> { buf: &'a [u8] }\n")
    );
    assert!(f_view.has_generated_code_info());
    let gci = f_view.generated_code_info().unwrap().unwrap();
    let ann = gci.annotation().next().unwrap().unwrap();
    assert_eq!(ann.source_file(), Ok("sensor.proto"));
    assert_eq!(ann.begin(), Ok(10));
    assert_eq!(ann.end(), Ok(42));
    assert_eq!(ann.semantic(), Ok(Semantic::Set));
    let path: [i32; 4] = [
        ann.path().next().unwrap().unwrap(),
        ann.path().nth(1).unwrap().unwrap(),
        ann.path().nth(2).unwrap().unwrap(),
        ann.path().nth(3).unwrap().unwrap(),
    ];
    assert_eq!(path, [4, 0, 2, 0]);

    let mut backing2 = [0u8; 256];
    let mut resp2 = CodeGeneratorResponseBuilder::new(&mut backing2);
    resp2.copy_unknown_fields(&resp_view).write_file(|f2| {
        f2.copy_unknown_fields(&f_view).write_name("sensor.rs");
    });
    let resp_view2 = CodeGeneratorResponseView::from_bytes(resp2.finish().unwrap()).unwrap();
    assert_eq!(resp_view2.unknown_fields().count(), 1);
    let f_view2 = resp_view2.file().next().unwrap().unwrap();
    assert_eq!(f_view2.name(), Ok("sensor.rs"));
    assert_eq!(f_view2.unknown_fields().count(), 1);

    let mut f_backing = [0u8; 64];
    let mut fb = FileBuilder::new(&mut f_backing);
    fb.write_content("// ok").write_name("mod.rs");
    let fv = FileView::from_bytes(fb.finish().unwrap()).unwrap();
    assert_eq!(fv.name(), Ok("mod.rs"));
    assert_eq!(fv.content(), Ok("// ok"));
}

#[test]
fn partial_update_with_copy_unset_fields() {
    let mut backing1 = [0u8; 512];
    let mut req1 = CodeGeneratorRequestBuilder::new(&mut backing1);
    req1.writer.write_varint(9999).unwrap();
    req1.writer.write_tag(77, WireType::Varint).unwrap();
    req1.write_compiler_version(|ver| {
        ver.writer.write_varint(8888).unwrap();
        ver.writer.write_tag(66, WireType::Varint).unwrap();
        ver.write_suffix("rc1")
            .write_patch(2)
            .write_minor(28)
            .write_major(4);
    })
    .write_parameter("old_param")
    .write_file_to_generate("b.proto")
    .write_file_to_generate("a.proto");

    let req1_view = CodeGeneratorRequestView::from_bytes(req1.finish().unwrap()).unwrap();
    let ver1_view = req1_view.compiler_version().unwrap().unwrap();

    // Update `parameter` on CodeGeneratorRequest and `patch` on Version, copying everything else
    let mut backing2 = [0u8; 512];
    let mut req2 = CodeGeneratorRequestBuilder::new(&mut backing2);
    req2.write_compiler_version(|ver2| {
        ver2.write_patch(3).copy_unset_fields(&ver1_view);
    })
    .write_parameter("new_param")
    .copy_unset_fields(&req1_view);

    let req2_view = CodeGeneratorRequestView::from_bytes(req2.finish().unwrap()).unwrap();
    assert_eq!(req2_view.parameter(), Ok("new_param"));
    let mut ftg = req2_view.file_to_generate();
    assert_eq!(ftg.next(), Some(Ok("a.proto")));
    assert_eq!(ftg.next(), Some(Ok("b.proto")));
    assert!(ftg.next().is_none());
    assert_eq!(req2_view.unknown_fields().count(), 1);

    let ver2_view = req2_view.compiler_version().unwrap().unwrap();
    assert_eq!(ver2_view.major(), Ok(4));
    assert_eq!(ver2_view.minor(), Ok(28));
    assert_eq!(ver2_view.patch(), Ok(3));
    assert_eq!(ver2_view.suffix(), Ok("rc1"));
    assert_eq!(ver2_view.unknown_fields().count(), 1);
}

#[test]
fn eager_scalar_view_layout_and_last_one_wins_semantics() {
    // MessageOptions has 5 bool singular fields + 1 message field (`features`); storing bool + [u32; 1]
    // instead of 8-byte FieldMarkers reduces MessageOptionsView from 64 bytes to 40 bytes.
    assert_eq!(core::mem::size_of::<MessageOptionsView<'_>>(), 40);
    // FieldOptions has 9 scalar singular fields (3 enums + 6 bools) + 2 message fields; storing values + [u32; 1]
    // reduces FieldOptionsView from 104 bytes to 56 bytes.
    assert_eq!(core::mem::size_of::<FieldOptionsView<'_>>(), 56);

    // Verify last-one-wins overwrite and presence bitmask during from_bytes on VersionView
    let mut backing = [0u8; 64];
    let mut b = VersionBuilder::new(&mut backing);
    // In BackwardWriter, earlier write calls appear later on the wire (last-wins)
    b.write_major(4).write_major(3).write_minor(28);
    let v = VersionView::from_bytes(b.finish().unwrap()).unwrap();
    assert!(v.has_major());
    assert!(v.has_minor());
    assert!(!v.has_patch());
    assert_eq!(v.major(), Ok(4));
    assert_eq!(v.minor(), Ok(28));
    assert_eq!(v.patch(), Ok(0));
}


