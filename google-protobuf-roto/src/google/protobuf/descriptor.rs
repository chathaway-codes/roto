use roto::runtime::{
    BackwardWriter, DecodeError, EncodeError, FieldIter, FieldMarker, MessageView,
    RepeatedMessageIter, RepeatedStrIter, RepeatedVarintIter, UnknownFieldsIter, VarintValue,
    WireType, decode_bool_at, decode_str_at, decode_varint_at,
};

// ============================================================================
// Enums
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Edition {
    EditionUnknown = 0,
    Edition1TestOnly = 1,
    Edition2TestOnly = 2,
    EditionLegacy = 900,
    EditionProto2 = 998,
    EditionProto3 = 999,
    Edition2023 = 1000,
    Edition2024 = 1001,
    Edition2026 = 1002,
    EditionUnstable = 9999,
    Edition99997TestOnly = 99997,
    Edition99998TestOnly = 99998,
    Edition99999TestOnly = 99999,
    EditionMax = 0x7FFFFFFF,
}

impl Edition {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::EditionUnknown),
            1 => Some(Self::Edition1TestOnly),
            2 => Some(Self::Edition2TestOnly),
            900 => Some(Self::EditionLegacy),
            998 => Some(Self::EditionProto2),
            999 => Some(Self::EditionProto3),
            1000 => Some(Self::Edition2023),
            1001 => Some(Self::Edition2024),
            1002 => Some(Self::Edition2026),
            9999 => Some(Self::EditionUnstable),
            99997 => Some(Self::Edition99997TestOnly),
            99998 => Some(Self::Edition99998TestOnly),
            99999 => Some(Self::Edition99999TestOnly),
            0x7FFFFFFF => Some(Self::EditionMax),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for Edition {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl VarintValue for Edition {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::EditionUnknown)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SymbolVisibility {
    VisibilityUnset = 0,
    VisibilityLocal = 1,
    VisibilityExport = 2,
}

impl SymbolVisibility {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::VisibilityUnset),
            1 => Some(Self::VisibilityLocal),
            2 => Some(Self::VisibilityExport),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for SymbolVisibility {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl VarintValue for SymbolVisibility {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::VisibilityUnset)
    }
}

pub mod field_descriptor_proto {
    use roto::runtime::VarintValue;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Type {
        TypeDouble = 1,
        TypeFloat = 2,
        TypeInt64 = 3,
        TypeUint64 = 4,
        TypeInt32 = 5,
        TypeFixed64 = 6,
        TypeFixed32 = 7,
        TypeBool = 8,
        TypeString = 9,
        TypeGroup = 10,
        TypeMessage = 11,
        TypeBytes = 12,
        TypeUint32 = 13,
        TypeEnum = 14,
        TypeSfixed32 = 15,
        TypeSfixed64 = 16,
        TypeSint32 = 17,
        TypeSint64 = 18,
    }

    impl Type {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                1 => Some(Self::TypeDouble),
                2 => Some(Self::TypeFloat),
                3 => Some(Self::TypeInt64),
                4 => Some(Self::TypeUint64),
                5 => Some(Self::TypeInt32),
                6 => Some(Self::TypeFixed64),
                7 => Some(Self::TypeFixed32),
                8 => Some(Self::TypeBool),
                9 => Some(Self::TypeString),
                10 => Some(Self::TypeGroup),
                11 => Some(Self::TypeMessage),
                12 => Some(Self::TypeBytes),
                13 => Some(Self::TypeUint32),
                14 => Some(Self::TypeEnum),
                15 => Some(Self::TypeSfixed32),
                16 => Some(Self::TypeSfixed64),
                17 => Some(Self::TypeSint32),
                18 => Some(Self::TypeSint64),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for Type {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for Type {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::TypeDouble)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Label {
        LabelOptional = 1,
        LabelRequired = 2,
        LabelRepeated = 3,
    }

    impl Label {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                1 => Some(Self::LabelOptional),
                2 => Some(Self::LabelRequired),
                3 => Some(Self::LabelRepeated),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl TryFrom<i32> for Label {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl VarintValue for Label {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::LabelOptional)
        }
    }
}

// ============================================================================
// FileDescriptorSet (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FileDescriptorSetView<'a> {
    buf: &'a [u8],
}

impl<'a> FileDescriptorSetView<'a> {
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
    pub const fn file(&self) -> RepeatedMessageIter<'a, FileDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 1)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FileDescriptorSetView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FileDescriptorSetBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FileDescriptorSetBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &FileDescriptorSetView<'_>,
    ) -> Result<(), EncodeError> {
        for raw_field in view.unknown_fields() {
            self.writer.write_bytes(raw_field)?;
        }
        Ok(())
    }

