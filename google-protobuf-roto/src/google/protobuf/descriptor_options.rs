use roto::runtime::{
    decode_bool_at, decode_bytes_at, decode_f64_at, decode_str_at, decode_varint_at,
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker, MessageView,
    RepeatedMessageIter, RepeatedStrIter, RepeatedVarintIter, UnknownFieldsIter, VarintValue,
    WireType,
};

use super::Edition;

// ============================================================================
// Options & Metadata Enums and Nested Submodules
// ============================================================================

pub mod file_options {
    use roto::runtime::VarintValue;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum OptimizeMode {
        Speed = 1,
        CodeSize = 2,
        LiteRuntime = 3,
    }

    impl OptimizeMode {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                1 => Some(Self::Speed),
                2 => Some(Self::CodeSize),
                3 => Some(Self::LiteRuntime),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for OptimizeMode {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for OptimizeMode {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::Speed)
        }
    }
}

pub mod field_options {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum CType {
        String = 0,
        Cord = 1,
        StringPiece = 2,
    }

    impl CType {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::String),
                1 => Some(Self::Cord),
                2 => Some(Self::StringPiece),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for CType {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for CType {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::String)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum JSType {
        JsNormal = 0,
        JsString = 1,
        JsNumber = 2,
    }

    impl JSType {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::JsNormal),
                1 => Some(Self::JsString),
                2 => Some(Self::JsNumber),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for JSType {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for JSType {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::JsNormal)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum OptionRetention {
        RetentionUnknown = 0,
        RetentionRuntime = 1,
        RetentionSource = 2,
    }

    impl OptionRetention {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::RetentionUnknown),
                1 => Some(Self::RetentionRuntime),
                2 => Some(Self::RetentionSource),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for OptionRetention {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for OptionRetention {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::RetentionUnknown)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum OptionTargetType {
        TargetTypeUnknown = 0,
        TargetTypeFile = 1,
        TargetTypeExtensionRange = 2,
        TargetTypeMessage = 3,
        TargetTypeField = 4,
        TargetTypeOneof = 5,
        TargetTypeEnum = 6,
        TargetTypeEnumEntry = 7,
        TargetTypeService = 8,
        TargetTypeMethod = 9,
    }

    impl OptionTargetType {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::TargetTypeUnknown),
                1 => Some(Self::TargetTypeFile),
                2 => Some(Self::TargetTypeExtensionRange),
                3 => Some(Self::TargetTypeMessage),
                4 => Some(Self::TargetTypeField),
                5 => Some(Self::TargetTypeOneof),
                6 => Some(Self::TargetTypeEnum),
                7 => Some(Self::TargetTypeEnumEntry),
                8 => Some(Self::TargetTypeService),
                9 => Some(Self::TargetTypeMethod),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for OptionTargetType {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for OptionTargetType {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::TargetTypeUnknown)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct EditionDefaultView<'a> {
        buf: &'a [u8],
        value: FieldMarker,
        edition: FieldMarker,
    }

    impl<'a> EditionDefaultView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut value = FieldMarker::none();
            let mut edition = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (2, WireType::LengthDelimited) => value = field.marker,
                    (3, WireType::Varint) => edition = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                value,
                edition,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_edition(&self) -> bool {
            self.edition.is_present()
        }

        #[inline]
        pub fn edition(&self) -> Result<Edition, DecodeError> {
            if !self.edition.is_present() {
                return Ok(Edition::EditionUnknown);
            }
            let raw = decode_varint_at(self.buf, self.edition)?;
            Ok(Edition::from_raw_varint(raw))
        }

        #[inline]
        pub const fn has_value(&self) -> bool {
            self.value.is_present()
        }

        #[inline]
        pub fn value(&self) -> Result<&'a str, DecodeError> {
            if !self.value.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.value)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for EditionDefaultView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct EditionDefaultBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> EditionDefaultBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &EditionDefaultView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_value(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(2, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_edition(&mut self, val: Edition) -> Result<(), EncodeError> {
            self.writer.write_varint(val.as_i32() as u64)?;
            self.writer.write_tag(3, WireType::Varint)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct FeatureSupportView<'a> {
        buf: &'a [u8],
        edition_introduced: FieldMarker,
        edition_deprecated: FieldMarker,
        deprecation_warning: FieldMarker,
        edition_removed: FieldMarker,
    }

    impl<'a> FeatureSupportView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut edition_introduced = FieldMarker::none();
            let mut edition_deprecated = FieldMarker::none();
            let mut deprecation_warning = FieldMarker::none();
            let mut edition_removed = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::Varint) => edition_introduced = field.marker,
                    (2, WireType::Varint) => edition_deprecated = field.marker,
                    (3, WireType::LengthDelimited) => deprecation_warning = field.marker,
                    (4, WireType::Varint) => edition_removed = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                edition_introduced,
                edition_deprecated,
                deprecation_warning,
                edition_removed,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_edition_introduced(&self) -> bool {
            self.edition_introduced.is_present()
        }

        #[inline]
        pub fn edition_introduced(&self) -> Result<Edition, DecodeError> {
            if !self.edition_introduced.is_present() {
                return Ok(Edition::EditionUnknown);
            }
            let raw = decode_varint_at(self.buf, self.edition_introduced)?;
            Ok(Edition::from_raw_varint(raw))
        }

        #[inline]
        pub const fn has_edition_deprecated(&self) -> bool {
            self.edition_deprecated.is_present()
        }

        #[inline]
        pub fn edition_deprecated(&self) -> Result<Edition, DecodeError> {
            if !self.edition_deprecated.is_present() {
                return Ok(Edition::EditionUnknown);
            }
            let raw = decode_varint_at(self.buf, self.edition_deprecated)?;
            Ok(Edition::from_raw_varint(raw))
        }

        #[inline]
        pub const fn has_deprecation_warning(&self) -> bool {
            self.deprecation_warning.is_present()
        }

        #[inline]
        pub fn deprecation_warning(&self) -> Result<&'a str, DecodeError> {
            if !self.deprecation_warning.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.deprecation_warning)
        }

        #[inline]
        pub const fn has_edition_removed(&self) -> bool {
            self.edition_removed.is_present()
        }

        #[inline]
        pub fn edition_removed(&self) -> Result<Edition, DecodeError> {
            if !self.edition_removed.is_present() {
                return Ok(Edition::EditionUnknown);
            }
            let raw = decode_varint_at(self.buf, self.edition_removed)?;
            Ok(Edition::from_raw_varint(raw))
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for FeatureSupportView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct FeatureSupportBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> FeatureSupportBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &FeatureSupportView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_edition_introduced(&mut self, val: Edition) -> Result<(), EncodeError> {
            self.writer.write_varint(val.as_i32() as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_edition_deprecated(&mut self, val: Edition) -> Result<(), EncodeError> {
            self.writer.write_varint(val.as_i32() as u64)?;
            self.writer.write_tag(2, WireType::Varint)
        }

        #[inline]
        pub fn write_deprecation_warning(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(3, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_edition_removed(&mut self, val: Edition) -> Result<(), EncodeError> {
            self.writer.write_varint(val.as_i32() as u64)?;
            self.writer.write_tag(4, WireType::Varint)
        }
    }
}

pub mod method_options {
    use roto::runtime::VarintValue;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum IdempotencyLevel {
        IdempotencyUnknown = 0,
        NoSideEffects = 1,
        Idempotent = 2,
    }

    impl IdempotencyLevel {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::IdempotencyUnknown),
                1 => Some(Self::NoSideEffects),
                2 => Some(Self::Idempotent),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for IdempotencyLevel {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for IdempotencyLevel {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::IdempotencyUnknown)
        }
    }
}

pub mod extension_range_options {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum VerificationState {
        Declaration = 0,
        Unverified = 1,
    }

    impl VerificationState {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::Declaration),
                1 => Some(Self::Unverified),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for VerificationState {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for VerificationState {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::Unverified)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct DeclarationView<'a> {
        buf: &'a [u8],
        number: FieldMarker,
        full_name: FieldMarker,
        r#type: FieldMarker,
        reserved: FieldMarker,
        repeated: FieldMarker,
    }

    impl<'a> DeclarationView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 5, 6];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut number = FieldMarker::none();
            let mut full_name = FieldMarker::none();
            let mut r#type = FieldMarker::none();
            let mut reserved = FieldMarker::none();
            let mut repeated = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::Varint) => number = field.marker,
                    (2, WireType::LengthDelimited) => full_name = field.marker,
                    (3, WireType::LengthDelimited) => r#type = field.marker,
                    (5, WireType::Varint) => reserved = field.marker,
                    (6, WireType::Varint) => repeated = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                number,
                full_name,
                r#type,
                reserved,
                repeated,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_number(&self) -> bool {
            self.number.is_present()
        }

        #[inline]
        pub fn number(&self) -> Result<i32, DecodeError> {
            if !self.number.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.number)? as i32)
        }

        #[inline]
        pub const fn has_full_name(&self) -> bool {
            self.full_name.is_present()
        }

        #[inline]
        pub fn full_name(&self) -> Result<&'a str, DecodeError> {
            if !self.full_name.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.full_name)
        }

        #[inline]
        pub const fn has_type(&self) -> bool {
            self.r#type.is_present()
        }

        #[inline]
        pub fn r#type(&self) -> Result<&'a str, DecodeError> {
            if !self.r#type.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.r#type)
        }

        #[inline]
        pub const fn has_reserved(&self) -> bool {
            self.reserved.is_present()
        }

        #[inline]
        pub fn reserved(&self) -> Result<bool, DecodeError> {
            if !self.reserved.is_present() {
                return Ok(false);
            }
            decode_bool_at(self.buf, self.reserved)
        }

        #[inline]
        pub const fn has_repeated(&self) -> bool {
            self.repeated.is_present()
        }

        #[inline]
        pub fn repeated(&self) -> Result<bool, DecodeError> {
            if !self.repeated.is_present() {
                return Ok(false);
            }
            decode_bool_at(self.buf, self.repeated)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for DeclarationView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct DeclarationBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> DeclarationBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &DeclarationView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_number(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_full_name(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(2, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_type(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(3, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_reserved(&mut self, val: bool) -> Result<(), EncodeError> {
            self.writer.write_bool(val)?;
            self.writer.write_tag(5, WireType::Varint)
        }

        #[inline]
        pub fn write_repeated(&mut self, val: bool) -> Result<(), EncodeError> {
            self.writer.write_bool(val)?;
            self.writer.write_tag(6, WireType::Varint)
        }
    }
}

// ============================================================================
// FileOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FileOptionsView<'a> {
    buf: &'a [u8],
    java_package: FieldMarker,
    java_outer_classname: FieldMarker,
    optimize_for: FieldMarker,
    java_multiple_files: FieldMarker,
    go_package: FieldMarker,
    cc_generic_services: FieldMarker,
    java_generic_services: FieldMarker,
    py_generic_services: FieldMarker,
    java_generate_equals_and_hash: FieldMarker,
    deprecated: FieldMarker,
    java_string_check_utf8: FieldMarker,
    cc_enable_arenas: FieldMarker,
    objc_class_prefix: FieldMarker,
    csharp_namespace: FieldMarker,
    swift_prefix: FieldMarker,
    php_class_prefix: FieldMarker,
    php_namespace: FieldMarker,
    php_metadata_namespace: FieldMarker,
    ruby_package: FieldMarker,
    features: FieldMarker,
}

impl<'a> FileOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[
        1, 8, 9, 10, 11, 16, 17, 18, 20, 23, 27, 31, 36, 37, 39, 40, 41, 44, 45, 50, 999,
    ];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut java_package = FieldMarker::none();
        let mut java_outer_classname = FieldMarker::none();
        let mut optimize_for = FieldMarker::none();
        let mut java_multiple_files = FieldMarker::none();
        let mut go_package = FieldMarker::none();
        let mut cc_generic_services = FieldMarker::none();
        let mut java_generic_services = FieldMarker::none();
        let mut py_generic_services = FieldMarker::none();
        let mut java_generate_equals_and_hash = FieldMarker::none();
        let mut deprecated = FieldMarker::none();
        let mut java_string_check_utf8 = FieldMarker::none();
        let mut cc_enable_arenas = FieldMarker::none();
        let mut objc_class_prefix = FieldMarker::none();
        let mut csharp_namespace = FieldMarker::none();
        let mut swift_prefix = FieldMarker::none();
        let mut php_class_prefix = FieldMarker::none();
        let mut php_namespace = FieldMarker::none();
        let mut php_metadata_namespace = FieldMarker::none();
        let mut ruby_package = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => java_package = field.marker,
                (8, WireType::LengthDelimited) => java_outer_classname = field.marker,
                (9, WireType::Varint) => optimize_for = field.marker,
                (10, WireType::Varint) => java_multiple_files = field.marker,
                (11, WireType::LengthDelimited) => go_package = field.marker,
                (16, WireType::Varint) => cc_generic_services = field.marker,
                (17, WireType::Varint) => java_generic_services = field.marker,
                (18, WireType::Varint) => py_generic_services = field.marker,
                (20, WireType::Varint) => java_generate_equals_and_hash = field.marker,
                (23, WireType::Varint) => deprecated = field.marker,
                (27, WireType::Varint) => java_string_check_utf8 = field.marker,
                (31, WireType::Varint) => cc_enable_arenas = field.marker,
                (36, WireType::LengthDelimited) => objc_class_prefix = field.marker,
                (37, WireType::LengthDelimited) => csharp_namespace = field.marker,
                (39, WireType::LengthDelimited) => swift_prefix = field.marker,
                (40, WireType::LengthDelimited) => php_class_prefix = field.marker,
                (41, WireType::LengthDelimited) => php_namespace = field.marker,
                (44, WireType::LengthDelimited) => php_metadata_namespace = field.marker,
                (45, WireType::LengthDelimited) => ruby_package = field.marker,
                (50, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            java_package,
            java_outer_classname,
            optimize_for,
            java_multiple_files,
            go_package,
            cc_generic_services,
            java_generic_services,
            py_generic_services,
            java_generate_equals_and_hash,
            deprecated,
            java_string_check_utf8,
            cc_enable_arenas,
            objc_class_prefix,
            csharp_namespace,
            swift_prefix,
            php_class_prefix,
            php_namespace,
            php_metadata_namespace,
            ruby_package,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_java_package(&self) -> bool {
        self.java_package.is_present()
    }

    #[inline]
    pub fn java_package(&self) -> Result<&'a str, DecodeError> {
        if !self.java_package.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.java_package)
    }

    #[inline]
    pub const fn has_java_outer_classname(&self) -> bool {
        self.java_outer_classname.is_present()
    }

    #[inline]
    pub fn java_outer_classname(&self) -> Result<&'a str, DecodeError> {
        if !self.java_outer_classname.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.java_outer_classname)
    }

    #[inline]
    pub const fn has_optimize_for(&self) -> bool {
        self.optimize_for.is_present()
    }

    #[inline]
    pub fn optimize_for(&self) -> Result<file_options::OptimizeMode, DecodeError> {
        if !self.optimize_for.is_present() {
            return Ok(file_options::OptimizeMode::Speed);
        }
        let raw = decode_varint_at(self.buf, self.optimize_for)?;
        Ok(file_options::OptimizeMode::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_java_multiple_files(&self) -> bool {
        self.java_multiple_files.is_present()
    }

    #[inline]
    pub fn java_multiple_files(&self) -> Result<bool, DecodeError> {
        if !self.java_multiple_files.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.java_multiple_files)
    }

    #[inline]
    pub const fn has_go_package(&self) -> bool {
        self.go_package.is_present()
    }

    #[inline]
    pub fn go_package(&self) -> Result<&'a str, DecodeError> {
        if !self.go_package.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.go_package)
    }

    #[inline]
    pub const fn has_cc_generic_services(&self) -> bool {
        self.cc_generic_services.is_present()
    }

    #[inline]
    pub fn cc_generic_services(&self) -> Result<bool, DecodeError> {
        if !self.cc_generic_services.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.cc_generic_services)
    }

    #[inline]
    pub const fn has_java_generic_services(&self) -> bool {
        self.java_generic_services.is_present()
    }

    #[inline]
    pub fn java_generic_services(&self) -> Result<bool, DecodeError> {
        if !self.java_generic_services.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.java_generic_services)
    }

    #[inline]
    pub const fn has_py_generic_services(&self) -> bool {
        self.py_generic_services.is_present()
    }

    #[inline]
    pub fn py_generic_services(&self) -> Result<bool, DecodeError> {
        if !self.py_generic_services.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.py_generic_services)
    }

    #[inline]
    pub const fn has_java_generate_equals_and_hash(&self) -> bool {
        self.java_generate_equals_and_hash.is_present()
    }

    #[inline]
    pub fn java_generate_equals_and_hash(&self) -> Result<bool, DecodeError> {
        if !self.java_generate_equals_and_hash.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.java_generate_equals_and_hash)
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_java_string_check_utf8(&self) -> bool {
        self.java_string_check_utf8.is_present()
    }

    #[inline]
    pub fn java_string_check_utf8(&self) -> Result<bool, DecodeError> {
        if !self.java_string_check_utf8.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.java_string_check_utf8)
    }

    #[inline]
    pub const fn has_cc_enable_arenas(&self) -> bool {
        self.cc_enable_arenas.is_present()
    }

    #[inline]
    pub fn cc_enable_arenas(&self) -> Result<bool, DecodeError> {
        if !self.cc_enable_arenas.is_present() {
            return Ok(true);
        }
        decode_bool_at(self.buf, self.cc_enable_arenas)
    }

    #[inline]
    pub const fn has_objc_class_prefix(&self) -> bool {
        self.objc_class_prefix.is_present()
    }

    #[inline]
    pub fn objc_class_prefix(&self) -> Result<&'a str, DecodeError> {
        if !self.objc_class_prefix.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.objc_class_prefix)
    }

    #[inline]
    pub const fn has_csharp_namespace(&self) -> bool {
        self.csharp_namespace.is_present()
    }

    #[inline]
    pub fn csharp_namespace(&self) -> Result<&'a str, DecodeError> {
        if !self.csharp_namespace.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.csharp_namespace)
    }

    #[inline]
    pub const fn has_swift_prefix(&self) -> bool {
        self.swift_prefix.is_present()
    }

    #[inline]
    pub fn swift_prefix(&self) -> Result<&'a str, DecodeError> {
        if !self.swift_prefix.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.swift_prefix)
    }

    #[inline]
    pub const fn has_php_class_prefix(&self) -> bool {
        self.php_class_prefix.is_present()
    }

    #[inline]
    pub fn php_class_prefix(&self) -> Result<&'a str, DecodeError> {
        if !self.php_class_prefix.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.php_class_prefix)
    }

    #[inline]
    pub const fn has_php_namespace(&self) -> bool {
        self.php_namespace.is_present()
    }

    #[inline]
    pub fn php_namespace(&self) -> Result<&'a str, DecodeError> {
        if !self.php_namespace.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.php_namespace)
    }

    #[inline]
    pub const fn has_php_metadata_namespace(&self) -> bool {
        self.php_metadata_namespace.is_present()
    }

    #[inline]
    pub fn php_metadata_namespace(&self) -> Result<&'a str, DecodeError> {
        if !self.php_metadata_namespace.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.php_metadata_namespace)
    }

    #[inline]
    pub const fn has_ruby_package(&self) -> bool {
        self.ruby_package.is_present()
    }

    #[inline]
    pub fn ruby_package(&self) -> Result<&'a str, DecodeError> {
        if !self.ruby_package.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.ruby_package)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FileOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FileOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FileOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &FileOptionsView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_java_package(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_java_outer_classname(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(8, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_optimize_for(
        &mut self,
        val: file_options::OptimizeMode,
    ) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(9, WireType::Varint)
    }

    #[inline]
    pub fn write_java_multiple_files(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(10, WireType::Varint)
    }

    #[inline]
    pub fn write_go_package(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(11, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_cc_generic_services(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(16, WireType::Varint)
    }

    #[inline]
    pub fn write_java_generic_services(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(17, WireType::Varint)
    }

    #[inline]
    pub fn write_py_generic_services(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(18, WireType::Varint)
    }

    #[inline]
    pub fn write_java_generate_equals_and_hash(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(20, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(23, WireType::Varint)
    }

    #[inline]
    pub fn write_java_string_check_utf8(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(27, WireType::Varint)
    }

    #[inline]
    pub fn write_cc_enable_arenas(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(31, WireType::Varint)
    }

    #[inline]
    pub fn write_objc_class_prefix(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(36, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_csharp_namespace(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(37, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_swift_prefix(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(39, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_php_class_prefix(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(40, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_php_namespace(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(41, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_php_metadata_namespace(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(44, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_ruby_package(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(45, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(50, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// MessageOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct MessageOptionsView<'a> {
    buf: &'a [u8],
    message_set_wire_format: FieldMarker,
    no_standard_descriptor_accessor: FieldMarker,
    deprecated: FieldMarker,
    map_entry: FieldMarker,
    deprecated_legacy_json_field_conflicts: FieldMarker,
    features: FieldMarker,
}

impl<'a> MessageOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 7, 11, 12, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut message_set_wire_format = FieldMarker::none();
        let mut no_standard_descriptor_accessor = FieldMarker::none();
        let mut deprecated = FieldMarker::none();
        let mut map_entry = FieldMarker::none();
        let mut deprecated_legacy_json_field_conflicts = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => message_set_wire_format = field.marker,
                (2, WireType::Varint) => no_standard_descriptor_accessor = field.marker,
                (3, WireType::Varint) => deprecated = field.marker,
                (7, WireType::Varint) => map_entry = field.marker,
                (11, WireType::Varint) => deprecated_legacy_json_field_conflicts = field.marker,
                (12, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            message_set_wire_format,
            no_standard_descriptor_accessor,
            deprecated,
            map_entry,
            deprecated_legacy_json_field_conflicts,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_message_set_wire_format(&self) -> bool {
        self.message_set_wire_format.is_present()
    }

    #[inline]
    pub fn message_set_wire_format(&self) -> Result<bool, DecodeError> {
        if !self.message_set_wire_format.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.message_set_wire_format)
    }

    #[inline]
    pub const fn has_no_standard_descriptor_accessor(&self) -> bool {
        self.no_standard_descriptor_accessor.is_present()
    }

    #[inline]
    pub fn no_standard_descriptor_accessor(&self) -> Result<bool, DecodeError> {
        if !self.no_standard_descriptor_accessor.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.no_standard_descriptor_accessor)
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_map_entry(&self) -> bool {
        self.map_entry.is_present()
    }

    #[inline]
    pub fn map_entry(&self) -> Result<bool, DecodeError> {
        if !self.map_entry.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.map_entry)
    }

    #[inline]
    pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
        self.deprecated_legacy_json_field_conflicts.is_present()
    }

    #[inline]
    pub fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, DecodeError> {
        if !self.deprecated_legacy_json_field_conflicts.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated_legacy_json_field_conflicts)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for MessageOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct MessageOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> MessageOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &MessageOptionsView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_message_set_wire_format(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(1, WireType::Varint)
    }

    #[inline]
    pub fn write_no_standard_descriptor_accessor(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_map_entry(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(7, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated_legacy_json_field_conflicts(
        &mut self,
        val: bool,
    ) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(11, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(12, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// FieldOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FieldOptionsView<'a> {
    buf: &'a [u8],
    ctype: FieldMarker,
    packed: FieldMarker,
    deprecated: FieldMarker,
    lazy: FieldMarker,
    jstype: FieldMarker,
    weak: FieldMarker,
    unverified_lazy: FieldMarker,
    debug_redact: FieldMarker,
    retention: FieldMarker,
    features: FieldMarker,
    feature_support: FieldMarker,
}

impl<'a> FieldOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 5, 6, 10, 15, 16, 17, 19, 20, 21, 22, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut ctype = FieldMarker::none();
        let mut packed = FieldMarker::none();
        let mut deprecated = FieldMarker::none();
        let mut lazy = FieldMarker::none();
        let mut jstype = FieldMarker::none();
        let mut weak = FieldMarker::none();
        let mut unverified_lazy = FieldMarker::none();
        let mut debug_redact = FieldMarker::none();
        let mut retention = FieldMarker::none();
        let mut features = FieldMarker::none();
        let mut feature_support = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => ctype = field.marker,
                (2, WireType::Varint) => packed = field.marker,
                (3, WireType::Varint) => deprecated = field.marker,
                (5, WireType::Varint) => lazy = field.marker,
                (6, WireType::Varint) => jstype = field.marker,
                (10, WireType::Varint) => weak = field.marker,
                (15, WireType::Varint) => unverified_lazy = field.marker,
                (16, WireType::Varint) => debug_redact = field.marker,
                (17, WireType::Varint) => retention = field.marker,
                (21, WireType::LengthDelimited) => features = field.marker,
                (22, WireType::LengthDelimited) => feature_support = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            ctype,
            packed,
            deprecated,
            lazy,
            jstype,
            weak,
            unverified_lazy,
            debug_redact,
            retention,
            features,
            feature_support,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_ctype(&self) -> bool {
        self.ctype.is_present()
    }

    #[inline]
    pub fn ctype(&self) -> Result<field_options::CType, DecodeError> {
        if !self.ctype.is_present() {
            return Ok(field_options::CType::String);
        }
        let raw = decode_varint_at(self.buf, self.ctype)?;
        Ok(field_options::CType::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_packed(&self) -> bool {
        self.packed.is_present()
    }

    #[inline]
    pub fn packed(&self) -> Result<bool, DecodeError> {
        if !self.packed.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.packed)
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_lazy(&self) -> bool {
        self.lazy.is_present()
    }

    #[inline]
    pub fn lazy(&self) -> Result<bool, DecodeError> {
        if !self.lazy.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.lazy)
    }

    #[inline]
    pub const fn has_jstype(&self) -> bool {
        self.jstype.is_present()
    }

    #[inline]
    pub fn jstype(&self) -> Result<field_options::JSType, DecodeError> {
        if !self.jstype.is_present() {
            return Ok(field_options::JSType::JsNormal);
        }
        let raw = decode_varint_at(self.buf, self.jstype)?;
        Ok(field_options::JSType::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_weak(&self) -> bool {
        self.weak.is_present()
    }

    #[inline]
    pub fn weak(&self) -> Result<bool, DecodeError> {
        if !self.weak.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.weak)
    }

    #[inline]
    pub const fn has_unverified_lazy(&self) -> bool {
        self.unverified_lazy.is_present()
    }

    #[inline]
    pub fn unverified_lazy(&self) -> Result<bool, DecodeError> {
        if !self.unverified_lazy.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.unverified_lazy)
    }

    #[inline]
    pub const fn has_debug_redact(&self) -> bool {
        self.debug_redact.is_present()
    }

    #[inline]
    pub fn debug_redact(&self) -> Result<bool, DecodeError> {
        if !self.debug_redact.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.debug_redact)
    }

    #[inline]
    pub const fn has_retention(&self) -> bool {
        self.retention.is_present()
    }

    #[inline]
    pub fn retention(&self) -> Result<field_options::OptionRetention, DecodeError> {
        if !self.retention.is_present() {
            return Ok(field_options::OptionRetention::RetentionUnknown);
        }
        let raw = decode_varint_at(self.buf, self.retention)?;
        Ok(field_options::OptionRetention::from_raw_varint(raw))
    }

    #[inline]
    pub const fn targets(&self) -> RepeatedVarintIter<'a, field_options::OptionTargetType> {
        RepeatedVarintIter::new(self.buf, 19)
    }

    #[inline]
    pub const fn edition_defaults(
        &self,
    ) -> RepeatedMessageIter<'a, field_options::EditionDefaultView<'a>> {
        RepeatedMessageIter::new(self.buf, 20)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_feature_support(&self) -> bool {
        self.feature_support.is_present()
    }

    #[inline]
    pub fn feature_support(
        &self,
    ) -> Result<Option<field_options::FeatureSupportView<'a>>, DecodeError> {
        if !self.feature_support.is_present() {
            return Ok(None);
        }
        field_options::FeatureSupportView::from_bytes(self.feature_support.slice(self.buf))
            .map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FieldOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FieldOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FieldOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &FieldOptionsView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_ctype(&mut self, val: field_options::CType) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(1, WireType::Varint)
    }

    #[inline]
    pub fn write_packed(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_lazy(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(5, WireType::Varint)
    }

    #[inline]
    pub fn write_jstype(&mut self, val: field_options::JSType) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(6, WireType::Varint)
    }

    #[inline]
    pub fn write_weak(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(10, WireType::Varint)
    }

    #[inline]
    pub fn write_unverified_lazy(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(15, WireType::Varint)
    }

    #[inline]
    pub fn write_debug_redact(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(16, WireType::Varint)
    }

    #[inline]
    pub fn write_retention(
        &mut self,
        val: field_options::OptionRetention,
    ) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(17, WireType::Varint)
    }

    #[inline]
    pub fn write_targets(
        &mut self,
        val: field_options::OptionTargetType,
    ) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(19, WireType::Varint)
    }

    #[inline]
    pub fn write_packed_targets(
        &mut self,
        vals: &[field_options::OptionTargetType],
    ) -> Result<(), EncodeError> {
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            self.writer.write_varint(val.as_i32() as u64)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(19, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_edition_defaults<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut field_options::EditionDefaultBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = field_options::EditionDefaultBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(20, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(21, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_feature_support<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut field_options::FeatureSupportBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = field_options::FeatureSupportBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(22, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// OneofOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct OneofOptionsView<'a> {
    buf: &'a [u8],
    features: FieldMarker,
}

impl<'a> OneofOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            if let (1, WireType::LengthDelimited) = (field.tag, field.wire_type) {
                features = field.marker;
            }
        }

        Ok(Self { buf, features })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for OneofOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct OneofOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> OneofOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &OneofOptionsView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// EnumOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct EnumOptionsView<'a> {
    buf: &'a [u8],
    allow_alias: FieldMarker,
    deprecated: FieldMarker,
    deprecated_legacy_json_field_conflicts: FieldMarker,
    features: FieldMarker,
}

impl<'a> EnumOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 6, 7, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut allow_alias = FieldMarker::none();
        let mut deprecated = FieldMarker::none();
        let mut deprecated_legacy_json_field_conflicts = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (2, WireType::Varint) => allow_alias = field.marker,
                (3, WireType::Varint) => deprecated = field.marker,
                (6, WireType::Varint) => deprecated_legacy_json_field_conflicts = field.marker,
                (7, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            allow_alias,
            deprecated,
            deprecated_legacy_json_field_conflicts,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_allow_alias(&self) -> bool {
        self.allow_alias.is_present()
    }

    #[inline]
    pub fn allow_alias(&self) -> Result<bool, DecodeError> {
        if !self.allow_alias.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.allow_alias)
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
        self.deprecated_legacy_json_field_conflicts.is_present()
    }

    #[inline]
    pub fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, DecodeError> {
        if !self.deprecated_legacy_json_field_conflicts.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated_legacy_json_field_conflicts)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for EnumOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct EnumOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> EnumOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &EnumOptionsView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_allow_alias(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_deprecated_legacy_json_field_conflicts(
        &mut self,
        val: bool,
    ) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(6, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(7, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// EnumValueOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct EnumValueOptionsView<'a> {
    buf: &'a [u8],
    deprecated: FieldMarker,
    features: FieldMarker,
    debug_redact: FieldMarker,
    feature_support: FieldMarker,
}

impl<'a> EnumValueOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut deprecated = FieldMarker::none();
        let mut features = FieldMarker::none();
        let mut debug_redact = FieldMarker::none();
        let mut feature_support = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => deprecated = field.marker,
                (2, WireType::LengthDelimited) => features = field.marker,
                (3, WireType::Varint) => debug_redact = field.marker,
                (4, WireType::LengthDelimited) => feature_support = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            deprecated,
            features,
            debug_redact,
            feature_support,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_debug_redact(&self) -> bool {
        self.debug_redact.is_present()
    }

    #[inline]
    pub fn debug_redact(&self) -> Result<bool, DecodeError> {
        if !self.debug_redact.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.debug_redact)
    }

    #[inline]
    pub const fn has_feature_support(&self) -> bool {
        self.feature_support.is_present()
    }

    #[inline]
    pub fn feature_support(
        &self,
    ) -> Result<Option<field_options::FeatureSupportView<'a>>, DecodeError> {
        if !self.feature_support.is_present() {
            return Ok(None);
        }
        field_options::FeatureSupportView::from_bytes(self.feature_support.slice(self.buf))
            .map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for EnumValueOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct EnumValueOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> EnumValueOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &EnumValueOptionsView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(1, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_debug_redact(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_feature_support<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut field_options::FeatureSupportBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = field_options::FeatureSupportBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// ServiceOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct ServiceOptionsView<'a> {
    buf: &'a [u8],
    deprecated: FieldMarker,
    features: FieldMarker,
}

impl<'a> ServiceOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[33, 34, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut deprecated = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (33, WireType::Varint) => deprecated = field.marker,
                (34, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            deprecated,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for ServiceOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ServiceOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> ServiceOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &ServiceOptionsView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(33, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(34, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// MethodOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct MethodOptionsView<'a> {
    buf: &'a [u8],
    deprecated: FieldMarker,
    idempotency_level: FieldMarker,
    features: FieldMarker,
}

impl<'a> MethodOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[33, 34, 35, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut deprecated = FieldMarker::none();
        let mut idempotency_level = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (33, WireType::Varint) => deprecated = field.marker,
                (34, WireType::Varint) => idempotency_level = field.marker,
                (35, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            deprecated,
            idempotency_level,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_deprecated(&self) -> bool {
        self.deprecated.is_present()
    }

    #[inline]
    pub fn deprecated(&self) -> Result<bool, DecodeError> {
        if !self.deprecated.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.deprecated)
    }

    #[inline]
    pub const fn has_idempotency_level(&self) -> bool {
        self.idempotency_level.is_present()
    }

    #[inline]
    pub fn idempotency_level(&self) -> Result<method_options::IdempotencyLevel, DecodeError> {
        if !self.idempotency_level.is_present() {
            return Ok(method_options::IdempotencyLevel::IdempotencyUnknown);
        }
        let raw = decode_varint_at(self.buf, self.idempotency_level)?;
        Ok(method_options::IdempotencyLevel::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for MethodOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct MethodOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> MethodOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &MethodOptionsView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_deprecated(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(33, WireType::Varint)
    }

    #[inline]
    pub fn write_idempotency_level(
        &mut self,
        val: method_options::IdempotencyLevel,
    ) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(34, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(35, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// ExtensionRangeOptions (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct ExtensionRangeOptionsView<'a> {
    buf: &'a [u8],
    verification: FieldMarker,
    features: FieldMarker,
}

impl<'a> ExtensionRangeOptionsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 50, 999];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut verification = FieldMarker::none();
        let mut features = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (3, WireType::Varint) => verification = field.marker,
                (50, WireType::LengthDelimited) => features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            verification,
            features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn declaration(
        &self,
    ) -> RepeatedMessageIter<'a, extension_range_options::DeclarationView<'a>> {
        RepeatedMessageIter::new(self.buf, 2)
    }

    #[inline]
    pub const fn has_verification(&self) -> bool {
        self.verification.is_present()
    }

    #[inline]
    pub fn verification(&self) -> Result<extension_range_options::VerificationState, DecodeError> {
        if !self.verification.is_present() {
            return Ok(extension_range_options::VerificationState::Unverified);
        }
        let raw = decode_varint_at(self.buf, self.verification)?;
        Ok(extension_range_options::VerificationState::from_raw_varint(
            raw,
        ))
    }

    #[inline]
    pub const fn has_features(&self) -> bool {
        self.features.is_present()
    }

    #[inline]
    pub fn features(&self) -> Result<Option<FeatureSetView<'a>>, DecodeError> {
        if !self.features.is_present() {
            return Ok(None);
        }
        FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn uninterpreted_option(
        &self,
    ) -> RepeatedMessageIter<'a, UninterpretedOptionView<'a>> {
        RepeatedMessageIter::new(self.buf, 999)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for ExtensionRangeOptionsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ExtensionRangeOptionsBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> ExtensionRangeOptionsBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &ExtensionRangeOptionsView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_declaration<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(
            &mut extension_range_options::DeclarationBuilder<'_, 'b>,
        ) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = extension_range_options::DeclarationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_verification(
        &mut self,
        val: extension_range_options::VerificationState,
    ) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_features<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FeatureSetBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FeatureSetBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(50, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_uninterpreted_option<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut UninterpretedOptionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = UninterpretedOptionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(999, WireType::LengthDelimited)
    }
}

// ============================================================================
// FeatureSet (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FeatureSetView<'a> {
    buf: &'a [u8],
    field_presence: FieldMarker,
    enum_type: FieldMarker,
    repeated_field_encoding: FieldMarker,
    utf8_validation: FieldMarker,
    message_encoding: FieldMarker,
    json_format: FieldMarker,
    enforce_naming_style: FieldMarker,
    default_symbol_visibility: FieldMarker,
}

impl<'a> FeatureSetView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut field_presence = FieldMarker::none();
        let mut enum_type = FieldMarker::none();
        let mut repeated_field_encoding = FieldMarker::none();
        let mut utf8_validation = FieldMarker::none();
        let mut message_encoding = FieldMarker::none();
        let mut json_format = FieldMarker::none();
        let mut enforce_naming_style = FieldMarker::none();
        let mut default_symbol_visibility = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => field_presence = field.marker,
                (2, WireType::Varint) => enum_type = field.marker,
                (3, WireType::Varint) => repeated_field_encoding = field.marker,
                (4, WireType::Varint) => utf8_validation = field.marker,
                (5, WireType::Varint) => message_encoding = field.marker,
                (6, WireType::Varint) => json_format = field.marker,
                (7, WireType::Varint) => enforce_naming_style = field.marker,
                (8, WireType::Varint) => default_symbol_visibility = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            field_presence,
            enum_type,
            repeated_field_encoding,
            utf8_validation,
            message_encoding,
            json_format,
            enforce_naming_style,
            default_symbol_visibility,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_field_presence(&self) -> bool {
        self.field_presence.is_present()
    }

    #[inline]
    pub fn field_presence(&self) -> Result<i32, DecodeError> {
        if !self.field_presence.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.field_presence)? as i32)
    }

    #[inline]
    pub const fn has_enum_type(&self) -> bool {
        self.enum_type.is_present()
    }

    #[inline]
    pub fn enum_type(&self) -> Result<i32, DecodeError> {
        if !self.enum_type.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.enum_type)? as i32)
    }

    #[inline]
    pub const fn has_repeated_field_encoding(&self) -> bool {
        self.repeated_field_encoding.is_present()
    }

    #[inline]
    pub fn repeated_field_encoding(&self) -> Result<i32, DecodeError> {
        if !self.repeated_field_encoding.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.repeated_field_encoding)? as i32)
    }

    #[inline]
    pub const fn has_utf8_validation(&self) -> bool {
        self.utf8_validation.is_present()
    }

    #[inline]
    pub fn utf8_validation(&self) -> Result<i32, DecodeError> {
        if !self.utf8_validation.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.utf8_validation)? as i32)
    }

    #[inline]
    pub const fn has_message_encoding(&self) -> bool {
        self.message_encoding.is_present()
    }

    #[inline]
    pub fn message_encoding(&self) -> Result<i32, DecodeError> {
        if !self.message_encoding.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.message_encoding)? as i32)
    }

    #[inline]
    pub const fn has_json_format(&self) -> bool {
        self.json_format.is_present()
    }

    #[inline]
    pub fn json_format(&self) -> Result<i32, DecodeError> {
        if !self.json_format.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.json_format)? as i32)
    }

    #[inline]
    pub const fn has_enforce_naming_style(&self) -> bool {
        self.enforce_naming_style.is_present()
    }

    #[inline]
    pub fn enforce_naming_style(&self) -> Result<i32, DecodeError> {
        if !self.enforce_naming_style.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.enforce_naming_style)? as i32)
    }

    #[inline]
    pub const fn has_default_symbol_visibility(&self) -> bool {
        self.default_symbol_visibility.is_present()
    }

    #[inline]
    pub fn default_symbol_visibility(&self) -> Result<i32, DecodeError> {
        if !self.default_symbol_visibility.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.default_symbol_visibility)? as i32)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FeatureSetView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FeatureSetBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FeatureSetBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &FeatureSetView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_field_presence(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(1, WireType::Varint)
    }

    #[inline]
    pub fn write_enum_type(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_repeated_field_encoding(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_utf8_validation(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(4, WireType::Varint)
    }

    #[inline]
    pub fn write_message_encoding(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(5, WireType::Varint)
    }

    #[inline]
    pub fn write_json_format(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(6, WireType::Varint)
    }

    #[inline]
    pub fn write_enforce_naming_style(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(7, WireType::Varint)
    }

    #[inline]
    pub fn write_default_symbol_visibility(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(8, WireType::Varint)
    }
}

// ============================================================================
// UninterpretedOption (Reader View & Backward Builder)
// ============================================================================

pub mod uninterpreted_option {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct NamePartView<'a> {
        buf: &'a [u8],
        name_part: FieldMarker,
        is_extension: FieldMarker,
    }

    impl<'a> NamePartView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut name_part = FieldMarker::none();
            let mut is_extension = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::LengthDelimited) => name_part = field.marker,
                    (2, WireType::Varint) => is_extension = field.marker,
                    _ => {}
                }
            }

            if !name_part.is_present() {
                return Err(DecodeError::MissingRequiredField("NamePart.name_part"));
            }
            if !is_extension.is_present() {
                return Err(DecodeError::MissingRequiredField("NamePart.is_extension"));
            }

            Ok(Self {
                buf,
                name_part,
                is_extension,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_name_part(&self) -> bool {
            self.name_part.is_present()
        }

        #[inline]
        pub fn name_part(&self) -> Result<&'a str, DecodeError> {
            decode_str_at(self.buf, self.name_part)
        }

        #[inline]
        pub const fn has_is_extension(&self) -> bool {
            self.is_extension.is_present()
        }

        #[inline]
        pub fn is_extension(&self) -> Result<bool, DecodeError> {
            decode_bool_at(self.buf, self.is_extension)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for NamePartView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct NamePartBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> NamePartBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(&mut self, view: &NamePartView<'_>) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_name_part(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(1, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_is_extension(&mut self, val: bool) -> Result<(), EncodeError> {
            self.writer.write_bool(val)?;
            self.writer.write_tag(2, WireType::Varint)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UninterpretedOptionView<'a> {
    buf: &'a [u8],
    identifier_value: FieldMarker,
    positive_int_value: FieldMarker,
    negative_int_value: FieldMarker,
    double_value: FieldMarker,
    string_value: FieldMarker,
    aggregate_value: FieldMarker,
}

impl<'a> UninterpretedOptionView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 4, 5, 6, 7, 8];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut identifier_value = FieldMarker::none();
        let mut positive_int_value = FieldMarker::none();
        let mut negative_int_value = FieldMarker::none();
        let mut double_value = FieldMarker::none();
        let mut string_value = FieldMarker::none();
        let mut aggregate_value = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (3, WireType::LengthDelimited) => identifier_value = field.marker,
                (4, WireType::Varint) => positive_int_value = field.marker,
                (5, WireType::Varint) => negative_int_value = field.marker,
                (6, WireType::Fixed64) => double_value = field.marker,
                (7, WireType::LengthDelimited) => string_value = field.marker,
                (8, WireType::LengthDelimited) => aggregate_value = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            identifier_value,
            positive_int_value,
            negative_int_value,
            double_value,
            string_value,
            aggregate_value,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn name(&self) -> RepeatedMessageIter<'a, uninterpreted_option::NamePartView<'a>> {
        RepeatedMessageIter::new(self.buf, 2)
    }

    #[inline]
    pub const fn has_identifier_value(&self) -> bool {
        self.identifier_value.is_present()
    }

    #[inline]
    pub fn identifier_value(&self) -> Result<&'a str, DecodeError> {
        if !self.identifier_value.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.identifier_value)
    }

    #[inline]
    pub const fn has_positive_int_value(&self) -> bool {
        self.positive_int_value.is_present()
    }

    #[inline]
    pub fn positive_int_value(&self) -> Result<u64, DecodeError> {
        if !self.positive_int_value.is_present() {
            return Ok(0);
        }
        decode_varint_at(self.buf, self.positive_int_value)
    }

    #[inline]
    pub const fn has_negative_int_value(&self) -> bool {
        self.negative_int_value.is_present()
    }

    #[inline]
    pub fn negative_int_value(&self) -> Result<i64, DecodeError> {
        if !self.negative_int_value.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.negative_int_value)? as i64)
    }

    #[inline]
    pub const fn has_double_value(&self) -> bool {
        self.double_value.is_present()
    }

    #[inline]
    pub fn double_value(&self) -> Result<f64, DecodeError> {
        if !self.double_value.is_present() {
            return Ok(0.0);
        }
        decode_f64_at(self.buf, self.double_value)
    }

    #[inline]
    pub const fn has_string_value(&self) -> bool {
        self.string_value.is_present()
    }

    #[inline]
    pub fn string_value(&self) -> Result<&'a [u8], DecodeError> {
        if !self.string_value.is_present() {
            return Ok(&[]);
        }
        decode_bytes_at(self.buf, self.string_value)
    }

    #[inline]
    pub const fn has_aggregate_value(&self) -> bool {
        self.aggregate_value.is_present()
    }

    #[inline]
    pub fn aggregate_value(&self) -> Result<&'a str, DecodeError> {
        if !self.aggregate_value.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.aggregate_value)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for UninterpretedOptionView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct UninterpretedOptionBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> UninterpretedOptionBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &UninterpretedOptionView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_name<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut uninterpreted_option::NamePartBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = uninterpreted_option::NamePartBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_identifier_value(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_positive_int_value(&mut self, val: u64) -> Result<(), EncodeError> {
        self.writer.write_varint(val)?;
        self.writer.write_tag(4, WireType::Varint)
    }

    #[inline]
    pub fn write_negative_int_value(&mut self, val: i64) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(5, WireType::Varint)
    }

    #[inline]
    pub fn write_double_value(&mut self, val: f64) -> Result<(), EncodeError> {
        self.writer.write_f64(val)?;
        self.writer.write_tag(6, WireType::Fixed64)
    }

    #[inline]
    pub fn write_string_value(&mut self, val: &[u8]) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val)?;
        self.writer.write_tag(7, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_aggregate_value(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(8, WireType::LengthDelimited)
    }
}

// ============================================================================
// SourceCodeInfo (Reader View & Backward Builder)
// ============================================================================

pub mod source_code_info {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct LocationView<'a> {
        buf: &'a [u8],
        leading_comments: FieldMarker,
        trailing_comments: FieldMarker,
    }

    impl<'a> LocationView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 6];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut leading_comments = FieldMarker::none();
            let mut trailing_comments = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (3, WireType::LengthDelimited) => leading_comments = field.marker,
                    (4, WireType::LengthDelimited) => trailing_comments = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                leading_comments,
                trailing_comments,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn path(&self) -> RepeatedVarintIter<'a, i32> {
            RepeatedVarintIter::new(self.buf, 1)
        }

        #[inline]
        pub const fn span(&self) -> RepeatedVarintIter<'a, i32> {
            RepeatedVarintIter::new(self.buf, 2)
        }

        #[inline]
        pub const fn has_leading_comments(&self) -> bool {
            self.leading_comments.is_present()
        }

        #[inline]
        pub fn leading_comments(&self) -> Result<&'a str, DecodeError> {
            if !self.leading_comments.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.leading_comments)
        }

        #[inline]
        pub const fn has_trailing_comments(&self) -> bool {
            self.trailing_comments.is_present()
        }

        #[inline]
        pub fn trailing_comments(&self) -> Result<&'a str, DecodeError> {
            if !self.trailing_comments.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.trailing_comments)
        }

        #[inline]
        pub const fn leading_detached_comments(&self) -> RepeatedStrIter<'a> {
            RepeatedStrIter::new(self.buf, 6)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for LocationView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct LocationBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> LocationBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_path(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_packed_path(&mut self, vals: &[i32]) -> Result<(), EncodeError> {
            let end_pos = self.writer.cursor();
            for &val in vals.iter().rev() {
                self.writer.write_varint(val as u64)?;
            }
            let nested_len = end_pos - self.writer.cursor();
            self.writer.write_varint(nested_len as u64)?;
            self.writer.write_tag(1, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_span(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(2, WireType::Varint)
        }

        #[inline]
        pub fn write_packed_span(&mut self, vals: &[i32]) -> Result<(), EncodeError> {
            let end_pos = self.writer.cursor();
            for &val in vals.iter().rev() {
                self.writer.write_varint(val as u64)?;
            }
            let nested_len = end_pos - self.writer.cursor();
            self.writer.write_varint(nested_len as u64)?;
            self.writer.write_tag(2, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_leading_comments(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(3, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_trailing_comments(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(4, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_leading_detached_comments(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(6, WireType::LengthDelimited)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SourceCodeInfoView<'a> {
    buf: &'a [u8],
}

impl<'a> SourceCodeInfoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        for item in FieldIter::new(buf) {
            let _field = item?;
        }
        Ok(Self { buf })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn location(&self) -> RepeatedMessageIter<'a, source_code_info::LocationView<'a>> {
        RepeatedMessageIter::new(self.buf, 1)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for SourceCodeInfoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct SourceCodeInfoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> SourceCodeInfoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &SourceCodeInfoView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_location<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut source_code_info::LocationBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = source_code_info::LocationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }
}

// ============================================================================
// GeneratedCodeInfo (Reader View & Backward Builder)
// ============================================================================

pub mod generated_code_info {
    use super::*;

    pub mod annotation {
        use roto::runtime::VarintValue;

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum Semantic {
            None = 0,
            Set = 1,
            Alias = 2,
        }

        impl Semantic {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::None),
                    1 => Some(Self::Set),
                    2 => Some(Self::Alias),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl TryFrom<i32> for Semantic {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl VarintValue for Semantic {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::None)
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct AnnotationView<'a> {
        buf: &'a [u8],
        source_file: FieldMarker,
        begin: FieldMarker,
        end: FieldMarker,
        semantic: FieldMarker,
    }

    impl<'a> AnnotationView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut source_file = FieldMarker::none();
            let mut begin = FieldMarker::none();
            let mut end = FieldMarker::none();
            let mut semantic = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (2, WireType::LengthDelimited) => source_file = field.marker,
                    (3, WireType::Varint) => begin = field.marker,
                    (4, WireType::Varint) => end = field.marker,
                    (5, WireType::Varint) => semantic = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                source_file,
                begin,
                end,
                semantic,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn path(&self) -> RepeatedVarintIter<'a, i32> {
            RepeatedVarintIter::new(self.buf, 1)
        }

        #[inline]
        pub const fn has_source_file(&self) -> bool {
            self.source_file.is_present()
        }

        #[inline]
        pub fn source_file(&self) -> Result<&'a str, DecodeError> {
            if !self.source_file.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.source_file)
        }

        #[inline]
        pub const fn has_begin(&self) -> bool {
            self.begin.is_present()
        }

        #[inline]
        pub fn begin(&self) -> Result<i32, DecodeError> {
            if !self.begin.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.begin)? as i32)
        }

        #[inline]
        pub const fn has_end(&self) -> bool {
            self.end.is_present()
        }

        #[inline]
        pub fn end(&self) -> Result<i32, DecodeError> {
            if !self.end.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.end)? as i32)
        }

        #[inline]
        pub const fn has_semantic(&self) -> bool {
            self.semantic.is_present()
        }

        #[inline]
        pub fn semantic(&self) -> Result<annotation::Semantic, DecodeError> {
            if !self.semantic.is_present() {
                return Ok(annotation::Semantic::None);
            }
            let raw = decode_varint_at(self.buf, self.semantic)?;
            Ok(annotation::Semantic::from_raw_varint(raw))
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for AnnotationView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct AnnotationBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> AnnotationBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &AnnotationView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_path(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_packed_path(&mut self, vals: &[i32]) -> Result<(), EncodeError> {
            let end_pos = self.writer.cursor();
            for &val in vals.iter().rev() {
                self.writer.write_varint(val as u64)?;
            }
            let nested_len = end_pos - self.writer.cursor();
            self.writer.write_varint(nested_len as u64)?;
            self.writer.write_tag(1, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_source_file(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(2, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_begin(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(3, WireType::Varint)
        }

        #[inline]
        pub fn write_end(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(4, WireType::Varint)
        }

        #[inline]
        pub fn write_semantic(&mut self, val: annotation::Semantic) -> Result<(), EncodeError> {
            self.writer.write_varint(val.as_i32() as u64)?;
            self.writer.write_tag(5, WireType::Varint)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GeneratedCodeInfoView<'a> {
    buf: &'a [u8],
}

impl<'a> GeneratedCodeInfoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        for item in FieldIter::new(buf) {
            let _field = item?;
        }
        Ok(Self { buf })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn annotation(
        &self,
    ) -> RepeatedMessageIter<'a, generated_code_info::AnnotationView<'a>> {
        RepeatedMessageIter::new(self.buf, 1)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for GeneratedCodeInfoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct GeneratedCodeInfoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> GeneratedCodeInfoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &GeneratedCodeInfoView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_annotation<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut generated_code_info::AnnotationBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = generated_code_info::AnnotationBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }
}
