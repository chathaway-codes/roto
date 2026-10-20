//! Round-trip encoding and decoding tests for the full `google/protobuf` Well-Known Types (WKT)
//! standard library in `google-protobuf-roto`.
//!
//! Covers the zero-allocation `*Builder<'b>` (`new(&mut buf)`, `&mut Self` chained `write_*`,
//! `copy_unknown_fields`, `copy_unset_fields`, `check()`, and `finish()`) and shallow zero-copy
//! `*View<'a>` (`from_bytes`, field accessors, `has_*`, repeated iterators, oneof accessors,
//! and `unknown_fields`) contracts across:
//! - `Timestamp` (`TimestampBuilder` / `TimestampView`, `copy_unset_fields`, `copy_unknown_fields`)
//! - `Duration` (`DurationBuilder` / `DurationView`)
//! - `Any` (`AnyBuilder` / `AnyView` wrapping a serialized inner `Timestamp`)
//! - `Empty` (`EmptyBuilder` / `EmptyView` with unknown field preservation)
//! - `FieldMask` (`FieldMaskBuilder` / `FieldMaskView` repeated `paths`)
//! - `SourceContext` (`SourceContextBuilder` / `SourceContextView`)
//! - `Struct`, `Value`, `ListValue`, `NullValue` (recursive `Struct` <-> `Value` <-> `ListValue`,
//!   all 6 `value::Kind` oneof variants: `NullValue`, `NumberValue`, `StringValue`, `BoolValue`,
//!   `StructValue`, `ListValue`, and `r#struct::FieldsEntry` map entries)
//! - `Type`, `Field`, `Enum`, `EnumValue`, `Option` (`field::{Cardinality, Kind}`, `Syntax`)
//! - `Api`, `Method`, `Mixin`
//! - All 9 wrapper types in `wrappers.proto`: `DoubleValue`, `FloatValue`, `Int64Value`,
//!   `UInt64Value`, `Int32Value`, `UInt32Value`, `BoolValue`, `StringValue`, `BytesValue`
//! - Edition feature set messages in `pb`: `CSharpFeatures`, `CppFeatures`, `GoFeatures`,
//!   `JavaFeatures`

use google_protobuf_roto::{
    google::protobuf::{
        field::{Cardinality, Kind as FieldKind},
        r#struct::{FieldsEntryBuilder, FieldsEntryView},
        value::Kind as ValueKind,
        AnyBuilder, AnyView, ApiBuilder, ApiView, BoolValueBuilder, BoolValueView,
        BytesValueBuilder, BytesValueView, DoubleValueBuilder, DoubleValueView, DurationBuilder,
        DurationView, EmptyBuilder, EmptyView, EnumBuilder, EnumValueBuilder, EnumValueView,
        EnumView, FieldBuilder, FieldMaskBuilder, FieldMaskView, FieldView, FloatValueBuilder,
        FloatValueView, Int32ValueBuilder, Int32ValueView, Int64ValueBuilder, Int64ValueView,
        ListValueBuilder, ListValueView, MethodBuilder, MethodView, MixinBuilder, MixinView,
        NullValue, OptionBuilder, OptionView, SourceContextBuilder, SourceContextView,
        StringValueBuilder, StringValueView, StructBuilder, StructView, Syntax, TimestampBuilder,
        TimestampView, TypeBuilder, TypeView, UInt32ValueBuilder, UInt32ValueView,
        UInt64ValueBuilder, UInt64ValueView, ValueBuilder, ValueView,
    },
    pb::{
        cpp_features::{RepeatedType, StringType},
        go_features::{
            optimize_mode_feature::OptimizeMode as GoOptimizeMode, APILevel, StripEnumPrefix,
        },
        java_features::{
            nest_in_file_class_feature::NestInFileClass, Utf8Validation as JavaUtf8Validation,
        },
        CSharpFeaturesBuilder, CSharpFeaturesView, CppFeaturesBuilder, CppFeaturesView,
        GoFeaturesBuilder, GoFeaturesView, JavaFeaturesBuilder, JavaFeaturesView,
    },
};
use roto::runtime::WireType;

