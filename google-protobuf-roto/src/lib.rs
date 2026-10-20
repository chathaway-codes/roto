#![no_std]
#![forbid(unsafe_code)]
#![allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]

pub mod google {
    pub mod protobuf {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum Edition {
            EditionUnknown = 0,
            EditionLegacy = 900,
            EditionProto2 = 998,
            EditionProto3 = 999,
            Edition2023 = 1000,
            Edition2024 = 1001,
            Edition2026 = 1002,
            EditionUnstable = 9999,
            Edition1TestOnly = 1,
            Edition2TestOnly = 2,
            Edition99997TestOnly = 99997,
            Edition99998TestOnly = 99998,
            Edition99999TestOnly = 99999,
            EditionMax = 2147483647,
        }

        impl Edition {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::EditionUnknown),
                    900 => Some(Self::EditionLegacy),
                    998 => Some(Self::EditionProto2),
                    999 => Some(Self::EditionProto3),
                    1000 => Some(Self::Edition2023),
                    1001 => Some(Self::Edition2024),
                    1002 => Some(Self::Edition2026),
                    9999 => Some(Self::EditionUnstable),
                    1 => Some(Self::Edition1TestOnly),
                    2 => Some(Self::Edition2TestOnly),
                    99997 => Some(Self::Edition99997TestOnly),
                    99998 => Some(Self::Edition99998TestOnly),
                    99999 => Some(Self::Edition99999TestOnly),
                    2147483647 => Some(Self::EditionMax),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for Edition {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for Edition {
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

        impl core::convert::TryFrom<i32> for SymbolVisibility {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for SymbolVisibility {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::VisibilityUnset)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FileDescriptorSetView<'a> {
            buf: &'a [u8],
        }

        impl<'a> FileDescriptorSetView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

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
            pub const fn file(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FileDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FileDescriptorSetView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FileDescriptorSetBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FileDescriptorSetBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FileDescriptorSetView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_file<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FileDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            package: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            source_code_info: ::roto::runtime::FieldMarker,
            syntax: ::roto::runtime::FieldMarker,
            edition: ::roto::runtime::FieldMarker,
        }

        impl<'a> FileDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 10, 11, 15, 4, 5, 6, 7, 8, 9, 12, 14];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut package = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut source_code_info = ::roto::runtime::FieldMarker::none();
                let mut syntax = ::roto::runtime::FieldMarker::none();
                let mut edition = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => package = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (9, ::roto::runtime::WireType::LengthDelimited) => source_code_info = field.marker,
                        (12, ::roto::runtime::WireType::LengthDelimited) => syntax = field.marker,
                        (14, ::roto::runtime::WireType::Varint) => edition = field.marker,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn has_package(&self) -> bool {
                self.package.is_present()
            }

            #[inline]
            pub fn package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.package.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.package)
            }

            #[inline]
            pub const fn dependency(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.buf, 3)
            }

            #[inline]
            pub const fn public_dependency(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                ::roto::runtime::RepeatedVarintIter::new(self.buf, 10)
            }

            #[inline]
            pub const fn weak_dependency(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                ::roto::runtime::RepeatedVarintIter::new(self.buf, 11)
            }