    #[inline]
    pub fn write_file<F>(&mut self, f: F) -> Result<(), EncodeError>
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
        self.writer.write_tag(1, WireType::LengthDelimited)
    }
}

// ============================================================================
// FileDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FileDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    package: FieldMarker,
    options: FieldMarker,
    source_code_info: FieldMarker,
    syntax: FieldMarker,
    edition: FieldMarker,
}

impl<'a> FileDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 14, 15];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut package = FieldMarker::none();
        let mut options = FieldMarker::none();
        let mut source_code_info = FieldMarker::none();
        let mut syntax = FieldMarker::none();
        let mut edition = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (2, WireType::LengthDelimited) => package = field.marker,
                (8, WireType::LengthDelimited) => options = field.marker,
                (9, WireType::LengthDelimited) => source_code_info = field.marker,
                (12, WireType::LengthDelimited) => syntax = field.marker,
                (14, WireType::Varint) => edition = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            package,
            options,
            source_code_info,
            syntax,
            edition,
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
    pub const fn has_package(&self) -> bool {
        self.package.is_present()
    }

    #[inline]
    pub fn package(&self) -> Result<&'a str, DecodeError> {
        if !self.package.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.package)
    }

    #[inline]
    pub const fn dependency(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 3)
    }

    #[inline]
    pub const fn public_dependency(&self) -> RepeatedVarintIter<'a, i32> {
        RepeatedVarintIter::new(self.buf, 10)
    }

    #[inline]
    pub const fn weak_dependency(&self) -> RepeatedVarintIter<'a, i32> {
        RepeatedVarintIter::new(self.buf, 11)
    }

    #[inline]
    pub const fn option_dependency(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 15)
    }

    #[inline]
    pub const fn message_type(&self) -> RepeatedMessageIter<'a, DescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 4)
    }

    #[inline]
    pub const fn enum_type(&self) -> RepeatedMessageIter<'a, EnumDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 5)
    }

    #[inline]
    pub const fn service(&self) -> RepeatedMessageIter<'a, ServiceDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 6)
    }

    #[inline]
    pub const fn extension(&self) -> RepeatedMessageIter<'a, FieldDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 7)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<FileOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        FileOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_source_code_info(&self) -> bool {
        self.source_code_info.is_present()
    }

    #[inline]
    pub fn source_code_info(&self) -> Result<Option<SourceCodeInfoView<'a>>, DecodeError> {
        if !self.source_code_info.is_present() {
            return Ok(None);
        }
        SourceCodeInfoView::from_bytes(self.source_code_info.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_syntax(&self) -> bool {
        self.syntax.is_present()
    }

    #[inline]
    pub fn syntax(&self) -> Result<&'a str, DecodeError> {
        if !self.syntax.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.syntax)
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
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FileDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FileDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FileDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &FileDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_package(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_dependency(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_public_dependency(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(10, WireType::Varint)
    }

    #[inline]
    pub fn write_weak_dependency(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(11, WireType::Varint)
    }

    #[inline]
    pub fn write_option_dependency(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(15, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_message_type<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut DescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = DescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_enum_type<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut EnumDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = EnumDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(5, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_service<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut ServiceDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = ServiceDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(6, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_extension<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FieldDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FieldDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(7, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FileOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FileOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(8, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_source_code_info<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut SourceCodeInfoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = SourceCodeInfoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(9, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_syntax(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(12, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_edition(&mut self, val: Edition) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(14, WireType::Varint)
    }
}

// ============================================================================
// DescriptorProto (Reader View & Backward Builder)
// ============================================================================

pub mod descriptor_proto {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct ExtensionRangeView<'a> {
        buf: &'a [u8],
        start: FieldMarker,
        end: FieldMarker,
        options: FieldMarker,
    }

    impl<'a> ExtensionRangeView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut start = FieldMarker::none();
            let mut end = FieldMarker::none();
            let mut options = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::Varint) => start = field.marker,
                    (2, WireType::Varint) => end = field.marker,
                    (3, WireType::LengthDelimited) => options = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                start,
                end,
                options,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_start(&self) -> bool {
            self.start.is_present()
        }

        #[inline]
        pub fn start(&self) -> Result<i32, DecodeError> {
            if !self.start.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.start)? as i32)
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
        pub const fn has_options(&self) -> bool {
            self.options.is_present()
        }

        #[inline]
        pub fn options(&self) -> Result<Option<ExtensionRangeOptionsView<'a>>, DecodeError> {
            if !self.options.is_present() {
                return Ok(None);
            }
            ExtensionRangeOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for ExtensionRangeView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct ExtensionRangeBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> ExtensionRangeBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &ExtensionRangeView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_start(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_end(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(2, WireType::Varint)
        }

        #[inline]
        pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
        where
            F: FnOnce(&mut ExtensionRangeOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
        {
            let end_pos = self.writer.cursor();
            {
                let mut sub_builder = ExtensionRangeOptionsBuilder::new(self.writer);
                f(&mut sub_builder)?;
            }
            let nested_len = end_pos - self.writer.cursor();
            self.writer.write_varint(nested_len as u64)?;
            self.writer.write_tag(3, WireType::LengthDelimited)
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct ReservedRangeView<'a> {
        buf: &'a [u8],
        start: FieldMarker,
        end: FieldMarker,
    }

    impl<'a> ReservedRangeView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut start = FieldMarker::none();
            let mut end = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::Varint) => start = field.marker,
                    (2, WireType::Varint) => end = field.marker,
                    _ => {}
                }
            }

            Ok(Self { buf, start, end })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_start(&self) -> bool {
            self.start.is_present()
        }

        #[inline]
        pub fn start(&self) -> Result<i32, DecodeError> {
            if !self.start.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.start)? as i32)
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
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for ReservedRangeView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct ReservedRangeBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> ReservedRangeBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &ReservedRangeView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_start(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_end(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(2, WireType::Varint)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    options: FieldMarker,
    visibility: FieldMarker,
}

impl<'a> DescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut options = FieldMarker::none();
        let mut visibility = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (7, WireType::LengthDelimited) => options = field.marker,
                (11, WireType::Varint) => visibility = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            options,
            visibility,
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
    pub const fn field(&self) -> RepeatedMessageIter<'a, FieldDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 2)
    }

    #[inline]
    pub const fn extension(&self) -> RepeatedMessageIter<'a, FieldDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 6)
    }

    #[inline]
    pub const fn nested_type(&self) -> RepeatedMessageIter<'a, DescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 3)
    }

    #[inline]
    pub const fn enum_type(&self) -> RepeatedMessageIter<'a, EnumDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 4)
    }

    #[inline]
    pub const fn extension_range(
        &self,
    ) -> RepeatedMessageIter<'a, descriptor_proto::ExtensionRangeView<'a>> {
        RepeatedMessageIter::new(self.buf, 5)
    }

    #[inline]
    pub const fn oneof_decl(&self) -> RepeatedMessageIter<'a, OneofDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 8)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<MessageOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        MessageOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn reserved_range(
        &self,
    ) -> RepeatedMessageIter<'a, descriptor_proto::ReservedRangeView<'a>> {
        RepeatedMessageIter::new(self.buf, 9)
    }

    #[inline]
    pub const fn reserved_name(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 10)
    }

    #[inline]
    pub const fn has_visibility(&self) -> bool {
        self.visibility.is_present()
    }

    #[inline]
    pub fn visibility(&self) -> Result<SymbolVisibility, DecodeError> {
        if !self.visibility.is_present() {
            return Ok(SymbolVisibility::VisibilityUnset);
        }
        let raw = decode_varint_at(self.buf, self.visibility)?;
        Ok(SymbolVisibility::from_raw_varint(raw))
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for DescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct DescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> DescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &DescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_field<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FieldDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FieldDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_nested_type<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut DescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = DescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_enum_type<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut EnumDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = EnumDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_extension_range<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut descriptor_proto::ExtensionRangeBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = descriptor_proto::ExtensionRangeBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(5, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_extension<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FieldDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FieldDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(6, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut MessageOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = MessageOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(7, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_oneof_decl<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut OneofDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = OneofDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(8, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_reserved_range<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut descriptor_proto::ReservedRangeBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = descriptor_proto::ReservedRangeBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(9, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_reserved_name(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(10, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_visibility(&mut self, val: SymbolVisibility) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(11, WireType::Varint)
    }
}

// ============================================================================
// FieldDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct FieldDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    extendee: FieldMarker,
    number: FieldMarker,
    label: FieldMarker,
    r#type: FieldMarker,
    type_name: FieldMarker,
    default_value: FieldMarker,
    options: FieldMarker,
    oneof_index: FieldMarker,
    json_name: FieldMarker,
    proto3_optional: FieldMarker,
}

impl<'a> FieldDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 17];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut extendee = FieldMarker::none();
        let mut number = FieldMarker::none();
        let mut label = FieldMarker::none();
        let mut r#type = FieldMarker::none();
        let mut type_name = FieldMarker::none();
        let mut default_value = FieldMarker::none();
        let mut options = FieldMarker::none();
        let mut oneof_index = FieldMarker::none();
        let mut json_name = FieldMarker::none();
        let mut proto3_optional = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (2, WireType::LengthDelimited) => extendee = field.marker,
                (3, WireType::Varint) => number = field.marker,
                (4, WireType::Varint) => label = field.marker,
                (5, WireType::Varint) => r#type = field.marker,
                (6, WireType::LengthDelimited) => type_name = field.marker,
                (7, WireType::LengthDelimited) => default_value = field.marker,
                (8, WireType::LengthDelimited) => options = field.marker,
                (9, WireType::Varint) => oneof_index = field.marker,
                (10, WireType::LengthDelimited) => json_name = field.marker,
                (17, WireType::Varint) => proto3_optional = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            extendee,
            number,
            label,
            r#type,
            type_name,
            default_value,
            options,
            oneof_index,
            json_name,
            proto3_optional,
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
    pub const fn has_extendee(&self) -> bool {
        self.extendee.is_present()
    }

    #[inline]
    pub fn extendee(&self) -> Result<&'a str, DecodeError> {
        if !self.extendee.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.extendee)
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
    pub const fn has_label(&self) -> bool {
        self.label.is_present()
    }

    #[inline]
    pub fn label(&self) -> Result<field_descriptor_proto::Label, DecodeError> {
        if !self.label.is_present() {
            return Ok(field_descriptor_proto::Label::LabelOptional);
        }
        let raw = decode_varint_at(self.buf, self.label)?;
        Ok(field_descriptor_proto::Label::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_type(&self) -> bool {
        self.r#type.is_present()
    }

    #[inline]
    pub fn r#type(&self) -> Result<field_descriptor_proto::Type, DecodeError> {
        if !self.r#type.is_present() {
            return Ok(field_descriptor_proto::Type::TypeDouble);
        }
        let raw = decode_varint_at(self.buf, self.r#type)?;
        Ok(field_descriptor_proto::Type::from_raw_varint(raw))
    }

    #[inline]
    pub const fn has_type_name(&self) -> bool {
        self.type_name.is_present()
    }

    #[inline]
    pub fn type_name(&self) -> Result<&'a str, DecodeError> {
        if !self.type_name.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.type_name)
    }

    #[inline]
    pub const fn has_default_value(&self) -> bool {
        self.default_value.is_present()
    }

    #[inline]
    pub fn default_value(&self) -> Result<&'a str, DecodeError> {
        if !self.default_value.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.default_value)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<FieldOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        FieldOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_oneof_index(&self) -> bool {
        self.oneof_index.is_present()
    }

    #[inline]
    pub fn oneof_index(&self) -> Result<i32, DecodeError> {
        if !self.oneof_index.is_present() {
            return Ok(0);
        }
        Ok(decode_varint_at(self.buf, self.oneof_index)? as i32)
    }

    #[inline]
    pub const fn has_json_name(&self) -> bool {
        self.json_name.is_present()
    }

    #[inline]
    pub fn json_name(&self) -> Result<&'a str, DecodeError> {
        if !self.json_name.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.json_name)
    }

    #[inline]
    pub const fn has_proto3_optional(&self) -> bool {
        self.proto3_optional.is_present()
    }

    #[inline]
    pub fn proto3_optional(&self) -> Result<bool, DecodeError> {
        if !self.proto3_optional.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.proto3_optional)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for FieldDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct FieldDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> FieldDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &FieldDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_extendee(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_number(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(3, WireType::Varint)
    }

    #[inline]
    pub fn write_label(&mut self, val: field_descriptor_proto::Label) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(4, WireType::Varint)
    }

    #[inline]
    pub fn write_type(&mut self, val: field_descriptor_proto::Type) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(5, WireType::Varint)
    }

    #[inline]
    pub fn write_type_name(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(6, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_default_value(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(7, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut FieldOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = FieldOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(8, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_oneof_index(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(9, WireType::Varint)
    }

    #[inline]
    pub fn write_json_name(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(10, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_proto3_optional(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(17, WireType::Varint)
    }
}

// ============================================================================
// OneofDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct OneofDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    options: FieldMarker,
}

impl<'a> OneofDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut options = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (2, WireType::LengthDelimited) => options = field.marker,
                _ => {}
            }
        }

        Ok(Self { buf, name, options })
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
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<OneofOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        OneofOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for OneofDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct OneofDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> OneofDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &OneofDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut OneofOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = OneofOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }
}

// ============================================================================
// EnumDescriptorProto (Reader View & Backward Builder)
// ============================================================================

pub mod enum_descriptor_proto {
    use super::*;

    #[derive(Clone, Copy, Debug)]
    pub struct EnumReservedRangeView<'a> {
        buf: &'a [u8],
        start: FieldMarker,
        end: FieldMarker,
    }

    impl<'a> EnumReservedRangeView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            let mut start = FieldMarker::none();
            let mut end = FieldMarker::none();

            for item in FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, WireType::Varint) => start = field.marker,
                    (2, WireType::Varint) => end = field.marker,
                    _ => {}
                }
            }

            Ok(Self { buf, start, end })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_start(&self) -> bool {
            self.start.is_present()
        }

        #[inline]
        pub fn start(&self) -> Result<i32, DecodeError> {
            if !self.start.is_present() {
                return Ok(0);
            }
            Ok(decode_varint_at(self.buf, self.start)? as i32)
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
        pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
            UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> MessageView<'a> for EnumReservedRangeView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct EnumReservedRangeBuilder<'a, 'b> {
        pub writer: &'a mut BackwardWriter<'b>,
    }

    impl<'a, 'b> EnumReservedRangeBuilder<'a, 'b> {
        #[inline]
        pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
            Self { writer }
        }

        #[inline]
        pub fn copy_unknown_fields(
            &mut self,
            view: &EnumReservedRangeView<'_>,
        ) -> Result<(), EncodeError> {
            for raw_field in view.unknown_fields() {
                self.writer.write_bytes(raw_field)?;
            }
            Ok(())
        }

        #[inline]
        pub fn write_start(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(1, WireType::Varint)
        }

        #[inline]
        pub fn write_end(&mut self, val: i32) -> Result<(), EncodeError> {
            self.writer.write_varint(val as u64)?;
            self.writer.write_tag(2, WireType::Varint)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct EnumDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    options: FieldMarker,
    visibility: FieldMarker,
}