#[test]
fn timestamp_and_duration_roundtrip_and_copy_unset_fields() {
    let mut ts_buf = [0u8; 64];
    let mut ts_builder = TimestampBuilder::new(&mut ts_buf);
    ts_builder.writer_mut().write_varint(999).unwrap();
    ts_builder
        .writer_mut()
        .write_tag(77, WireType::Varint)
        .unwrap();
    ts_builder
        .write_nanos(123_456_789)
        .write_seconds(1_700_000_000);
    ts_builder.check().unwrap();
    let ts_bytes = ts_builder.finish().unwrap();

    let ts_view = TimestampView::from_bytes(ts_bytes).unwrap();
    assert!(ts_view.has_seconds());
    assert_eq!(ts_view.seconds(), Ok(1_700_000_000));
    assert!(ts_view.has_nanos());
    assert_eq!(ts_view.nanos(), Ok(123_456_789));
    assert_eq!(ts_view.unknown_fields().count(), 1);

    // Partial update via copy_unset_fields: override `seconds` and preserve `nanos` + unknown tag 77
    let mut ts_buf2 = [0u8; 64];
    let mut ts_builder2 = TimestampBuilder::new(&mut ts_buf2);
    ts_builder2
        .write_seconds(1_700_000_042)
        .copy_unset_fields(&ts_view);
    let ts_bytes2 = ts_builder2.finish().unwrap();

    let ts_view2 = TimestampView::from_bytes(ts_bytes2).unwrap();
    assert_eq!(ts_view2.seconds(), Ok(1_700_000_042));
    assert_eq!(ts_view2.nanos(), Ok(123_456_789));
    assert_eq!(ts_view2.unknown_fields().count(), 1);

    // Duration round-trip (including negative duration values)
    let mut dur_buf = [0u8; 64];
    let mut dur_builder = DurationBuilder::new(&mut dur_buf);
    dur_builder.write_nanos(-500_000_000).write_seconds(-3600);
    let dur_bytes = dur_builder.finish().unwrap();

    let dur_view = DurationView::from_bytes(dur_bytes).unwrap();
    assert!(dur_view.has_seconds());
    assert_eq!(dur_view.seconds(), Ok(-3600));
    assert!(dur_view.has_nanos());
    assert_eq!(dur_view.nanos(), Ok(-500_000_000));
}

#[test]
fn any_empty_field_mask_and_source_context_roundtrip() {
    // 1. Serialize an inner Timestamp
    let mut inner_buf = [0u8; 32];
    let mut inner_ts = TimestampBuilder::new(&mut inner_buf);
    inner_ts.write_nanos(42).write_seconds(1_710_000_000);
    let inner_bytes = inner_ts.finish().unwrap();

    // 2. Wrap inside google.protobuf.Any
    let mut any_buf = [0u8; 128];
    let mut any_builder = AnyBuilder::new(&mut any_buf);
    any_builder
        .write_value(inner_bytes)
        .write_type_url("type.googleapis.com/google.protobuf.Timestamp");
    let any_bytes = any_builder.finish().unwrap();

    let any_view = AnyView::from_bytes(any_bytes).unwrap();
    assert!(any_view.has_type_url());
    assert_eq!(
        any_view.type_url(),
        Ok("type.googleapis.com/google.protobuf.Timestamp")
    );
    assert!(any_view.has_value());
    let unpacked_ts = TimestampView::from_bytes(any_view.value().unwrap()).unwrap();
    assert_eq!(unpacked_ts.seconds(), Ok(1_710_000_000));
    assert_eq!(unpacked_ts.nanos(), Ok(42));

    // 3. Empty message with unknown field preservation
    let mut empty_buf = [0u8; 32];
    let empty_builder = EmptyBuilder::new(&mut empty_buf);
    let empty_bytes = empty_builder.finish().unwrap();
    assert!(empty_bytes.is_empty());
    let empty_view = EmptyView::from_bytes(empty_bytes).unwrap();
    assert_eq!(empty_view.unknown_fields().count(), 0);

    let mut empty_with_unknown_buf = [0u8; 32];
    let mut empty_with_unknown = EmptyBuilder::new(&mut empty_with_unknown_buf);
    empty_with_unknown.writer_mut().write_varint(123).unwrap();
    empty_with_unknown
        .writer_mut()
        .write_tag(15, WireType::Varint)
        .unwrap();
    let empty_unk_bytes = empty_with_unknown.finish().unwrap();
    let empty_unk_view = EmptyView::from_bytes(empty_unk_bytes).unwrap();
    assert_eq!(empty_unk_view.unknown_fields().count(), 1);

    let mut empty_copy_buf = [0u8; 32];
    let mut empty_copy_builder = EmptyBuilder::new(&mut empty_copy_buf);
    empty_copy_builder.copy_unknown_fields(&empty_unk_view);
    let empty_copy_view = EmptyView::from_bytes(empty_copy_builder.finish().unwrap()).unwrap();
    assert_eq!(empty_copy_view.unknown_fields().count(), 1);

    // 4. FieldMask repeated paths (written in reverse order for backward writer)
    let mut mask_buf = [0u8; 128];
    let mut mask_builder = FieldMaskBuilder::new(&mut mask_buf);
    mask_builder
        .write_path("user.Output.email")
        .write_path("user.profile.display_name")
        .write_paths("user.id");
    let mask_bytes = mask_builder.finish().unwrap();

    let mask_view = FieldMaskView::from_bytes(mask_bytes).unwrap();
    let paths: Vec<&str> = mask_view.paths().map(|p| p.unwrap()).collect();
    assert_eq!(
        paths,
        vec![
            "user.id",
            "user.profile.display_name",
            "user.Output.email"
        ]
    );

    // 5. SourceContext
    let mut sc_buf = [0u8; 64];
    let mut sc_builder = SourceContextBuilder::new(&mut sc_buf);
    sc_builder.write_file_name("google/protobuf/timestamp.proto");
    let sc_bytes = sc_builder.finish().unwrap();

    let sc_view = SourceContextView::from_bytes(sc_bytes).unwrap();
    assert!(sc_view.has_file_name());
    assert_eq!(sc_view.file_name(), Ok("google/protobuf/timestamp.proto"));
}

