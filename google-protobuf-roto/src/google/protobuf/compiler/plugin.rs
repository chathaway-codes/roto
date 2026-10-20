use roto::runtime::{
    decode_str_at, decode_varint_at, BackwardWriter, DecodeError, EncodeError, FieldIter,
    FieldMarker, MessageView, RepeatedMessageIter, RepeatedStrIter, UnknownFieldsIter, WireType,
};

use crate::google::protobuf::descriptor::{
    FileDescriptorProtoBuilder, FileDescriptorProtoView, GeneratedCodeInfoBuilder,
    GeneratedCodeInfoView,
};

// ============================================================================
// Version (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct VersionView<'a> {
    buf: &'a [u8],
    major: FieldMarker,
    minor: FieldMarker,
    patch: FieldMarker,
    suffix: FieldMarker,
}

impl<'a> VersionView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut major = FieldMarker::none();
        let mut minor = FieldMarker::none();
        let mut patch = FieldMarker::none();
        let mut suffix = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::Varint) => major = field.marker,
                (2, WireType::Varint) => minor = field.marker,
                (3, WireType::Varint) => patch = field.marker,
                (4, WireType::LengthDelimited) => suffix = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            major,
            minor,
            patch,
            suffix,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_major(&self) -> bool {
        self.major.is_present()
    }

    #[inline]
    pub fn major(&self) -> Result<i32, DecodeError> {
        if !self.major.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.major)? as i32)
    }

    #[inline]
    pub const fn has_minor(&self) -> bool {
        self.minor.is_present()
    }

    #[inline]
    pub fn minor(&self) -> Result<i32, DecodeError> {
        if !self.minor.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.minor)? as i32)
    }

    #[inline]
    pub const fn has_patch(&self) -> bool {
        self.patch.is_present()
    }

    #[inline]
    pub fn patch(&self) -> Result<i32, DecodeError> {
        if !self.patch.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.patch)? as i32)
    }

    #[inline]
    pub const fn has_suffix(&self) -> bool {
        self.suffix.is_present()
    }

    #[inline]
    pub fn suffix(&self) -> Result<&'a str, DecodeError> {
        if !self.suffix.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.suffix)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for VersionView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct VersionBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> VersionBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(&mut self, view: &VersionView<'_>) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_major(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(1, WireType::Varint)
    }

    #[inline]
    pub fn write_minor(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_patch(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_suffix(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }
}

// ============================================================================
// CodeGeneratorRequest (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct CodeGeneratorRequestView<'a> {
    buf: &'a [u8],
    parameter: FieldMarker,
    compiler_version: FieldMarker,
}