impl<'a> EnumDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut options = FieldMarker::none();
        let mut visibility = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (3, WireType::LengthDelimited) => options = field.marker,
                (6, WireType::Varint) => visibility = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            options,
            visibility,
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
    pub const fn value(&self) -> RepeatedMessageIter<'a, EnumValueDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 2)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<EnumOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        EnumOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn reserved_range(
        &self,
    ) -> RepeatedMessageIter<'a, enum_descriptor_proto::EnumReservedRangeView<'a>> {
        RepeatedMessageIter::new(self.buf, 4)
    }

    #[inline]
    pub const fn reserved_name(&self) -> RepeatedStrIter<'a> {
        RepeatedStrIter::new(self.buf, 5)
    }

    #[inline]
    pub const fn has_visibility(&self) -> bool {
        self.visibility.is_present()
    }

    #[inline]
    pub fn visibility(&self) -> Result<SymbolVisibility, DecodeError> {
        if !self.visibility.is_present() {
            return Ok(SymbolVisibility::VisibilityUnset);
        }
        let raw = decode_varint_at(self.buf, self.visibility)?;
        Ok(SymbolVisibility::from_raw_varint(raw))
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for EnumDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct EnumDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> EnumDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &EnumDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_value<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut EnumValueDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = EnumValueDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut EnumOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = EnumOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_reserved_range<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(
            &mut enum_descriptor_proto::EnumReservedRangeBuilder<'_, 'b>,
        ) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = enum_descriptor_proto::EnumReservedRangeBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_reserved_name(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(5, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_visibility(&mut self, val: SymbolVisibility) -> Result<(), EncodeError> {
        self.writer.write_varint(val.as_i32() as u64)?;
        self.writer.write_tag(6, WireType::Varint)
    }
}

// ============================================================================
// EnumValueDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct EnumValueDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    number: FieldMarker,
    options: FieldMarker,
}

impl<'a> EnumValueDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut number = FieldMarker::none();
        let mut options = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (2, WireType::Varint) => number = field.marker,
                (3, WireType::LengthDelimited) => options = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            number,
            options,
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
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<EnumValueOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        EnumValueOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for EnumValueDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct EnumValueDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> EnumValueDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &EnumValueDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_number(&mut self, val: i32) -> Result<(), EncodeError> {
        self.writer.write_varint(val as u64)?;
        self.writer.write_tag(2, WireType::Varint)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut EnumValueOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = EnumValueOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }
}