#[test]
fn recursive_struct_value_list_value_and_null_value_roundtrip() {
    // Construct a rich recursive google.protobuf.Struct containing all 6 Value::Kind variants:
    // {
    //   "null_key": null,
    //   "num_key": 3.14159,
    //   "str_key": "roto-wkt",
    //   "bool_key": true,
    //   "nested_struct": { "inner_flag": false },
    //   "nested_list": [ 42.0, "second", { "list_obj_key": "ok" }, [ true, null ] ]
    // }
    let mut buf = [0u8; 1024];
    let mut builder = StructBuilder::new(&mut buf);

    // Write entries in reverse order so wire order matches the list above
    builder
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_list_value(|list| {
                        // 4th element: nested list [ true, null ]
                        list.write_value(|elem4| {
                            elem4.write_list_value(|inner_list| {
                                inner_list
                                    .write_value(|iv2| {
                                        iv2.write_null_value(NullValue::NullValue);
                                    })
                                    .write_value(|iv1| {
                                        iv1.write_bool_value(true);
                                    });
                            });
                        })
                        // 3rd element: nested struct { "list_obj_key": "ok" }
                        .write_value(|elem3| {
                            elem3.write_struct_value(|inner_st| {
                                inner_st.write_field(|ie| {
                                    ie.write_value(|iv| {
                                        iv.write_string_value("ok");
                                    })
                                    .write_key("list_obj_key");
                                });
                            });
                        })
                        // 2nd element: "second"
                        .write_value(|elem2| {
                            elem2.write_string_value("second");
                        })
                        // 1st element: 42.0
                        .write_value(|elem1| {
                            elem1.write_number_value(42.0);
                        });
                    });
                })
                .write_key("nested_list");
        })
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_struct_value(|st| {
                        st.write_field(|ie| {
                            ie.write_value(|iv| {
                                iv.write_bool_value(false);
                            })
                            .write_key("inner_flag");
                        });
                    });
                })
                .write_key("nested_struct");
        })
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_bool_value(true);
                })
                .write_key("bool_key");
        })
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_string_value("roto-wkt");
                })
                .write_key("str_key");
        })
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_number_value(std::f64::consts::PI);
                })
                .write_key("num_key");
        })
        .write_field(|entry| {
            entry
                .write_value(|v| {
                    v.write_null_value(NullValue::NullValue);
                })
                .write_key("null_key");
        });

    let bytes = builder.finish().unwrap();
    let st_view = StructView::from_bytes(bytes).unwrap();

    let entries: Vec<FieldsEntryView<'_>> = st_view.fields().map(|e| e.unwrap()).collect();
    assert_eq!(entries.len(), 6);

    // 1. null_key
    assert_eq!(entries[0].key(), Ok("null_key"));
    let v0 = entries[0].value().unwrap().unwrap();
    assert!(v0.has_null_value());
    assert_eq!(v0.null_value(), Ok(NullValue::NullValue));
    assert!(matches!(
        v0.kind(),
        Ok(Some(ValueKind::NullValue(NullValue::NullValue)))
    ));

    // 2. num_key
    assert_eq!(entries[1].key(), Ok("num_key"));
    let v1 = entries[1].value().unwrap().unwrap();
    assert!(v1.has_number_value());
    assert_eq!(v1.number_value(), Ok(std::f64::consts::PI));
    assert!(matches!(
        v1.kind(),
        Ok(Some(ValueKind::NumberValue(n))) if (n - std::f64::consts::PI).abs() < 1e-12
    ));

    // 3. str_key
    assert_eq!(entries[2].key(), Ok("str_key"));
    let v2 = entries[2].value().unwrap().unwrap();
    assert!(v2.has_string_value());
    assert_eq!(v2.string_value(), Ok("roto-wkt"));
    assert!(matches!(
        v2.kind(),
        Ok(Some(ValueKind::StringValue("roto-wkt")))
    ));

    // 4. bool_key
    assert_eq!(entries[3].key(), Ok("bool_key"));
    let v3 = entries[3].value().unwrap().unwrap();
    assert!(v3.has_bool_value());
    assert_eq!(v3.bool_value(), Ok(true));
    assert!(matches!(v3.kind(), Ok(Some(ValueKind::BoolValue(true)))));

    // 5. nested_struct
    assert_eq!(entries[4].key(), Ok("nested_struct"));
    let v4 = entries[4].value().unwrap().unwrap();
    assert!(v4.has_struct_value());
    let Some(ValueKind::StructValue(nested_st)) = v4.kind().unwrap() else {
        panic!("expected StructValue");
    };
    let nested_entries: Vec<_> = nested_st.fields().map(|e| e.unwrap()).collect();
    assert_eq!(nested_entries.len(), 1);
    assert_eq!(nested_entries[0].key(), Ok("inner_flag"));
    let inner_flag_val = nested_entries[0].value().unwrap().unwrap();
    assert_eq!(inner_flag_val.bool_value(), Ok(false));

    // 6. nested_list
    assert_eq!(entries[5].key(), Ok("nested_list"));
    let v5 = entries[5].value().unwrap().unwrap();
    assert!(v5.has_list_value());
    let Some(ValueKind::ListValue(list_view)) = v5.kind().unwrap() else {
        panic!("expected ListValue");
    };
    let list_items: Vec<ValueView<'_>> = list_view.values().map(|x| x.unwrap()).collect();
    assert_eq!(list_items.len(), 4);
    assert_eq!(list_items[0].number_value(), Ok(42.0));
    assert_eq!(list_items[1].string_value(), Ok("second"));

    let item2_st = list_items[2].struct_value().unwrap().unwrap();
    let item2_entry = item2_st.fields().next().unwrap().unwrap();
    assert_eq!(item2_entry.key(), Ok("list_obj_key"));
    assert_eq!(
        item2_entry.value().unwrap().unwrap().string_value(),
        Ok("ok")
    );

    let item3_list: ListValueView<'_> = list_items[3].list_value().unwrap().unwrap();
    let item3_vals: Vec<ValueView<'_>> = item3_list.values().map(|x| x.unwrap()).collect();
    assert_eq!(item3_vals.len(), 2);
    assert_eq!(item3_vals[0].bool_value(), Ok(true));
    assert_eq!(item3_vals[1].null_value(), Ok(NullValue::NullValue));

    // Also test standalone FieldsEntryBuilder / ListValueBuilder / ValueBuilder::copy_unset_fields
    let mut entry_buf = [0u8; 64];
    let mut entry_builder = FieldsEntryBuilder::new(&mut entry_buf);
    entry_builder
        .write_value(|v| {
            v.write_string_value("standalone");
        })
        .write_key("k");
    let entry_view = FieldsEntryView::from_bytes(entry_builder.finish().unwrap()).unwrap();
    assert_eq!(entry_view.key(), Ok("k"));
    assert_eq!(
        entry_view.value().unwrap().unwrap().string_value(),
        Ok("standalone")
    );

    let mut list_buf = [0u8; 64];
    let mut list_builder = ListValueBuilder::new(&mut list_buf);
    list_builder.write_value(|v| {
        v.write_number_value(100.5);
    });
    let lv = ListValueView::from_bytes(list_builder.finish().unwrap()).unwrap();
    assert_eq!(lv.values().count(), 1);

    let mut val_copy_buf = [0u8; 64];
    let mut val_copy_builder = ValueBuilder::new(&mut val_copy_buf);
    val_copy_builder.copy_unset_fields(&v2);
    let val_copy_view = ValueView::from_bytes(val_copy_builder.finish().unwrap()).unwrap();
    assert_eq!(val_copy_view.string_value(), Ok("roto-wkt"));
}

