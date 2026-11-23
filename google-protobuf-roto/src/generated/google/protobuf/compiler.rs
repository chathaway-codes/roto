#[derive(Clone, Copy, Debug)]
pub struct VersionView<'a> {
    buf: &'a [u8],
    major: i32,
    minor: i32,
    patch: i32,
    suffix: ::roto::runtime::FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> VersionView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut major: i32 = 0i32;
        let mut minor: i32 = 0i32;
        let mut patch: i32 = 0i32;
        let mut suffix = ::roto::runtime::FieldMarker::none();
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    major = field.val as i32;
                    has_fields[0] |= 1;
                }
                (2, ::roto::runtime::WireType::Varint) => {
                    minor = field.val as i32;
                    has_fields[0] |= 2;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    patch = field.val as i32;
                    has_fields[0] |= 4;
                }
                (4, ::roto::runtime::WireType::LengthDelimited) => suffix = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            major,
            minor,
            patch,
            suffix,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_major(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn major(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.major)
    }

    #[inline]
    pub const fn has_minor(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn minor(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.minor)
    }

    #[inline]
    pub const fn has_patch(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn patch(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.patch)
    }

    #[inline]
    pub const fn has_suffix(&self) -> bool {
        self.suffix.is_present()
    }

    #[inline]
    pub fn suffix(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.suffix.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.suffix)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for VersionView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct VersionBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> VersionBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &VersionView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !VersionView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &VersionView<'_>) -> &mut Self {
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
    pub fn write_major(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_minor(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_patch(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_suffix(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CodeGeneratorRequestView<'a> {
    buf: &'a [u8],
    file_to_generate: ::roto::runtime::FieldMarker,
    parameter: ::roto::runtime::FieldMarker,
    proto_file: ::roto::runtime::FieldMarker,
    source_file_descriptors: ::roto::runtime::FieldMarker,
    compiler_version: ::roto::runtime::FieldMarker,
}

impl<'a> CodeGeneratorRequestView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 15, 17, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut file_to_generate = ::roto::runtime::FieldMarker::none();
        let mut parameter = ::roto::runtime::FieldMarker::none();
        let mut proto_file = ::roto::runtime::FieldMarker::none();
        let mut source_file_descriptors = ::roto::runtime::FieldMarker::none();
        let mut compiler_version = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, _) => file_to_generate.extend(field.raw_span),
                (2, ::roto::runtime::WireType::LengthDelimited) => parameter = field.marker,
                (15, _) => proto_file.extend(field.raw_span),
                (17, _) => source_file_descriptors.extend(field.raw_span),
                (3, ::roto::runtime::WireType::LengthDelimited) => compiler_version = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            file_to_generate,
            parameter,
            proto_file,
            source_file_descriptors,
            compiler_version,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn file_to_generate(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.file_to_generate.slice(self.buf), 1)
    }

    #[inline]
    pub const fn has_parameter(&self) -> bool {
        self.parameter.is_present()
    }

    #[inline]
    pub fn parameter(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.parameter.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.parameter)
    }

    #[inline]
    pub const fn proto_file(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FileDescriptorProtoView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.proto_file.slice(self.buf), 15)
    }

    #[inline]
    pub const fn source_file_descriptors(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FileDescriptorProtoView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.source_file_descriptors.slice(self.buf), 17)
    }

    #[inline]
    pub const fn has_compiler_version(&self) -> bool {
        self.compiler_version.is_present()
    }

    #[inline]
    pub fn compiler_version(&self) -> Result<Option<crate::google::protobuf::compiler::VersionView<'a>>, ::roto::runtime::DecodeError> {
        if !self.compiler_version.is_present() {
            return Ok(None);
        }
        crate::google::protobuf::compiler::VersionView::from_bytes(self.compiler_version.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CodeGeneratorRequestView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CodeGeneratorRequestBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CodeGeneratorRequestBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &CodeGeneratorRequestView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CodeGeneratorRequestView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CodeGeneratorRequestView<'_>) -> &mut Self {
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
            15 => {
                self.set_fields[0] |= 4;
                (prev_set[0] & 4) == 0
            }
            17 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            3 => {
                self.set_fields[0] |= 16;
                (prev_set[0] & 16) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_file_to_generate(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_parameter(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_proto_file<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_source_file_descriptors<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_>),
    {
        self.set_fields[0] |= 8;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(17, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_source_file_descriptor<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_>),
    {
        self.write_source_file_descriptors(f)
    }

    #[inline]
    pub fn write_compiler_version<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::compiler::VersionBuilder<'_>),
    {
        self.set_fields[0] |= 16;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::protobuf::compiler::VersionBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CodeGeneratorResponseView<'a> {
    buf: &'a [u8],
    error: ::roto::runtime::FieldMarker,
    supported_features: u64,
    minimum_edition: i32,
    maximum_edition: i32,
    file: ::roto::runtime::FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> CodeGeneratorResponseView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 15];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut error = ::roto::runtime::FieldMarker::none();
        let mut supported_features: u64 = 0u64;
        let mut minimum_edition: i32 = 0i32;
        let mut maximum_edition: i32 = 0i32;
        let mut file = ::roto::runtime::FieldMarker::none();
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => error = field.marker,
                (2, ::roto::runtime::WireType::Varint) => {
                    supported_features = field.val;
                    has_fields[0] |= 1;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    minimum_edition = field.val as i32;
                    has_fields[0] |= 2;
                }
                (4, ::roto::runtime::WireType::Varint) => {
                    maximum_edition = field.val as i32;
                    has_fields[0] |= 4;
                }
                (15, _) => file.extend(field.raw_span),
                _ => {}
            }
        }

        Ok(Self {
            buf,
            error,
            supported_features,
            minimum_edition,
            maximum_edition,
            file,
            has_fields,
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
    pub fn error(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.error.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.error)
    }

    #[inline]
    pub const fn has_supported_features(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn supported_features(&self) -> Result<u64, ::roto::runtime::DecodeError> {
        Ok(self.supported_features)
    }

    #[inline]
    pub const fn has_minimum_edition(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn minimum_edition(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.minimum_edition)
    }

    #[inline]
    pub const fn has_maximum_edition(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn maximum_edition(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.maximum_edition)
    }

    #[inline]
    pub const fn file(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::compiler::code_generator_response::FileView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.file.slice(self.buf), 15)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CodeGeneratorResponseView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CodeGeneratorResponseBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CodeGeneratorResponseBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &CodeGeneratorResponseView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CodeGeneratorResponseView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CodeGeneratorResponseView<'_>) -> &mut Self {
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
            15 => {
                self.set_fields[0] |= 16;
                (prev_set[0] & 16) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_error(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_supported_features(&mut self, val: u64) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_minimum_edition(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_maximum_edition(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_file<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::compiler::code_generator_response::FileBuilder<'_>),
    {
        self.set_fields[0] |= 16;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::protobuf::compiler::code_generator_response::FileBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

pub mod code_generator_response {
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

    impl core::convert::TryFrom<i32> for Feature {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for Feature {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::FeatureNone)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct FileView<'a> {
        buf: &'a [u8],
        name: ::roto::runtime::FieldMarker,
        insertion_point: ::roto::runtime::FieldMarker,
        content: ::roto::runtime::FieldMarker,
        generated_code_info: ::roto::runtime::FieldMarker,
    }

    impl<'a> FileView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 15, 16];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut name = ::roto::runtime::FieldMarker::none();
            let mut insertion_point = ::roto::runtime::FieldMarker::none();
            let mut content = ::roto::runtime::FieldMarker::none();
            let mut generated_code_info = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                    (2, ::roto::runtime::WireType::LengthDelimited) => insertion_point = field.marker,
                    (15, ::roto::runtime::WireType::LengthDelimited) => content = field.marker,
                    (16, ::roto::runtime::WireType::LengthDelimited) => generated_code_info = field.marker,
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
        pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.name.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.name)
        }

        #[inline]
        pub const fn has_insertion_point(&self) -> bool {
            self.insertion_point.is_present()
        }

        #[inline]
        pub fn insertion_point(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.insertion_point.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.insertion_point)
        }

        #[inline]
        pub const fn has_content(&self) -> bool {
            self.content.is_present()
        }

        #[inline]
        pub fn content(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.content.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.content)
        }

        #[inline]
        pub const fn has_generated_code_info(&self) -> bool {
            self.generated_code_info.is_present()
        }

        #[inline]
        pub fn generated_code_info(&self) -> Result<Option<crate::google::protobuf::GeneratedCodeInfoView<'a>>, ::roto::runtime::DecodeError> {
            if !self.generated_code_info.is_present() {
                return Ok(None);
            }
            crate::google::protobuf::GeneratedCodeInfoView::from_bytes(self.generated_code_info.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for FileView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct FileBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> FileBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &FileView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FileView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &FileView<'_>) -> &mut Self {
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
                15 => {
                    self.set_fields[0] |= 4;
                    (prev_set[0] & 4) == 0
                }
                16 => {
                    self.set_fields[0] |= 8;
                    (prev_set[0] & 8) == 0
                }
                _ => true,
            });
            self
        }

        #[inline]
        pub fn write_name(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_insertion_point(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_content(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 4;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_generated_code_info<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::google::protobuf::GeneratedCodeInfoBuilder<'_>),
        {
            self.set_fields[0] |= 8;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::google::protobuf::GeneratedCodeInfoBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(16, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }
}