// ============================================================================
// ServiceDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct ServiceDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    options: FieldMarker,
}

impl<'a> ServiceDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut options = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (3, WireType::LengthDelimited) => options = field.marker,
                _ => {}
            }
        }

        Ok(Self { buf, name, options })
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
    pub const fn method(&self) -> RepeatedMessageIter<'a, MethodDescriptorProtoView<'a>> {
        RepeatedMessageIter::new(self.buf, 2)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<ServiceOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        ServiceOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for ServiceDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ServiceDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> ServiceDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &ServiceDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_method<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut MethodDescriptorProtoBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = MethodDescriptorProtoBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut ServiceOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = ServiceOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }
}

// ============================================================================
// MethodDescriptorProto (Reader View & Backward Builder)
// ============================================================================

#[derive(Clone, Copy, Debug)]
pub struct MethodDescriptorProtoView<'a> {
    buf: &'a [u8],
    name: FieldMarker,
    input_type: FieldMarker,
    output_type: FieldMarker,
    options: FieldMarker,
    client_streaming: FieldMarker,
    server_streaming: FieldMarker,
}

impl<'a> MethodDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        let mut name = FieldMarker::none();
        let mut input_type = FieldMarker::none();
        let mut output_type = FieldMarker::none();
        let mut options = FieldMarker::none();
        let mut client_streaming = FieldMarker::none();
        let mut server_streaming = FieldMarker::none();

        for item in FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, WireType::LengthDelimited) => name = field.marker,
                (2, WireType::LengthDelimited) => input_type = field.marker,
                (3, WireType::LengthDelimited) => output_type = field.marker,
                (4, WireType::LengthDelimited) => options = field.marker,
                (5, WireType::Varint) => client_streaming = field.marker,
                (6, WireType::Varint) => server_streaming = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            name,
            input_type,
            output_type,
            options,
            client_streaming,
            server_streaming,
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
    pub const fn has_input_type(&self) -> bool {
        self.input_type.is_present()
    }

    #[inline]
    pub fn input_type(&self) -> Result<&'a str, DecodeError> {
        if !self.input_type.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.input_type)
    }

    #[inline]
    pub const fn has_output_type(&self) -> bool {
        self.output_type.is_present()
    }

    #[inline]
    pub fn output_type(&self) -> Result<&'a str, DecodeError> {
        if !self.output_type.is_present() {
            return Ok("");
        }
        decode_str_at(self.buf, self.output_type)
    }

    #[inline]
    pub const fn has_options(&self) -> bool {
        self.options.is_present()
    }

    #[inline]
    pub fn options(&self) -> Result<Option<MethodOptionsView<'a>>, DecodeError> {
        if !self.options.is_present() {
            return Ok(None);
        }
        MethodOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_client_streaming(&self) -> bool {
        self.client_streaming.is_present()
    }

    #[inline]
    pub fn client_streaming(&self) -> Result<bool, DecodeError> {
        if !self.client_streaming.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.client_streaming)
    }

    #[inline]
    pub const fn has_server_streaming(&self) -> bool {
        self.server_streaming.is_present()
    }

    #[inline]
    pub fn server_streaming(&self) -> Result<bool, DecodeError> {
        if !self.server_streaming.is_present() {
            return Ok(false);
        }
        decode_bool_at(self.buf, self.server_streaming)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
        UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> MessageView<'a> for MethodDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct MethodDescriptorProtoBuilder<'a, 'b> {
    pub writer: &'a mut BackwardWriter<'b>,
}