#[test]
fn type_field_enum_enum_value_and_option_roundtrip() {
    let mut type_buf = [0u8; 512];
    let mut type_builder = TypeBuilder::new(&mut type_buf);
    type_builder
        .write_edition("2023")
        .write_syntax(Syntax::SyntaxProto3)
        .write_source_context(|sc| {
            sc.write_file_name("telemetry/sensor.proto");
        })
        .write_option(|opt| {
            opt.write_value(|any| {
                any.write_value(b"\x08\x01")
                    .write_type_url("type.googleapis.com/google.protobuf.BoolValue");
            })
            .write_name("deprecated");
        })
        .write_oneof("payload")
        .write_field(|f| {
            f.write_default_value("0")
                .write_json_name("timestampMs")
                .write_option(|opt| {
                    opt.write_name("packed");
                })
                .write_packed(true)
                .write_oneof_index(1)
                .write_type_url("")
                .write_name("timestamp_ms")
                .write_number(1)
                .write_cardinality(Cardinality::CardinalityOptional)
                .write_kind(FieldKind::TypeUint64);
        })
        .write_name("telemetry.SensorReading");

    let type_bytes = type_builder.finish().unwrap();
    let type_view = TypeView::from_bytes(type_bytes).unwrap();
    assert_eq!(type_view.name(), Ok("telemetry.SensorReading"));
    assert_eq!(type_view.syntax(), Ok(Syntax::SyntaxProto3));
    assert_eq!(type_view.edition(), Ok("2023"));
    assert_eq!(
        type_view
            .source_context()
            .unwrap()
            .unwrap()
            .file_name(),
        Ok("telemetry/sensor.proto")
    );

    let oneofs: Vec<&str> = type_view.oneofs().map(|o| o.unwrap()).collect();
    assert_eq!(oneofs, vec!["payload"]);

    let fields: Vec<FieldView<'_>> = type_view.fields().map(|f| f.unwrap()).collect();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name(), Ok("timestamp_ms"));
    assert_eq!(fields[0].number(), Ok(1));
    assert_eq!(fields[0].kind(), Ok(FieldKind::TypeUint64));
    assert_eq!(
        fields[0].cardinality(),
        Ok(Cardinality::CardinalityOptional)
    );
    assert_eq!(fields[0].oneof_index(), Ok(1));
    assert_eq!(fields[0].packed(), Ok(true));
    assert_eq!(fields[0].json_name(), Ok("timestampMs"));
    assert_eq!(fields[0].default_value(), Ok("0"));

    let opts: Vec<OptionView<'_>> = type_view.options().map(|o| o.unwrap()).collect();
    assert_eq!(opts.len(), 1);
    assert_eq!(opts[0].name(), Ok("deprecated"));
    let opt_any = opts[0].value().unwrap().unwrap();
    assert_eq!(
        opt_any.type_url(),
        Ok("type.googleapis.com/google.protobuf.BoolValue")
    );
    let opt_bool = BoolValueView::from_bytes(opt_any.value().unwrap()).unwrap();
    assert_eq!(opt_bool.value(), Ok(true));

    // Standalone FieldBuilder / EnumBuilder / EnumValueBuilder / OptionBuilder
    let mut f_buf = [0u8; 64];
    let mut f_builder = FieldBuilder::new(&mut f_buf);
    f_builder.write_number(2).write_name("device_id");
    let f_view = FieldView::from_bytes(f_builder.finish().unwrap()).unwrap();
    assert_eq!(f_view.name(), Ok("device_id"));
    assert_eq!(f_view.number(), Ok(2));

    let mut ev_buf = [0u8; 64];
    let mut ev_builder = EnumValueBuilder::new(&mut ev_buf);
    ev_builder.write_number(1).write_name("STATUS_ACTIVE");
    let ev_view = EnumValueView::from_bytes(ev_builder.finish().unwrap()).unwrap();
    assert_eq!(ev_view.name(), Ok("STATUS_ACTIVE"));
    assert_eq!(ev_view.number(), Ok(1));

    let mut opt_buf = [0u8; 64];
    let mut opt_builder = OptionBuilder::new(&mut opt_buf);
    opt_builder.write_name("allow_alias");
    let opt_view = OptionView::from_bytes(opt_builder.finish().unwrap()).unwrap();
    assert_eq!(opt_view.name(), Ok("allow_alias"));

    let mut enum_buf = [0u8; 256];
    let mut enum_builder = EnumBuilder::new(&mut enum_buf);
    enum_builder
        .write_edition("2023")
        .write_syntax(Syntax::SyntaxEditions)
        .write_source_context(|sc| {
            sc.write_file_name("telemetry/status.proto");
        })
        .write_option(|opt| {
            opt.write_name("allow_alias");
        })
        .write_enumvalue(|ev| {
            ev.write_option(|o| {
                o.write_name("deprecated");
            })
            .write_number(1)
            .write_name("DEVICE_ACTIVE");
        })
        .write_enumvalue(|ev| {
            ev.write_number(0).write_name("DEVICE_UNKNOWN");
        })
        .write_name("DeviceStatus");

    let enum_view = EnumView::from_bytes(enum_builder.finish().unwrap()).unwrap();
    assert_eq!(enum_view.name(), Ok("DeviceStatus"));
    assert_eq!(enum_view.syntax(), Ok(Syntax::SyntaxEditions));
    assert_eq!(enum_view.edition(), Ok("2023"));
    let evs: Vec<EnumValueView<'_>> = enum_view.enumvalue().map(|v| v.unwrap()).collect();
    assert_eq!(evs.len(), 2);
    assert_eq!(evs[0].name(), Ok("DEVICE_UNKNOWN"));
    assert_eq!(evs[0].number(), Ok(0));
    assert_eq!(evs[1].name(), Ok("DEVICE_ACTIVE"));
    assert_eq!(evs[1].number(), Ok(1));
}