impl<'a> CodeGeneratorRequestView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 15, 17];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut parameter = FieldMarker::none();
        let mut compiler_version = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (2, WireType::LengthDelimited) => parameter = field.marker,
                (3, WireType::LengthDelimited) => compiler_version = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            parameter,
            compiler_version,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn file_to_generate(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 1)
    }

    #[inline]
    pub const fn has_parameter(&self) -> bool {
        self.parameter.is_present()
    }

    #[inline]
    pub fn parameter(&self) -> Result<&'a str, DecodeError> {
        if !self.parameter.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.parameter)
    }

    #[inline]
    pub const fn proto_file(&self) -> RepeatedMessageIter<'a, FileDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 15)
    }

    #[inline]
    pub const fn source_file_descriptors(
        &self,
    ) -> RepeatedMessageIter<'a, FileDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 17)
    }

    #[inline]
    pub const fn has_compiler_version(&self) -> bool {
        self.compiler_version.is_present()
    }

    #[inline]
    pub fn compiler_version(&self) -> Result<Option<VersionView<'a>>, DecodeError> {
        if !self.compiler_version.is_present() {
            return Ok(None);
        }
        VersionView::from_bytes(self.compiler_version.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for CodeGeneratorRequestView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CodeGeneratorRequestBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> CodeGeneratorRequestBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &CodeGeneratorRequestView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_file_to_generate(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_parameter(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_proto_file<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FileDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FileDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(15, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_source_file_descriptors<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FileDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FileDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(17, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_source_file_descriptor<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FileDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        self.write_source_file_descriptors(f)
    }

    #[inline]
    pub fn write_compiler_version<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut VersionBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = VersionBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }
}

// ============================================================================
// CodeGeneratorResponse (Reader View, Backward Builder & Nested Types)
// ============================================================================

pub mod code_generator_response {
    use roto::runtime::{
        decode_str_at, BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker,
        MessageView, UnknownFieldsIter, VarintValue, WireType,
    };

    use super::{GeneratedCodeInfoBuilder, GeneratedCodeInfoView};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Feature {
        FeatureNone = 0,
        FeatureProto3Optional = 1,
        FeatureSupportsEditions = 2,
    }

    impl Feature {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::FeatureNone),
                1 => Some(Self::FeatureProto3Optional),
                2 => Some(Self::FeatureSupportsEditions),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for Feature {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for Feature {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::FeatureNone)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct FileView<'a> {
        buf: &'a [u8],
        name: FieldMarker,
        insertion_point: FieldMarker,
        content: FieldMarker,
        generated_code_info: FieldMarker,
    }

    impl<'a> FileView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 15, 16];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut name = FieldMarker::none();
            let mut insertion_point = FieldMarker::none();
            let mut content = FieldMarker::none();
            let mut generated_code_info = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::LengthDelimited) => name = field.marker,
                    (2, WireType::LengthDelimited) => insertion_point = field.marker,
                    (15, WireType::LengthDelimited) => content = field.marker,
                    (16, WireType::LengthDelimited) => generated_code_info = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                name,
                insertion_point,
                content,
                generated_code_info,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_name(&self) -> bool {
            self.name.is_present()
        }

        #[inline]
        pub fn name(&self) -> Result<&'a str, DecodeError> {
            if !self.name.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.name)
        }

        #[inline]
        pub const fn has_insertion_point(&self) -> bool {
            self.insertion_point.is_present()
        }

        #[inline]
        pub fn insertion_point(&self) -> Result<&'a str, DecodeError> {
            if !self.insertion_point.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.insertion_point)
        }

        #[inline]
        pub const fn has_content(&self) -> bool {
            self.content.is_present()
        }

        #[inline]
        pub fn content(&self) -> Result<&'a str, DecodeError> {
            if !self.content.is_present() {
                return Ok("");
            }
            decode_str_at(self.buf, self.content)
        }

        #[inline]
        pub const fn has_generated_code_info(&self) -> bool {
            self.generated_code_info.is_present()
        }

        #[inline]
        pub fn generated_code_info(
            &self,
        ) -> Result<Option<GeneratedCodeInfoView<'a>>, DecodeError> {
            if !self.generated_code_info.is_present() {
                return Ok(None);
            }
            GeneratedCodeInfoView::from_bytes(self.generated_code_info.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for FileView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct FileBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> FileBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(&mut self, view: &FileView<'_>) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_name(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(1, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_insertion_point(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(2, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_content(&mut self, val: &str) -> Result<(), EncodeError> {
            self.writer.write_length_delimited(val.as_bytes())?;
            self.writer.write_tag(15, WireType::LengthDelimited)
        }

        #[inline]
        pub fn write_generated_code_info<F>(&mut self, f: F) -> Result<(), EncodeError>
        where
            F: FnOnce(&mut GeneratedCodeInfoBuilder<'_, 'b>) -> Result<(), EncodeError>,
        {
            let end_pos = self.writer.cursor();
            {
                let mut sub_builder = GeneratedCodeInfoBuilder::new(self.writer);
                f(&mut sub_builder)?;
            }
            let nested_len = end_pos - self.writer.cursor();
            self.writer.write_varint(nested_len as u64)?;
            self.writer.write_tag(16, WireType::LengthDelimited)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CodeGeneratorResponseView<'a> {
    buf: &'a [u8],
    error: FieldMarker,
    supported_features: FieldMarker,
    minimum_edition: FieldMarker,
    maximum_edition: FieldMarker,
}

impl<'a> CodeGeneratorResponseView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 15];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut error = FieldMarker::none();
        let mut supported_features = FieldMarker::none();
        let mut minimum_edition = FieldMarker::none();
        let mut maximum_edition = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => error = field.marker,
                (2, WireType::Varint) => supported_features = field.marker,
                (3, WireType::Varint) => minimum_edition = field.marker,
                (4, WireType::Varint) => maximum_edition = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            error,
            supported_features,
            minimum_edition,
            maximum_edition,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_error(&self) -> bool {
        self.error.is_present()
    }

    #[inline]
    pub fn error(&self) -> Result<&'a str, DecodeError> {
        if !self.error.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.error)
    }

    #[inline]
    pub const fn has_supported_features(&self) -> bool {
        self.supported_features.is_present()
    }

    #[inline]
    pub fn supported_features(&self) -> Result<u64, DecodeError> {
        if !self.supported_features.is_present() {
            return Ok(0);
        }
        decode_varint_at(self.buf, self.supported_features)
    }

    #[inline]
    pub const fn has_minimum_edition(&self) -> bool {
        self.minimum_edition.is_present()
    }

    #[inline]
    pub fn minimum_edition(&self) -> Result<i32, DecodeError> {
        if !self.minimum_edition.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.minimum_edition)? as i32)
    }

    #[inline]
    pub const fn has_maximum_edition(&self) -> bool {
        self.maximum_edition.is_present()
    }

    #[inline]
    pub fn maximum_edition(&self) -> Result<i32, DecodeError> {
        if !self.maximum_edition.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.maximum_edition)? as i32)
    }

    #[inline]
    pub const fn file(&self) -> RepeatedMessageIter<'a, code_generator_response::FileView<'a>> {
        RepeatedMessageIter::new(self.buf, 15)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for CodeGeneratorResponseView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CodeGeneratorResponseBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> CodeGeneratorResponseBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &CodeGeneratorResponseView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_error(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(1, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_supported_features(&mut self, val: u64) -> Result<(), EncodeError> {
        self.writer.write_varint(val)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_minimum_edition(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_maximum_edition(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(4, WireType::Varint)
    }

    #[inline]
    pub fn write_file<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut code_generator_response::FileBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = code_generator_response::FileBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(15, WireType::LengthDelimited)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::google::protobuf::descriptor::{
        field_descriptor_proto::{Label, Type},
        generated_code_info::annotation::Semantic,
        Edition,
    };
    use code_generator_response::{Feature, FileBuilder, FileView};
    use roto::runtime::VarintValue;

    #[test]
    fn feature_enum_conversions() {
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
    fn version_and_request_roundtrip_with_unknown_fields() {
        let mut backing = [0u8; 1024];
        let mut writer = BackwardWriter::new(&mut backing);
        {
            // Unknown field 99 on CodeGeneratorRequest
            writer.write_varint(4242).unwrap();
            writer.write_tag(99, WireType::Varint).unwrap();

            let mut req = CodeGeneratorRequestBuilder::new(&mut writer);
            req.write_compiler_version(|ver| {
                // Unknown field 88 on Version
                ver.writer.write_varint(999).unwrap();
                ver.writer.write_tag(88, WireType::Varint).unwrap();

                ver.write_suffix("rc1")?;
                ver.write_patch(2)?;
                ver.write_minor(28)?;
                ver.write_major(4)
            })
            .unwrap();
            req.write_source_file_descriptor(|fdp| {
                fdp.write_syntax("proto2")?;
                fdp.write_package("demo")?;
                fdp.write_name("sensor.proto")
            })
            .unwrap();
            req.write_proto_file(|fdp| {
                fdp.write_message_type(|msg| {
                    msg.write_field(|fld| {
                        fld.write_type(Type::TypeInt32)?;
                        fld.write_label(Label::LabelRequired)?;
                        fld.write_number(1)?;
                        fld.write_name("lat_e7")
                    })?;
                    msg.write_name("Location")
                })?;
                fdp.write_package("demo")?;
                fdp.write_name("sensor.proto")
            })
            .unwrap();
            req.write_proto_file(|fdp| {
                fdp.write_package("google.protobuf")?;
                fdp.write_name("google/protobuf/descriptor.proto")
            })
            .unwrap();
            req.write_parameter("no_std=true,edition=2023").unwrap();
            req.write_file_to_generate("sensor.proto").unwrap();
        }
        let wire = writer.finish();

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

        // Copy unknown fields into a re-encoded request
        let mut backing2 = [0u8; 256];
        let mut writer2 = BackwardWriter::new(&mut backing2);
        {
            let mut req2 = CodeGeneratorRequestBuilder::new(&mut writer2);
            req2.copy_unknown_fields(&req_view).unwrap();
            req2.write_compiler_version(|ver2| {
                ver2.copy_unknown_fields(&ver_view)?;
                ver2.write_major(4)
            })
            .unwrap();
        }
        let req_view2 = CodeGeneratorRequestView::from_bytes(writer2.finish()).unwrap();
        assert_eq!(req_view2.unknown_fields().count(), 1);
        let ver_view2 = req_view2.compiler_version().unwrap().unwrap();
        assert_eq!(ver_view2.major(), Ok(4));
        assert_eq!(ver_view2.unknown_fields().count(), 1);
    }

    #[test]
    fn response_and_file_roundtrip_with_generated_code_info_and_unknown_fields() {
        let mut backing = [0u8; 1024];
        let mut writer = BackwardWriter::new(&mut backing);
        {
            // Unknown field 77 on CodeGeneratorResponse
            writer.write_varint(7777).unwrap();
            writer.write_tag(77, WireType::Varint).unwrap();

            let mut resp = CodeGeneratorResponseBuilder::new(&mut writer);
            resp.write_file(|file| {
                // Unknown field 66 on File
                file.writer.write_varint(6666).unwrap();
                file.writer.write_tag(66, WireType::Varint).unwrap();

                file.write_generated_code_info(|gci| {
                    gci.write_annotation(|ann| {
                        ann.write_semantic(Semantic::Set)?;
                        ann.write_end(42)?;
                        ann.write_begin(10)?;
                        ann.write_source_file("sensor.proto")?;
                        ann.write_packed_path(&[4, 0, 2, 0])
                    })
                })?;
                file.write_content("pub struct LocationView<'a> { buf: &'a [u8] }\n")?;
                file.write_insertion_point("module_scope")?;
                file.write_name("sensor.rs")
            })
            .unwrap();
            resp.write_maximum_edition(Edition::Edition2024.as_i32())
                .unwrap();
            resp.write_minimum_edition(Edition::EditionProto2.as_i32())
                .unwrap();
            resp.write_supported_features(
                (Feature::FeatureProto3Optional.as_i32()
                    | Feature::FeatureSupportsEditions.as_i32()) as u64,
            )
            .unwrap();
            resp.write_error("").unwrap();
        }
        let wire = writer.finish();

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

        // Copy unknown fields on both Response and File
        let mut backing2 = [0u8; 256];
        let mut writer2 = BackwardWriter::new(&mut backing2);
        {
            let mut resp2 = CodeGeneratorResponseBuilder::new(&mut writer2);
            resp2.copy_unknown_fields(&resp_view).unwrap();
            resp2
                .write_file(|f2| {
                    f2.copy_unknown_fields(&f_view)?;
                    f2.write_name("sensor.rs")
                })
                .unwrap();
        }
        let resp_view2 = CodeGeneratorResponseView::from_bytes(writer2.finish()).unwrap();
        assert_eq!(resp_view2.unknown_fields().count(), 1);
        let f_view2 = resp_view2.file().next().unwrap().unwrap();
        assert_eq!(f_view2.name(), Ok("sensor.rs"));
        assert_eq!(f_view2.unknown_fields().count(), 1);

        // Standalone FileBuilder test
        let mut f_backing = [0u8; 64];
        let mut f_writer = BackwardWriter::new(&mut f_backing);
        {
            let mut fb = FileBuilder::new(&mut f_writer);
            fb.write_content("// ok").unwrap();
            fb.write_name("mod.rs").unwrap();
        }
        let fv = FileView::from_bytes(f_writer.finish()).unwrap();
        assert_eq!(fv.name(), Ok("mod.rs"));
        assert_eq!(fv.content(), Ok("// ok"));
    }
}