impl<'a, 'b> MethodDescriptorProtoBuilder<'a, 'b> {
    #[inline]
    pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
        Self { writer }
    }

    #[inline]
    pub fn copy_unknown_fields(
        &mut self,
        view: &MethodDescriptorProtoView<'_>,
    ) -> Result<(), EncodeError> {
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
    pub fn write_input_type(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(2, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_output_type(&mut self, val: &str) -> Result<(), EncodeError> {
        self.writer.write_length_delimited(val.as_bytes())?;
        self.writer.write_tag(3, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_options<F>(&mut self, f: F) -> Result<(), EncodeError>
    where
        F: FnOnce(&mut MethodOptionsBuilder<'_, 'b>) -> Result<(), EncodeError>,
    {
        let end_pos = self.writer.cursor();
        {
            let mut sub_builder = MethodOptionsBuilder::new(self.writer);
            f(&mut sub_builder)?;
        }
        let nested_len = end_pos - self.writer.cursor();
        self.writer.write_varint(nested_len as u64)?;
        self.writer.write_tag(4, WireType::LengthDelimited)
    }

    #[inline]
    pub fn write_client_streaming(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(5, WireType::Varint)
    }

    #[inline]
    pub fn write_server_streaming(&mut self, val: bool) -> Result<(), EncodeError> {
        self.writer.write_bool(val)?;
        self.writer.write_tag(6, WireType::Varint)
    }
}

// ============================================================================
// Options & Metadata Views and Builders (Expanded in Issue #007)
// ============================================================================

macro_rules! define_minimal_options_view_and_builder {
    ($view:ident, $builder:ident) => {
        #[derive(Clone, Copy, Debug)]
        pub struct $view<'a> {
            buf: &'a [u8],
        }

        impl<'a> $view<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[];

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
            pub const fn unknown_fields(&self) -> UnknownFieldsIter<'a> {
                UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> MessageView<'a> for $view<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct $builder<'a, 'b> {
            pub writer: &'a mut BackwardWriter<'b>,
        }

        impl<'a, 'b> $builder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &$view<'_>) -> Result<(), EncodeError> {
                for raw_field in view.unknown_fields() {
                    self.writer.write_bytes(raw_field)?;
                }
                Ok(())
            }
        }
    };
}