#[test]
fn api_method_and_mixin_roundtrip() {
    let mut api_buf = [0u8; 512];
    let mut api_builder = ApiBuilder::new(&mut api_buf);
    api_builder
        .write_edition("2023")
        .write_syntax(Syntax::SyntaxProto3)
        .write_mixin(|mx| {
            mx.write_root("/v1/operations")
                .write_name("google.longrunning.Operations");
        })
        .write_source_context(|sc| {
            sc.write_file_name("telemetry/service.proto");
        })
        .write_version("v1")
        .write_option(|opt| {
            opt.write_name("java_multiple_files");
        })
        .write_method(|m| {
            m.write_edition("2023")
                .write_syntax(Syntax::SyntaxProto3)
                .write_option(|opt| {
                    opt.write_name("idempotency_level");
                })
                .write_response_streaming(true)
                .write_response_type_url("type.googleapis.com/telemetry.SensorAck")
                .write_request_streaming(false)
                .write_request_type_url("type.googleapis.com/telemetry.SensorReading")
                .write_name("StreamReadings");
        })
        .write_name("telemetry.TelemetryService");

    let api_view = ApiView::from_bytes(api_builder.finish().unwrap()).unwrap();
    assert_eq!(api_view.name(), Ok("telemetry.TelemetryService"));
    assert_eq!(api_view.version(), Ok("v1"));
    assert_eq!(api_view.syntax(), Ok(Syntax::SyntaxProto3));
    assert_eq!(api_view.edition(), Ok("2023"));
    assert_eq!(
        api_view.source_context().unwrap().unwrap().file_name(),
        Ok("telemetry/service.proto")
    );

    let methods: Vec<MethodView<'_>> = api_view.methods().map(|m| m.unwrap()).collect();
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0].name(), Ok("StreamReadings"));
    assert_eq!(
        methods[0].request_type_url(),
        Ok("type.googleapis.com/telemetry.SensorReading")
    );
    assert_eq!(methods[0].request_streaming(), Ok(false));
    assert_eq!(
        methods[0].response_type_url(),
        Ok("type.googleapis.com/telemetry.SensorAck")
    );
    assert_eq!(methods[0].response_streaming(), Ok(true));
    assert_eq!(methods[0].syntax(), Ok(Syntax::SyntaxProto3));
    assert_eq!(methods[0].edition(), Ok("2023"));

    let mixins: Vec<MixinView<'_>> = api_view.mixins().map(|m| m.unwrap()).collect();
    assert_eq!(mixins.len(), 1);
    assert_eq!(mixins[0].name(), Ok("google.longrunning.Operations"));
    assert_eq!(mixins[0].root(), Ok("/v1/operations"));

    // Standalone MethodBuilder and MixinBuilder
    let mut m_buf = [0u8; 64];
    let mut m_builder = MethodBuilder::new(&mut m_buf);
    m_builder.write_name("Ping");
    let m_view = MethodView::from_bytes(m_builder.finish().unwrap()).unwrap();
    assert_eq!(m_view.name(), Ok("Ping"));

    let mut mx_buf = [0u8; 64];
    let mut mx_builder = MixinBuilder::new(&mut mx_buf);
    mx_builder.write_root("/health").write_name("HealthCheck");
    let mx_view = MixinView::from_bytes(mx_builder.finish().unwrap()).unwrap();
    assert_eq!(mx_view.name(), Ok("HealthCheck"));
    assert_eq!(mx_view.root(), Ok("/health"));
}