            #[inline]
            pub const fn option_dependency(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.buf, 15)
            }

            #[inline]
            pub const fn message_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::DescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 4)
            }

            #[inline]
            pub const fn enum_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 5)
            }

            #[inline]
            pub const fn service(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::ServiceDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 6)
            }

            #[inline]
            pub const fn extension(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 7)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::FileOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FileOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_source_code_info(&self) -> bool {
                self.source_code_info.is_present()
            }

            #[inline]
            pub fn source_code_info(&self) -> Result<Option<crate::google::protobuf::SourceCodeInfoView<'a>>, ::roto::runtime::DecodeError> {
                if !self.source_code_info.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::SourceCodeInfoView::from_bytes(self.source_code_info.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_syntax(&self) -> bool {
                self.syntax.is_present()
            }

            #[inline]
            pub fn syntax(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.syntax.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.syntax)
            }

            #[inline]
            pub const fn has_edition(&self) -> bool {
                self.edition.is_present()
            }

            #[inline]
            pub fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                if !self.edition.is_present() {
                    return Ok(crate::google::protobuf::Edition::EditionUnknown);
                }
                Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FileDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FileDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FileDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FileDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_package(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_dependency(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_public_dependency(&mut self, val: i32) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed_public_dependency(&mut self, vals: &[i32]) -> &mut Self {
                let end_pos = self.writer.cursor();
                for &val in vals.iter().rev() {
                    let _ = self.writer.write_varint(val as u64);
                }
                let packed_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(packed_len as u64);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_weak_dependency(&mut self, val: i32) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed_weak_dependency(&mut self, vals: &[i32]) -> &mut Self {
                let end_pos = self.writer.cursor();
                for &val in vals.iter().rev() {
                    let _ = self.writer.write_varint(val as u64);
                }
                let packed_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(packed_len as u64);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option_dependency(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_message_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::DescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::DescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_enum_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::EnumDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_service<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ServiceDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::ServiceDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FileOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FileOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_source_code_info<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::SourceCodeInfoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::SourceCodeInfoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_syntax(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct DescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            visibility: ::roto::runtime::FieldMarker,
        }

        impl<'a> DescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 6, 3, 4, 5, 8, 7, 9, 10, 11];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut visibility = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (7, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (11, ::roto::runtime::WireType::Varint) => visibility = field.marker,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn field(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 2)
            }

            #[inline]
            pub const fn extension(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 6)
            }

            #[inline]
            pub const fn nested_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::DescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 3)
            }

            #[inline]
            pub const fn enum_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 4)
            }

            #[inline]
            pub const fn extension_range(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::descriptor_proto::ExtensionRangeView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 5)
            }

            #[inline]
            pub const fn oneof_decl(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OneofDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 8)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::MessageOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::MessageOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn reserved_range(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::descriptor_proto::ReservedRangeView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 9)
            }

            #[inline]
            pub const fn reserved_name(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.buf, 10)
            }

            #[inline]
            pub const fn has_visibility(&self) -> bool {
                self.visibility.is_present()
            }

            #[inline]
            pub fn visibility(&self) -> Result<crate::google::protobuf::SymbolVisibility, ::roto::runtime::DecodeError> {
                if !self.visibility.is_present() {
                    return Ok(crate::google::protobuf::SymbolVisibility::VisibilityUnset);
                }
                Ok(<crate::google::protobuf::SymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.visibility)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for DescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct DescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> DescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &DescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_nested_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::DescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::DescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_enum_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::EnumDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::descriptor_proto::ExtensionRangeBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::descriptor_proto::ExtensionRangeBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof_decl<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OneofDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::OneofDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MessageOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::MessageOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::descriptor_proto::ReservedRangeBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::descriptor_proto::ReservedRangeBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_visibility(&mut self, val: crate::google::protobuf::SymbolVisibility) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod descriptor_proto {
            #[derive(Clone, Copy, Debug)]
            pub struct ExtensionRangeView<'a> {
                buf: &'a [u8],
                start: ::roto::runtime::FieldMarker,
                end: ::roto::runtime::FieldMarker,
                options: ::roto::runtime::FieldMarker,
            }

            impl<'a> ExtensionRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start = ::roto::runtime::FieldMarker::none();
                    let mut end = ::roto::runtime::FieldMarker::none();
                    let mut options = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => start = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => end = field.marker,
                            (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
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
                pub fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.start.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.start)? as i32)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    self.end.is_present()
                }

                #[inline]
                pub fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.end.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.end)? as i32)
                }

                #[inline]
                pub const fn has_options(&self) -> bool {
                    self.options.is_present()
                }

                #[inline]
                pub fn options(&self) -> Result<Option<crate::google::protobuf::ExtensionRangeOptionsView<'a>>, ::roto::runtime::DecodeError> {
                    if !self.options.is_present() {
                        return Ok(None);
                    }
                    crate::google::protobuf::ExtensionRangeOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for ExtensionRangeView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct ExtensionRangeBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> ExtensionRangeBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &ExtensionRangeView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_options<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::ExtensionRangeOptionsBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::ExtensionRangeOptionsBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct ReservedRangeView<'a> {
                buf: &'a [u8],
                start: ::roto::runtime::FieldMarker,
                end: ::roto::runtime::FieldMarker,
            }

            impl<'a> ReservedRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start = ::roto::runtime::FieldMarker::none();
                    let mut end = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => start = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => end = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        start,
                        end,
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
                pub fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.start.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.start)? as i32)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    self.end.is_present()
                }

                #[inline]
                pub fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.end.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.end)? as i32)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for ReservedRangeView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct ReservedRangeBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> ReservedRangeBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &ReservedRangeView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ExtensionRangeOptionsView<'a> {
            buf: &'a [u8],
            features: ::roto::runtime::FieldMarker,
            verification: ::roto::runtime::FieldMarker,
        }

        impl<'a> ExtensionRangeOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[999, 2, 50, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut verification = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (50, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => verification = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    features,
                    verification,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn declaration(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::extension_range_options::DeclarationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 2)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_verification(&self) -> bool {
                self.verification.is_present()
            }

            #[inline]
            pub fn verification(&self) -> Result<crate::google::protobuf::extension_range_options::VerificationState, ::roto::runtime::DecodeError> {
                if !self.verification.is_present() {
                    return Ok(crate::google::protobuf::extension_range_options::VerificationState::Unverified);
                }
                Ok(<crate::google::protobuf::extension_range_options::VerificationState as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.verification)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ExtensionRangeOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ExtensionRangeOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> ExtensionRangeOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &ExtensionRangeOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_declaration<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::extension_range_options::DeclarationBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::extension_range_options::DeclarationBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(50, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_verification(&mut self, val: crate::google::protobuf::extension_range_options::VerificationState) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod extension_range_options {
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

            impl core::convert::TryFrom<i32> for VerificationState {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for VerificationState {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::Declaration)
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct DeclarationView<'a> {
                buf: &'a [u8],
                number: ::roto::runtime::FieldMarker,
                full_name: ::roto::runtime::FieldMarker,
                r#type: ::roto::runtime::FieldMarker,
                reserved: ::roto::runtime::FieldMarker,
                repeated: ::roto::runtime::FieldMarker,
            }

            impl<'a> DeclarationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 5, 6];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut number = ::roto::runtime::FieldMarker::none();
                    let mut full_name = ::roto::runtime::FieldMarker::none();
                    let mut r#type = ::roto::runtime::FieldMarker::none();
                    let mut reserved = ::roto::runtime::FieldMarker::none();
                    let mut repeated = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => number = field.marker,
                            (2, ::roto::runtime::WireType::LengthDelimited) => full_name = field.marker,
                            (3, ::roto::runtime::WireType::LengthDelimited) => r#type = field.marker,
                            (5, ::roto::runtime::WireType::Varint) => reserved = field.marker,
                            (6, ::roto::runtime::WireType::Varint) => repeated = field.marker,
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
                pub fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.number.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.number)? as i32)
                }

                #[inline]
                pub const fn has_full_name(&self) -> bool {
                    self.full_name.is_present()
                }

                #[inline]
                pub fn full_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.full_name.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.full_name)
                }

                #[inline]
                pub const fn has_type(&self) -> bool {
                    self.r#type.is_present()
                }

                #[inline]
                pub fn r#type(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.r#type.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.r#type)
                }

                #[inline]
                pub const fn has_reserved(&self) -> bool {
                    self.reserved.is_present()
                }

                #[inline]
                pub fn reserved(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    if !self.reserved.is_present() {
                        return Ok(false);
                    }
                    ::roto::runtime::decode_bool_at(self.buf, self.reserved)
                }

                #[inline]
                pub const fn has_repeated(&self) -> bool {
                    self.repeated.is_present()
                }

                #[inline]
                pub fn repeated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    if !self.repeated.is_present() {
                        return Ok(false);
                    }
                    ::roto::runtime::decode_bool_at(self.buf, self.repeated)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for DeclarationView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct DeclarationBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> DeclarationBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &DeclarationView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_number(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_full_name(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_type(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_reserved(&mut self, val: bool) -> &mut Self {
                    let _ = self.writer.write_bool(val);
                    let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_repeated(&mut self, val: bool) -> &mut Self {
                    let _ = self.writer.write_bool(val);
                    let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FieldDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            number: ::roto::runtime::FieldMarker,
            label: ::roto::runtime::FieldMarker,
            r#type: ::roto::runtime::FieldMarker,
            type_name: ::roto::runtime::FieldMarker,
            extendee: ::roto::runtime::FieldMarker,
            default_value: ::roto::runtime::FieldMarker,
            f_9: ::roto::runtime::FieldMarker,
            json_name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            proto3_optional: ::roto::runtime::FieldMarker,
        }

        impl<'a> FieldDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 3, 4, 5, 6, 2, 7, 9, 10, 8, 17];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut number = ::roto::runtime::FieldMarker::none();
                let mut label = ::roto::runtime::FieldMarker::none();
                let mut r#type = ::roto::runtime::FieldMarker::none();
                let mut type_name = ::roto::runtime::FieldMarker::none();
                let mut extendee = ::roto::runtime::FieldMarker::none();
                let mut default_value = ::roto::runtime::FieldMarker::none();
                let mut f_9 = ::roto::runtime::FieldMarker::none();
                let mut json_name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut proto3_optional = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => number = field.marker,
                        (4, ::roto::runtime::WireType::Varint) => label = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => r#type = field.marker,
                        (6, ::roto::runtime::WireType::LengthDelimited) => type_name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => extendee = field.marker,
                        (7, ::roto::runtime::WireType::LengthDelimited) => default_value = field.marker,
                        (9, ::roto::runtime::WireType::Varint) => f_9 = field.marker,
                        (10, ::roto::runtime::WireType::LengthDelimited) => json_name = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (17, ::roto::runtime::WireType::Varint) => proto3_optional = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    number,
                    label,
                    r#type,
                    type_name,
                    extendee,
                    default_value,
                    f_9,
                    json_name,
                    options,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn has_number(&self) -> bool {
                self.number.is_present()
            }

            #[inline]
            pub fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                if !self.number.is_present() {
                    return Ok(0i32);
                }
                Ok(::roto::runtime::decode_varint_at(self.buf, self.number)? as i32)
            }

            #[inline]
            pub const fn has_label(&self) -> bool {
                self.label.is_present()
            }

            #[inline]
            pub fn label(&self) -> Result<crate::google::protobuf::field_descriptor_proto::Label, ::roto::runtime::DecodeError> {
                if !self.label.is_present() {
                    return Ok(crate::google::protobuf::field_descriptor_proto::Label::LabelOptional);
                }
                Ok(<crate::google::protobuf::field_descriptor_proto::Label as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.label)?))
            }

            #[inline]
            pub const fn has_type(&self) -> bool {
                self.r#type.is_present()
            }

            #[inline]
            pub fn r#type(&self) -> Result<crate::google::protobuf::field_descriptor_proto::Type, ::roto::runtime::DecodeError> {
                if !self.r#type.is_present() {
                    return Ok(crate::google::protobuf::field_descriptor_proto::Type::TypeDouble);
                }
                Ok(<crate::google::protobuf::field_descriptor_proto::Type as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.r#type)?))
            }

            #[inline]
            pub const fn has_type_name(&self) -> bool {
                self.type_name.is_present()
            }

            #[inline]
            pub fn type_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.type_name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.type_name)
            }

            #[inline]
            pub const fn has_extendee(&self) -> bool {
                self.extendee.is_present()
            }

            #[inline]
            pub fn extendee(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.extendee.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.extendee)
            }

            #[inline]
            pub const fn has_default_value(&self) -> bool {
                self.default_value.is_present()
            }

            #[inline]
            pub fn default_value(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.default_value.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.default_value)
            }

            #[inline]
            pub const fn has_oneof_index(&self) -> bool {
                self.f_9.is_present()
            }

            #[inline]
            pub fn oneof_index(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                if !self.f_9.is_present() {
                    return Ok(0i32);
                }
                Ok(::roto::runtime::decode_varint_at(self.buf, self.f_9)? as i32)
            }

            #[inline]
            pub const fn has_json_name(&self) -> bool {
                self.json_name.is_present()
            }

            #[inline]
            pub fn json_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.json_name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.json_name)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::FieldOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FieldOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_proto3_optional(&self) -> bool {
                self.proto3_optional.is_present()
            }

            #[inline]
            pub fn proto3_optional(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.proto3_optional.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.proto3_optional)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FieldDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FieldDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FieldDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FieldDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_label(&mut self, val: crate::google::protobuf::field_descriptor_proto::Label) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_type(&mut self, val: crate::google::protobuf::field_descriptor_proto::Type) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_type_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extendee(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_default_value(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof_index(&mut self, val: i32) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_json_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FieldOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_proto3_optional(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod field_descriptor_proto {
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

            impl core::convert::TryFrom<i32> for Type {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for Type {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::TypeDouble)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum Label {
                LabelOptional = 1,
                LabelRepeated = 3,
                LabelRequired = 2,
            }

            impl Label {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        1 => Some(Self::LabelOptional),
                        3 => Some(Self::LabelRepeated),
                        2 => Some(Self::LabelRequired),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for Label {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for Label {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::LabelOptional)
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct OneofDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
        }

        impl<'a> OneofDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::OneofOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::OneofOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for OneofDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct OneofDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> OneofDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &OneofDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OneofOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::OneofOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            visibility: ::roto::runtime::FieldMarker,
        }

        impl<'a> EnumDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut visibility = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => visibility = field.marker,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn value(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumValueDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 2)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::EnumOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::EnumOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn reserved_range(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::enum_descriptor_proto::EnumReservedRangeView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 4)
            }

            #[inline]
            pub const fn reserved_name(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.buf, 5)
            }

            #[inline]
            pub const fn has_visibility(&self) -> bool {
                self.visibility.is_present()
            }

            #[inline]
            pub fn visibility(&self) -> Result<crate::google::protobuf::SymbolVisibility, ::roto::runtime::DecodeError> {
                if !self.visibility.is_present() {
                    return Ok(crate::google::protobuf::SymbolVisibility::VisibilityUnset);
                }
                Ok(<crate::google::protobuf::SymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.visibility)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> EnumDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &EnumDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumValueDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::EnumValueDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::EnumOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::enum_descriptor_proto::EnumReservedRangeBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::enum_descriptor_proto::EnumReservedRangeBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_visibility(&mut self, val: crate::google::protobuf::SymbolVisibility) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod enum_descriptor_proto {
            #[derive(Clone, Copy, Debug)]
            pub struct EnumReservedRangeView<'a> {
                buf: &'a [u8],
                start: ::roto::runtime::FieldMarker,
                end: ::roto::runtime::FieldMarker,
            }

            impl<'a> EnumReservedRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start = ::roto::runtime::FieldMarker::none();
                    let mut end = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => start = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => end = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        start,
                        end,
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
                pub fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.start.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.start)? as i32)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    self.end.is_present()
                }

                #[inline]
                pub fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.end.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.end)? as i32)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for EnumReservedRangeView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct EnumReservedRangeBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> EnumReservedRangeBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &EnumReservedRangeView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumValueDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            number: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
        }

        impl<'a> EnumValueDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut number = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => number = field.marker,
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn has_number(&self) -> bool {
                self.number.is_present()
            }

            #[inline]
            pub fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                if !self.number.is_present() {
                    return Ok(0i32);
                }
                Ok(::roto::runtime::decode_varint_at(self.buf, self.number)? as i32)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::EnumValueOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::EnumValueOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumValueDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumValueDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> EnumValueDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &EnumValueDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumValueOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::EnumValueOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ServiceDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
        }

        impl<'a> ServiceDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn method(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::MethodDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 2)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::ServiceOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::ServiceOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ServiceDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ServiceDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> ServiceDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &ServiceDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_method<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodDescriptorProtoBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::MethodDescriptorProtoBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ServiceOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::ServiceOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MethodDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            input_type: ::roto::runtime::FieldMarker,
            output_type: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            client_streaming: ::roto::runtime::FieldMarker,
            server_streaming: ::roto::runtime::FieldMarker,
        }

        impl<'a> MethodDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut input_type = ::roto::runtime::FieldMarker::none();
                let mut output_type = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut client_streaming = ::roto::runtime::FieldMarker::none();
                let mut server_streaming = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => input_type = field.marker,
                        (3, ::roto::runtime::WireType::LengthDelimited) => output_type = field.marker,
                        (4, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => client_streaming = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => server_streaming = field.marker,
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
            pub fn name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.name)
            }

            #[inline]
            pub const fn has_input_type(&self) -> bool {
                self.input_type.is_present()
            }

            #[inline]
            pub fn input_type(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.input_type.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.input_type)
            }

            #[inline]
            pub const fn has_output_type(&self) -> bool {
                self.output_type.is_present()
            }

            #[inline]
            pub fn output_type(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.output_type.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.output_type)
            }

            #[inline]
            pub const fn has_options(&self) -> bool {
                self.options.is_present()
            }

            #[inline]
            pub fn options(&self) -> Result<Option<crate::google::protobuf::MethodOptionsView<'a>>, ::roto::runtime::DecodeError> {
                if !self.options.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::MethodOptionsView::from_bytes(self.options.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_client_streaming(&self) -> bool {
                self.client_streaming.is_present()
            }

            #[inline]
            pub fn client_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.client_streaming.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.client_streaming)
            }

            #[inline]
            pub const fn has_server_streaming(&self) -> bool {
                self.server_streaming.is_present()
            }

            #[inline]
            pub fn server_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.server_streaming.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.server_streaming)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for MethodDescriptorProtoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct MethodDescriptorProtoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> MethodDescriptorProtoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &MethodDescriptorProtoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_input_type(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_output_type(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodOptionsBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::MethodOptionsBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_client_streaming(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_server_streaming(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FileOptionsView<'a> {
            buf: &'a [u8],
            java_package: ::roto::runtime::FieldMarker,
            java_outer_classname: ::roto::runtime::FieldMarker,
            java_multiple_files: ::roto::runtime::FieldMarker,
            java_generate_equals_and_hash: ::roto::runtime::FieldMarker,
            java_string_check_utf8: ::roto::runtime::FieldMarker,
            optimize_for: ::roto::runtime::FieldMarker,
            go_package: ::roto::runtime::FieldMarker,
            cc_generic_services: ::roto::runtime::FieldMarker,
            java_generic_services: ::roto::runtime::FieldMarker,
            py_generic_services: ::roto::runtime::FieldMarker,
            deprecated: ::roto::runtime::FieldMarker,
            cc_enable_arenas: ::roto::runtime::FieldMarker,
            objc_class_prefix: ::roto::runtime::FieldMarker,
            csharp_namespace: ::roto::runtime::FieldMarker,
            swift_prefix: ::roto::runtime::FieldMarker,
            php_class_prefix: ::roto::runtime::FieldMarker,
            php_namespace: ::roto::runtime::FieldMarker,
            php_metadata_namespace: ::roto::runtime::FieldMarker,
            ruby_package: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
        }

        impl<'a> FileOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 8, 10, 20, 27, 9, 11, 16, 17, 18, 23, 31, 36, 37, 39, 40, 41, 44, 45, 50, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut java_package = ::roto::runtime::FieldMarker::none();
                let mut java_outer_classname = ::roto::runtime::FieldMarker::none();
                let mut java_multiple_files = ::roto::runtime::FieldMarker::none();
                let mut java_generate_equals_and_hash = ::roto::runtime::FieldMarker::none();
                let mut java_string_check_utf8 = ::roto::runtime::FieldMarker::none();
                let mut optimize_for = ::roto::runtime::FieldMarker::none();
                let mut go_package = ::roto::runtime::FieldMarker::none();
                let mut cc_generic_services = ::roto::runtime::FieldMarker::none();
                let mut java_generic_services = ::roto::runtime::FieldMarker::none();
                let mut py_generic_services = ::roto::runtime::FieldMarker::none();
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut cc_enable_arenas = ::roto::runtime::FieldMarker::none();
                let mut objc_class_prefix = ::roto::runtime::FieldMarker::none();
                let mut csharp_namespace = ::roto::runtime::FieldMarker::none();
                let mut swift_prefix = ::roto::runtime::FieldMarker::none();
                let mut php_class_prefix = ::roto::runtime::FieldMarker::none();
                let mut php_namespace = ::roto::runtime::FieldMarker::none();
                let mut php_metadata_namespace = ::roto::runtime::FieldMarker::none();
                let mut ruby_package = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => java_package = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => java_outer_classname = field.marker,
                        (10, ::roto::runtime::WireType::Varint) => java_multiple_files = field.marker,
                        (20, ::roto::runtime::WireType::Varint) => java_generate_equals_and_hash = field.marker,
                        (27, ::roto::runtime::WireType::Varint) => java_string_check_utf8 = field.marker,
                        (9, ::roto::runtime::WireType::Varint) => optimize_for = field.marker,
                        (11, ::roto::runtime::WireType::LengthDelimited) => go_package = field.marker,
                        (16, ::roto::runtime::WireType::Varint) => cc_generic_services = field.marker,
                        (17, ::roto::runtime::WireType::Varint) => java_generic_services = field.marker,
                        (18, ::roto::runtime::WireType::Varint) => py_generic_services = field.marker,
                        (23, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (31, ::roto::runtime::WireType::Varint) => cc_enable_arenas = field.marker,
                        (36, ::roto::runtime::WireType::LengthDelimited) => objc_class_prefix = field.marker,
                        (37, ::roto::runtime::WireType::LengthDelimited) => csharp_namespace = field.marker,
                        (39, ::roto::runtime::WireType::LengthDelimited) => swift_prefix = field.marker,
                        (40, ::roto::runtime::WireType::LengthDelimited) => php_class_prefix = field.marker,
                        (41, ::roto::runtime::WireType::LengthDelimited) => php_namespace = field.marker,
                        (44, ::roto::runtime::WireType::LengthDelimited) => php_metadata_namespace = field.marker,
                        (45, ::roto::runtime::WireType::LengthDelimited) => ruby_package = field.marker,
                        (50, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    java_package,
                    java_outer_classname,
                    java_multiple_files,
                    java_generate_equals_and_hash,
                    java_string_check_utf8,
                    optimize_for,
                    go_package,
                    cc_generic_services,
                    java_generic_services,
                    py_generic_services,
                    deprecated,
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
            pub fn java_package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.java_package.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.java_package)
            }

            #[inline]
            pub const fn has_java_outer_classname(&self) -> bool {
                self.java_outer_classname.is_present()
            }

            #[inline]
            pub fn java_outer_classname(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.java_outer_classname.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.java_outer_classname)
            }

            #[inline]
            pub const fn has_java_multiple_files(&self) -> bool {
                self.java_multiple_files.is_present()
            }

            #[inline]
            pub fn java_multiple_files(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.java_multiple_files.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.java_multiple_files)
            }

            #[inline]
            pub const fn has_java_generate_equals_and_hash(&self) -> bool {
                self.java_generate_equals_and_hash.is_present()
            }

            #[inline]
            pub fn java_generate_equals_and_hash(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.java_generate_equals_and_hash.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.java_generate_equals_and_hash)
            }

            #[inline]
            pub const fn has_java_string_check_utf8(&self) -> bool {
                self.java_string_check_utf8.is_present()
            }

            #[inline]
            pub fn java_string_check_utf8(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.java_string_check_utf8.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.java_string_check_utf8)
            }

            #[inline]
            pub const fn has_optimize_for(&self) -> bool {
                self.optimize_for.is_present()
            }

            #[inline]
            pub fn optimize_for(&self) -> Result<crate::google::protobuf::file_options::OptimizeMode, ::roto::runtime::DecodeError> {
                if !self.optimize_for.is_present() {
                    return Ok(crate::google::protobuf::file_options::OptimizeMode::Speed);
                }
                Ok(<crate::google::protobuf::file_options::OptimizeMode as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.optimize_for)?))
            }

            #[inline]
            pub const fn has_go_package(&self) -> bool {
                self.go_package.is_present()
            }

            #[inline]
            pub fn go_package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.go_package.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.go_package)
            }

            #[inline]
            pub const fn has_cc_generic_services(&self) -> bool {
                self.cc_generic_services.is_present()
            }

            #[inline]
            pub fn cc_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.cc_generic_services.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.cc_generic_services)
            }

            #[inline]
            pub const fn has_java_generic_services(&self) -> bool {
                self.java_generic_services.is_present()
            }

            #[inline]
            pub fn java_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.java_generic_services.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.java_generic_services)
            }

            #[inline]
            pub const fn has_py_generic_services(&self) -> bool {
                self.py_generic_services.is_present()
            }

            #[inline]
            pub fn py_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.py_generic_services.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.py_generic_services)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                self.deprecated.is_present()
            }

            #[inline]
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_cc_enable_arenas(&self) -> bool {
                self.cc_enable_arenas.is_present()
            }

            #[inline]
            pub fn cc_enable_arenas(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.cc_enable_arenas.is_present() {
                    return Ok(true);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.cc_enable_arenas)
            }

            #[inline]
            pub const fn has_objc_class_prefix(&self) -> bool {
                self.objc_class_prefix.is_present()
            }

            #[inline]
            pub fn objc_class_prefix(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.objc_class_prefix.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.objc_class_prefix)
            }

            #[inline]
            pub const fn has_csharp_namespace(&self) -> bool {
                self.csharp_namespace.is_present()
            }

            #[inline]
            pub fn csharp_namespace(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.csharp_namespace.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.csharp_namespace)
            }

            #[inline]
            pub const fn has_swift_prefix(&self) -> bool {
                self.swift_prefix.is_present()
            }

            #[inline]
            pub fn swift_prefix(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.swift_prefix.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.swift_prefix)
            }

            #[inline]
            pub const fn has_php_class_prefix(&self) -> bool {
                self.php_class_prefix.is_present()
            }

            #[inline]
            pub fn php_class_prefix(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.php_class_prefix.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.php_class_prefix)
            }

            #[inline]
            pub const fn has_php_namespace(&self) -> bool {
                self.php_namespace.is_present()
            }

            #[inline]
            pub fn php_namespace(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.php_namespace.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.php_namespace)
            }

            #[inline]
            pub const fn has_php_metadata_namespace(&self) -> bool {
                self.php_metadata_namespace.is_present()
            }

            #[inline]
            pub fn php_metadata_namespace(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.php_metadata_namespace.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.php_metadata_namespace)
            }

            #[inline]
            pub const fn has_ruby_package(&self) -> bool {
                self.ruby_package.is_present()
            }

            #[inline]
            pub fn ruby_package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.ruby_package.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.ruby_package)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FileOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FileOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FileOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FileOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_java_package(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_java_outer_classname(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_java_multiple_files(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_generate_equals_and_hash(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_string_check_utf8(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(27, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_optimize_for(&mut self, val: crate::google::protobuf::file_options::OptimizeMode) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_go_package(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_cc_generic_services(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_generic_services(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_py_generic_services(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(18, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_cc_enable_arenas(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(31, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_objc_class_prefix(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(36, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_csharp_namespace(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(37, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_swift_prefix(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(39, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_class_prefix(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(40, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_namespace(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(41, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_metadata_namespace(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(44, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_ruby_package(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(45, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(50, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod file_options {
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
                    Self::from_i32(val as i32).unwrap_or(Self::Speed)
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MessageOptionsView<'a> {
            buf: &'a [u8],
            message_set_wire_format: ::roto::runtime::FieldMarker,
            no_standard_descriptor_accessor: ::roto::runtime::FieldMarker,
            deprecated: ::roto::runtime::FieldMarker,
            map_entry: ::roto::runtime::FieldMarker,
            deprecated_legacy_json_field_conflicts: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
        }

        impl<'a> MessageOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 7, 11, 12, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut message_set_wire_format = ::roto::runtime::FieldMarker::none();
                let mut no_standard_descriptor_accessor = ::roto::runtime::FieldMarker::none();
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut map_entry = ::roto::runtime::FieldMarker::none();
                let mut deprecated_legacy_json_field_conflicts = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => message_set_wire_format = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => no_standard_descriptor_accessor = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (7, ::roto::runtime::WireType::Varint) => map_entry = field.marker,
                        (11, ::roto::runtime::WireType::Varint) => deprecated_legacy_json_field_conflicts = field.marker,
                        (12, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
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
            pub fn message_set_wire_format(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.message_set_wire_format.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.message_set_wire_format)
            }

            #[inline]
            pub const fn has_no_standard_descriptor_accessor(&self) -> bool {
                self.no_standard_descriptor_accessor.is_present()
            }

            #[inline]
            pub fn no_standard_descriptor_accessor(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.no_standard_descriptor_accessor.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.no_standard_descriptor_accessor)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                self.deprecated.is_present()
            }

            #[inline]
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_map_entry(&self) -> bool {
                self.map_entry.is_present()
            }

            #[inline]
            pub fn map_entry(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.map_entry.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.map_entry)
            }

            #[inline]
            pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
                self.deprecated_legacy_json_field_conflicts.is_present()
            }

            #[inline]
            pub fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated_legacy_json_field_conflicts.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated_legacy_json_field_conflicts)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for MessageOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct MessageOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> MessageOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &MessageOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_message_set_wire_format(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_no_standard_descriptor_accessor(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_map_entry(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated_legacy_json_field_conflicts(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FieldOptionsView<'a> {
            buf: &'a [u8],
            ctype: ::roto::runtime::FieldMarker,
            packed: ::roto::runtime::FieldMarker,
            jstype: ::roto::runtime::FieldMarker,
            lazy: ::roto::runtime::FieldMarker,
            unverified_lazy: ::roto::runtime::FieldMarker,
            deprecated: ::roto::runtime::FieldMarker,
            weak: ::roto::runtime::FieldMarker,
            debug_redact: ::roto::runtime::FieldMarker,
            retention: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
            feature_support: ::roto::runtime::FieldMarker,
        }

        impl<'a> FieldOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 6, 5, 15, 3, 10, 16, 17, 19, 20, 21, 22, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut ctype = ::roto::runtime::FieldMarker::none();
                let mut packed = ::roto::runtime::FieldMarker::none();
                let mut jstype = ::roto::runtime::FieldMarker::none();
                let mut lazy = ::roto::runtime::FieldMarker::none();
                let mut unverified_lazy = ::roto::runtime::FieldMarker::none();
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut weak = ::roto::runtime::FieldMarker::none();
                let mut debug_redact = ::roto::runtime::FieldMarker::none();
                let mut retention = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut feature_support = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => ctype = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => packed = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => jstype = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => lazy = field.marker,
                        (15, ::roto::runtime::WireType::Varint) => unverified_lazy = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (10, ::roto::runtime::WireType::Varint) => weak = field.marker,
                        (16, ::roto::runtime::WireType::Varint) => debug_redact = field.marker,
                        (17, ::roto::runtime::WireType::Varint) => retention = field.marker,
                        (21, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (22, ::roto::runtime::WireType::LengthDelimited) => feature_support = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    ctype,
                    packed,
                    jstype,
                    lazy,
                    unverified_lazy,
                    deprecated,
                    weak,
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
            pub fn ctype(&self) -> Result<crate::google::protobuf::field_options::CType, ::roto::runtime::DecodeError> {
                if !self.ctype.is_present() {
                    return Ok(crate::google::protobuf::field_options::CType::String);
                }
                Ok(<crate::google::protobuf::field_options::CType as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.ctype)?))
            }

            #[inline]
            pub const fn has_packed(&self) -> bool {
                self.packed.is_present()
            }

            #[inline]
            pub fn packed(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.packed.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.packed)
            }

            #[inline]
            pub const fn has_jstype(&self) -> bool {
                self.jstype.is_present()
            }

            #[inline]
            pub fn jstype(&self) -> Result<crate::google::protobuf::field_options::JSType, ::roto::runtime::DecodeError> {
                if !self.jstype.is_present() {
                    return Ok(crate::google::protobuf::field_options::JSType::JsNormal);
                }
                Ok(<crate::google::protobuf::field_options::JSType as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.jstype)?))
            }

            #[inline]
            pub const fn has_lazy(&self) -> bool {
                self.lazy.is_present()
            }

            #[inline]
            pub fn lazy(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.lazy.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.lazy)
            }

            #[inline]
            pub const fn has_unverified_lazy(&self) -> bool {
                self.unverified_lazy.is_present()
            }

            #[inline]
            pub fn unverified_lazy(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.unverified_lazy.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.unverified_lazy)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                self.deprecated.is_present()
            }

            #[inline]
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_weak(&self) -> bool {
                self.weak.is_present()
            }

            #[inline]
            pub fn weak(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.weak.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.weak)
            }

            #[inline]
            pub const fn has_debug_redact(&self) -> bool {
                self.debug_redact.is_present()
            }

            #[inline]
            pub fn debug_redact(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.debug_redact.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.debug_redact)
            }

            #[inline]
            pub const fn has_retention(&self) -> bool {
                self.retention.is_present()
            }

            #[inline]
            pub fn retention(&self) -> Result<crate::google::protobuf::field_options::OptionRetention, ::roto::runtime::DecodeError> {
                if !self.retention.is_present() {
                    return Ok(crate::google::protobuf::field_options::OptionRetention::RetentionUnknown);
                }
                Ok(<crate::google::protobuf::field_options::OptionRetention as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.retention)?))
            }

            #[inline]
            pub const fn targets(&self) -> ::roto::runtime::RepeatedVarintIter<'a, crate::google::protobuf::field_options::OptionTargetType> {
                ::roto::runtime::RepeatedVarintIter::new(self.buf, 19)
            }

            #[inline]
            pub const fn edition_defaults(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::field_options::EditionDefaultView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 20)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_feature_support(&self) -> bool {
                self.feature_support.is_present()
            }

            #[inline]
            pub fn feature_support(&self) -> Result<Option<crate::google::protobuf::field_options::FeatureSupportView<'a>>, ::roto::runtime::DecodeError> {
                if !self.feature_support.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::field_options::FeatureSupportView::from_bytes(self.feature_support.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FieldOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FieldOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FieldOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FieldOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_ctype(&mut self, val: crate::google::protobuf::field_options::CType) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_jstype(&mut self, val: crate::google::protobuf::field_options::JSType) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_lazy(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_unverified_lazy(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_weak(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_debug_redact(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_retention(&mut self, val: crate::google::protobuf::field_options::OptionRetention) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_targets(&mut self, val: crate::google::protobuf::field_options::OptionTargetType) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(19, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_target(&mut self, val: crate::google::protobuf::field_options::OptionTargetType) -> &mut Self {
                self.write_targets(val)
            }

            #[inline]
            pub fn write_packed_targets(&mut self, vals: &[crate::google::protobuf::field_options::OptionTargetType]) -> &mut Self {
                let end_pos = self.writer.cursor();
                for &val in vals.iter().rev() {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                }
                let packed_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(packed_len as u64);
                let _ = self.writer.write_tag(19, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_edition_defaults<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::EditionDefaultBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::field_options::EditionDefaultBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_edition_default<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::EditionDefaultBuilder<'_, 'b>),
            {
                self.write_edition_defaults(f)
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(21, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_feature_support<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::FeatureSupportBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::field_options::FeatureSupportBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod field_options {
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

            impl core::convert::TryFrom<i32> for CType {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for CType {
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

            impl core::convert::TryFrom<i32> for JSType {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for JSType {
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

            impl core::convert::TryFrom<i32> for OptionRetention {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for OptionRetention {
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

            impl core::convert::TryFrom<i32> for OptionTargetType {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for OptionTargetType {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::TargetTypeUnknown)
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct EditionDefaultView<'a> {
                buf: &'a [u8],
                edition: ::roto::runtime::FieldMarker,
                value: ::roto::runtime::FieldMarker,
            }

            impl<'a> EditionDefaultView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[3, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition = ::roto::runtime::FieldMarker::none();
                    let mut value = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (3, ::roto::runtime::WireType::Varint) => edition = field.marker,
                            (2, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        edition,
                        value,
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
                pub fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    if !self.edition.is_present() {
                        return Ok(crate::google::protobuf::Edition::EditionUnknown);
                    }
                    Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition)?))
                }

                #[inline]
                pub const fn has_value(&self) -> bool {
                    self.value.is_present()
                }

                #[inline]
                pub fn value(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.value.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.value)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for EditionDefaultView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct EditionDefaultBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> EditionDefaultBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &EditionDefaultView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_value(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct FeatureSupportView<'a> {
                buf: &'a [u8],
                edition_introduced: ::roto::runtime::FieldMarker,
                edition_deprecated: ::roto::runtime::FieldMarker,
                deprecation_warning: ::roto::runtime::FieldMarker,
                edition_removed: ::roto::runtime::FieldMarker,
                removal_error: ::roto::runtime::FieldMarker,
            }

            impl<'a> FeatureSupportView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition_introduced = ::roto::runtime::FieldMarker::none();
                    let mut edition_deprecated = ::roto::runtime::FieldMarker::none();
                    let mut deprecation_warning = ::roto::runtime::FieldMarker::none();
                    let mut edition_removed = ::roto::runtime::FieldMarker::none();
                    let mut removal_error = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => edition_introduced = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => edition_deprecated = field.marker,
                            (3, ::roto::runtime::WireType::LengthDelimited) => deprecation_warning = field.marker,
                            (4, ::roto::runtime::WireType::Varint) => edition_removed = field.marker,
                            (5, ::roto::runtime::WireType::LengthDelimited) => removal_error = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        edition_introduced,
                        edition_deprecated,
                        deprecation_warning,
                        edition_removed,
                        removal_error,
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
                pub fn edition_introduced(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    if !self.edition_introduced.is_present() {
                        return Ok(crate::google::protobuf::Edition::EditionUnknown);
                    }
                    Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition_introduced)?))
                }

                #[inline]
                pub const fn has_edition_deprecated(&self) -> bool {
                    self.edition_deprecated.is_present()
                }

                #[inline]
                pub fn edition_deprecated(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    if !self.edition_deprecated.is_present() {
                        return Ok(crate::google::protobuf::Edition::EditionUnknown);
                    }
                    Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition_deprecated)?))
                }

                #[inline]
                pub const fn has_deprecation_warning(&self) -> bool {
                    self.deprecation_warning.is_present()
                }

                #[inline]
                pub fn deprecation_warning(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.deprecation_warning.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.deprecation_warning)
                }

                #[inline]
                pub const fn has_edition_removed(&self) -> bool {
                    self.edition_removed.is_present()
                }

                #[inline]
                pub fn edition_removed(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    if !self.edition_removed.is_present() {
                        return Ok(crate::google::protobuf::Edition::EditionUnknown);
                    }
                    Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition_removed)?))
                }

                #[inline]
                pub const fn has_removal_error(&self) -> bool {
                    self.removal_error.is_present()
                }

                #[inline]
                pub fn removal_error(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.removal_error.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.removal_error)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for FeatureSupportView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct FeatureSupportBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> FeatureSupportBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &FeatureSupportView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_edition_introduced(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_edition_deprecated(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_deprecation_warning(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_edition_removed(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_removal_error(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct OneofOptionsView<'a> {
            buf: &'a [u8],
            features: ::roto::runtime::FieldMarker,
        }

        impl<'a> OneofOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut features = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    features,
                })
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
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for OneofOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct OneofOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> OneofOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &OneofOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumOptionsView<'a> {
            buf: &'a [u8],
            allow_alias: ::roto::runtime::FieldMarker,
            deprecated: ::roto::runtime::FieldMarker,
            deprecated_legacy_json_field_conflicts: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
        }

        impl<'a> EnumOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 6, 7, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut allow_alias = ::roto::runtime::FieldMarker::none();
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut deprecated_legacy_json_field_conflicts = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (2, ::roto::runtime::WireType::Varint) => allow_alias = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => deprecated_legacy_json_field_conflicts = field.marker,
                        (7, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
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
            pub fn allow_alias(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.allow_alias.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.allow_alias)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                self.deprecated.is_present()
            }

            #[inline]
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
                self.deprecated_legacy_json_field_conflicts.is_present()
            }

            #[inline]
            pub fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated_legacy_json_field_conflicts.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated_legacy_json_field_conflicts)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> EnumOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &EnumOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_allow_alias(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated_legacy_json_field_conflicts(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumValueOptionsView<'a> {
            buf: &'a [u8],
            deprecated: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
            debug_redact: ::roto::runtime::FieldMarker,
            feature_support: ::roto::runtime::FieldMarker,
        }

        impl<'a> EnumValueOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut debug_redact = ::roto::runtime::FieldMarker::none();
                let mut feature_support = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => debug_redact = field.marker,
                        (4, ::roto::runtime::WireType::LengthDelimited) => feature_support = field.marker,
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
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_debug_redact(&self) -> bool {
                self.debug_redact.is_present()
            }

            #[inline]
            pub fn debug_redact(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.debug_redact.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.debug_redact)
            }

            #[inline]
            pub const fn has_feature_support(&self) -> bool {
                self.feature_support.is_present()
            }

            #[inline]
            pub fn feature_support(&self) -> Result<Option<crate::google::protobuf::field_options::FeatureSupportView<'a>>, ::roto::runtime::DecodeError> {
                if !self.feature_support.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::field_options::FeatureSupportView::from_bytes(self.feature_support.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumValueOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumValueOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> EnumValueOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &EnumValueOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_debug_redact(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_feature_support<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::FeatureSupportBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::field_options::FeatureSupportBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ServiceOptionsView<'a> {
            buf: &'a [u8],
            features: ::roto::runtime::FieldMarker,
            deprecated: ::roto::runtime::FieldMarker,
        }

        impl<'a> ServiceOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[34, 33, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut deprecated = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (34, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (33, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    features,
                    deprecated,
                })
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
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                self.deprecated.is_present()
            }

            #[inline]
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ServiceOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ServiceOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> ServiceOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &ServiceOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(34, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(33, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MethodOptionsView<'a> {
            buf: &'a [u8],
            deprecated: ::roto::runtime::FieldMarker,
            idempotency_level: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
        }

        impl<'a> MethodOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[33, 34, 35, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut deprecated = ::roto::runtime::FieldMarker::none();
                let mut idempotency_level = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (33, ::roto::runtime::WireType::Varint) => deprecated = field.marker,
                        (34, ::roto::runtime::WireType::Varint) => idempotency_level = field.marker,
                        (35, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
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
            pub fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if !self.deprecated.is_present() {
                    return Ok(false);
                }
                ::roto::runtime::decode_bool_at(self.buf, self.deprecated)
            }

            #[inline]
            pub const fn has_idempotency_level(&self) -> bool {
                self.idempotency_level.is_present()
            }

            #[inline]
            pub fn idempotency_level(&self) -> Result<crate::google::protobuf::method_options::IdempotencyLevel, ::roto::runtime::DecodeError> {
                if !self.idempotency_level.is_present() {
                    return Ok(crate::google::protobuf::method_options::IdempotencyLevel::IdempotencyUnknown);
                }
                Ok(<crate::google::protobuf::method_options::IdempotencyLevel as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.idempotency_level)?))
            }

            #[inline]
            pub const fn has_features(&self) -> bool {
                self.features.is_present()
            }

            #[inline]
            pub fn features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                if !self.features.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::FeatureSetView::from_bytes(self.features.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 999)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for MethodOptionsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct MethodOptionsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> MethodOptionsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &MethodOptionsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(33, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_idempotency_level(&mut self, val: crate::google::protobuf::method_options::IdempotencyLevel) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(34, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(35, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod method_options {
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

            impl core::convert::TryFrom<i32> for IdempotencyLevel {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for IdempotencyLevel {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::IdempotencyUnknown)
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct UninterpretedOptionView<'a> {
            buf: &'a [u8],
            identifier_value: ::roto::runtime::FieldMarker,
            positive_int_value: ::roto::runtime::FieldMarker,
            negative_int_value: ::roto::runtime::FieldMarker,
            double_value: ::roto::runtime::FieldMarker,
            string_value: ::roto::runtime::FieldMarker,
            aggregate_value: ::roto::runtime::FieldMarker,
        }

        impl<'a> UninterpretedOptionView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 4, 5, 6, 7, 8];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut identifier_value = ::roto::runtime::FieldMarker::none();
                let mut positive_int_value = ::roto::runtime::FieldMarker::none();
                let mut negative_int_value = ::roto::runtime::FieldMarker::none();
                let mut double_value = ::roto::runtime::FieldMarker::none();
                let mut string_value = ::roto::runtime::FieldMarker::none();
                let mut aggregate_value = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (3, ::roto::runtime::WireType::LengthDelimited) => identifier_value = field.marker,
                        (4, ::roto::runtime::WireType::Varint) => positive_int_value = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => negative_int_value = field.marker,
                        (6, ::roto::runtime::WireType::Fixed64) => double_value = field.marker,
                        (7, ::roto::runtime::WireType::LengthDelimited) => string_value = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => aggregate_value = field.marker,
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
            pub const fn name(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::uninterpreted_option::NamePartView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 2)
            }

            #[inline]
            pub const fn has_identifier_value(&self) -> bool {
                self.identifier_value.is_present()
            }

            #[inline]
            pub fn identifier_value(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.identifier_value.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.identifier_value)
            }

            #[inline]
            pub const fn has_positive_int_value(&self) -> bool {
                self.positive_int_value.is_present()
            }

            #[inline]
            pub fn positive_int_value(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                if !self.positive_int_value.is_present() {
                    return Ok(0u64);
                }
                ::roto::runtime::decode_varint_at(self.buf, self.positive_int_value)
            }

            #[inline]
            pub const fn has_negative_int_value(&self) -> bool {
                self.negative_int_value.is_present()
            }

            #[inline]
            pub fn negative_int_value(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                if !self.negative_int_value.is_present() {
                    return Ok(0i64);
                }
                Ok(::roto::runtime::decode_varint_at(self.buf, self.negative_int_value)? as i64)
            }

            #[inline]
            pub const fn has_double_value(&self) -> bool {
                self.double_value.is_present()
            }

            #[inline]
            pub fn double_value(&self) -> Result<f64, ::roto::runtime::DecodeError> {
                if !self.double_value.is_present() {
                    return Ok(0.0f64);
                }
                ::roto::runtime::decode_f64_at(self.buf, self.double_value)
            }

            #[inline]
            pub const fn has_string_value(&self) -> bool {
                self.string_value.is_present()
            }

            #[inline]
            pub fn string_value(&self) -> Result<&'a [u8], ::roto::runtime::DecodeError> {
                if !self.string_value.is_present() {
                    return Ok(&b""[..]);
                }
                ::roto::runtime::decode_bytes_at(self.buf, self.string_value)
            }

            #[inline]
            pub const fn has_aggregate_value(&self) -> bool {
                self.aggregate_value.is_present()
            }

            #[inline]
            pub fn aggregate_value(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.aggregate_value.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.aggregate_value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for UninterpretedOptionView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct UninterpretedOptionBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> UninterpretedOptionBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &UninterpretedOptionView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_name<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::uninterpreted_option::NamePartBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::uninterpreted_option::NamePartBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_identifier_value(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_positive_int_value(&mut self, val: u64) -> &mut Self {
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_negative_int_value(&mut self, val: i64) -> &mut Self {
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_double_value(&mut self, val: f64) -> &mut Self {
                let _ = self.writer.write_f64(val);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_string_value(&mut self, val: &[u8]) -> &mut Self {
                let _ = self.writer.write_length_delimited(val);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_aggregate_value(&mut self, val: &str) -> &mut Self {
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod uninterpreted_option {
            #[derive(Clone, Copy, Debug)]
            pub struct NamePartView<'a> {
                buf: &'a [u8],
                name_part: ::roto::runtime::FieldMarker,
                is_extension: ::roto::runtime::FieldMarker,
            }

            impl<'a> NamePartView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut name_part = ::roto::runtime::FieldMarker::none();
                    let mut is_extension = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::LengthDelimited) => name_part = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => is_extension = field.marker,
                            _ => {}
                        }
                    }

                    if !name_part.is_present() {
                        return Err(::roto::runtime::DecodeError::MissingRequiredField("NamePart.name_part"));
                    }
                    if !is_extension.is_present() {
                        return Err(::roto::runtime::DecodeError::MissingRequiredField("NamePart.is_extension"));
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
                pub fn name_part(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    ::roto::runtime::decode_str_at(self.buf, self.name_part)
                }

                #[inline]
                pub const fn has_is_extension(&self) -> bool {
                    self.is_extension.is_present()
                }

                #[inline]
                pub fn is_extension(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    ::roto::runtime::decode_bool_at(self.buf, self.is_extension)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for NamePartView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct NamePartBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> NamePartBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &NamePartView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_name_part(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_is_extension(&mut self, val: bool) -> &mut Self {
                    let _ = self.writer.write_bool(val);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FeatureSetView<'a> {
            buf: &'a [u8],
            field_presence: ::roto::runtime::FieldMarker,
            enum_type: ::roto::runtime::FieldMarker,
            repeated_field_encoding: ::roto::runtime::FieldMarker,
            utf8_validation: ::roto::runtime::FieldMarker,
            message_encoding: ::roto::runtime::FieldMarker,
            json_format: ::roto::runtime::FieldMarker,
            enforce_naming_style: ::roto::runtime::FieldMarker,
            default_symbol_visibility: ::roto::runtime::FieldMarker,
            enforce_proto_limits: ::roto::runtime::FieldMarker,
        }

        impl<'a> FeatureSetView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field_presence = ::roto::runtime::FieldMarker::none();
                let mut enum_type = ::roto::runtime::FieldMarker::none();
                let mut repeated_field_encoding = ::roto::runtime::FieldMarker::none();
                let mut utf8_validation = ::roto::runtime::FieldMarker::none();
                let mut message_encoding = ::roto::runtime::FieldMarker::none();
                let mut json_format = ::roto::runtime::FieldMarker::none();
                let mut enforce_naming_style = ::roto::runtime::FieldMarker::none();
                let mut default_symbol_visibility = ::roto::runtime::FieldMarker::none();
                let mut enforce_proto_limits = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => field_presence = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => enum_type = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => repeated_field_encoding = field.marker,
                        (4, ::roto::runtime::WireType::Varint) => utf8_validation = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => message_encoding = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => json_format = field.marker,
                        (7, ::roto::runtime::WireType::Varint) => enforce_naming_style = field.marker,
                        (8, ::roto::runtime::WireType::Varint) => default_symbol_visibility = field.marker,
                        (9, ::roto::runtime::WireType::Varint) => enforce_proto_limits = field.marker,
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
                    enforce_proto_limits,
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
            pub fn field_presence(&self) -> Result<crate::google::protobuf::feature_set::FieldPresence, ::roto::runtime::DecodeError> {
                if !self.field_presence.is_present() {
                    return Ok(crate::google::protobuf::feature_set::FieldPresence::FieldPresenceUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::FieldPresence as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.field_presence)?))
            }

            #[inline]
            pub const fn has_enum_type(&self) -> bool {
                self.enum_type.is_present()
            }

            #[inline]
            pub fn enum_type(&self) -> Result<crate::google::protobuf::feature_set::EnumType, ::roto::runtime::DecodeError> {
                if !self.enum_type.is_present() {
                    return Ok(crate::google::protobuf::feature_set::EnumType::EnumTypeUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::EnumType as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.enum_type)?))
            }

            #[inline]
            pub const fn has_repeated_field_encoding(&self) -> bool {
                self.repeated_field_encoding.is_present()
            }

            #[inline]
            pub fn repeated_field_encoding(&self) -> Result<crate::google::protobuf::feature_set::RepeatedFieldEncoding, ::roto::runtime::DecodeError> {
                if !self.repeated_field_encoding.is_present() {
                    return Ok(crate::google::protobuf::feature_set::RepeatedFieldEncoding::RepeatedFieldEncodingUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::RepeatedFieldEncoding as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.repeated_field_encoding)?))
            }

            #[inline]
            pub const fn has_utf8_validation(&self) -> bool {
                self.utf8_validation.is_present()
            }

            #[inline]
            pub fn utf8_validation(&self) -> Result<crate::google::protobuf::feature_set::Utf8Validation, ::roto::runtime::DecodeError> {
                if !self.utf8_validation.is_present() {
                    return Ok(crate::google::protobuf::feature_set::Utf8Validation::Utf8ValidationUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::Utf8Validation as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.utf8_validation)?))
            }

            #[inline]
            pub const fn has_message_encoding(&self) -> bool {
                self.message_encoding.is_present()
            }

            #[inline]
            pub fn message_encoding(&self) -> Result<crate::google::protobuf::feature_set::MessageEncoding, ::roto::runtime::DecodeError> {
                if !self.message_encoding.is_present() {
                    return Ok(crate::google::protobuf::feature_set::MessageEncoding::MessageEncodingUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::MessageEncoding as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.message_encoding)?))
            }

            #[inline]
            pub const fn has_json_format(&self) -> bool {
                self.json_format.is_present()
            }

            #[inline]
            pub fn json_format(&self) -> Result<crate::google::protobuf::feature_set::JsonFormat, ::roto::runtime::DecodeError> {
                if !self.json_format.is_present() {
                    return Ok(crate::google::protobuf::feature_set::JsonFormat::JsonFormatUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::JsonFormat as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.json_format)?))
            }

            #[inline]
            pub const fn has_enforce_naming_style(&self) -> bool {
                self.enforce_naming_style.is_present()
            }

            #[inline]
            pub fn enforce_naming_style(&self) -> Result<crate::google::protobuf::feature_set::EnforceNamingStyle, ::roto::runtime::DecodeError> {
                if !self.enforce_naming_style.is_present() {
                    return Ok(crate::google::protobuf::feature_set::EnforceNamingStyle::EnforceNamingStyleUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::EnforceNamingStyle as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.enforce_naming_style)?))
            }

            #[inline]
            pub const fn has_default_symbol_visibility(&self) -> bool {
                self.default_symbol_visibility.is_present()
            }

            #[inline]
            pub fn default_symbol_visibility(&self) -> Result<crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility, ::roto::runtime::DecodeError> {
                if !self.default_symbol_visibility.is_present() {
                    return Ok(crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility::DefaultSymbolVisibilityUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.default_symbol_visibility)?))
            }

            #[inline]
            pub const fn has_enforce_proto_limits(&self) -> bool {
                self.enforce_proto_limits.is_present()
            }

            #[inline]
            pub fn enforce_proto_limits(&self) -> Result<crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits, ::roto::runtime::DecodeError> {
                if !self.enforce_proto_limits.is_present() {
                    return Ok(crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits::ProtoLimitsUnknown);
                }
                Ok(<crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.enforce_proto_limits)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FeatureSetView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FeatureSetBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FeatureSetBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FeatureSetView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_field_presence(&mut self, val: crate::google::protobuf::feature_set::FieldPresence) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enum_type(&mut self, val: crate::google::protobuf::feature_set::EnumType) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_repeated_field_encoding(&mut self, val: crate::google::protobuf::feature_set::RepeatedFieldEncoding) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_utf8_validation(&mut self, val: crate::google::protobuf::feature_set::Utf8Validation) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_message_encoding(&mut self, val: crate::google::protobuf::feature_set::MessageEncoding) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_json_format(&mut self, val: crate::google::protobuf::feature_set::JsonFormat) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enforce_naming_style(&mut self, val: crate::google::protobuf::feature_set::EnforceNamingStyle) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_default_symbol_visibility(&mut self, val: crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enforce_proto_limits(&mut self, val: crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod feature_set {
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum FieldPresence {
                FieldPresenceUnknown = 0,
                Explicit = 1,
                Implicit = 2,
                LegacyRequired = 3,
            }

            impl FieldPresence {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::FieldPresenceUnknown),
                        1 => Some(Self::Explicit),
                        2 => Some(Self::Implicit),
                        3 => Some(Self::LegacyRequired),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for FieldPresence {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for FieldPresence {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::FieldPresenceUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum EnumType {
                EnumTypeUnknown = 0,
                Open = 1,
                Closed = 2,
            }

            impl EnumType {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::EnumTypeUnknown),
                        1 => Some(Self::Open),
                        2 => Some(Self::Closed),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for EnumType {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for EnumType {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::EnumTypeUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum RepeatedFieldEncoding {
                RepeatedFieldEncodingUnknown = 0,
                Packed = 1,
                Expanded = 2,
            }

            impl RepeatedFieldEncoding {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::RepeatedFieldEncodingUnknown),
                        1 => Some(Self::Packed),
                        2 => Some(Self::Expanded),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for RepeatedFieldEncoding {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for RepeatedFieldEncoding {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::RepeatedFieldEncodingUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum Utf8Validation {
                Utf8ValidationUnknown = 0,
                Verify = 2,
                None = 3,
            }

            impl Utf8Validation {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::Utf8ValidationUnknown),
                        2 => Some(Self::Verify),
                        3 => Some(Self::None),
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

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum MessageEncoding {
                MessageEncodingUnknown = 0,
                LengthPrefixed = 1,
                Delimited = 2,
            }

            impl MessageEncoding {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::MessageEncodingUnknown),
                        1 => Some(Self::LengthPrefixed),
                        2 => Some(Self::Delimited),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for MessageEncoding {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for MessageEncoding {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::MessageEncodingUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum JsonFormat {
                JsonFormatUnknown = 0,
                Allow = 1,
                LegacyBestEffort = 2,
            }

            impl JsonFormat {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::JsonFormatUnknown),
                        1 => Some(Self::Allow),
                        2 => Some(Self::LegacyBestEffort),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for JsonFormat {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for JsonFormat {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::JsonFormatUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum EnforceNamingStyle {
                EnforceNamingStyleUnknown = 0,
                Style2024 = 1,
                StyleLegacy = 2,
                Style2026 = 3,
            }

            impl EnforceNamingStyle {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::EnforceNamingStyleUnknown),
                        1 => Some(Self::Style2024),
                        2 => Some(Self::StyleLegacy),
                        3 => Some(Self::Style2026),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for EnforceNamingStyle {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for EnforceNamingStyle {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::EnforceNamingStyleUnknown)
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct VisibilityFeatureView<'a> {
                buf: &'a [u8],
            }

            impl<'a> VisibilityFeatureView<'a> {
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

            impl<'a> ::roto::runtime::MessageView<'a> for VisibilityFeatureView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct VisibilityFeatureBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> VisibilityFeatureBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &VisibilityFeatureView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }
            }

            pub mod visibility_feature {
                #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                #[repr(i32)]
                pub enum DefaultSymbolVisibility {
                    DefaultSymbolVisibilityUnknown = 0,
                    ExportAll = 1,
                    ExportTopLevel = 2,
                    LocalAll = 3,
                    Strict = 4,
                }

                impl DefaultSymbolVisibility {
                    #[inline]
                    pub const fn from_i32(val: i32) -> Option<Self> {
                        match val {
                            0 => Some(Self::DefaultSymbolVisibilityUnknown),
                            1 => Some(Self::ExportAll),
                            2 => Some(Self::ExportTopLevel),
                            3 => Some(Self::LocalAll),
                            4 => Some(Self::Strict),
                            _ => None,
                        }
                    }

                    #[inline]
                    pub const fn as_i32(self) -> i32 {
                        self as i32
                    }
                }

                impl core::convert::TryFrom<i32> for DefaultSymbolVisibility {
                    type Error = i32;

                    #[inline]
                    fn try_from(val: i32) -> Result<Self, Self::Error> {
                        Self::from_i32(val).ok_or(val)
                    }
                }

                impl ::roto::runtime::VarintValue for DefaultSymbolVisibility {
                    #[inline]
                    fn from_raw_varint(val: u64) -> Self {
                        Self::from_i32(val as i32).unwrap_or(Self::DefaultSymbolVisibilityUnknown)
                    }
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct ProtoLimitsFeatureView<'a> {
                buf: &'a [u8],
            }

            impl<'a> ProtoLimitsFeatureView<'a> {
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

            impl<'a> ::roto::runtime::MessageView<'a> for ProtoLimitsFeatureView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct ProtoLimitsFeatureBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> ProtoLimitsFeatureBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &ProtoLimitsFeatureView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }
            }

            pub mod proto_limits_feature {
                #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                #[repr(i32)]
                pub enum EnforceProtoLimits {
                    ProtoLimitsUnknown = 0,
                    LegacyNoExplicitLimits = 1,
                    ProtoLimits2026 = 2,
                }

                impl EnforceProtoLimits {
                    #[inline]
                    pub const fn from_i32(val: i32) -> Option<Self> {
                        match val {
                            0 => Some(Self::ProtoLimitsUnknown),
                            1 => Some(Self::LegacyNoExplicitLimits),
                            2 => Some(Self::ProtoLimits2026),
                            _ => None,
                        }
                    }

                    #[inline]
                    pub const fn as_i32(self) -> i32 {
                        self as i32
                    }
                }

                impl core::convert::TryFrom<i32> for EnforceProtoLimits {
                    type Error = i32;

                    #[inline]
                    fn try_from(val: i32) -> Result<Self, Self::Error> {
                        Self::from_i32(val).ok_or(val)
                    }
                }

                impl ::roto::runtime::VarintValue for EnforceProtoLimits {
                    #[inline]
                    fn from_raw_varint(val: u64) -> Self {
                        Self::from_i32(val as i32).unwrap_or(Self::ProtoLimitsUnknown)
                    }
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FeatureSetDefaultsView<'a> {
            buf: &'a [u8],
            minimum_edition: ::roto::runtime::FieldMarker,
            maximum_edition: ::roto::runtime::FieldMarker,
        }

        impl<'a> FeatureSetDefaultsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 4, 5];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut minimum_edition = ::roto::runtime::FieldMarker::none();
                let mut maximum_edition = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (4, ::roto::runtime::WireType::Varint) => minimum_edition = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => maximum_edition = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    minimum_edition,
                    maximum_edition,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn defaults(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 1)
            }

            #[inline]
            pub const fn has_minimum_edition(&self) -> bool {
                self.minimum_edition.is_present()
            }

            #[inline]
            pub fn minimum_edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                if !self.minimum_edition.is_present() {
                    return Ok(crate::google::protobuf::Edition::EditionUnknown);
                }
                Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.minimum_edition)?))
            }

            #[inline]
            pub const fn has_maximum_edition(&self) -> bool {
                self.maximum_edition.is_present()
            }

            #[inline]
            pub fn maximum_edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                if !self.maximum_edition.is_present() {
                    return Ok(crate::google::protobuf::Edition::EditionUnknown);
                }
                Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.maximum_edition)?))
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FeatureSetDefaultsView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FeatureSetDefaultsBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> FeatureSetDefaultsBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &FeatureSetDefaultsView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_defaults<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_default<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder<'_, 'b>),
            {
                self.write_defaults(f)
            }

            #[inline]
            pub fn write_minimum_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_maximum_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod feature_set_defaults {
            #[derive(Clone, Copy, Debug)]
            pub struct FeatureSetEditionDefaultView<'a> {
                buf: &'a [u8],
                edition: ::roto::runtime::FieldMarker,
                overridable_features: ::roto::runtime::FieldMarker,
                fixed_features: ::roto::runtime::FieldMarker,
            }

            impl<'a> FeatureSetEditionDefaultView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition = ::roto::runtime::FieldMarker::none();
                    let mut overridable_features = ::roto::runtime::FieldMarker::none();
                    let mut fixed_features = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (3, ::roto::runtime::WireType::Varint) => edition = field.marker,
                            (4, ::roto::runtime::WireType::LengthDelimited) => overridable_features = field.marker,
                            (5, ::roto::runtime::WireType::LengthDelimited) => fixed_features = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        edition,
                        overridable_features,
                        fixed_features,
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
                pub fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    if !self.edition.is_present() {
                        return Ok(crate::google::protobuf::Edition::EditionUnknown);
                    }
                    Ok(<crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.edition)?))
                }

                #[inline]
                pub const fn has_overridable_features(&self) -> bool {
                    self.overridable_features.is_present()
                }

                #[inline]
                pub fn overridable_features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                    if !self.overridable_features.is_present() {
                        return Ok(None);
                    }
                    crate::google::protobuf::FeatureSetView::from_bytes(self.overridable_features.slice(self.buf)).map(Some)
                }

                #[inline]
                pub const fn has_fixed_features(&self) -> bool {
                    self.fixed_features.is_present()
                }

                #[inline]
                pub fn fixed_features(&self) -> Result<Option<crate::google::protobuf::FeatureSetView<'a>>, ::roto::runtime::DecodeError> {
                    if !self.fixed_features.is_present() {
                        return Ok(None);
                    }
                    crate::google::protobuf::FeatureSetView::from_bytes(self.fixed_features.slice(self.buf)).map(Some)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for FeatureSetEditionDefaultView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct FeatureSetEditionDefaultBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> FeatureSetEditionDefaultBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &FeatureSetEditionDefaultView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_overridable_features<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_fixed_features<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct SourceCodeInfoView<'a> {
            buf: &'a [u8],
        }

        impl<'a> SourceCodeInfoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

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
            pub const fn location(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::source_code_info::LocationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for SourceCodeInfoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct SourceCodeInfoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> SourceCodeInfoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &SourceCodeInfoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_location<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::source_code_info::LocationBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::source_code_info::LocationBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod source_code_info {
            #[derive(Clone, Copy, Debug)]
            pub struct LocationView<'a> {
                buf: &'a [u8],
                leading_comments: ::roto::runtime::FieldMarker,
                trailing_comments: ::roto::runtime::FieldMarker,
            }

            impl<'a> LocationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 6];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut leading_comments = ::roto::runtime::FieldMarker::none();
                    let mut trailing_comments = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (3, ::roto::runtime::WireType::LengthDelimited) => leading_comments = field.marker,
                            (4, ::roto::runtime::WireType::LengthDelimited) => trailing_comments = field.marker,
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
                pub const fn path(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.buf, 1)
                }

                #[inline]
                pub const fn span(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.buf, 2)
                }

                #[inline]
                pub const fn has_leading_comments(&self) -> bool {
                    self.leading_comments.is_present()
                }

                #[inline]
                pub fn leading_comments(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.leading_comments.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.leading_comments)
                }

                #[inline]
                pub const fn has_trailing_comments(&self) -> bool {
                    self.trailing_comments.is_present()
                }

                #[inline]
                pub fn trailing_comments(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.trailing_comments.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.trailing_comments)
                }

                #[inline]
                pub const fn leading_detached_comments(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                    ::roto::runtime::RepeatedStrIter::new(self.buf, 6)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for LocationView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct LocationBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> LocationBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_path(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_path(&mut self, vals: &[i32]) -> &mut Self {
                    let end_pos = self.writer.cursor();
                    for &val in vals.iter().rev() {
                        let _ = self.writer.write_varint(val as u64);
                    }
                    let packed_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(packed_len as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_span(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_span(&mut self, vals: &[i32]) -> &mut Self {
                    let end_pos = self.writer.cursor();
                    for &val in vals.iter().rev() {
                        let _ = self.writer.write_varint(val as u64);
                    }
                    let packed_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(packed_len as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_leading_comments(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_trailing_comments(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_leading_detached_comments(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_leading_detached_comment(&mut self, val: &str) -> &mut Self {
                    self.write_leading_detached_comments(val)
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct GeneratedCodeInfoView<'a> {
            buf: &'a [u8],
        }

        impl<'a> GeneratedCodeInfoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

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
            pub const fn annotation(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::generated_code_info::AnnotationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.buf, 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for GeneratedCodeInfoView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct GeneratedCodeInfoBuilder<'a, 'b> {
            pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'a, 'b> GeneratedCodeInfoBuilder<'a, 'b> {
            #[inline]
            pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                Self { writer }
            }

            #[inline]
            pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                self.writer.check()
            }

            #[inline]
            pub fn copy_unknown_fields(&mut self, view: &GeneratedCodeInfoView<'_>) -> &mut Self {
                for raw_field in view.unknown_fields() {
                    let _ = self.writer.write_bytes(raw_field);
                }
                self
            }

            #[inline]
            pub fn write_annotation<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::generated_code_info::AnnotationBuilder<'_, 'b>),
            {
                let end_pos = self.writer.cursor();
                {
                    let mut sub_builder = crate::google::protobuf::generated_code_info::AnnotationBuilder::new(self.writer);
                    f(&mut sub_builder);
                }
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod generated_code_info {
            #[derive(Clone, Copy, Debug)]
            pub struct AnnotationView<'a> {
                buf: &'a [u8],
                source_file: ::roto::runtime::FieldMarker,
                begin: ::roto::runtime::FieldMarker,
                end: ::roto::runtime::FieldMarker,
                semantic: ::roto::runtime::FieldMarker,
            }

            impl<'a> AnnotationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut source_file = ::roto::runtime::FieldMarker::none();
                    let mut begin = ::roto::runtime::FieldMarker::none();
                    let mut end = ::roto::runtime::FieldMarker::none();
                    let mut semantic = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (2, ::roto::runtime::WireType::LengthDelimited) => source_file = field.marker,
                            (3, ::roto::runtime::WireType::Varint) => begin = field.marker,
                            (4, ::roto::runtime::WireType::Varint) => end = field.marker,
                            (5, ::roto::runtime::WireType::Varint) => semantic = field.marker,
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
                pub const fn path(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.buf, 1)
                }

                #[inline]
                pub const fn has_source_file(&self) -> bool {
                    self.source_file.is_present()
                }

                #[inline]
                pub fn source_file(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.source_file.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.source_file)
                }

                #[inline]
                pub const fn has_begin(&self) -> bool {
                    self.begin.is_present()
                }

                #[inline]
                pub fn begin(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.begin.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.begin)? as i32)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    self.end.is_present()
                }

                #[inline]
                pub fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.end.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.end)? as i32)
                }

                #[inline]
                pub const fn has_semantic(&self) -> bool {
                    self.semantic.is_present()
                }

                #[inline]
                pub fn semantic(&self) -> Result<crate::google::protobuf::generated_code_info::annotation::Semantic, ::roto::runtime::DecodeError> {
                    if !self.semantic.is_present() {
                        return Ok(crate::google::protobuf::generated_code_info::annotation::Semantic::None);
                    }
                    Ok(<crate::google::protobuf::generated_code_info::annotation::Semantic as ::roto::runtime::VarintValue>::from_raw_varint(::roto::runtime::decode_varint_at(self.buf, self.semantic)?))
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for AnnotationView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct AnnotationBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> AnnotationBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &AnnotationView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_path(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_path(&mut self, vals: &[i32]) -> &mut Self {
                    let end_pos = self.writer.cursor();
                    for &val in vals.iter().rev() {
                        let _ = self.writer.write_varint(val as u64);
                    }
                    let packed_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(packed_len as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_source_file(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_begin(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_semantic(&mut self, val: crate::google::protobuf::generated_code_info::annotation::Semantic) -> &mut Self {
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                    self
                }
            }

            pub mod annotation {
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

                impl core::convert::TryFrom<i32> for Semantic {
                    type Error = i32;

                    #[inline]
                    fn try_from(val: i32) -> Result<Self, Self::Error> {
                        Self::from_i32(val).ok_or(val)
                    }
                }

                impl ::roto::runtime::VarintValue for Semantic {
                    #[inline]
                    fn from_raw_varint(val: u64) -> Self {
                        Self::from_i32(val as i32).unwrap_or(Self::None)
                    }
                }
            }
        }

        pub mod compiler {
            #[derive(Clone, Copy, Debug)]
            pub struct VersionView<'a> {
                buf: &'a [u8],
                major: ::roto::runtime::FieldMarker,
                minor: ::roto::runtime::FieldMarker,
                patch: ::roto::runtime::FieldMarker,
                suffix: ::roto::runtime::FieldMarker,
            }

            impl<'a> VersionView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut major = ::roto::runtime::FieldMarker::none();
                    let mut minor = ::roto::runtime::FieldMarker::none();
                    let mut patch = ::roto::runtime::FieldMarker::none();
                    let mut suffix = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => major = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => minor = field.marker,
                            (3, ::roto::runtime::WireType::Varint) => patch = field.marker,
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
                pub fn major(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.major.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.major)? as i32)
                }

                #[inline]
                pub const fn has_minor(&self) -> bool {
                    self.minor.is_present()
                }

                #[inline]
                pub fn minor(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.minor.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.minor)? as i32)
                }

                #[inline]
                pub const fn has_patch(&self) -> bool {
                    self.patch.is_present()
                }

                #[inline]
                pub fn patch(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.patch.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.patch)? as i32)
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

            pub struct VersionBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> VersionBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &VersionView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_major(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_minor(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_patch(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_suffix(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct CodeGeneratorRequestView<'a> {
                buf: &'a [u8],
                parameter: ::roto::runtime::FieldMarker,
                compiler_version: ::roto::runtime::FieldMarker,
            }

            impl<'a> CodeGeneratorRequestView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 15, 17, 3];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut parameter = ::roto::runtime::FieldMarker::none();
                    let mut compiler_version = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (2, ::roto::runtime::WireType::LengthDelimited) => parameter = field.marker,
                            (3, ::roto::runtime::WireType::LengthDelimited) => compiler_version = field.marker,
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
                pub const fn file_to_generate(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                    ::roto::runtime::RepeatedStrIter::new(self.buf, 1)
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
                    ::roto::runtime::RepeatedMessageIter::new(self.buf, 15)
                }

                #[inline]
                pub const fn source_file_descriptors(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FileDescriptorProtoView<'a>> {
                    ::roto::runtime::RepeatedMessageIter::new(self.buf, 17)
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

            pub struct CodeGeneratorRequestBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> CodeGeneratorRequestBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &CodeGeneratorRequestView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_file_to_generate(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_parameter(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_proto_file<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_source_file_descriptors<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(17, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_source_file_descriptor<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_, 'b>),
                {
                    self.write_source_file_descriptors(f)
                }

                #[inline]
                pub fn write_compiler_version<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::compiler::VersionBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::compiler::VersionBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
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
                supported_features: ::roto::runtime::FieldMarker,
                minimum_edition: ::roto::runtime::FieldMarker,
                maximum_edition: ::roto::runtime::FieldMarker,
            }

            impl<'a> CodeGeneratorResponseView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 15];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut error = ::roto::runtime::FieldMarker::none();
                    let mut supported_features = ::roto::runtime::FieldMarker::none();
                    let mut minimum_edition = ::roto::runtime::FieldMarker::none();
                    let mut maximum_edition = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::LengthDelimited) => error = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => supported_features = field.marker,
                            (3, ::roto::runtime::WireType::Varint) => minimum_edition = field.marker,
                            (4, ::roto::runtime::WireType::Varint) => maximum_edition = field.marker,
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
                pub fn error(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.error.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.error)
                }

                #[inline]
                pub const fn has_supported_features(&self) -> bool {
                    self.supported_features.is_present()
                }

                #[inline]
                pub fn supported_features(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                    if !self.supported_features.is_present() {
                        return Ok(0u64);
                    }
                    ::roto::runtime::decode_varint_at(self.buf, self.supported_features)
                }

                #[inline]
                pub const fn has_minimum_edition(&self) -> bool {
                    self.minimum_edition.is_present()
                }

                #[inline]
                pub fn minimum_edition(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.minimum_edition.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.minimum_edition)? as i32)
                }

                #[inline]
                pub const fn has_maximum_edition(&self) -> bool {
                    self.maximum_edition.is_present()
                }

                #[inline]
                pub fn maximum_edition(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    if !self.maximum_edition.is_present() {
                        return Ok(0i32);
                    }
                    Ok(::roto::runtime::decode_varint_at(self.buf, self.maximum_edition)? as i32)
                }

                #[inline]
                pub const fn file(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::compiler::code_generator_response::FileView<'a>> {
                    ::roto::runtime::RepeatedMessageIter::new(self.buf, 15)
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

            pub struct CodeGeneratorResponseBuilder<'a, 'b> {
                pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'a, 'b> CodeGeneratorResponseBuilder<'a, 'b> {
                #[inline]
                pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                    Self { writer }
                }

                #[inline]
                pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                    self.writer.check()
                }

                #[inline]
                pub fn copy_unknown_fields(&mut self, view: &CodeGeneratorResponseView<'_>) -> &mut Self {
                    for raw_field in view.unknown_fields() {
                        let _ = self.writer.write_bytes(raw_field);
                    }
                    self
                }

                #[inline]
                pub fn write_error(&mut self, val: &str) -> &mut Self {
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_supported_features(&mut self, val: u64) -> &mut Self {
                    let _ = self.writer.write_varint(val);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_minimum_edition(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_maximum_edition(&mut self, val: i32) -> &mut Self {
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_file<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::compiler::code_generator_response::FileBuilder<'_, 'b>),
                {
                    let end_pos = self.writer.cursor();
                    {
                        let mut sub_builder = crate::google::protobuf::compiler::code_generator_response::FileBuilder::new(self.writer);
                        f(&mut sub_builder);
                    }
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

                pub struct FileBuilder<'a, 'b> {
                    pub writer: &'a mut ::roto::runtime::BackwardWriter<'b>,
                }

                impl<'a, 'b> FileBuilder<'a, 'b> {
                    #[inline]
                    pub fn new(writer: &'a mut ::roto::runtime::BackwardWriter<'b>) -> Self {
                        Self { writer }
                    }

                    #[inline]
                    pub const fn check(&self) -> Result<(), ::roto::runtime::EncodeError> {
                        self.writer.check()
                    }

                    #[inline]
                    pub fn copy_unknown_fields(&mut self, view: &FileView<'_>) -> &mut Self {
                        for raw_field in view.unknown_fields() {
                            let _ = self.writer.write_bytes(raw_field);
                        }
                        self
                    }

                    #[inline]
                    pub fn write_name(&mut self, val: &str) -> &mut Self {
                        let _ = self.writer.write_length_delimited(val.as_bytes());
                        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                        self
                    }

                    #[inline]
                    pub fn write_insertion_point(&mut self, val: &str) -> &mut Self {
                        let _ = self.writer.write_length_delimited(val.as_bytes());
                        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                        self
                    }

                    #[inline]
                    pub fn write_content(&mut self, val: &str) -> &mut Self {
                        let _ = self.writer.write_length_delimited(val.as_bytes());
                        let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                        self
                    }

                    #[inline]
                    pub fn write_generated_code_info<F>(&mut self, f: F) -> &mut Self
                    where
                        F: FnOnce(&mut crate::google::protobuf::GeneratedCodeInfoBuilder<'_, 'b>),
                    {
                        let end_pos = self.writer.cursor();
                        {
                            let mut sub_builder = crate::google::protobuf::GeneratedCodeInfoBuilder::new(self.writer);
                            f(&mut sub_builder);
                        }
                        let nested_len = end_pos - self.writer.cursor();
                        let _ = self.writer.write_varint(nested_len as u64);
                        let _ = self.writer.write_tag(16, ::roto::runtime::WireType::LengthDelimited);
                        self
                    }
                }
            }
        }
    }
}
