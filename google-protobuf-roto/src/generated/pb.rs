#[derive(Clone, Copy, Debug)]
pub struct CSharpFeaturesView<'a> {
    buf: &'a [u8],
    nullable_reference_types: bool,
    has_fields: [u32; 1],
}

impl<'a> CSharpFeaturesView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut nullable_reference_types: bool = false;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            #[allow(clippy::single_match)]
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    nullable_reference_types = field.val != 0;
                    has_fields[0] |= 1;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            nullable_reference_types,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_nullable_reference_types(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn nullable_reference_types(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.nullable_reference_types)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CSharpFeaturesView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CSharpFeaturesBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CSharpFeaturesBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { writer: ::roto::runtime::BackwardWriter::new(buf), set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer, set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
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
    pub fn copy_unknown_fields(&mut self, view: &CSharpFeaturesView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CSharpFeaturesView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CSharpFeaturesView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            1 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_nullable_reference_types(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CppFeaturesView<'a> {
    buf: &'a [u8],
    legacy_closed_enum: bool,
    string_type: crate::pb::cpp_features::StringType,
    enum_name_uses_string_view: bool,
    repeated_type: crate::pb::cpp_features::RepeatedType,
    has_fields: [u32; 1],
}

impl<'a> CppFeaturesView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut legacy_closed_enum: bool = false;
        let mut string_type: crate::pb::cpp_features::StringType = crate::pb::cpp_features::StringType::StringTypeUnknown;
        let mut enum_name_uses_string_view: bool = false;
        let mut repeated_type: crate::pb::cpp_features::RepeatedType = crate::pb::cpp_features::RepeatedType::RepeatedTypeUnknown;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    legacy_closed_enum = field.val != 0;
                    has_fields[0] |= 1;
                }
                (2, ::roto::runtime::WireType::Varint) => {
                    string_type = <crate::pb::cpp_features::StringType as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 2;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    enum_name_uses_string_view = field.val != 0;
                    has_fields[0] |= 4;
                }
                (4, ::roto::runtime::WireType::Varint) => {
                    repeated_type = <crate::pb::cpp_features::RepeatedType as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 8;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            legacy_closed_enum,
            string_type,
            enum_name_uses_string_view,
            repeated_type,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_legacy_closed_enum(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn legacy_closed_enum(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.legacy_closed_enum)
    }

    #[inline]
    pub const fn has_string_type(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn string_type(&self) -> Result<crate::pb::cpp_features::StringType, ::roto::runtime::DecodeError> {
        Ok(self.string_type)
    }

    #[inline]
    pub const fn has_enum_name_uses_string_view(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn enum_name_uses_string_view(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.enum_name_uses_string_view)
    }

    #[inline]
    pub const fn has_repeated_type(&self) -> bool {
        (self.has_fields[0] & 8) != 0
    }

    #[inline]
    pub const fn repeated_type(&self) -> Result<crate::pb::cpp_features::RepeatedType, ::roto::runtime::DecodeError> {
        Ok(self.repeated_type)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CppFeaturesView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CppFeaturesBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CppFeaturesBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { writer: ::roto::runtime::BackwardWriter::new(buf), set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer, set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
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
    pub fn copy_unknown_fields(&mut self, view: &CppFeaturesView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CppFeaturesView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CppFeaturesView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            1 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            2 => {
                self.set_fields[0] |= 2;
                (prev_set[0] & 2) == 0
            }
            3 => {
                self.set_fields[0] |= 4;
                (prev_set[0] & 4) == 0
            }
            4 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_legacy_closed_enum(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_string_type(&mut self, val: crate::pb::cpp_features::StringType) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_enum_name_uses_string_view(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_repeated_type(&mut self, val: crate::pb::cpp_features::RepeatedType) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }
}

pub mod cpp_features {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum StringType {
        StringTypeUnknown = 0,
        View = 1,
        Cord = 2,
        String = 3,
    }

    impl StringType {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::StringTypeUnknown),
                1 => Some(Self::View),
                2 => Some(Self::Cord),
                3 => Some(Self::String),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for StringType {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for StringType {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::StringTypeUnknown)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum RepeatedType {
        RepeatedTypeUnknown = 0,
        Legacy = 1,
        Proxy = 2,
    }

    impl RepeatedType {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::RepeatedTypeUnknown),
                1 => Some(Self::Legacy),
                2 => Some(Self::Proxy),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for RepeatedType {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for RepeatedType {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::RepeatedTypeUnknown)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GoFeaturesView<'a> {
    buf: &'a [u8],
    legacy_unmarshal_json_enum: bool,
    api_level: crate::pb::go_features::APILevel,
    strip_enum_prefix: crate::pb::go_features::StripEnumPrefix,
    optimize_mode: crate::pb::go_features::optimize_mode_feature::OptimizeMode,
    has_fields: [u32; 1],
}

impl<'a> GoFeaturesView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut legacy_unmarshal_json_enum: bool = false;
        let mut api_level: crate::pb::go_features::APILevel = crate::pb::go_features::APILevel::ApiLevelUnspecified;
        let mut strip_enum_prefix: crate::pb::go_features::StripEnumPrefix = crate::pb::go_features::StripEnumPrefix::StripEnumPrefixUnspecified;
        let mut optimize_mode: crate::pb::go_features::optimize_mode_feature::OptimizeMode = crate::pb::go_features::optimize_mode_feature::OptimizeMode::OptimizeModeUnspecified;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    legacy_unmarshal_json_enum = field.val != 0;
                    has_fields[0] |= 1;
                }
                (2, ::roto::runtime::WireType::Varint) => {
                    api_level = <crate::pb::go_features::APILevel as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 2;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    strip_enum_prefix = <crate::pb::go_features::StripEnumPrefix as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 4;
                }
                (4, ::roto::runtime::WireType::Varint) => {
                    optimize_mode = <crate::pb::go_features::optimize_mode_feature::OptimizeMode as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 8;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            legacy_unmarshal_json_enum,
            api_level,
            strip_enum_prefix,
            optimize_mode,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_legacy_unmarshal_json_enum(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn legacy_unmarshal_json_enum(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.legacy_unmarshal_json_enum)
    }

    #[inline]
    pub const fn has_api_level(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn api_level(&self) -> Result<crate::pb::go_features::APILevel, ::roto::runtime::DecodeError> {
        Ok(self.api_level)
    }

    #[inline]
    pub const fn has_strip_enum_prefix(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn strip_enum_prefix(&self) -> Result<crate::pb::go_features::StripEnumPrefix, ::roto::runtime::DecodeError> {
        Ok(self.strip_enum_prefix)
    }

    #[inline]
    pub const fn has_optimize_mode(&self) -> bool {
        (self.has_fields[0] & 8) != 0
    }

    #[inline]
    pub const fn optimize_mode(&self) -> Result<crate::pb::go_features::optimize_mode_feature::OptimizeMode, ::roto::runtime::DecodeError> {
        Ok(self.optimize_mode)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for GoFeaturesView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct GoFeaturesBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> GoFeaturesBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { writer: ::roto::runtime::BackwardWriter::new(buf), set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer, set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
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
    pub fn copy_unknown_fields(&mut self, view: &GoFeaturesView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoFeaturesView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &GoFeaturesView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            1 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            2 => {
                self.set_fields[0] |= 2;
                (prev_set[0] & 2) == 0
            }
            3 => {
                self.set_fields[0] |= 4;
                (prev_set[0] & 4) == 0
            }
            4 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_legacy_unmarshal_json_enum(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_api_level(&mut self, val: crate::pb::go_features::APILevel) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_strip_enum_prefix(&mut self, val: crate::pb::go_features::StripEnumPrefix) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_optimize_mode(&mut self, val: crate::pb::go_features::optimize_mode_feature::OptimizeMode) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }
}

pub mod go_features {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum APILevel {
        ApiLevelUnspecified = 0,
        ApiOpen = 1,
        ApiHybrid = 2,
        ApiOpaque = 3,
    }

    impl APILevel {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::ApiLevelUnspecified),
                1 => Some(Self::ApiOpen),
                2 => Some(Self::ApiHybrid),
                3 => Some(Self::ApiOpaque),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for APILevel {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for APILevel {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::ApiLevelUnspecified)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum StripEnumPrefix {
        StripEnumPrefixUnspecified = 0,
        StripEnumPrefixKeep = 1,
        StripEnumPrefixGenerateBoth = 2,
        StripEnumPrefixStrip = 3,
    }

    impl StripEnumPrefix {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::StripEnumPrefixUnspecified),
                1 => Some(Self::StripEnumPrefixKeep),
                2 => Some(Self::StripEnumPrefixGenerateBoth),
                3 => Some(Self::StripEnumPrefixStrip),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for StripEnumPrefix {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for StripEnumPrefix {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::StripEnumPrefixUnspecified)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct OptimizeModeFeatureView<'a> {
        buf: &'a [u8],
    }

    impl<'a> OptimizeModeFeatureView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            for item in ::roto::runtime::FieldIter::new(buf) {
                let _field = item?;
            }

            Ok(Self { buf })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for OptimizeModeFeatureView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct OptimizeModeFeatureBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
    }

    impl<'b> OptimizeModeFeatureBuilder<'b> {
        #[inline]
        pub fn new(buf: &'b mut [u8]) -> Self {
            Self { writer: ::roto::runtime::BackwardWriter::new(buf) }
        }

        #[inline]
        pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
            &self.writer
        }

        #[inline]
        pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
            &mut self.writer
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
        pub fn copy_unknown_fields(&mut self, view: &OptimizeModeFeatureView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !OptimizeModeFeatureView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &OptimizeModeFeatureView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |_| true);
            self
        }
    }

    pub mod optimize_mode_feature {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum OptimizeMode {
            OptimizeModeUnspecified = 0,
            Speed = 1,
            CodeSize = 2,
        }

        impl OptimizeMode {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::OptimizeModeUnspecified),
                    1 => Some(Self::Speed),
                    2 => Some(Self::CodeSize),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for OptimizeMode {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for OptimizeMode {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::OptimizeModeUnspecified)
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JavaFeaturesView<'a> {
    buf: &'a [u8],
    legacy_closed_enum: bool,
    utf8_validation: crate::pb::java_features::Utf8Validation,
    large_enum: bool,
    use_old_outer_classname_default: bool,
    nest_in_file_class: crate::pb::java_features::nest_in_file_class_feature::NestInFileClass,
    has_fields: [u32; 1],
}

impl<'a> JavaFeaturesView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut legacy_closed_enum: bool = false;
        let mut utf8_validation: crate::pb::java_features::Utf8Validation = crate::pb::java_features::Utf8Validation::Utf8ValidationUnknown;
        let mut large_enum: bool = false;
        let mut use_old_outer_classname_default: bool = false;
        let mut nest_in_file_class: crate::pb::java_features::nest_in_file_class_feature::NestInFileClass = crate::pb::java_features::nest_in_file_class_feature::NestInFileClass::NestInFileClassUnknown;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    legacy_closed_enum = field.val != 0;
                    has_fields[0] |= 1;
                }
                (2, ::roto::runtime::WireType::Varint) => {
                    utf8_validation = <crate::pb::java_features::Utf8Validation as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 2;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    large_enum = field.val != 0;
                    has_fields[0] |= 4;
                }
                (4, ::roto::runtime::WireType::Varint) => {
                    use_old_outer_classname_default = field.val != 0;
                    has_fields[0] |= 8;
                }
                (5, ::roto::runtime::WireType::Varint) => {
                    nest_in_file_class = <crate::pb::java_features::nest_in_file_class_feature::NestInFileClass as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 16;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            legacy_closed_enum,
            utf8_validation,
            large_enum,
            use_old_outer_classname_default,
            nest_in_file_class,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_legacy_closed_enum(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn legacy_closed_enum(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.legacy_closed_enum)
    }

    #[inline]
    pub const fn has_utf8_validation(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn utf8_validation(&self) -> Result<crate::pb::java_features::Utf8Validation, ::roto::runtime::DecodeError> {
        Ok(self.utf8_validation)
    }

    #[inline]
    pub const fn has_large_enum(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn large_enum(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.large_enum)
    }

    #[inline]
    pub const fn has_use_old_outer_classname_default(&self) -> bool {
        (self.has_fields[0] & 8) != 0
    }

    #[inline]
    pub const fn use_old_outer_classname_default(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.use_old_outer_classname_default)
    }

    #[inline]
    pub const fn has_nest_in_file_class(&self) -> bool {
        (self.has_fields[0] & 16) != 0
    }

    #[inline]
    pub const fn nest_in_file_class(&self) -> Result<crate::pb::java_features::nest_in_file_class_feature::NestInFileClass, ::roto::runtime::DecodeError> {
        Ok(self.nest_in_file_class)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for JavaFeaturesView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct JavaFeaturesBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> JavaFeaturesBuilder<'b> {
    #[inline]
    pub fn new(buf: &'b mut [u8]) -> Self {
        Self { writer: ::roto::runtime::BackwardWriter::new(buf), set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
        Self { writer, set_fields: [0u64; 1] }
    }

    #[inline]
    pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
        &self.writer
    }

    #[inline]
    pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
        &mut self.writer
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
    pub fn copy_unknown_fields(&mut self, view: &JavaFeaturesView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !JavaFeaturesView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &JavaFeaturesView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            1 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            2 => {
                self.set_fields[0] |= 2;
                (prev_set[0] & 2) == 0
            }
            3 => {
                self.set_fields[0] |= 4;
                (prev_set[0] & 4) == 0
            }
            4 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            5 => {
                self.set_fields[0] |= 16;
                (prev_set[0] & 16) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_legacy_closed_enum(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_utf8_validation(&mut self, val: crate::pb::java_features::Utf8Validation) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_large_enum(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_use_old_outer_classname_default(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_nest_in_file_class(&mut self, val: crate::pb::java_features::nest_in_file_class_feature::NestInFileClass) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
        self
    }
}

pub mod java_features {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Utf8Validation {
        Utf8ValidationUnknown = 0,
        Default = 1,
        Verify = 2,
    }

    impl Utf8Validation {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::Utf8ValidationUnknown),
                1 => Some(Self::Default),
                2 => Some(Self::Verify),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for Utf8Validation {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for Utf8Validation {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::Utf8ValidationUnknown)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct NestInFileClassFeatureView<'a> {
        buf: &'a [u8],
    }

    impl<'a> NestInFileClassFeatureView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            for item in ::roto::runtime::FieldIter::new(buf) {
                let _field = item?;
            }

            Ok(Self { buf })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for NestInFileClassFeatureView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct NestInFileClassFeatureBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
    }

    impl<'b> NestInFileClassFeatureBuilder<'b> {
        #[inline]
        pub fn new(buf: &'b mut [u8]) -> Self {
            Self { writer: ::roto::runtime::BackwardWriter::new(buf) }
        }

        #[inline]
        pub const fn from_writer(writer: ::roto::runtime::BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub const fn writer(&self) -> &::roto::runtime::BackwardWriter<'b> {
            &self.writer
        }

        #[inline]
        pub fn writer_mut(&mut self) -> &mut ::roto::runtime::BackwardWriter<'b> {
            &mut self.writer
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
        pub fn copy_unknown_fields(&mut self, view: &NestInFileClassFeatureView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !NestInFileClassFeatureView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &NestInFileClassFeatureView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |_| true);
            self
        }
    }

    pub mod nest_in_file_class_feature {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum NestInFileClass {
            NestInFileClassUnknown = 0,
            No = 1,
            Yes = 2,
            Legacy = 3,
        }

        impl NestInFileClass {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::NestInFileClassUnknown),
                    1 => Some(Self::No),
                    2 => Some(Self::Yes),
                    3 => Some(Self::Legacy),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for NestInFileClass {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for NestInFileClass {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::NestInFileClassUnknown)
            }
        }
    }
}