#[test]
fn all_nine_wrapper_types_roundtrip() {
    // 1. DoubleValue
    let mut b1 = [0u8; 32];
    let mut w1 = DoubleValueBuilder::new(&mut b1);
    w1.write_value(-std::f64::consts::E);
    let v1 = DoubleValueView::from_bytes(w1.finish().unwrap()).unwrap();
    assert!(v1.has_value());
    assert_eq!(v1.value(), Ok(-std::f64::consts::E));

    // 2. FloatValue
    let mut b2 = [0u8; 32];
    let mut w2 = FloatValueBuilder::new(&mut b2);
    w2.write_value(1.5f32);
    let v2 = FloatValueView::from_bytes(w2.finish().unwrap()).unwrap();
    assert!(v2.has_value());
    assert_eq!(v2.value(), Ok(1.5f32));

    // 3. Int64Value
    let mut b3 = [0u8; 32];
    let mut w3 = Int64ValueBuilder::new(&mut b3);
    w3.write_value(-9_000_000_000_000_000_000i64);
    let v3 = Int64ValueView::from_bytes(w3.finish().unwrap()).unwrap();
    assert!(v3.has_value());
    assert_eq!(v3.value(), Ok(-9_000_000_000_000_000_000i64));

    // 4. UInt64Value
    let mut b4 = [0u8; 32];
    let mut w4 = UInt64ValueBuilder::new(&mut b4);
    w4.write_value(18_000_000_000_000_000_000u64);
    let v4 = UInt64ValueView::from_bytes(w4.finish().unwrap()).unwrap();
    assert!(v4.has_value());
    assert_eq!(v4.value(), Ok(18_000_000_000_000_000_000u64));

    // 5. Int32Value
    let mut b5 = [0u8; 32];
    let mut w5 = Int32ValueBuilder::new(&mut b5);
    w5.write_value(-123_456);
    let v5 = Int32ValueView::from_bytes(w5.finish().unwrap()).unwrap();
    assert!(v5.has_value());
    assert_eq!(v5.value(), Ok(-123_456));

    // 6. UInt32Value
    let mut b6 = [0u8; 32];
    let mut w6 = UInt32ValueBuilder::new(&mut b6);
    w6.write_value(4_000_000_000u32);
    let v6 = UInt32ValueView::from_bytes(w6.finish().unwrap()).unwrap();
    assert!(v6.has_value());
    assert_eq!(v6.value(), Ok(4_000_000_000u32));

    // 7. BoolValue
    let mut b7 = [0u8; 32];
    let mut w7 = BoolValueBuilder::new(&mut b7);
    w7.write_value(true);
    let v7 = BoolValueView::from_bytes(w7.finish().unwrap()).unwrap();
    assert!(v7.has_value());
    assert_eq!(v7.value(), Ok(true));

    // 8. StringValue
    let mut b8 = [0u8; 64];
    let mut w8 = StringValueBuilder::new(&mut b8);
    w8.write_value("zero-copy wrapper string");
    let v8 = StringValueView::from_bytes(w8.finish().unwrap()).unwrap();
    assert!(v8.has_value());
    assert_eq!(v8.value(), Ok("zero-copy wrapper string"));

    // 9. BytesValue
    let mut b9 = [0u8; 64];
    let mut w9 = BytesValueBuilder::new(&mut b9);
    w9.write_value(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF]);
    let v9 = BytesValueView::from_bytes(w9.finish().unwrap()).unwrap();
    assert!(v9.has_value());
    assert_eq!(
        v9.value(),
        Ok(&[0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF][..])
    );
}

