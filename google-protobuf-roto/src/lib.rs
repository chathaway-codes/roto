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
        #[derive(Clone, Copy, Debug)]
        pub struct AnyView<'a> {
            buf: &'a [u8],
            type_url: ::roto::runtime::FieldMarker,
            value: ::roto::runtime::FieldMarker,
        }

        impl<'a> AnyView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut type_url = ::roto::runtime::FieldMarker::none();
                let mut value = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => type_url = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    type_url,
                    value,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_type_url(&self) -> bool {
                self.type_url.is_present()
            }

            #[inline]
            pub fn type_url(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.type_url.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.type_url)
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                self.value.is_present()
            }

            #[inline]
            pub fn value(&self) -> Result<&'a [u8], ::roto::runtime::DecodeError> {
                if !self.value.is_present() {
                    return Ok(&b""[..]);
                }
                ::roto::runtime::decode_bytes_at(self.buf, self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for AnyView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct AnyBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> AnyBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &AnyView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !AnyView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &AnyView<'_>) -> &mut Self {
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
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_type_url(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_value(&mut self, val: &[u8]) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ApiView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            methods: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            version: ::roto::runtime::FieldMarker,
            source_context: ::roto::runtime::FieldMarker,
            mixins: ::roto::runtime::FieldMarker,
            syntax: crate::google::protobuf::Syntax,
            edition: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> ApiView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut methods = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut version = ::roto::runtime::FieldMarker::none();
                let mut source_context = ::roto::runtime::FieldMarker::none();
                let mut mixins = ::roto::runtime::FieldMarker::none();
                let mut syntax: crate::google::protobuf::Syntax = crate::google::protobuf::Syntax::SyntaxProto2;
                let mut edition = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => methods.extend(field.raw_span),
                        (3, _) => options.extend(field.raw_span),
                        (4, ::roto::runtime::WireType::LengthDelimited) => version = field.marker,
                        (5, ::roto::runtime::WireType::LengthDelimited) => source_context = field.marker,
                        (6, _) => mixins.extend(field.raw_span),
                        (7, ::roto::runtime::WireType::Varint) => {
                            syntax = <crate::google::protobuf::Syntax as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (8, ::roto::runtime::WireType::LengthDelimited) => edition = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    methods,
                    options,
                    version,
                    source_context,
                    mixins,
                    syntax,
                    edition,
                    has_fields,
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
            pub const fn methods(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::MethodView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.methods.slice(self.buf), 2)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 3)
            }

            #[inline]
            pub const fn has_version(&self) -> bool {
                self.version.is_present()
            }

            #[inline]
            pub fn version(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.version.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.version)
            }

            #[inline]
            pub const fn has_source_context(&self) -> bool {
                self.source_context.is_present()
            }

            #[inline]
            pub fn source_context(&self) -> Result<Option<crate::google::protobuf::SourceContextView<'a>>, ::roto::runtime::DecodeError> {
                if !self.source_context.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::SourceContextView::from_bytes(self.source_context.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn mixins(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::MixinView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.mixins.slice(self.buf), 6)
            }

            #[inline]
            pub const fn has_syntax(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn syntax(&self) -> Result<crate::google::protobuf::Syntax, ::roto::runtime::DecodeError> {
                Ok(self.syntax)
            }

            #[inline]
            pub const fn has_edition(&self) -> bool {
                self.edition.is_present()
            }

            #[inline]
            pub fn edition(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.edition.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.edition)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ApiView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ApiBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ApiBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ApiView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ApiView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ApiView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
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
            pub fn write_methods<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::MethodBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_method<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodBuilder<'_>),
            {
                self.write_methods(f)
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }

            #[inline]
            pub fn write_version(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_source_context<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::SourceContextBuilder<'_>),
            {
                self.set_fields[0] |= 16;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::SourceContextBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_mixins<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MixinBuilder<'_>),
            {
                self.set_fields[0] |= 32;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::MixinBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_mixin<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MixinBuilder<'_>),
            {
                self.write_mixins(f)
            }

            #[inline]
            pub fn write_syntax(&mut self, val: crate::google::protobuf::Syntax) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MethodView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            request_type_url: ::roto::runtime::FieldMarker,
            request_streaming: bool,
            response_type_url: ::roto::runtime::FieldMarker,
            response_streaming: bool,
            options: ::roto::runtime::FieldMarker,
            syntax: crate::google::protobuf::Syntax,
            edition: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> MethodView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut request_type_url = ::roto::runtime::FieldMarker::none();
                let mut request_streaming: bool = false;
                let mut response_type_url = ::roto::runtime::FieldMarker::none();
                let mut response_streaming: bool = false;
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut syntax: crate::google::protobuf::Syntax = crate::google::protobuf::Syntax::SyntaxProto2;
                let mut edition = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => request_type_url = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => {
                            request_streaming = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (4, ::roto::runtime::WireType::LengthDelimited) => response_type_url = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => {
                            response_streaming = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (6, _) => options.extend(field.raw_span),
                        (7, ::roto::runtime::WireType::Varint) => {
                            syntax = <crate::google::protobuf::Syntax as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 4;
                        }
                        (8, ::roto::runtime::WireType::LengthDelimited) => edition = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    request_type_url,
                    request_streaming,
                    response_type_url,
                    response_streaming,
                    options,
                    syntax,
                    edition,
                    has_fields,
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
            pub const fn has_request_type_url(&self) -> bool {
                self.request_type_url.is_present()
            }

            #[inline]
            pub fn request_type_url(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.request_type_url.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.request_type_url)
            }

            #[inline]
            pub const fn has_request_streaming(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn request_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.request_streaming)
            }

            #[inline]
            pub const fn has_response_type_url(&self) -> bool {
                self.response_type_url.is_present()
            }

            #[inline]
            pub fn response_type_url(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.response_type_url.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.response_type_url)
            }

            #[inline]
            pub const fn has_response_streaming(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn response_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.response_streaming)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 6)
            }

            #[inline]
            pub const fn has_syntax(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn syntax(&self) -> Result<crate::google::protobuf::Syntax, ::roto::runtime::DecodeError> {
                Ok(self.syntax)
            }

            #[inline]
            pub const fn has_edition(&self) -> bool {
                self.edition.is_present()
            }

            #[inline]
            pub fn edition(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.edition.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.edition)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for MethodView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct MethodBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> MethodBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &MethodView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MethodView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &MethodView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
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
            pub fn write_request_type_url(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_request_streaming(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_response_type_url(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_response_streaming(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 32;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }

            #[inline]
            pub fn write_syntax(&mut self, val: crate::google::protobuf::Syntax) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MixinView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            root: ::roto::runtime::FieldMarker,
        }

        impl<'a> MixinView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut root = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => root = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    root,
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
            pub const fn has_root(&self) -> bool {
                self.root.is_present()
            }

            #[inline]
            pub fn root(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.root.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.root)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for MixinView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct MixinBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> MixinBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &MixinView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MixinView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &MixinView<'_>) -> &mut Self {
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
            pub fn write_root(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

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
            file: ::roto::runtime::FieldMarker,
        }

        impl<'a> FileDescriptorSetView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut file = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => file.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    file,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn file(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FileDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.file.slice(self.buf), 1)
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

        pub struct FileDescriptorSetBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FileDescriptorSetBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FileDescriptorSetView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FileDescriptorSetView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FileDescriptorSetView<'_>) -> &mut Self {
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
            pub fn write_file<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FileDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FileDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
            dependency: ::roto::runtime::FieldMarker,
            public_dependency: ::roto::runtime::FieldMarker,
            weak_dependency: ::roto::runtime::FieldMarker,
            option_dependency: ::roto::runtime::FieldMarker,
            message_type: ::roto::runtime::FieldMarker,
            enum_type: ::roto::runtime::FieldMarker,
            service: ::roto::runtime::FieldMarker,
            extension: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            source_code_info: ::roto::runtime::FieldMarker,
            syntax: ::roto::runtime::FieldMarker,
            edition: crate::google::protobuf::Edition,
            has_fields: [u32; 1],
        }

        impl<'a> FileDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 10, 11, 15, 4, 5, 6, 7, 8, 9, 12, 14];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut package = ::roto::runtime::FieldMarker::none();
                let mut dependency = ::roto::runtime::FieldMarker::none();
                let mut public_dependency = ::roto::runtime::FieldMarker::none();
                let mut weak_dependency = ::roto::runtime::FieldMarker::none();
                let mut option_dependency = ::roto::runtime::FieldMarker::none();
                let mut message_type = ::roto::runtime::FieldMarker::none();
                let mut enum_type = ::roto::runtime::FieldMarker::none();
                let mut service = ::roto::runtime::FieldMarker::none();
                let mut extension = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut source_code_info = ::roto::runtime::FieldMarker::none();
                let mut syntax = ::roto::runtime::FieldMarker::none();
                let mut edition: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => package = field.marker,
                        (3, _) => dependency.extend(field.raw_span),
                        (10, _) => public_dependency.extend(field.raw_span),
                        (11, _) => weak_dependency.extend(field.raw_span),
                        (15, _) => option_dependency.extend(field.raw_span),
                        (4, _) => message_type.extend(field.raw_span),
                        (5, _) => enum_type.extend(field.raw_span),
                        (6, _) => service.extend(field.raw_span),
                        (7, _) => extension.extend(field.raw_span),
                        (8, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (9, ::roto::runtime::WireType::LengthDelimited) => source_code_info = field.marker,
                        (12, ::roto::runtime::WireType::LengthDelimited) => syntax = field.marker,
                        (14, ::roto::runtime::WireType::Varint) => {
                            edition = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    package,
                    dependency,
                    public_dependency,
                    weak_dependency,
                    option_dependency,
                    message_type,
                    enum_type,
                    service,
                    extension,
                    options,
                    source_code_info,
                    syntax,
                    edition,
                    has_fields,
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
                ::roto::runtime::RepeatedStrIter::new(self.dependency.slice(self.buf), 3)
            }

            #[inline]
            pub const fn public_dependency(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                ::roto::runtime::RepeatedVarintIter::new(self.public_dependency.slice(self.buf), 10)
            }

            #[inline]
            pub const fn weak_dependency(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                ::roto::runtime::RepeatedVarintIter::new(self.weak_dependency.slice(self.buf), 11)
            }

            #[inline]
            pub const fn option_dependency(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.option_dependency.slice(self.buf), 15)
            }

            #[inline]
            pub const fn message_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::DescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.message_type.slice(self.buf), 4)
            }

            #[inline]
            pub const fn enum_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.enum_type.slice(self.buf), 5)
            }

            #[inline]
            pub const fn service(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::ServiceDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.service.slice(self.buf), 6)
            }

            #[inline]
            pub const fn extension(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.extension.slice(self.buf), 7)
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                Ok(self.edition)
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

        pub struct FileDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FileDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FileDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FileDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FileDescriptorProtoView<'_>) -> &mut Self {
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
                    10 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    11 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    15 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 2048;
                        (prev_set[0] & 2048) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 4096;
                        (prev_set[0] & 4096) == 0
                    }
                    14 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
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
            pub fn write_package(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_dependency(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_public_dependency(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed_public_dependency(&mut self, vals: &[i32]) -> &mut Self {
                self.set_fields[0] |= 8;
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
                self.set_fields[0] |= 16;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed_weak_dependency(&mut self, vals: &[i32]) -> &mut Self {
                self.set_fields[0] |= 16;
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
                self.set_fields[0] |= 32;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_message_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::DescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 64;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::DescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_enum_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 128;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_service<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ServiceDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 256;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::ServiceDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 512;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FileOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 1024;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FileOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_source_code_info<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::SourceCodeInfoBuilder<'_>),
            {
                self.set_fields[0] |= 2048;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::SourceCodeInfoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_syntax(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct DescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            f_2: ::roto::runtime::FieldMarker,
            extension: ::roto::runtime::FieldMarker,
            nested_type: ::roto::runtime::FieldMarker,
            enum_type: ::roto::runtime::FieldMarker,
            extension_range: ::roto::runtime::FieldMarker,
            f_8: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            reserved_range: ::roto::runtime::FieldMarker,
            reserved_name: ::roto::runtime::FieldMarker,
            visibility: crate::google::protobuf::SymbolVisibility,
            has_fields: [u32; 1],
        }

        impl<'a> DescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 6, 3, 4, 5, 8, 7, 9, 10, 11];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut f_2 = ::roto::runtime::FieldMarker::none();
                let mut extension = ::roto::runtime::FieldMarker::none();
                let mut nested_type = ::roto::runtime::FieldMarker::none();
                let mut enum_type = ::roto::runtime::FieldMarker::none();
                let mut extension_range = ::roto::runtime::FieldMarker::none();
                let mut f_8 = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut reserved_range = ::roto::runtime::FieldMarker::none();
                let mut reserved_name = ::roto::runtime::FieldMarker::none();
                let mut visibility: crate::google::protobuf::SymbolVisibility = crate::google::protobuf::SymbolVisibility::VisibilityUnset;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => f_2.extend(field.raw_span),
                        (6, _) => extension.extend(field.raw_span),
                        (3, _) => nested_type.extend(field.raw_span),
                        (4, _) => enum_type.extend(field.raw_span),
                        (5, _) => extension_range.extend(field.raw_span),
                        (8, _) => f_8.extend(field.raw_span),
                        (7, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (9, _) => reserved_range.extend(field.raw_span),
                        (10, _) => reserved_name.extend(field.raw_span),
                        (11, ::roto::runtime::WireType::Varint) => {
                            visibility = <crate::google::protobuf::SymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    f_2,
                    extension,
                    nested_type,
                    enum_type,
                    extension_range,
                    f_8,
                    options,
                    reserved_range,
                    reserved_name,
                    visibility,
                    has_fields,
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
                ::roto::runtime::RepeatedMessageIter::new(self.f_2.slice(self.buf), 2)
            }

            #[inline]
            pub const fn extension(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.extension.slice(self.buf), 6)
            }

            #[inline]
            pub const fn nested_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::DescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.nested_type.slice(self.buf), 3)
            }

            #[inline]
            pub const fn enum_type(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.enum_type.slice(self.buf), 4)
            }

            #[inline]
            pub const fn extension_range(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::descriptor_proto::ExtensionRangeView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.extension_range.slice(self.buf), 5)
            }

            #[inline]
            pub const fn oneof_decl(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OneofDescriptorProtoView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.f_8.slice(self.buf), 8)
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
                ::roto::runtime::RepeatedMessageIter::new(self.reserved_range.slice(self.buf), 9)
            }

            #[inline]
            pub const fn reserved_name(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.reserved_name.slice(self.buf), 10)
            }

            #[inline]
            pub const fn has_visibility(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn visibility(&self) -> Result<crate::google::protobuf::SymbolVisibility, ::roto::runtime::DecodeError> {
                Ok(self.visibility)
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

        pub struct DescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> DescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &DescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !DescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &DescriptorProtoView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    10 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    11 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
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
            pub fn write_field<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FieldDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_nested_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::DescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::DescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_enum_type<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 16;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extension_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::descriptor_proto::ExtensionRangeBuilder<'_>),
            {
                self.set_fields[0] |= 32;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::descriptor_proto::ExtensionRangeBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof_decl<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OneofDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 64;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OneofDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MessageOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 128;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::MessageOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::descriptor_proto::ReservedRangeBuilder<'_>),
            {
                self.set_fields[0] |= 256;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::descriptor_proto::ReservedRangeBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_visibility(&mut self, val: crate::google::protobuf::SymbolVisibility) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod descriptor_proto {
            #[derive(Clone, Copy, Debug)]
            pub struct ExtensionRangeView<'a> {
                buf: &'a [u8],
                start: i32,
                end: i32,
                options: ::roto::runtime::FieldMarker,
                has_fields: [u32; 1],
            }

            impl<'a> ExtensionRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start: i32 = 0i32;
                    let mut end: i32 = 0i32;
                    let mut options = ::roto::runtime::FieldMarker::none();
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => {
                                start = field.val as i32;
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::Varint) => {
                                end = field.val as i32;
                                has_fields[0] |= 2;
                            }
                            (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        start,
                        end,
                        options,
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_start(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.start)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.end)
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

            pub struct ExtensionRangeBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> ExtensionRangeBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &ExtensionRangeView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ExtensionRangeView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &ExtensionRangeView<'_>) -> &mut Self {
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
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_options<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::ExtensionRangeOptionsBuilder<'_>),
                {
                    self.set_fields[0] |= 4;
                    let end_pos = self.writer.cursor();
                    self.writer.with_sub_writer(|sub_writer| {
                        let mut sub_builder = crate::google::protobuf::ExtensionRangeOptionsBuilder::from_writer(sub_writer);
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
            pub struct ReservedRangeView<'a> {
                buf: &'a [u8],
                start: i32,
                end: i32,
                has_fields: [u32; 1],
            }

            impl<'a> ReservedRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start: i32 = 0i32;
                    let mut end: i32 = 0i32;
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => {
                                start = field.val as i32;
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::Varint) => {
                                end = field.val as i32;
                                has_fields[0] |= 2;
                            }
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        start,
                        end,
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_start(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.start)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.end)
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

            pub struct ReservedRangeBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> ReservedRangeBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &ReservedRangeView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ReservedRangeView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &ReservedRangeView<'_>) -> &mut Self {
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
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ExtensionRangeOptionsView<'a> {
            buf: &'a [u8],
            uninterpreted_option: ::roto::runtime::FieldMarker,
            declaration: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
            verification: crate::google::protobuf::extension_range_options::VerificationState,
            has_fields: [u32; 1],
        }

        impl<'a> ExtensionRangeOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[999, 2, 50, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut declaration = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut verification: crate::google::protobuf::extension_range_options::VerificationState = crate::google::protobuf::extension_range_options::VerificationState::Unverified;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        (2, _) => declaration.extend(field.raw_span),
                        (50, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => {
                            verification = <crate::google::protobuf::extension_range_options::VerificationState as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    uninterpreted_option,
                    declaration,
                    features,
                    verification,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
            }

            #[inline]
            pub const fn declaration(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::extension_range_options::DeclarationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.declaration.slice(self.buf), 2)
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn verification(&self) -> Result<crate::google::protobuf::extension_range_options::VerificationState, ::roto::runtime::DecodeError> {
                Ok(self.verification)
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

        pub struct ExtensionRangeOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ExtensionRangeOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ExtensionRangeOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ExtensionRangeOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ExtensionRangeOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    999 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    2 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    50 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_declaration<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::extension_range_options::DeclarationBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::extension_range_options::DeclarationBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(50, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_verification(&mut self, val: crate::google::protobuf::extension_range_options::VerificationState) -> &mut Self {
                self.set_fields[0] |= 8;
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
                number: i32,
                full_name: ::roto::runtime::FieldMarker,
                r#type: ::roto::runtime::FieldMarker,
                reserved: bool,
                repeated: bool,
                has_fields: [u32; 1],
            }

            impl<'a> DeclarationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 5, 6];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut number: i32 = 0i32;
                    let mut full_name = ::roto::runtime::FieldMarker::none();
                    let mut r#type = ::roto::runtime::FieldMarker::none();
                    let mut reserved: bool = false;
                    let mut repeated: bool = false;
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => {
                                number = field.val as i32;
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::LengthDelimited) => full_name = field.marker,
                            (3, ::roto::runtime::WireType::LengthDelimited) => r#type = field.marker,
                            (5, ::roto::runtime::WireType::Varint) => {
                                reserved = field.val != 0;
                                has_fields[0] |= 2;
                            }
                            (6, ::roto::runtime::WireType::Varint) => {
                                repeated = field.val != 0;
                                has_fields[0] |= 4;
                            }
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
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_number(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.number)
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
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn reserved(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    Ok(self.reserved)
                }

                #[inline]
                pub const fn has_repeated(&self) -> bool {
                    (self.has_fields[0] & 4) != 0
                }

                #[inline]
                pub const fn repeated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    Ok(self.repeated)
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

            pub struct DeclarationBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> DeclarationBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &DeclarationView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !DeclarationView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &DeclarationView<'_>) -> &mut Self {
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
                        5 => {
                            self.set_fields[0] |= 8;
                            (prev_set[0] & 8) == 0
                        }
                        6 => {
                            self.set_fields[0] |= 16;
                            (prev_set[0] & 16) == 0
                        }
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_number(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_full_name(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_type(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 4;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_reserved(&mut self, val: bool) -> &mut Self {
                    self.set_fields[0] |= 8;
                    let _ = self.writer.write_bool(val);
                    let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_repeated(&mut self, val: bool) -> &mut Self {
                    self.set_fields[0] |= 16;
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
            number: i32,
            label: crate::google::protobuf::field_descriptor_proto::Label,
            r#type: crate::google::protobuf::field_descriptor_proto::Type,
            type_name: ::roto::runtime::FieldMarker,
            extendee: ::roto::runtime::FieldMarker,
            default_value: ::roto::runtime::FieldMarker,
            f_9: i32,
            json_name: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            proto3_optional: bool,
            has_fields: [u32; 1],
        }

        impl<'a> FieldDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 3, 4, 5, 6, 2, 7, 9, 10, 8, 17];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut number: i32 = 0i32;
                let mut label: crate::google::protobuf::field_descriptor_proto::Label = crate::google::protobuf::field_descriptor_proto::Label::LabelOptional;
                let mut r#type: crate::google::protobuf::field_descriptor_proto::Type = crate::google::protobuf::field_descriptor_proto::Type::TypeDouble;
                let mut type_name = ::roto::runtime::FieldMarker::none();
                let mut extendee = ::roto::runtime::FieldMarker::none();
                let mut default_value = ::roto::runtime::FieldMarker::none();
                let mut f_9: i32 = 0i32;
                let mut json_name = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut proto3_optional: bool = false;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => {
                            number = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (4, ::roto::runtime::WireType::Varint) => {
                            label = <crate::google::protobuf::field_descriptor_proto::Label as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 2;
                        }
                        (5, ::roto::runtime::WireType::Varint) => {
                            r#type = <crate::google::protobuf::field_descriptor_proto::Type as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 4;
                        }
                        (6, ::roto::runtime::WireType::LengthDelimited) => type_name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => extendee = field.marker,
                        (7, ::roto::runtime::WireType::LengthDelimited) => default_value = field.marker,
                        (9, ::roto::runtime::WireType::Varint) => {
                            f_9 = field.val as i32;
                            has_fields[0] |= 8;
                        }
                        (10, ::roto::runtime::WireType::LengthDelimited) => json_name = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (17, ::roto::runtime::WireType::Varint) => {
                            proto3_optional = field.val != 0;
                            has_fields[0] |= 16;
                        }
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
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.number)
            }

            #[inline]
            pub const fn has_label(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn label(&self) -> Result<crate::google::protobuf::field_descriptor_proto::Label, ::roto::runtime::DecodeError> {
                Ok(self.label)
            }

            #[inline]
            pub const fn has_type(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn r#type(&self) -> Result<crate::google::protobuf::field_descriptor_proto::Type, ::roto::runtime::DecodeError> {
                Ok(self.r#type)
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
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn oneof_index(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.f_9)
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
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn proto3_optional(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.proto3_optional)
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

        pub struct FieldDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FieldDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FieldDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FieldDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FieldDescriptorProtoView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    2 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    10 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    17 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
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
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_label(&mut self, val: crate::google::protobuf::field_descriptor_proto::Label) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_type(&mut self, val: crate::google::protobuf::field_descriptor_proto::Type) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_type_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_extendee(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_default_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof_index(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_json_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 512;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FieldOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_proto3_optional(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1024;
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

        pub struct OneofDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> OneofDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &OneofDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !OneofDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &OneofDescriptorProtoView<'_>) -> &mut Self {
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
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OneofOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OneofOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
            value: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            reserved_range: ::roto::runtime::FieldMarker,
            reserved_name: ::roto::runtime::FieldMarker,
            visibility: crate::google::protobuf::SymbolVisibility,
            has_fields: [u32; 1],
        }

        impl<'a> EnumDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut value = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut reserved_range = ::roto::runtime::FieldMarker::none();
                let mut reserved_name = ::roto::runtime::FieldMarker::none();
                let mut visibility: crate::google::protobuf::SymbolVisibility = crate::google::protobuf::SymbolVisibility::VisibilityUnset;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => value.extend(field.raw_span),
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (4, _) => reserved_range.extend(field.raw_span),
                        (5, _) => reserved_name.extend(field.raw_span),
                        (6, ::roto::runtime::WireType::Varint) => {
                            visibility = <crate::google::protobuf::SymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    value,
                    options,
                    reserved_range,
                    reserved_name,
                    visibility,
                    has_fields,
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
                ::roto::runtime::RepeatedMessageIter::new(self.value.slice(self.buf), 2)
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
                ::roto::runtime::RepeatedMessageIter::new(self.reserved_range.slice(self.buf), 4)
            }

            #[inline]
            pub const fn reserved_name(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.reserved_name.slice(self.buf), 5)
            }

            #[inline]
            pub const fn has_visibility(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn visibility(&self) -> Result<crate::google::protobuf::SymbolVisibility, ::roto::runtime::DecodeError> {
                Ok(self.visibility)
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

        pub struct EnumDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumDescriptorProtoView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
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
            pub fn write_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumValueDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumValueDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_range<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::enum_descriptor_proto::EnumReservedRangeBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::enum_descriptor_proto::EnumReservedRangeBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_reserved_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_visibility(&mut self, val: crate::google::protobuf::SymbolVisibility) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod enum_descriptor_proto {
            #[derive(Clone, Copy, Debug)]
            pub struct EnumReservedRangeView<'a> {
                buf: &'a [u8],
                start: i32,
                end: i32,
                has_fields: [u32; 1],
            }

            impl<'a> EnumReservedRangeView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut start: i32 = 0i32;
                    let mut end: i32 = 0i32;
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => {
                                start = field.val as i32;
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::Varint) => {
                                end = field.val as i32;
                                has_fields[0] |= 2;
                            }
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        start,
                        end,
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_start(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn start(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.start)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.end)
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

            pub struct EnumReservedRangeBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> EnumReservedRangeBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &EnumReservedRangeView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumReservedRangeView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &EnumReservedRangeView<'_>) -> &mut Self {
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
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_start(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 2;
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
            number: i32,
            options: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> EnumValueDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut number: i32 = 0i32;
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => {
                            number = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    number,
                    options,
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.number)
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

        pub struct EnumValueDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumValueDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumValueDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumValueDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumValueDescriptorProtoView<'_>) -> &mut Self {
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
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumValueOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumValueOptionsBuilder::from_writer(sub_writer);
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
        pub struct ServiceDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            method: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
        }

        impl<'a> ServiceDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut method = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => method.extend(field.raw_span),
                        (3, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    method,
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
                ::roto::runtime::RepeatedMessageIter::new(self.method.slice(self.buf), 2)
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

        pub struct ServiceDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ServiceDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ServiceDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ServiceDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ServiceDescriptorProtoView<'_>) -> &mut Self {
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
            pub fn write_method<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodDescriptorProtoBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::MethodDescriptorProtoBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ServiceOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::ServiceOptionsBuilder::from_writer(sub_writer);
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
        pub struct MethodDescriptorProtoView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            input_type: ::roto::runtime::FieldMarker,
            output_type: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            client_streaming: bool,
            server_streaming: bool,
            has_fields: [u32; 1],
        }

        impl<'a> MethodDescriptorProtoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut input_type = ::roto::runtime::FieldMarker::none();
                let mut output_type = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut client_streaming: bool = false;
                let mut server_streaming: bool = false;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => input_type = field.marker,
                        (3, ::roto::runtime::WireType::LengthDelimited) => output_type = field.marker,
                        (4, ::roto::runtime::WireType::LengthDelimited) => options = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => {
                            client_streaming = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            server_streaming = field.val != 0;
                            has_fields[0] |= 2;
                        }
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
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn client_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.client_streaming)
            }

            #[inline]
            pub const fn has_server_streaming(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn server_streaming(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.server_streaming)
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

        pub struct MethodDescriptorProtoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> MethodDescriptorProtoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &MethodDescriptorProtoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MethodDescriptorProtoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &MethodDescriptorProtoView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
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
            pub fn write_input_type(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_output_type(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::MethodOptionsBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::MethodOptionsBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_client_streaming(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_server_streaming(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 32;
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
            java_multiple_files: bool,
            java_generate_equals_and_hash: bool,
            java_string_check_utf8: bool,
            optimize_for: crate::google::protobuf::file_options::OptimizeMode,
            go_package: ::roto::runtime::FieldMarker,
            cc_generic_services: bool,
            java_generic_services: bool,
            py_generic_services: bool,
            deprecated: bool,
            cc_enable_arenas: bool,
            objc_class_prefix: ::roto::runtime::FieldMarker,
            csharp_namespace: ::roto::runtime::FieldMarker,
            swift_prefix: ::roto::runtime::FieldMarker,
            php_class_prefix: ::roto::runtime::FieldMarker,
            php_namespace: ::roto::runtime::FieldMarker,
            php_metadata_namespace: ::roto::runtime::FieldMarker,
            ruby_package: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> FileOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 8, 10, 20, 27, 9, 11, 16, 17, 18, 23, 31, 36, 37, 39, 40, 41, 44, 45, 50, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut java_package = ::roto::runtime::FieldMarker::none();
                let mut java_outer_classname = ::roto::runtime::FieldMarker::none();
                let mut java_multiple_files: bool = false;
                let mut java_generate_equals_and_hash: bool = false;
                let mut java_string_check_utf8: bool = false;
                let mut optimize_for: crate::google::protobuf::file_options::OptimizeMode = crate::google::protobuf::file_options::OptimizeMode::Speed;
                let mut go_package = ::roto::runtime::FieldMarker::none();
                let mut cc_generic_services: bool = false;
                let mut java_generic_services: bool = false;
                let mut py_generic_services: bool = false;
                let mut deprecated: bool = false;
                let mut cc_enable_arenas: bool = true;
                let mut objc_class_prefix = ::roto::runtime::FieldMarker::none();
                let mut csharp_namespace = ::roto::runtime::FieldMarker::none();
                let mut swift_prefix = ::roto::runtime::FieldMarker::none();
                let mut php_class_prefix = ::roto::runtime::FieldMarker::none();
                let mut php_namespace = ::roto::runtime::FieldMarker::none();
                let mut php_metadata_namespace = ::roto::runtime::FieldMarker::none();
                let mut ruby_package = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => java_package = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => java_outer_classname = field.marker,
                        (10, ::roto::runtime::WireType::Varint) => {
                            java_multiple_files = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (20, ::roto::runtime::WireType::Varint) => {
                            java_generate_equals_and_hash = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (27, ::roto::runtime::WireType::Varint) => {
                            java_string_check_utf8 = field.val != 0;
                            has_fields[0] |= 4;
                        }
                        (9, ::roto::runtime::WireType::Varint) => {
                            optimize_for = <crate::google::protobuf::file_options::OptimizeMode as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 8;
                        }
                        (11, ::roto::runtime::WireType::LengthDelimited) => go_package = field.marker,
                        (16, ::roto::runtime::WireType::Varint) => {
                            cc_generic_services = field.val != 0;
                            has_fields[0] |= 16;
                        }
                        (17, ::roto::runtime::WireType::Varint) => {
                            java_generic_services = field.val != 0;
                            has_fields[0] |= 32;
                        }
                        (18, ::roto::runtime::WireType::Varint) => {
                            py_generic_services = field.val != 0;
                            has_fields[0] |= 64;
                        }
                        (23, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 128;
                        }
                        (31, ::roto::runtime::WireType::Varint) => {
                            cc_enable_arenas = field.val != 0;
                            has_fields[0] |= 256;
                        }
                        (36, ::roto::runtime::WireType::LengthDelimited) => objc_class_prefix = field.marker,
                        (37, ::roto::runtime::WireType::LengthDelimited) => csharp_namespace = field.marker,
                        (39, ::roto::runtime::WireType::LengthDelimited) => swift_prefix = field.marker,
                        (40, ::roto::runtime::WireType::LengthDelimited) => php_class_prefix = field.marker,
                        (41, ::roto::runtime::WireType::LengthDelimited) => php_namespace = field.marker,
                        (44, ::roto::runtime::WireType::LengthDelimited) => php_metadata_namespace = field.marker,
                        (45, ::roto::runtime::WireType::LengthDelimited) => ruby_package = field.marker,
                        (50, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
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
                    uninterpreted_option,
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn java_multiple_files(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.java_multiple_files)
            }

            #[inline]
            pub const fn has_java_generate_equals_and_hash(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn java_generate_equals_and_hash(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.java_generate_equals_and_hash)
            }

            #[inline]
            pub const fn has_java_string_check_utf8(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn java_string_check_utf8(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.java_string_check_utf8)
            }

            #[inline]
            pub const fn has_optimize_for(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn optimize_for(&self) -> Result<crate::google::protobuf::file_options::OptimizeMode, ::roto::runtime::DecodeError> {
                Ok(self.optimize_for)
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
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn cc_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.cc_generic_services)
            }

            #[inline]
            pub const fn has_java_generic_services(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn java_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.java_generic_services)
            }

            #[inline]
            pub const fn has_py_generic_services(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn py_generic_services(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.py_generic_services)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn has_cc_enable_arenas(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn cc_enable_arenas(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.cc_enable_arenas)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct FileOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FileOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FileOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FileOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FileOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    10 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    20 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    27 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    11 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    17 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    18 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    23 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
                    }
                    31 => {
                        self.set_fields[0] |= 2048;
                        (prev_set[0] & 2048) == 0
                    }
                    36 => {
                        self.set_fields[0] |= 4096;
                        (prev_set[0] & 4096) == 0
                    }
                    37 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    39 => {
                        self.set_fields[0] |= 16384;
                        (prev_set[0] & 16384) == 0
                    }
                    40 => {
                        self.set_fields[0] |= 32768;
                        (prev_set[0] & 32768) == 0
                    }
                    41 => {
                        self.set_fields[0] |= 65536;
                        (prev_set[0] & 65536) == 0
                    }
                    44 => {
                        self.set_fields[0] |= 131072;
                        (prev_set[0] & 131072) == 0
                    }
                    45 => {
                        self.set_fields[0] |= 262144;
                        (prev_set[0] & 262144) == 0
                    }
                    50 => {
                        self.set_fields[0] |= 524288;
                        (prev_set[0] & 524288) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 1048576;
                        (prev_set[0] & 1048576) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_java_package(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_java_outer_classname(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_java_multiple_files(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_generate_equals_and_hash(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_string_check_utf8(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(27, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_optimize_for(&mut self, val: crate::google::protobuf::file_options::OptimizeMode) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_go_package(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_cc_generic_services(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_java_generic_services(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_py_generic_services(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(18, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_cc_enable_arenas(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 2048;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(31, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_objc_class_prefix(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(36, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_csharp_namespace(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(37, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_swift_prefix(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16384;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(39, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_class_prefix(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 32768;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(40, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_namespace(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 65536;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(41, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_php_metadata_namespace(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 131072;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(44, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_ruby_package(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 262144;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(45, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 524288;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(50, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 1048576;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
            message_set_wire_format: bool,
            no_standard_descriptor_accessor: bool,
            deprecated: bool,
            map_entry: bool,
            deprecated_legacy_json_field_conflicts: bool,
            features: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> MessageOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 7, 11, 12, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut message_set_wire_format: bool = false;
                let mut no_standard_descriptor_accessor: bool = false;
                let mut deprecated: bool = false;
                let mut map_entry: bool = false;
                let mut deprecated_legacy_json_field_conflicts: bool = false;
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            message_set_wire_format = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            no_standard_descriptor_accessor = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 4;
                        }
                        (7, ::roto::runtime::WireType::Varint) => {
                            map_entry = field.val != 0;
                            has_fields[0] |= 8;
                        }
                        (11, ::roto::runtime::WireType::Varint) => {
                            deprecated_legacy_json_field_conflicts = field.val != 0;
                            has_fields[0] |= 16;
                        }
                        (12, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
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
                    uninterpreted_option,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_message_set_wire_format(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn message_set_wire_format(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.message_set_wire_format)
            }

            #[inline]
            pub const fn has_no_standard_descriptor_accessor(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn no_standard_descriptor_accessor(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.no_standard_descriptor_accessor)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn has_map_entry(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn map_entry(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.map_entry)
            }

            #[inline]
            pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated_legacy_json_field_conflicts)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct MessageOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> MessageOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &MessageOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MessageOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &MessageOptionsView<'_>) -> &mut Self {
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
                    7 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    11 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_message_set_wire_format(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_no_standard_descriptor_accessor(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_map_entry(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated_legacy_json_field_conflicts(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 32;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 64;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FieldOptionsView<'a> {
            buf: &'a [u8],
            ctype: crate::google::protobuf::field_options::CType,
            packed: bool,
            jstype: crate::google::protobuf::field_options::JSType,
            lazy: bool,
            unverified_lazy: bool,
            deprecated: bool,
            weak: bool,
            debug_redact: bool,
            retention: crate::google::protobuf::field_options::OptionRetention,
            targets: ::roto::runtime::FieldMarker,
            edition_defaults: ::roto::runtime::FieldMarker,
            features: ::roto::runtime::FieldMarker,
            feature_support: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> FieldOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 6, 5, 15, 3, 10, 16, 17, 19, 20, 21, 22, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut ctype: crate::google::protobuf::field_options::CType = crate::google::protobuf::field_options::CType::String;
                let mut packed: bool = false;
                let mut jstype: crate::google::protobuf::field_options::JSType = crate::google::protobuf::field_options::JSType::JsNormal;
                let mut lazy: bool = false;
                let mut unverified_lazy: bool = false;
                let mut deprecated: bool = false;
                let mut weak: bool = false;
                let mut debug_redact: bool = false;
                let mut retention: crate::google::protobuf::field_options::OptionRetention = crate::google::protobuf::field_options::OptionRetention::RetentionUnknown;
                let mut targets = ::roto::runtime::FieldMarker::none();
                let mut edition_defaults = ::roto::runtime::FieldMarker::none();
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut feature_support = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            ctype = <crate::google::protobuf::field_options::CType as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            packed = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            jstype = <crate::google::protobuf::field_options::JSType as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 4;
                        }
                        (5, ::roto::runtime::WireType::Varint) => {
                            lazy = field.val != 0;
                            has_fields[0] |= 8;
                        }
                        (15, ::roto::runtime::WireType::Varint) => {
                            unverified_lazy = field.val != 0;
                            has_fields[0] |= 16;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 32;
                        }
                        (10, ::roto::runtime::WireType::Varint) => {
                            weak = field.val != 0;
                            has_fields[0] |= 64;
                        }
                        (16, ::roto::runtime::WireType::Varint) => {
                            debug_redact = field.val != 0;
                            has_fields[0] |= 128;
                        }
                        (17, ::roto::runtime::WireType::Varint) => {
                            retention = <crate::google::protobuf::field_options::OptionRetention as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 256;
                        }
                        (19, _) => targets.extend(field.raw_span),
                        (20, _) => edition_defaults.extend(field.raw_span),
                        (21, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (22, ::roto::runtime::WireType::LengthDelimited) => feature_support = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
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
                    targets,
                    edition_defaults,
                    features,
                    feature_support,
                    uninterpreted_option,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_ctype(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn ctype(&self) -> Result<crate::google::protobuf::field_options::CType, ::roto::runtime::DecodeError> {
                Ok(self.ctype)
            }

            #[inline]
            pub const fn has_packed(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn packed(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.packed)
            }

            #[inline]
            pub const fn has_jstype(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn jstype(&self) -> Result<crate::google::protobuf::field_options::JSType, ::roto::runtime::DecodeError> {
                Ok(self.jstype)
            }

            #[inline]
            pub const fn has_lazy(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn lazy(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.lazy)
            }

            #[inline]
            pub const fn has_unverified_lazy(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn unverified_lazy(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.unverified_lazy)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn has_weak(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn weak(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.weak)
            }

            #[inline]
            pub const fn has_debug_redact(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn debug_redact(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.debug_redact)
            }

            #[inline]
            pub const fn has_retention(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn retention(&self) -> Result<crate::google::protobuf::field_options::OptionRetention, ::roto::runtime::DecodeError> {
                Ok(self.retention)
            }

            #[inline]
            pub const fn targets(&self) -> ::roto::runtime::RepeatedVarintIter<'a, crate::google::protobuf::field_options::OptionTargetType> {
                ::roto::runtime::RepeatedVarintIter::new(self.targets.slice(self.buf), 19)
            }

            #[inline]
            pub const fn edition_defaults(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::field_options::EditionDefaultView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.edition_defaults.slice(self.buf), 20)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct FieldOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FieldOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FieldOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FieldOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FieldOptionsView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    15 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    10 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    17 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    19 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    20 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
                    }
                    21 => {
                        self.set_fields[0] |= 2048;
                        (prev_set[0] & 2048) == 0
                    }
                    22 => {
                        self.set_fields[0] |= 4096;
                        (prev_set[0] & 4096) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_ctype(&mut self, val: crate::google::protobuf::field_options::CType) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_jstype(&mut self, val: crate::google::protobuf::field_options::JSType) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_lazy(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_unverified_lazy(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_weak(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_debug_redact(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_retention(&mut self, val: crate::google::protobuf::field_options::OptionRetention) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_targets(&mut self, val: crate::google::protobuf::field_options::OptionTargetType) -> &mut Self {
                self.set_fields[0] |= 512;
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
                self.set_fields[0] |= 512;
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
                F: FnOnce(&mut crate::google::protobuf::field_options::EditionDefaultBuilder<'_>),
            {
                self.set_fields[0] |= 1024;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::field_options::EditionDefaultBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_edition_default<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::EditionDefaultBuilder<'_>),
            {
                self.write_edition_defaults(f)
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 2048;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(21, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_feature_support<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::FeatureSupportBuilder<'_>),
            {
                self.set_fields[0] |= 4096;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::field_options::FeatureSupportBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 8192;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
                edition: crate::google::protobuf::Edition,
                value: ::roto::runtime::FieldMarker,
                has_fields: [u32; 1],
            }

            impl<'a> EditionDefaultView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[3, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                    let mut value = ::roto::runtime::FieldMarker::none();
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (3, ::roto::runtime::WireType::Varint) => {
                                edition = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        edition,
                        value,
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_edition(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    Ok(self.edition)
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

            pub struct EditionDefaultBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> EditionDefaultBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &EditionDefaultView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EditionDefaultView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &EditionDefaultView<'_>) -> &mut Self {
                    let prev_set = self.set_fields;
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                        3 => {
                            self.set_fields[0] |= 1;
                            (prev_set[0] & 1) == 0
                        }
                        2 => {
                            self.set_fields[0] |= 2;
                            (prev_set[0] & 2) == 0
                        }
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_value(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }

            #[derive(Clone, Copy, Debug)]
            pub struct FeatureSupportView<'a> {
                buf: &'a [u8],
                edition_introduced: crate::google::protobuf::Edition,
                edition_deprecated: crate::google::protobuf::Edition,
                deprecation_warning: ::roto::runtime::FieldMarker,
                edition_removed: crate::google::protobuf::Edition,
                removal_error: ::roto::runtime::FieldMarker,
                has_fields: [u32; 1],
            }

            impl<'a> FeatureSupportView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition_introduced: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                    let mut edition_deprecated: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                    let mut deprecation_warning = ::roto::runtime::FieldMarker::none();
                    let mut edition_removed: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                    let mut removal_error = ::roto::runtime::FieldMarker::none();
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::Varint) => {
                                edition_introduced = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 1;
                            }
                            (2, ::roto::runtime::WireType::Varint) => {
                                edition_deprecated = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 2;
                            }
                            (3, ::roto::runtime::WireType::LengthDelimited) => deprecation_warning = field.marker,
                            (4, ::roto::runtime::WireType::Varint) => {
                                edition_removed = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 4;
                            }
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
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_edition_introduced(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn edition_introduced(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    Ok(self.edition_introduced)
                }

                #[inline]
                pub const fn has_edition_deprecated(&self) -> bool {
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn edition_deprecated(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    Ok(self.edition_deprecated)
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
                    (self.has_fields[0] & 4) != 0
                }

                #[inline]
                pub const fn edition_removed(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    Ok(self.edition_removed)
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

            pub struct FeatureSupportBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> FeatureSupportBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &FeatureSupportView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FeatureSupportView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &FeatureSupportView<'_>) -> &mut Self {
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
                pub fn write_edition_introduced(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_edition_deprecated(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_deprecation_warning(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 4;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_edition_removed(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    self.set_fields[0] |= 8;
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_removal_error(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 16;
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
            uninterpreted_option: ::roto::runtime::FieldMarker,
        }

        impl<'a> OneofOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    features,
                    uninterpreted_option,
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct OneofOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> OneofOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &OneofOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !OneofOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &OneofOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumOptionsView<'a> {
            buf: &'a [u8],
            allow_alias: bool,
            deprecated: bool,
            deprecated_legacy_json_field_conflicts: bool,
            features: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> EnumOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 6, 7, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut allow_alias: bool = false;
                let mut deprecated: bool = false;
                let mut deprecated_legacy_json_field_conflicts: bool = false;
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (2, ::roto::runtime::WireType::Varint) => {
                            allow_alias = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            deprecated_legacy_json_field_conflicts = field.val != 0;
                            has_fields[0] |= 4;
                        }
                        (7, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    allow_alias,
                    deprecated,
                    deprecated_legacy_json_field_conflicts,
                    features,
                    uninterpreted_option,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_allow_alias(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn allow_alias(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.allow_alias)
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn has_deprecated_legacy_json_field_conflicts(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn deprecated_legacy_json_field_conflicts(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated_legacy_json_field_conflicts)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct EnumOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    2 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_allow_alias(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_deprecated_legacy_json_field_conflicts(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 16;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumValueOptionsView<'a> {
            buf: &'a [u8],
            deprecated: bool,
            features: ::roto::runtime::FieldMarker,
            debug_redact: bool,
            feature_support: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> EnumValueOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut deprecated: bool = false;
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut debug_redact: bool = false;
                let mut feature_support = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (3, ::roto::runtime::WireType::Varint) => {
                            debug_redact = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (4, ::roto::runtime::WireType::LengthDelimited) => feature_support = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    deprecated,
                    features,
                    debug_redact,
                    feature_support,
                    uninterpreted_option,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
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
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn debug_redact(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.debug_redact)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct EnumValueOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumValueOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumValueOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumValueOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumValueOptionsView<'_>) -> &mut Self {
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
                    999 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_debug_redact(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_feature_support<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::field_options::FeatureSupportBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::field_options::FeatureSupportBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 16;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
            deprecated: bool,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> ServiceOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[34, 33, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut deprecated: bool = false;
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (34, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (33, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    features,
                    deprecated,
                    uninterpreted_option,
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn uninterpreted_option(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::UninterpretedOptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct ServiceOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ServiceOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ServiceOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ServiceOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ServiceOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    34 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    33 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(34, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(33, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(999, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct MethodOptionsView<'a> {
            buf: &'a [u8],
            deprecated: bool,
            idempotency_level: crate::google::protobuf::method_options::IdempotencyLevel,
            features: ::roto::runtime::FieldMarker,
            uninterpreted_option: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> MethodOptionsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[33, 34, 35, 999];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut deprecated: bool = false;
                let mut idempotency_level: crate::google::protobuf::method_options::IdempotencyLevel = crate::google::protobuf::method_options::IdempotencyLevel::IdempotencyUnknown;
                let mut features = ::roto::runtime::FieldMarker::none();
                let mut uninterpreted_option = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (33, ::roto::runtime::WireType::Varint) => {
                            deprecated = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (34, ::roto::runtime::WireType::Varint) => {
                            idempotency_level = <crate::google::protobuf::method_options::IdempotencyLevel as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 2;
                        }
                        (35, ::roto::runtime::WireType::LengthDelimited) => features = field.marker,
                        (999, _) => uninterpreted_option.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    deprecated,
                    idempotency_level,
                    features,
                    uninterpreted_option,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_deprecated(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn deprecated(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.deprecated)
            }

            #[inline]
            pub const fn has_idempotency_level(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn idempotency_level(&self) -> Result<crate::google::protobuf::method_options::IdempotencyLevel, ::roto::runtime::DecodeError> {
                Ok(self.idempotency_level)
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
                ::roto::runtime::RepeatedMessageIter::new(self.uninterpreted_option.slice(self.buf), 999)
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

        pub struct MethodOptionsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> MethodOptionsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &MethodOptionsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MethodOptionsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &MethodOptionsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    33 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    34 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    35 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    999 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_deprecated(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(33, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_idempotency_level(&mut self, val: crate::google::protobuf::method_options::IdempotencyLevel) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(34, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_features<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(35, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_uninterpreted_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::UninterpretedOptionBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::UninterpretedOptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
            name: ::roto::runtime::FieldMarker,
            identifier_value: ::roto::runtime::FieldMarker,
            positive_int_value: u64,
            negative_int_value: i64,
            double_value: f64,
            string_value: ::roto::runtime::FieldMarker,
            aggregate_value: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> UninterpretedOptionView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[2, 3, 4, 5, 6, 7, 8];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut identifier_value = ::roto::runtime::FieldMarker::none();
                let mut positive_int_value: u64 = 0u64;
                let mut negative_int_value: i64 = 0i64;
                let mut double_value: f64 = 0.0f64;
                let mut string_value = ::roto::runtime::FieldMarker::none();
                let mut aggregate_value = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (2, _) => name.extend(field.raw_span),
                        (3, ::roto::runtime::WireType::LengthDelimited) => identifier_value = field.marker,
                        (4, ::roto::runtime::WireType::Varint) => {
                            positive_int_value = field.val;
                            has_fields[0] |= 1;
                        }
                        (5, ::roto::runtime::WireType::Varint) => {
                            negative_int_value = field.val as i64;
                            has_fields[0] |= 2;
                        }
                        (6, ::roto::runtime::WireType::Fixed64) => {
                            double_value = f64::from_bits(field.val);
                            has_fields[0] |= 4;
                        }
                        (7, ::roto::runtime::WireType::LengthDelimited) => string_value = field.marker,
                        (8, ::roto::runtime::WireType::LengthDelimited) => aggregate_value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    identifier_value,
                    positive_int_value,
                    negative_int_value,
                    double_value,
                    string_value,
                    aggregate_value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn name(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::uninterpreted_option::NamePartView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.name.slice(self.buf), 2)
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn positive_int_value(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.positive_int_value)
            }

            #[inline]
            pub const fn has_negative_int_value(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn negative_int_value(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.negative_int_value)
            }

            #[inline]
            pub const fn has_double_value(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn double_value(&self) -> Result<f64, ::roto::runtime::DecodeError> {
                Ok(self.double_value)
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

        pub struct UninterpretedOptionBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> UninterpretedOptionBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &UninterpretedOptionView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !UninterpretedOptionView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &UninterpretedOptionView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    2 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_name<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::uninterpreted_option::NamePartBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::uninterpreted_option::NamePartBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_identifier_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_positive_int_value(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_negative_int_value(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_double_value(&mut self, val: f64) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_f64(val);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_string_value(&mut self, val: &[u8]) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_length_delimited(val);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_aggregate_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 64;
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
                is_extension: bool,
                has_fields: [u32; 1],
            }

            impl<'a> NamePartView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut name_part = ::roto::runtime::FieldMarker::none();
                    let mut is_extension: bool = false;
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::LengthDelimited) => name_part = field.marker,
                            (2, ::roto::runtime::WireType::Varint) => {
                                is_extension = field.val != 0;
                                has_fields[0] |= 1;
                            }
                            _ => {}
                        }
                    }

                    if !name_part.is_present() {
                        return Err(::roto::runtime::DecodeError::MissingRequiredField("NamePart.name_part"));
                    }
                    if (has_fields[0] & 1) == 0 {
                        return Err(::roto::runtime::DecodeError::MissingRequiredField("NamePart.is_extension"));
                    }

                    Ok(Self {
                        buf,
                        name_part,
                        is_extension,
                        has_fields,
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
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn is_extension(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                    Ok(self.is_extension)
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

            pub struct NamePartBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> NamePartBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &NamePartView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !NamePartView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &NamePartView<'_>) -> &mut Self {
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
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_name_part(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_is_extension(&mut self, val: bool) -> &mut Self {
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_bool(val);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FeatureSetView<'a> {
            buf: &'a [u8],
            field_presence: crate::google::protobuf::feature_set::FieldPresence,
            enum_type: crate::google::protobuf::feature_set::EnumType,
            repeated_field_encoding: crate::google::protobuf::feature_set::RepeatedFieldEncoding,
            utf8_validation: crate::google::protobuf::feature_set::Utf8Validation,
            message_encoding: crate::google::protobuf::feature_set::MessageEncoding,
            json_format: crate::google::protobuf::feature_set::JsonFormat,
            enforce_naming_style: crate::google::protobuf::feature_set::EnforceNamingStyle,
            default_symbol_visibility: crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility,
            enforce_proto_limits: crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits,
            has_fields: [u32; 1],
        }

        impl<'a> FeatureSetView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field_presence: crate::google::protobuf::feature_set::FieldPresence = crate::google::protobuf::feature_set::FieldPresence::FieldPresenceUnknown;
                let mut enum_type: crate::google::protobuf::feature_set::EnumType = crate::google::protobuf::feature_set::EnumType::EnumTypeUnknown;
                let mut repeated_field_encoding: crate::google::protobuf::feature_set::RepeatedFieldEncoding = crate::google::protobuf::feature_set::RepeatedFieldEncoding::RepeatedFieldEncodingUnknown;
                let mut utf8_validation: crate::google::protobuf::feature_set::Utf8Validation = crate::google::protobuf::feature_set::Utf8Validation::Utf8ValidationUnknown;
                let mut message_encoding: crate::google::protobuf::feature_set::MessageEncoding = crate::google::protobuf::feature_set::MessageEncoding::MessageEncodingUnknown;
                let mut json_format: crate::google::protobuf::feature_set::JsonFormat = crate::google::protobuf::feature_set::JsonFormat::JsonFormatUnknown;
                let mut enforce_naming_style: crate::google::protobuf::feature_set::EnforceNamingStyle = crate::google::protobuf::feature_set::EnforceNamingStyle::EnforceNamingStyleUnknown;
                let mut default_symbol_visibility: crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility = crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility::DefaultSymbolVisibilityUnknown;
                let mut enforce_proto_limits: crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits = crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits::ProtoLimitsUnknown;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            field_presence = <crate::google::protobuf::feature_set::FieldPresence as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            enum_type = <crate::google::protobuf::feature_set::EnumType as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 2;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            repeated_field_encoding = <crate::google::protobuf::feature_set::RepeatedFieldEncoding as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 4;
                        }
                        (4, ::roto::runtime::WireType::Varint) => {
                            utf8_validation = <crate::google::protobuf::feature_set::Utf8Validation as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 8;
                        }
                        (5, ::roto::runtime::WireType::Varint) => {
                            message_encoding = <crate::google::protobuf::feature_set::MessageEncoding as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 16;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            json_format = <crate::google::protobuf::feature_set::JsonFormat as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 32;
                        }
                        (7, ::roto::runtime::WireType::Varint) => {
                            enforce_naming_style = <crate::google::protobuf::feature_set::EnforceNamingStyle as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 64;
                        }
                        (8, ::roto::runtime::WireType::Varint) => {
                            default_symbol_visibility = <crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 128;
                        }
                        (9, ::roto::runtime::WireType::Varint) => {
                            enforce_proto_limits = <crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 256;
                        }
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
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_field_presence(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn field_presence(&self) -> Result<crate::google::protobuf::feature_set::FieldPresence, ::roto::runtime::DecodeError> {
                Ok(self.field_presence)
            }

            #[inline]
            pub const fn has_enum_type(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn enum_type(&self) -> Result<crate::google::protobuf::feature_set::EnumType, ::roto::runtime::DecodeError> {
                Ok(self.enum_type)
            }

            #[inline]
            pub const fn has_repeated_field_encoding(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn repeated_field_encoding(&self) -> Result<crate::google::protobuf::feature_set::RepeatedFieldEncoding, ::roto::runtime::DecodeError> {
                Ok(self.repeated_field_encoding)
            }

            #[inline]
            pub const fn has_utf8_validation(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn utf8_validation(&self) -> Result<crate::google::protobuf::feature_set::Utf8Validation, ::roto::runtime::DecodeError> {
                Ok(self.utf8_validation)
            }

            #[inline]
            pub const fn has_message_encoding(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn message_encoding(&self) -> Result<crate::google::protobuf::feature_set::MessageEncoding, ::roto::runtime::DecodeError> {
                Ok(self.message_encoding)
            }

            #[inline]
            pub const fn has_json_format(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn json_format(&self) -> Result<crate::google::protobuf::feature_set::JsonFormat, ::roto::runtime::DecodeError> {
                Ok(self.json_format)
            }

            #[inline]
            pub const fn has_enforce_naming_style(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn enforce_naming_style(&self) -> Result<crate::google::protobuf::feature_set::EnforceNamingStyle, ::roto::runtime::DecodeError> {
                Ok(self.enforce_naming_style)
            }

            #[inline]
            pub const fn has_default_symbol_visibility(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn default_symbol_visibility(&self) -> Result<crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility, ::roto::runtime::DecodeError> {
                Ok(self.default_symbol_visibility)
            }

            #[inline]
            pub const fn has_enforce_proto_limits(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn enforce_proto_limits(&self) -> Result<crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits, ::roto::runtime::DecodeError> {
                Ok(self.enforce_proto_limits)
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

        pub struct FeatureSetBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FeatureSetBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FeatureSetView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FeatureSetView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FeatureSetView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_field_presence(&mut self, val: crate::google::protobuf::feature_set::FieldPresence) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enum_type(&mut self, val: crate::google::protobuf::feature_set::EnumType) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_repeated_field_encoding(&mut self, val: crate::google::protobuf::feature_set::RepeatedFieldEncoding) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_utf8_validation(&mut self, val: crate::google::protobuf::feature_set::Utf8Validation) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_message_encoding(&mut self, val: crate::google::protobuf::feature_set::MessageEncoding) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_json_format(&mut self, val: crate::google::protobuf::feature_set::JsonFormat) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enforce_naming_style(&mut self, val: crate::google::protobuf::feature_set::EnforceNamingStyle) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_default_symbol_visibility(&mut self, val: crate::google::protobuf::feature_set::visibility_feature::DefaultSymbolVisibility) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_enforce_proto_limits(&mut self, val: crate::google::protobuf::feature_set::proto_limits_feature::EnforceProtoLimits) -> &mut Self {
                self.set_fields[0] |= 256;
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

            pub struct VisibilityFeatureBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'b> VisibilityFeatureBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &VisibilityFeatureView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !VisibilityFeatureView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &VisibilityFeatureView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |_| true);
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

            pub struct ProtoLimitsFeatureBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
            }

            impl<'b> ProtoLimitsFeatureBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &ProtoLimitsFeatureView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ProtoLimitsFeatureView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &ProtoLimitsFeatureView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |_| true);
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
            defaults: ::roto::runtime::FieldMarker,
            minimum_edition: crate::google::protobuf::Edition,
            maximum_edition: crate::google::protobuf::Edition,
            has_fields: [u32; 1],
        }

        impl<'a> FeatureSetDefaultsView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 4, 5];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut defaults = ::roto::runtime::FieldMarker::none();
                let mut minimum_edition: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                let mut maximum_edition: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, _) => defaults.extend(field.raw_span),
                        (4, ::roto::runtime::WireType::Varint) => {
                            minimum_edition = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (5, ::roto::runtime::WireType::Varint) => {
                            maximum_edition = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 2;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    defaults,
                    minimum_edition,
                    maximum_edition,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn defaults(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.defaults.slice(self.buf), 1)
            }

            #[inline]
            pub const fn has_minimum_edition(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn minimum_edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                Ok(self.minimum_edition)
            }

            #[inline]
            pub const fn has_maximum_edition(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn maximum_edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                Ok(self.maximum_edition)
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

        pub struct FeatureSetDefaultsBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FeatureSetDefaultsBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FeatureSetDefaultsView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FeatureSetDefaultsView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FeatureSetDefaultsView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_defaults<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_default<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::feature_set_defaults::FeatureSetEditionDefaultBuilder<'_>),
            {
                self.write_defaults(f)
            }

            #[inline]
            pub fn write_minimum_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_maximum_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }
        }

        pub mod feature_set_defaults {
            #[derive(Clone, Copy, Debug)]
            pub struct FeatureSetEditionDefaultView<'a> {
                buf: &'a [u8],
                edition: crate::google::protobuf::Edition,
                overridable_features: ::roto::runtime::FieldMarker,
                fixed_features: ::roto::runtime::FieldMarker,
                has_fields: [u32; 1],
            }

            impl<'a> FeatureSetEditionDefaultView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut edition: crate::google::protobuf::Edition = crate::google::protobuf::Edition::EditionUnknown;
                    let mut overridable_features = ::roto::runtime::FieldMarker::none();
                    let mut fixed_features = ::roto::runtime::FieldMarker::none();
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (3, ::roto::runtime::WireType::Varint) => {
                                edition = <crate::google::protobuf::Edition as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 1;
                            }
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
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_edition(&self) -> bool {
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn edition(&self) -> Result<crate::google::protobuf::Edition, ::roto::runtime::DecodeError> {
                    Ok(self.edition)
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

            pub struct FeatureSetEditionDefaultBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> FeatureSetEditionDefaultBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &FeatureSetEditionDefaultView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FeatureSetEditionDefaultView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &FeatureSetEditionDefaultView<'_>) -> &mut Self {
                    let prev_set = self.set_fields;
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                        3 => {
                            self.set_fields[0] |= 1;
                            (prev_set[0] & 1) == 0
                        }
                        4 => {
                            self.set_fields[0] |= 2;
                            (prev_set[0] & 2) == 0
                        }
                        5 => {
                            self.set_fields[0] |= 4;
                            (prev_set[0] & 4) == 0
                        }
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_edition(&mut self, val: crate::google::protobuf::Edition) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val.as_i32() as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_overridable_features<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
                {
                    self.set_fields[0] |= 2;
                    let end_pos = self.writer.cursor();
                    self.writer.with_sub_writer(|sub_writer| {
                        let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                        f(&mut sub_builder);
                        sub_builder.writer
                    });
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_fixed_features<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::FeatureSetBuilder<'_>),
                {
                    self.set_fields[0] |= 4;
                    let end_pos = self.writer.cursor();
                    self.writer.with_sub_writer(|sub_writer| {
                        let mut sub_builder = crate::google::protobuf::FeatureSetBuilder::from_writer(sub_writer);
                        f(&mut sub_builder);
                        sub_builder.writer
                    });
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
            location: ::roto::runtime::FieldMarker,
        }

        impl<'a> SourceCodeInfoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut location = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => location.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    location,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn location(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::source_code_info::LocationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.location.slice(self.buf), 1)
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

        pub struct SourceCodeInfoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> SourceCodeInfoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &SourceCodeInfoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !SourceCodeInfoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &SourceCodeInfoView<'_>) -> &mut Self {
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
            pub fn write_location<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::source_code_info::LocationBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::source_code_info::LocationBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
                path: ::roto::runtime::FieldMarker,
                span: ::roto::runtime::FieldMarker,
                leading_comments: ::roto::runtime::FieldMarker,
                trailing_comments: ::roto::runtime::FieldMarker,
                leading_detached_comments: ::roto::runtime::FieldMarker,
            }

            impl<'a> LocationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 6];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut path = ::roto::runtime::FieldMarker::none();
                    let mut span = ::roto::runtime::FieldMarker::none();
                    let mut leading_comments = ::roto::runtime::FieldMarker::none();
                    let mut trailing_comments = ::roto::runtime::FieldMarker::none();
                    let mut leading_detached_comments = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, _) => path.extend(field.raw_span),
                            (2, _) => span.extend(field.raw_span),
                            (3, ::roto::runtime::WireType::LengthDelimited) => leading_comments = field.marker,
                            (4, ::roto::runtime::WireType::LengthDelimited) => trailing_comments = field.marker,
                            (6, _) => leading_detached_comments.extend(field.raw_span),
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        path,
                        span,
                        leading_comments,
                        trailing_comments,
                        leading_detached_comments,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn path(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.path.slice(self.buf), 1)
                }

                #[inline]
                pub const fn span(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.span.slice(self.buf), 2)
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
                    ::roto::runtime::RepeatedStrIter::new(self.leading_detached_comments.slice(self.buf), 6)
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

            pub struct LocationBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> LocationBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !LocationView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &LocationView<'_>) -> &mut Self {
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
                        6 => {
                            self.set_fields[0] |= 16;
                            (prev_set[0] & 16) == 0
                        }
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_path(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_path(&mut self, vals: &[i32]) -> &mut Self {
                    self.set_fields[0] |= 1;
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
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_span(&mut self, vals: &[i32]) -> &mut Self {
                    self.set_fields[0] |= 2;
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
                    self.set_fields[0] |= 4;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_trailing_comments(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 8;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_leading_detached_comments(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 16;
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
            annotation: ::roto::runtime::FieldMarker,
        }

        impl<'a> GeneratedCodeInfoView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut annotation = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => annotation.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    annotation,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn annotation(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::generated_code_info::AnnotationView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.annotation.slice(self.buf), 1)
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

        pub struct GeneratedCodeInfoBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> GeneratedCodeInfoBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &GeneratedCodeInfoView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GeneratedCodeInfoView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &GeneratedCodeInfoView<'_>) -> &mut Self {
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
            pub fn write_annotation<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::generated_code_info::AnnotationBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::generated_code_info::AnnotationBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
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
                path: ::roto::runtime::FieldMarker,
                source_file: ::roto::runtime::FieldMarker,
                begin: i32,
                end: i32,
                semantic: crate::google::protobuf::generated_code_info::annotation::Semantic,
                has_fields: [u32; 1],
            }

            impl<'a> AnnotationView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut path = ::roto::runtime::FieldMarker::none();
                    let mut source_file = ::roto::runtime::FieldMarker::none();
                    let mut begin: i32 = 0i32;
                    let mut end: i32 = 0i32;
                    let mut semantic: crate::google::protobuf::generated_code_info::annotation::Semantic = crate::google::protobuf::generated_code_info::annotation::Semantic::None;
                    let mut has_fields = [0u32; 1];

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, _) => path.extend(field.raw_span),
                            (2, ::roto::runtime::WireType::LengthDelimited) => source_file = field.marker,
                            (3, ::roto::runtime::WireType::Varint) => {
                                begin = field.val as i32;
                                has_fields[0] |= 1;
                            }
                            (4, ::roto::runtime::WireType::Varint) => {
                                end = field.val as i32;
                                has_fields[0] |= 2;
                            }
                            (5, ::roto::runtime::WireType::Varint) => {
                                semantic = <crate::google::protobuf::generated_code_info::annotation::Semantic as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                                has_fields[0] |= 4;
                            }
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        path,
                        source_file,
                        begin,
                        end,
                        semantic,
                        has_fields,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn path(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
                    ::roto::runtime::RepeatedVarintIter::new(self.path.slice(self.buf), 1)
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
                    (self.has_fields[0] & 1) != 0
                }

                #[inline]
                pub const fn begin(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.begin)
                }

                #[inline]
                pub const fn has_end(&self) -> bool {
                    (self.has_fields[0] & 2) != 0
                }

                #[inline]
                pub const fn end(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                    Ok(self.end)
                }

                #[inline]
                pub const fn has_semantic(&self) -> bool {
                    (self.has_fields[0] & 4) != 0
                }

                #[inline]
                pub const fn semantic(&self) -> Result<crate::google::protobuf::generated_code_info::annotation::Semantic, ::roto::runtime::DecodeError> {
                    Ok(self.semantic)
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

            pub struct AnnotationBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> AnnotationBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &AnnotationView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !AnnotationView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &AnnotationView<'_>) -> &mut Self {
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
                pub fn write_path(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_packed_path(&mut self, vals: &[i32]) -> &mut Self {
                    self.set_fields[0] |= 1;
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
                    self.set_fields[0] |= 2;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_begin(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 4;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_end(&mut self, val: i32) -> &mut Self {
                    self.set_fields[0] |= 8;
                    let _ = self.writer.write_varint(val as u64);
                    let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                    self
                }

                #[inline]
                pub fn write_semantic(&mut self, val: crate::google::protobuf::generated_code_info::annotation::Semantic) -> &mut Self {
                    self.set_fields[0] |= 16;
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

        #[derive(Clone, Copy, Debug)]
        pub struct DurationView<'a> {
            buf: &'a [u8],
            seconds: i64,
            nanos: i32,
            has_fields: [u32; 1],
        }

        impl<'a> DurationView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut seconds: i64 = 0i64;
                let mut nanos: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            seconds = field.val as i64;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            nanos = field.val as i32;
                            has_fields[0] |= 2;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    seconds,
                    nanos,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_seconds(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn seconds(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.seconds)
            }

            #[inline]
            pub const fn has_nanos(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn nanos(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.nanos)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for DurationView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct DurationBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> DurationBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &DurationView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !DurationView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &DurationView<'_>) -> &mut Self {
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
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_seconds(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_nanos(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EmptyView<'a> {
            buf: &'a [u8],
        }

        impl<'a> EmptyView<'a> {
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

        impl<'a> ::roto::runtime::MessageView<'a> for EmptyView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EmptyBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
        }

        impl<'b> EmptyBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EmptyView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EmptyView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EmptyView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |_| true);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FieldMaskView<'a> {
            buf: &'a [u8],
            paths: ::roto::runtime::FieldMarker,
        }

        impl<'a> FieldMaskView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut paths = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => paths.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    paths,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn paths(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.paths.slice(self.buf), 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FieldMaskView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FieldMaskBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FieldMaskBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FieldMaskView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FieldMaskView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FieldMaskView<'_>) -> &mut Self {
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
            pub fn write_paths(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_path(&mut self, val: &str) -> &mut Self {
                self.write_paths(val)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct SourceContextView<'a> {
            buf: &'a [u8],
            file_name: ::roto::runtime::FieldMarker,
        }

        impl<'a> SourceContextView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut file_name = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => file_name = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    file_name,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_file_name(&self) -> bool {
                self.file_name.is_present()
            }

            #[inline]
            pub fn file_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.file_name.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.file_name)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for SourceContextView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct SourceContextBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> SourceContextBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &SourceContextView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !SourceContextView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &SourceContextView<'_>) -> &mut Self {
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
            pub fn write_file_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum NullValue {
            NullValue = 0,
        }

        impl NullValue {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::NullValue),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for NullValue {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for NullValue {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::NullValue)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct StructView<'a> {
            buf: &'a [u8],
            fields: ::roto::runtime::FieldMarker,
        }

        impl<'a> StructView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut fields = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => fields.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn fields(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::r#struct::FieldsEntryView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.fields.slice(self.buf), 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for StructView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct StructBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> StructBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &StructView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !StructView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &StructView<'_>) -> &mut Self {
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
            pub fn write_fields<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::r#struct::FieldsEntryBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::r#struct::FieldsEntryBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::r#struct::FieldsEntryBuilder<'_>),
            {
                self.write_fields(f)
            }
        }

        pub mod r#struct {
            #[derive(Clone, Copy, Debug)]
            pub struct FieldsEntryView<'a> {
                buf: &'a [u8],
                key: ::roto::runtime::FieldMarker,
                value: ::roto::runtime::FieldMarker,
            }

            impl<'a> FieldsEntryView<'a> {
                pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

                pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    let mut key = ::roto::runtime::FieldMarker::none();
                    let mut value = ::roto::runtime::FieldMarker::none();

                    for item in ::roto::runtime::FieldIter::new(buf) {
                        let field = item?;
                        match (field.tag, field.wire_type) {
                            (1, ::roto::runtime::WireType::LengthDelimited) => key = field.marker,
                            (2, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                            _ => {}
                        }
                    }

                    Ok(Self {
                        buf,
                        key,
                        value,
                    })
                }

                #[inline]
                pub const fn as_bytes(&self) -> &'a [u8] {
                    self.buf
                }

                #[inline]
                pub const fn has_key(&self) -> bool {
                    self.key.is_present()
                }

                #[inline]
                pub fn key(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                    if !self.key.is_present() {
                        return Ok("");
                    }
                    ::roto::runtime::decode_str_at(self.buf, self.key)
                }

                #[inline]
                pub const fn has_value(&self) -> bool {
                    self.value.is_present()
                }

                #[inline]
                pub fn value(&self) -> Result<Option<crate::google::protobuf::ValueView<'a>>, ::roto::runtime::DecodeError> {
                    if !self.value.is_present() {
                        return Ok(None);
                    }
                    crate::google::protobuf::ValueView::from_bytes(self.value.slice(self.buf)).map(Some)
                }

                #[inline]
                pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                    ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
                }
            }

            impl<'a> ::roto::runtime::MessageView<'a> for FieldsEntryView<'a> {
                #[inline]
                fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                    Self::from_bytes(buf)
                }
            }

            pub struct FieldsEntryBuilder<'b> {
                pub writer: ::roto::runtime::BackwardWriter<'b>,
                set_fields: [u64; 1],
            }

            impl<'b> FieldsEntryBuilder<'b> {
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
                pub fn copy_unknown_fields(&mut self, view: &FieldsEntryView<'_>) -> &mut Self {
                    let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FieldsEntryView::KNOWN_TAGS.contains(&tag));
                    self
                }

                #[inline]
                pub fn copy_unset_fields(&mut self, view: &FieldsEntryView<'_>) -> &mut Self {
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
                        _ => true,
                    });
                    self
                }

                #[inline]
                pub fn write_key(&mut self, val: &str) -> &mut Self {
                    self.set_fields[0] |= 1;
                    let _ = self.writer.write_length_delimited(val.as_bytes());
                    let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                    self
                }

                #[inline]
                pub fn write_value<F>(&mut self, f: F) -> &mut Self
                where
                    F: FnOnce(&mut crate::google::protobuf::ValueBuilder<'_>),
                {
                    self.set_fields[0] |= 2;
                    let end_pos = self.writer.cursor();
                    self.writer.with_sub_writer(|sub_writer| {
                        let mut sub_builder = crate::google::protobuf::ValueBuilder::from_writer(sub_writer);
                        f(&mut sub_builder);
                        sub_builder.writer
                    });
                    let nested_len = end_pos - self.writer.cursor();
                    let _ = self.writer.write_varint(nested_len as u64);
                    let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                    self
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ValueView<'a> {
            buf: &'a [u8],
            null_value: crate::google::protobuf::NullValue,
            number_value: f64,
            string_value: ::roto::runtime::FieldMarker,
            bool_value: bool,
            struct_value: ::roto::runtime::FieldMarker,
            list_value: ::roto::runtime::FieldMarker,
            oneof_0: u32,
        }

        impl<'a> ValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut null_value: crate::google::protobuf::NullValue = crate::google::protobuf::NullValue::NullValue;
                let mut number_value: f64 = 0.0f64;
                let mut string_value = ::roto::runtime::FieldMarker::none();
                let mut bool_value: bool = false;
                let mut struct_value = ::roto::runtime::FieldMarker::none();
                let mut list_value = ::roto::runtime::FieldMarker::none();
                let mut oneof_0: u32 = 0;

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            null_value = <crate::google::protobuf::NullValue as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            oneof_0 = 1;
                        }
                        (2, ::roto::runtime::WireType::Fixed64) => {
                            number_value = f64::from_bits(field.val);
                            oneof_0 = 2;
                        }
                        (3, ::roto::runtime::WireType::LengthDelimited) => {
                            string_value = field.marker;
                            oneof_0 = 3;
                        }
                        (4, ::roto::runtime::WireType::Varint) => {
                            bool_value = field.val != 0;
                            oneof_0 = 4;
                        }
                        (5, ::roto::runtime::WireType::LengthDelimited) => {
                            struct_value = field.marker;
                            oneof_0 = 5;
                        }
                        (6, ::roto::runtime::WireType::LengthDelimited) => {
                            list_value = field.marker;
                            oneof_0 = 6;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    null_value,
                    number_value,
                    string_value,
                    bool_value,
                    struct_value,
                    list_value,
                    oneof_0,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_null_value(&self) -> bool {
                self.oneof_0 == 1
            }

            #[inline]
            pub const fn null_value(&self) -> Result<crate::google::protobuf::NullValue, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 1 {
                    return Ok(crate::google::protobuf::NullValue::NullValue);
                }
                Ok(self.null_value)
            }

            #[inline]
            pub const fn has_number_value(&self) -> bool {
                self.oneof_0 == 2
            }

            #[inline]
            pub const fn number_value(&self) -> Result<f64, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 2 {
                    return Ok(0.0f64);
                }
                Ok(self.number_value)
            }

            #[inline]
            pub const fn has_string_value(&self) -> bool {
                self.oneof_0 == 3
            }

            #[inline]
            pub fn string_value(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 3 {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.string_value)
            }

            #[inline]
            pub const fn has_bool_value(&self) -> bool {
                self.oneof_0 == 4
            }

            #[inline]
            pub const fn bool_value(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 4 {
                    return Ok(false);
                }
                Ok(self.bool_value)
            }

            #[inline]
            pub const fn has_struct_value(&self) -> bool {
                self.oneof_0 == 5
            }

            #[inline]
            pub fn struct_value(&self) -> Result<Option<crate::google::protobuf::StructView<'a>>, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 5 {
                    return Ok(None);
                }
                crate::google::protobuf::StructView::from_bytes(self.struct_value.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_list_value(&self) -> bool {
                self.oneof_0 == 6
            }

            #[inline]
            pub fn list_value(&self) -> Result<Option<crate::google::protobuf::ListValueView<'a>>, ::roto::runtime::DecodeError> {
                if self.oneof_0 != 6 {
                    return Ok(None);
                }
                crate::google::protobuf::ListValueView::from_bytes(self.list_value.slice(self.buf)).map(Some)
            }

            #[inline]
            pub fn kind(&self) -> Result<Option<value::Kind<'a>>, ::roto::runtime::DecodeError> {
                match self.oneof_0 {
                    1 => Ok(Some(value::Kind::NullValue(self.null_value))),
                    2 => Ok(Some(value::Kind::NumberValue(self.number_value))),
                    3 => Ok(Some(value::Kind::StringValue(::roto::runtime::decode_str_at(self.buf, self.string_value)?))),
                    4 => Ok(Some(value::Kind::BoolValue(self.bool_value))),
                    5 => Ok(Some(value::Kind::StructValue(crate::google::protobuf::StructView::from_bytes(self.struct_value.slice(self.buf))?))),
                    6 => Ok(Some(value::Kind::ListValue(crate::google::protobuf::ListValueView::from_bytes(self.list_value.slice(self.buf))?))),
                    _ => Ok(None),
                }
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ValueView<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 1) == 0
                    }
                    2 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 2) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 4) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 8) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 16) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 63;
                        (prev_set[0] & 32) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_null_value(&mut self, val: crate::google::protobuf::NullValue) -> &mut Self {
                self.set_fields[0] |= 63;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_number_value(&mut self, val: f64) -> &mut Self {
                self.set_fields[0] |= 63;
                let _ = self.writer.write_f64(val);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_string_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 63;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_bool_value(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 63;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_struct_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::StructBuilder<'_>),
            {
                self.set_fields[0] |= 63;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::StructBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_list_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ListValueBuilder<'_>),
            {
                self.set_fields[0] |= 63;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::ListValueBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod value {
            #[derive(Clone, Copy, Debug)]
            pub enum Kind<'a> {
                NullValue(crate::google::protobuf::NullValue),
                NumberValue(f64),
                StringValue(&'a str),
                BoolValue(bool),
                StructValue(crate::google::protobuf::StructView<'a>),
                ListValue(crate::google::protobuf::ListValueView<'a>),
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct ListValueView<'a> {
            buf: &'a [u8],
            values: ::roto::runtime::FieldMarker,
        }

        impl<'a> ListValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut values = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, _) => values.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    values,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn values(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::ValueView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.values.slice(self.buf), 1)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for ListValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct ListValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> ListValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &ListValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ListValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &ListValueView<'_>) -> &mut Self {
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
            pub fn write_values<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ValueBuilder<'_>),
            {
                self.set_fields[0] |= 1;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::ValueBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::ValueBuilder<'_>),
            {
                self.write_values(f)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct TimestampView<'a> {
            buf: &'a [u8],
            seconds: i64,
            nanos: i32,
            has_fields: [u32; 1],
        }

        impl<'a> TimestampView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut seconds: i64 = 0i64;
                let mut nanos: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            seconds = field.val as i64;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            nanos = field.val as i32;
                            has_fields[0] |= 2;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    seconds,
                    nanos,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_seconds(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn seconds(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.seconds)
            }

            #[inline]
            pub const fn has_nanos(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn nanos(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.nanos)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for TimestampView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct TimestampBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> TimestampBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &TimestampView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TimestampView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &TimestampView<'_>) -> &mut Self {
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
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_seconds(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_nanos(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(i32)]
        pub enum Syntax {
            SyntaxProto2 = 0,
            SyntaxProto3 = 1,
            SyntaxEditions = 2,
        }

        impl Syntax {
            #[inline]
            pub const fn from_i32(val: i32) -> Option<Self> {
                match val {
                    0 => Some(Self::SyntaxProto2),
                    1 => Some(Self::SyntaxProto3),
                    2 => Some(Self::SyntaxEditions),
                    _ => None,
                }
            }

            #[inline]
            pub const fn as_i32(self) -> i32 {
                self as i32
            }
        }

        impl core::convert::TryFrom<i32> for Syntax {
            type Error = i32;

            #[inline]
            fn try_from(val: i32) -> Result<Self, Self::Error> {
                Self::from_i32(val).ok_or(val)
            }
        }

        impl ::roto::runtime::VarintValue for Syntax {
            #[inline]
            fn from_raw_varint(val: u64) -> Self {
                Self::from_i32(val as i32).unwrap_or(Self::SyntaxProto2)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct TypeView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            fields: ::roto::runtime::FieldMarker,
            oneofs: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            source_context: ::roto::runtime::FieldMarker,
            syntax: crate::google::protobuf::Syntax,
            edition: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> TypeView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut fields = ::roto::runtime::FieldMarker::none();
                let mut oneofs = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut source_context = ::roto::runtime::FieldMarker::none();
                let mut syntax: crate::google::protobuf::Syntax = crate::google::protobuf::Syntax::SyntaxProto2;
                let mut edition = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => fields.extend(field.raw_span),
                        (3, _) => oneofs.extend(field.raw_span),
                        (4, _) => options.extend(field.raw_span),
                        (5, ::roto::runtime::WireType::LengthDelimited) => source_context = field.marker,
                        (6, ::roto::runtime::WireType::Varint) => {
                            syntax = <crate::google::protobuf::Syntax as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (7, ::roto::runtime::WireType::LengthDelimited) => edition = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    fields,
                    oneofs,
                    options,
                    source_context,
                    syntax,
                    edition,
                    has_fields,
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
            pub const fn fields(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::FieldView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.fields.slice(self.buf), 2)
            }

            #[inline]
            pub const fn oneofs(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
                ::roto::runtime::RepeatedStrIter::new(self.oneofs.slice(self.buf), 3)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 4)
            }

            #[inline]
            pub const fn has_source_context(&self) -> bool {
                self.source_context.is_present()
            }

            #[inline]
            pub fn source_context(&self) -> Result<Option<crate::google::protobuf::SourceContextView<'a>>, ::roto::runtime::DecodeError> {
                if !self.source_context.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::SourceContextView::from_bytes(self.source_context.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_syntax(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn syntax(&self) -> Result<crate::google::protobuf::Syntax, ::roto::runtime::DecodeError> {
                Ok(self.syntax)
            }

            #[inline]
            pub const fn has_edition(&self) -> bool {
                self.edition.is_present()
            }

            #[inline]
            pub fn edition(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.edition.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.edition)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for TypeView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct TypeBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> TypeBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &TypeView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TypeView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &TypeView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
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
            pub fn write_fields<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::FieldBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::FieldBuilder<'_>),
            {
                self.write_fields(f)
            }

            #[inline]
            pub fn write_oneofs(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof(&mut self, val: &str) -> &mut Self {
                self.write_oneofs(val)
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }

            #[inline]
            pub fn write_source_context<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::SourceContextBuilder<'_>),
            {
                self.set_fields[0] |= 16;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::SourceContextBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_syntax(&mut self, val: crate::google::protobuf::Syntax) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FieldView<'a> {
            buf: &'a [u8],
            kind: crate::google::protobuf::field::Kind,
            cardinality: crate::google::protobuf::field::Cardinality,
            number: i32,
            name: ::roto::runtime::FieldMarker,
            type_url: ::roto::runtime::FieldMarker,
            f_7: i32,
            packed: bool,
            options: ::roto::runtime::FieldMarker,
            json_name: ::roto::runtime::FieldMarker,
            default_value: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> FieldView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 6, 7, 8, 9, 10, 11];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut kind: crate::google::protobuf::field::Kind = crate::google::protobuf::field::Kind::TypeUnknown;
                let mut cardinality: crate::google::protobuf::field::Cardinality = crate::google::protobuf::field::Cardinality::CardinalityUnknown;
                let mut number: i32 = 0i32;
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut type_url = ::roto::runtime::FieldMarker::none();
                let mut f_7: i32 = 0i32;
                let mut packed: bool = false;
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut json_name = ::roto::runtime::FieldMarker::none();
                let mut default_value = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            kind = <crate::google::protobuf::field::Kind as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            cardinality = <crate::google::protobuf::field::Cardinality as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 2;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            number = field.val as i32;
                            has_fields[0] |= 4;
                        }
                        (4, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (6, ::roto::runtime::WireType::LengthDelimited) => type_url = field.marker,
                        (7, ::roto::runtime::WireType::Varint) => {
                            f_7 = field.val as i32;
                            has_fields[0] |= 8;
                        }
                        (8, ::roto::runtime::WireType::Varint) => {
                            packed = field.val != 0;
                            has_fields[0] |= 16;
                        }
                        (9, _) => options.extend(field.raw_span),
                        (10, ::roto::runtime::WireType::LengthDelimited) => json_name = field.marker,
                        (11, ::roto::runtime::WireType::LengthDelimited) => default_value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    kind,
                    cardinality,
                    number,
                    name,
                    type_url,
                    f_7,
                    packed,
                    options,
                    json_name,
                    default_value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_kind(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn kind(&self) -> Result<crate::google::protobuf::field::Kind, ::roto::runtime::DecodeError> {
                Ok(self.kind)
            }

            #[inline]
            pub const fn has_cardinality(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn cardinality(&self) -> Result<crate::google::protobuf::field::Cardinality, ::roto::runtime::DecodeError> {
                Ok(self.cardinality)
            }

            #[inline]
            pub const fn has_number(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.number)
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
            pub const fn has_type_url(&self) -> bool {
                self.type_url.is_present()
            }

            #[inline]
            pub fn type_url(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.type_url.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.type_url)
            }

            #[inline]
            pub const fn has_oneof_index(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn oneof_index(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.f_7)
            }

            #[inline]
            pub const fn has_packed(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn packed(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.packed)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 9)
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
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FieldView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FieldBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FieldBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FieldView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FieldView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FieldView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    8 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    10 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    11 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_kind(&mut self, val: crate::google::protobuf::field::Kind) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_cardinality(&mut self, val: crate::google::protobuf::field::Cardinality) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_type_url(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_oneof_index(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_packed(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 128;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }

            #[inline]
            pub fn write_json_name(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_default_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(11, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod field {
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum Kind {
                TypeUnknown = 0,
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

            impl Kind {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::TypeUnknown),
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

            impl core::convert::TryFrom<i32> for Kind {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for Kind {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::TypeUnknown)
                }
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            #[repr(i32)]
            pub enum Cardinality {
                CardinalityUnknown = 0,
                CardinalityOptional = 1,
                CardinalityRequired = 2,
                CardinalityRepeated = 3,
            }

            impl Cardinality {
                #[inline]
                pub const fn from_i32(val: i32) -> Option<Self> {
                    match val {
                        0 => Some(Self::CardinalityUnknown),
                        1 => Some(Self::CardinalityOptional),
                        2 => Some(Self::CardinalityRequired),
                        3 => Some(Self::CardinalityRepeated),
                        _ => None,
                    }
                }

                #[inline]
                pub const fn as_i32(self) -> i32 {
                    self as i32
                }
            }

            impl core::convert::TryFrom<i32> for Cardinality {
                type Error = i32;

                #[inline]
                fn try_from(val: i32) -> Result<Self, Self::Error> {
                    Self::from_i32(val).ok_or(val)
                }
            }

            impl ::roto::runtime::VarintValue for Cardinality {
                #[inline]
                fn from_raw_varint(val: u64) -> Self {
                    Self::from_i32(val as i32).unwrap_or(Self::CardinalityUnknown)
                }
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            enumvalue: ::roto::runtime::FieldMarker,
            options: ::roto::runtime::FieldMarker,
            source_context: ::roto::runtime::FieldMarker,
            syntax: crate::google::protobuf::Syntax,
            edition: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> EnumView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut enumvalue = ::roto::runtime::FieldMarker::none();
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut source_context = ::roto::runtime::FieldMarker::none();
                let mut syntax: crate::google::protobuf::Syntax = crate::google::protobuf::Syntax::SyntaxProto2;
                let mut edition = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, _) => enumvalue.extend(field.raw_span),
                        (3, _) => options.extend(field.raw_span),
                        (4, ::roto::runtime::WireType::LengthDelimited) => source_context = field.marker,
                        (5, ::roto::runtime::WireType::Varint) => {
                            syntax = <crate::google::protobuf::Syntax as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                            has_fields[0] |= 1;
                        }
                        (6, ::roto::runtime::WireType::LengthDelimited) => edition = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    enumvalue,
                    options,
                    source_context,
                    syntax,
                    edition,
                    has_fields,
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
            pub const fn enumvalue(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::EnumValueView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.enumvalue.slice(self.buf), 2)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 3)
            }

            #[inline]
            pub const fn has_source_context(&self) -> bool {
                self.source_context.is_present()
            }

            #[inline]
            pub fn source_context(&self) -> Result<Option<crate::google::protobuf::SourceContextView<'a>>, ::roto::runtime::DecodeError> {
                if !self.source_context.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::SourceContextView::from_bytes(self.source_context.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_syntax(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn syntax(&self) -> Result<crate::google::protobuf::Syntax, ::roto::runtime::DecodeError> {
                Ok(self.syntax)
            }

            #[inline]
            pub const fn has_edition(&self) -> bool {
                self.edition.is_present()
            }

            #[inline]
            pub fn edition(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.edition.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.edition)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumView<'_>) -> &mut Self {
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
                    6 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
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
            pub fn write_enumvalue<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::EnumValueBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::EnumValueBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }

            #[inline]
            pub fn write_source_context<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::SourceContextBuilder<'_>),
            {
                self.set_fields[0] |= 8;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::SourceContextBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_syntax(&mut self, val: crate::google::protobuf::Syntax) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_varint(val.as_i32() as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_edition(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct EnumValueView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            number: i32,
            options: ::roto::runtime::FieldMarker,
            has_fields: [u32; 1],
        }

        impl<'a> EnumValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut number: i32 = 0i32;
                let mut options = ::roto::runtime::FieldMarker::none();
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::Varint) => {
                            number = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (3, _) => options.extend(field.raw_span),
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    number,
                    options,
                    has_fields,
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
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn number(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.number)
            }

            #[inline]
            pub const fn options(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::protobuf::OptionView<'a>> {
                ::roto::runtime::RepeatedMessageIter::new(self.options.slice(self.buf), 3)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for EnumValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct EnumValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> EnumValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &EnumValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !EnumValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &EnumValueView<'_>) -> &mut Self {
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
            pub fn write_number(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_options<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.set_fields[0] |= 4;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::OptionBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_option<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::OptionBuilder<'_>),
            {
                self.write_options(f)
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct OptionView<'a> {
            buf: &'a [u8],
            name: ::roto::runtime::FieldMarker,
            value: ::roto::runtime::FieldMarker,
        }

        impl<'a> OptionView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut name = ::roto::runtime::FieldMarker::none();
                let mut value = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                        (2, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    name,
                    value,
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
            pub const fn has_value(&self) -> bool {
                self.value.is_present()
            }

            #[inline]
            pub fn value(&self) -> Result<Option<crate::google::protobuf::AnyView<'a>>, ::roto::runtime::DecodeError> {
                if !self.value.is_present() {
                    return Ok(None);
                }
                crate::google::protobuf::AnyView::from_bytes(self.value.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for OptionView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct OptionBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> OptionBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &OptionView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !OptionView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &OptionView<'_>) -> &mut Self {
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
            pub fn write_value<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::google::protobuf::AnyBuilder<'_>),
            {
                self.set_fields[0] |= 2;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::google::protobuf::AnyBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct DoubleValueView<'a> {
            buf: &'a [u8],
            value: f64,
            has_fields: [u32; 1],
        }

        impl<'a> DoubleValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: f64 = 0.0f64;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Fixed64) => {
                            value = f64::from_bits(field.val);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<f64, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for DoubleValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct DoubleValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> DoubleValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &DoubleValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !DoubleValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &DoubleValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: f64) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_f64(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Fixed64);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct FloatValueView<'a> {
            buf: &'a [u8],
            value: f32,
            has_fields: [u32; 1],
        }

        impl<'a> FloatValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: f32 = 0.0f32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Fixed32) => {
                            value = f32::from_bits(field.val as u32);
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<f32, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for FloatValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct FloatValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> FloatValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &FloatValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !FloatValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &FloatValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: f32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_f32(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Fixed32);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Int64ValueView<'a> {
            buf: &'a [u8],
            value: i64,
            has_fields: [u32; 1],
        }

        impl<'a> Int64ValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: i64 = 0i64;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            value = field.val as i64;
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for Int64ValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct Int64ValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> Int64ValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &Int64ValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !Int64ValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &Int64ValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct UInt64ValueView<'a> {
            buf: &'a [u8],
            value: u64,
            has_fields: [u32; 1],
        }

        impl<'a> UInt64ValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: u64 = 0u64;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            value = field.val;
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for UInt64ValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct UInt64ValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> UInt64ValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &UInt64ValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !UInt64ValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &UInt64ValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct Int32ValueView<'a> {
            buf: &'a [u8],
            value: i32,
            has_fields: [u32; 1],
        }

        impl<'a> Int32ValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            value = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for Int32ValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct Int32ValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> Int32ValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &Int32ValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !Int32ValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &Int32ValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct UInt32ValueView<'a> {
            buf: &'a [u8],
            value: u32,
            has_fields: [u32; 1],
        }

        impl<'a> UInt32ValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: u32 = 0u32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            value = field.val as u32;
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<u32, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for UInt32ValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct UInt32ValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> UInt32ValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &UInt32ValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !UInt32ValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &UInt32ValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: u32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BoolValueView<'a> {
            buf: &'a [u8],
            value: bool,
            has_fields: [u32; 1],
        }

        impl<'a> BoolValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value: bool = false;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            value = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn value(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for BoolValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct BoolValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> BoolValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &BoolValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BoolValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &BoolValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct StringValueView<'a> {
            buf: &'a [u8],
            value: ::roto::runtime::FieldMarker,
        }

        impl<'a> StringValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
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

        impl<'a> ::roto::runtime::MessageView<'a> for StringValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct StringValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> StringValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &StringValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !StringValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &StringValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct BytesValueView<'a> {
            buf: &'a [u8],
            value: ::roto::runtime::FieldMarker,
        }

        impl<'a> BytesValueView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut value = ::roto::runtime::FieldMarker::none();

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    #[allow(clippy::single_match)]
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => value = field.marker,
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    value,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_value(&self) -> bool {
                self.value.is_present()
            }

            #[inline]
            pub fn value(&self) -> Result<&'a [u8], ::roto::runtime::DecodeError> {
                if !self.value.is_present() {
                    return Ok(&b""[..]);
                }
                ::roto::runtime::decode_bytes_at(self.buf, self.value)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for BytesValueView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct BytesValueBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> BytesValueBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &BytesValueView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BytesValueView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &BytesValueView<'_>) -> &mut Self {
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
            pub fn write_value(&mut self, val: &[u8]) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }
        }

        pub mod compiler {
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
        }
    }
}

pub mod pb {
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
}