define_minimal_options_view_and_builder!(FileOptionsView, FileOptionsBuilder);
define_minimal_options_view_and_builder!(MessageOptionsView, MessageOptionsBuilder);
define_minimal_options_view_and_builder!(FieldOptionsView, FieldOptionsBuilder);
define_minimal_options_view_and_builder!(OneofOptionsView, OneofOptionsBuilder);
define_minimal_options_view_and_builder!(EnumOptionsView, EnumOptionsBuilder);
define_minimal_options_view_and_builder!(EnumValueOptionsView, EnumValueOptionsBuilder);
define_minimal_options_view_and_builder!(ServiceOptionsView, ServiceOptionsBuilder);
define_minimal_options_view_and_builder!(MethodOptionsView, MethodOptionsBuilder);
define_minimal_options_view_and_builder!(ExtensionRangeOptionsView, ExtensionRangeOptionsBuilder);
define_minimal_options_view_and_builder!(SourceCodeInfoView, SourceCodeInfoBuilder);

#[cfg(test)]
mod tests {
    use super::*;
    use field_descriptor_proto::{Label, Type};

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
    }

    #[test]
    fn full_descriptor_roundtrip_and_unknown_fields() {
        let mut backing = [0u8; 1024];
        let mut writer = BackwardWriter::new(&mut backing);
        {
            // Unknown field 99 on FileDescriptorSet
            writer.write_varint(12345).unwrap();
            writer.write_tag(99, WireType::Varint).unwrap();

            let mut fds = FileDescriptorSetBuilder::new(&mut writer);
            fds.write_file(|file| {
                // Unknown field 88 on FileDescriptorProto
                file.writer.write_varint(777).unwrap();
                file.writer.write_tag(88, WireType::Varint).unwrap();

                file.write_edition(Edition::Edition2024)?;
                file.write_syntax("editions")?;
                file.write_source_code_info(|sci| {
                    sci.writer.write_varint(1)?;
                    sci.writer.write_tag(1, WireType::Varint)
                })?;
                file.write_options(|opt| {
                    opt.writer.write_varint(1)?;
                    opt.writer.write_tag(1, WireType::Varint)
                })?;
                file.write_extension(|ext| {
                    ext.write_type(Type::TypeString)?;
                    ext.write_label(Label::LabelOptional)?;
                    ext.write_number(1000)?;
                    ext.write_extendee(".google.protobuf.MessageOptions")?;
                    ext.write_name("custom_ext")
                })?;
                file.write_service(|svc| {
                    svc.write_options(|opt| {
                        opt.writer.write_varint(1)?;
                        opt.writer.write_tag(33, WireType::Varint)
                    })?;
                    svc.write_method(|m| {
                        m.write_server_streaming(true)?;
                        m.write_client_streaming(false)?;
                        m.write_options(|opt| {
                            opt.writer.write_varint(1)?;
                            opt.writer.write_tag(33, WireType::Varint)
                        })?;
                        m.write_output_type(".demo.SensorReading")?;
                        m.write_input_type(".demo.Location")?;
                        m.write_name("ReadStream")
                    })?;
                    svc.write_name("TelemetryService")
                })?;
                file.write_enum_type(|en| {
                    en.write_visibility(SymbolVisibility::VisibilityExport)?;
                    en.write_reserved_name("OLD_STATUS")?;
                    en.write_reserved_range(|rr| {
                        rr.write_end(20)?;
                        rr.write_start(10)
                    })?;
                    en.write_options(|opt| {
                        opt.writer.write_varint(1)?;
                        opt.writer.write_tag(2, WireType::Varint)
                    })?;
                    en.write_value(|val| {
                        val.write_options(|opt| {
                            opt.writer.write_varint(1)?;
                            opt.writer.write_tag(1, WireType::Varint)
                        })?;
                        val.write_number(0)?;
                        val.write_name("UNKNOWN")
                    })?;
                    en.write_name("DeviceState")
                })?;
                file.write_message_type(|msg| {
                    msg.write_visibility(SymbolVisibility::VisibilityLocal)?;
                    msg.write_reserved_name("legacy_field")?;
                    msg.write_reserved_range(|rr| {
                        rr.write_end(200)?;
                        rr.write_start(100)
                    })?;
                    msg.write_oneof_decl(|oo| {
                        oo.write_options(|opt| {
                            opt.writer.write_varint(1)?;
                            opt.writer.write_tag(1, WireType::Varint)
                        })?;
                        oo.write_name("payload")
                    })?;
                    msg.write_options(|opt| {
                        opt.writer.write_varint(1)?;
                        opt.writer.write_tag(7, WireType::Varint)
                    })?;
                    msg.write_extension(|ext| {
                        ext.write_type(Type::TypeInt32)?;
                        ext.write_number(500)?;
                        ext.write_extendee(".demo.SensorReading")?;
                        ext.write_name("nested_ext")
                    })?;
                    msg.write_extension_range(|er| {
                        er.write_options(|opt| {
                            opt.writer.write_varint(1)?;
                            opt.writer.write_tag(3, WireType::Varint)
                        })?;
                        er.write_end(1000)?;
                        er.write_start(500)
                    })?;
                    msg.write_enum_type(|nes_en| nes_en.write_name("NestedEnum"))?;
                    msg.write_nested_type(|nes_msg| nes_msg.write_name("NestedMsg"))?;
                    msg.write_field(|fld| {
                        fld.write_proto3_optional(true)?;
                        fld.write_json_name("deviceId")?;
                        fld.write_oneof_index(0)?;
                        fld.write_options(|opt| {
                            opt.writer.write_varint(1)?;
                            opt.writer.write_tag(2, WireType::Varint)
                        })?;
                        fld.write_default_value("unknown")?;
                        fld.write_type_name(".demo.DeviceState")?;
                        fld.write_type(Type::TypeEnum)?;
                        fld.write_label(Label::LabelOptional)?;
                        fld.write_number(2)?;
                        fld.write_name("device_id")
                    })?;
                    msg.write_name("SensorReading")
                })?;
                file.write_option_dependency("custom_options.proto")?;
                file.write_weak_dependency(1)?;
                file.write_public_dependency(0)?;
                file.write_dependency("common.proto")?;
                file.write_package("demo")?;
                file.write_name("sensor.proto")
            })
            .unwrap();
        }
        let wire = writer.finish();

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
        assert!(file.options().unwrap().is_some());
        assert!(file.has_source_code_info());
        assert!(file.source_code_info().unwrap().is_some());

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
        assert!(msg.options().unwrap().is_some());

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
        assert!(fld.options().unwrap().is_some());

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
        assert!(er.options().unwrap().is_some());

        let mut msg_exts = msg.extension();
        let me = msg_exts.next().unwrap().unwrap();
        assert_eq!(me.name(), Ok("nested_ext"));
        assert_eq!(me.extendee(), Ok(".demo.SensorReading"));
        assert_eq!(me.number(), Ok(500));

        let mut oneofs = msg.oneof_decl();
        let oo = oneofs.next().unwrap().unwrap();
        assert_eq!(oo.name(), Ok("payload"));
        assert!(oo.options().unwrap().is_some());

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
        assert!(en.options().unwrap().is_some());
        let mut evs = en.value();
        let ev = evs.next().unwrap().unwrap();
        assert_eq!(ev.name(), Ok("UNKNOWN"));
        assert_eq!(ev.number(), Ok(0));
        assert!(ev.options().unwrap().is_some());
        let mut er_iter = en.reserved_range();
        let err = er_iter.next().unwrap().unwrap();
        assert_eq!(err.start(), Ok(10));
        assert_eq!(err.end(), Ok(20));
        let mut en_rn = en.reserved_name();
        assert_eq!(en_rn.next(), Some(Ok("OLD_STATUS")));

        let mut svcs = file.service();
        let svc = svcs.next().unwrap().unwrap();
        assert_eq!(svc.name(), Ok("TelemetryService"));
        assert!(svc.options().unwrap().is_some());
        let mut methods = svc.method();
        let m = methods.next().unwrap().unwrap();
        assert_eq!(m.name(), Ok("ReadStream"));
        assert_eq!(m.input_type(), Ok(".demo.Location"));
        assert_eq!(m.output_type(), Ok(".demo.SensorReading"));
        assert_eq!(m.client_streaming(), Ok(false));
        assert_eq!(m.server_streaming(), Ok(true));
        assert!(m.options().unwrap().is_some());

        let mut file_exts = file.extension();
        let fe = file_exts.next().unwrap().unwrap();
        assert_eq!(fe.name(), Ok("custom_ext"));
        assert_eq!(fe.extendee(), Ok(".google.protobuf.MessageOptions"));
        assert_eq!(fe.number(), Ok(1000));
    }
}