#[test]
fn edition_feature_sets_roundtrip() {
    // CSharpFeatures
    let mut cs_buf = [0u8; 32];
    let mut cs_b = CSharpFeaturesBuilder::new(&mut cs_buf);
    cs_b.write_nullable_reference_types(true);
    let cs_v = CSharpFeaturesView::from_bytes(cs_b.finish().unwrap()).unwrap();
    assert_eq!(cs_v.nullable_reference_types(), Ok(true));

    // CppFeatures
    let mut cpp_buf = [0u8; 32];
    let mut cpp_b = CppFeaturesBuilder::new(&mut cpp_buf);
    cpp_b
        .write_repeated_type(RepeatedType::Proxy)
        .write_string_type(StringType::View)
        .write_legacy_closed_enum(true);
    let cpp_v = CppFeaturesView::from_bytes(cpp_b.finish().unwrap()).unwrap();
    assert_eq!(cpp_v.legacy_closed_enum(), Ok(true));
    assert_eq!(cpp_v.string_type(), Ok(StringType::View));
    assert_eq!(cpp_v.repeated_type(), Ok(RepeatedType::Proxy));

    // GoFeatures
    let mut go_buf = [0u8; 32];
    let mut go_b = GoFeaturesBuilder::new(&mut go_buf);
    go_b.write_optimize_mode(GoOptimizeMode::CodeSize)
        .write_strip_enum_prefix(StripEnumPrefix::StripEnumPrefixStrip)
        .write_api_level(APILevel::ApiOpaque)
        .write_legacy_unmarshal_json_enum(false);
    let go_v = GoFeaturesView::from_bytes(go_b.finish().unwrap()).unwrap();
    assert_eq!(go_v.legacy_unmarshal_json_enum(), Ok(false));
    assert_eq!(go_v.api_level(), Ok(APILevel::ApiOpaque));
    assert_eq!(
        go_v.strip_enum_prefix(),
        Ok(StripEnumPrefix::StripEnumPrefixStrip)
    );
    assert_eq!(go_v.optimize_mode(), Ok(GoOptimizeMode::CodeSize));

    // JavaFeatures
    let mut java_buf = [0u8; 32];
    let mut java_b = JavaFeaturesBuilder::new(&mut java_buf);
    java_b
        .write_nest_in_file_class(NestInFileClass::Yes)
        .write_use_old_outer_classname_default(false)
        .write_large_enum(true)
        .write_utf8_validation(JavaUtf8Validation::Verify)
        .write_legacy_closed_enum(false);
    let java_v = JavaFeaturesView::from_bytes(java_b.finish().unwrap()).unwrap();
    assert_eq!(java_v.legacy_closed_enum(), Ok(false));
    assert_eq!(java_v.utf8_validation(), Ok(JavaUtf8Validation::Verify));
    assert_eq!(java_v.large_enum(), Ok(true));
    assert_eq!(java_v.use_old_outer_classname_default(), Ok(false));
    assert_eq!(java_v.nest_in_file_class(), Ok(NestInFileClass::Yes));
}
