#![allow(
    clippy::all,
    dead_code,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_imports
)]

pub mod benchmarks {
    #[derive(Clone, Copy, Debug)]
    pub struct BenchmarkDatasetView<'a> {
        buf: &'a [u8],
        name: ::roto::runtime::FieldMarker,
        message_name: ::roto::runtime::FieldMarker,
        payload: ::roto::runtime::FieldMarker,
    }

    impl<'a> BenchmarkDatasetView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut name = ::roto::runtime::FieldMarker::none();
            let mut message_name = ::roto::runtime::FieldMarker::none();
            let mut payload = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => name = field.marker,
                    (2, ::roto::runtime::WireType::LengthDelimited) => message_name = field.marker,
                    (3, _) => payload.extend(field.raw_span),
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                name,
                message_name,
                payload,
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
        pub const fn has_message_name(&self) -> bool {
            self.message_name.is_present()
        }

        #[inline]
        pub fn message_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.message_name.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.message_name)
        }

        #[inline]
        pub const fn payload(&self) -> ::roto::runtime::RepeatedBytesIter<'a> {
            ::roto::runtime::RepeatedBytesIter::new(self.payload.slice(self.buf), 3)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for BenchmarkDatasetView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct BenchmarkDatasetBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> BenchmarkDatasetBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &BenchmarkDatasetView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BenchmarkDatasetView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &BenchmarkDatasetView<'_>) -> &mut Self {
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
        pub fn write_message_name(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_payload(&mut self, val: &[u8]) -> &mut Self {
            self.set_fields[0] |= 4;
            let _ = self.writer.write_length_delimited(val);
            let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }

    pub mod proto2 {
        #[derive(Clone, Copy, Debug)]
        pub struct GoogleMessage1View<'a> {
            buf: &'a [u8],
            field1: ::roto::runtime::FieldMarker,
            field9: ::roto::runtime::FieldMarker,
            field18: ::roto::runtime::FieldMarker,
            field80: bool,
            field81: bool,
            field2: i32,
            field3: i32,
            field280: i32,
            field6: i32,
            field22: i64,
            field4: ::roto::runtime::FieldMarker,
            field5: ::roto::runtime::FieldMarker,
            field59: bool,
            field7: ::roto::runtime::FieldMarker,
            field16: i32,
            field130: i32,
            field12: bool,
            field17: bool,
            field13: bool,
            field14: bool,
            field104: i32,
            field100: i32,
            field101: i32,
            field102: ::roto::runtime::FieldMarker,
            field103: ::roto::runtime::FieldMarker,
            field29: i32,
            field30: bool,
            field60: i32,
            field271: i32,
            field272: i32,
            field150: i32,
            field23: i32,
            field24: bool,
            field25: i32,
            field15: ::roto::runtime::FieldMarker,
            field78: bool,
            field67: i32,
            field68: i32,
            field128: i32,
            field129: ::roto::runtime::FieldMarker,
            field131: i32,
            has_fields: [u32; 1],
        }

        impl<'a> GoogleMessage1View<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 9, 18, 80, 81, 2, 3, 280, 6, 22, 4, 5, 59, 7, 16, 130, 12, 17, 13, 14, 104, 100, 101, 102, 103, 29, 30, 60, 271, 272, 150, 23, 24, 25, 15, 78, 67, 68, 128, 129, 131];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field1 = ::roto::runtime::FieldMarker::none();
                let mut field9 = ::roto::runtime::FieldMarker::none();
                let mut field18 = ::roto::runtime::FieldMarker::none();
                let mut field80: bool = false;
                let mut field81: bool = true;
                let mut field2: i32 = 0i32;
                let mut field3: i32 = 0i32;
                let mut field280: i32 = 0i32;
                let mut field6: i32 = 0i32;
                let mut field22: i64 = 0i64;
                let mut field4 = ::roto::runtime::FieldMarker::none();
                let mut field5 = ::roto::runtime::FieldMarker::none();
                let mut field59: bool = false;
                let mut field7 = ::roto::runtime::FieldMarker::none();
                let mut field16: i32 = 0i32;
                let mut field130: i32 = 0i32;
                let mut field12: bool = true;
                let mut field17: bool = true;
                let mut field13: bool = true;
                let mut field14: bool = true;
                let mut field104: i32 = 0i32;
                let mut field100: i32 = 0i32;
                let mut field101: i32 = 0i32;
                let mut field102 = ::roto::runtime::FieldMarker::none();
                let mut field103 = ::roto::runtime::FieldMarker::none();
                let mut field29: i32 = 0i32;
                let mut field30: bool = false;
                let mut field60: i32 = -1i32;
                let mut field271: i32 = -1i32;
                let mut field272: i32 = -1i32;
                let mut field150: i32 = 0i32;
                let mut field23: i32 = 0i32;
                let mut field24: bool = false;
                let mut field25: i32 = 0i32;
                let mut field15 = ::roto::runtime::FieldMarker::none();
                let mut field78: bool = false;
                let mut field67: i32 = 0i32;
                let mut field68: i32 = 0i32;
                let mut field128: i32 = 0i32;
                let mut field129 = ::roto::runtime::FieldMarker::none();
                let mut field131: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => field1 = field.marker,
                        (9, ::roto::runtime::WireType::LengthDelimited) => field9 = field.marker,
                        (18, ::roto::runtime::WireType::LengthDelimited) => field18 = field.marker,
                        (80, ::roto::runtime::WireType::Varint) => {
                            field80 = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (81, ::roto::runtime::WireType::Varint) => {
                            field81 = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            field2 = field.val as i32;
                            has_fields[0] |= 4;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            field3 = field.val as i32;
                            has_fields[0] |= 8;
                        }
                        (280, ::roto::runtime::WireType::Varint) => {
                            field280 = field.val as i32;
                            has_fields[0] |= 16;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            field6 = field.val as i32;
                            has_fields[0] |= 32;
                        }
                        (22, ::roto::runtime::WireType::Varint) => {
                            field22 = field.val as i64;
                            has_fields[0] |= 64;
                        }
                        (4, ::roto::runtime::WireType::LengthDelimited) => field4 = field.marker,
                        (5, _) => field5.extend(field.raw_span),
                        (59, ::roto::runtime::WireType::Varint) => {
                            field59 = field.val != 0;
                            has_fields[0] |= 128;
                        }
                        (7, ::roto::runtime::WireType::LengthDelimited) => field7 = field.marker,
                        (16, ::roto::runtime::WireType::Varint) => {
                            field16 = field.val as i32;
                            has_fields[0] |= 256;
                        }
                        (130, ::roto::runtime::WireType::Varint) => {
                            field130 = field.val as i32;
                            has_fields[0] |= 512;
                        }
                        (12, ::roto::runtime::WireType::Varint) => {
                            field12 = field.val != 0;
                            has_fields[0] |= 1024;
                        }
                        (17, ::roto::runtime::WireType::Varint) => {
                            field17 = field.val != 0;
                            has_fields[0] |= 2048;
                        }
                        (13, ::roto::runtime::WireType::Varint) => {
                            field13 = field.val != 0;
                            has_fields[0] |= 4096;
                        }
                        (14, ::roto::runtime::WireType::Varint) => {
                            field14 = field.val != 0;
                            has_fields[0] |= 8192;
                        }
                        (104, ::roto::runtime::WireType::Varint) => {
                            field104 = field.val as i32;
                            has_fields[0] |= 16384;
                        }
                        (100, ::roto::runtime::WireType::Varint) => {
                            field100 = field.val as i32;
                            has_fields[0] |= 32768;
                        }
                        (101, ::roto::runtime::WireType::Varint) => {
                            field101 = field.val as i32;
                            has_fields[0] |= 65536;
                        }
                        (102, ::roto::runtime::WireType::LengthDelimited) => field102 = field.marker,
                        (103, ::roto::runtime::WireType::LengthDelimited) => field103 = field.marker,
                        (29, ::roto::runtime::WireType::Varint) => {
                            field29 = field.val as i32;
                            has_fields[0] |= 131072;
                        }
                        (30, ::roto::runtime::WireType::Varint) => {
                            field30 = field.val != 0;
                            has_fields[0] |= 262144;
                        }
                        (60, ::roto::runtime::WireType::Varint) => {
                            field60 = field.val as i32;
                            has_fields[0] |= 524288;
                        }
                        (271, ::roto::runtime::WireType::Varint) => {
                            field271 = field.val as i32;
                            has_fields[0] |= 1048576;
                        }
                        (272, ::roto::runtime::WireType::Varint) => {
                            field272 = field.val as i32;
                            has_fields[0] |= 2097152;
                        }
                        (150, ::roto::runtime::WireType::Varint) => {
                            field150 = field.val as i32;
                            has_fields[0] |= 4194304;
                        }
                        (23, ::roto::runtime::WireType::Varint) => {
                            field23 = field.val as i32;
                            has_fields[0] |= 8388608;
                        }
                        (24, ::roto::runtime::WireType::Varint) => {
                            field24 = field.val != 0;
                            has_fields[0] |= 16777216;
                        }
                        (25, ::roto::runtime::WireType::Varint) => {
                            field25 = field.val as i32;
                            has_fields[0] |= 33554432;
                        }
                        (15, ::roto::runtime::WireType::LengthDelimited) => field15 = field.marker,
                        (78, ::roto::runtime::WireType::Varint) => {
                            field78 = field.val != 0;
                            has_fields[0] |= 67108864;
                        }
                        (67, ::roto::runtime::WireType::Varint) => {
                            field67 = field.val as i32;
                            has_fields[0] |= 134217728;
                        }
                        (68, ::roto::runtime::WireType::Varint) => {
                            field68 = field.val as i32;
                            has_fields[0] |= 268435456;
                        }
                        (128, ::roto::runtime::WireType::Varint) => {
                            field128 = field.val as i32;
                            has_fields[0] |= 536870912;
                        }
                        (129, ::roto::runtime::WireType::LengthDelimited) => field129 = field.marker,
                        (131, ::roto::runtime::WireType::Varint) => {
                            field131 = field.val as i32;
                            has_fields[0] |= 1073741824;
                        }
                        _ => {}
                    }
                }

                if !field1.is_present() {
                    return Err(::roto::runtime::DecodeError::MissingRequiredField("GoogleMessage1.field1"));
                }
                if (has_fields[0] & 4) == 0 {
                    return Err(::roto::runtime::DecodeError::MissingRequiredField("GoogleMessage1.field2"));
                }
                if (has_fields[0] & 8) == 0 {
                    return Err(::roto::runtime::DecodeError::MissingRequiredField("GoogleMessage1.field3"));
                }

                Ok(Self {
                    buf,
                    field1,
                    field9,
                    field18,
                    field80,
                    field81,
                    field2,
                    field3,
                    field280,
                    field6,
                    field22,
                    field4,
                    field5,
                    field59,
                    field7,
                    field16,
                    field130,
                    field12,
                    field17,
                    field13,
                    field14,
                    field104,
                    field100,
                    field101,
                    field102,
                    field103,
                    field29,
                    field30,
                    field60,
                    field271,
                    field272,
                    field150,
                    field23,
                    field24,
                    field25,
                    field15,
                    field78,
                    field67,
                    field68,
                    field128,
                    field129,
                    field131,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_field1(&self) -> bool {
                self.field1.is_present()
            }

            #[inline]
            pub fn field1(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                ::roto::runtime::decode_str_at(self.buf, self.field1)
            }

            #[inline]
            pub const fn has_field9(&self) -> bool {
                self.field9.is_present()
            }

            #[inline]
            pub fn field9(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field9.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field9)
            }

            #[inline]
            pub const fn has_field18(&self) -> bool {
                self.field18.is_present()
            }

            #[inline]
            pub fn field18(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field18.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field18)
            }

            #[inline]
            pub const fn has_field80(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn field80(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field80)
            }

            #[inline]
            pub const fn has_field81(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn field81(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field81)
            }

            #[inline]
            pub const fn has_field2(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn field2(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field2)
            }

            #[inline]
            pub const fn has_field3(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn field3(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field3)
            }

            #[inline]
            pub const fn has_field280(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn field280(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field280)
            }

            #[inline]
            pub const fn has_field6(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn field6(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field6)
            }

            #[inline]
            pub const fn has_field22(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn field22(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field22)
            }

            #[inline]
            pub const fn has_field4(&self) -> bool {
                self.field4.is_present()
            }

            #[inline]
            pub fn field4(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field4.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field4)
            }

            #[inline]
            pub const fn field5(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, u64> {
                ::roto::runtime::RepeatedFixed64Iter::new(self.field5.slice(self.buf), 5)
            }

            #[inline]
            pub const fn has_field59(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn field59(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field59)
            }

            #[inline]
            pub const fn has_field7(&self) -> bool {
                self.field7.is_present()
            }

            #[inline]
            pub fn field7(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field7.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field7)
            }

            #[inline]
            pub const fn has_field16(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn field16(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field16)
            }

            #[inline]
            pub const fn has_field130(&self) -> bool {
                (self.has_fields[0] & 512) != 0
            }

            #[inline]
            pub const fn field130(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field130)
            }

            #[inline]
            pub const fn has_field12(&self) -> bool {
                (self.has_fields[0] & 1024) != 0
            }

            #[inline]
            pub const fn field12(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field12)
            }

            #[inline]
            pub const fn has_field17(&self) -> bool {
                (self.has_fields[0] & 2048) != 0
            }

            #[inline]
            pub const fn field17(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field17)
            }

            #[inline]
            pub const fn has_field13(&self) -> bool {
                (self.has_fields[0] & 4096) != 0
            }

            #[inline]
            pub const fn field13(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field13)
            }

            #[inline]
            pub const fn has_field14(&self) -> bool {
                (self.has_fields[0] & 8192) != 0
            }

            #[inline]
            pub const fn field14(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field14)
            }

            #[inline]
            pub const fn has_field104(&self) -> bool {
                (self.has_fields[0] & 16384) != 0
            }

            #[inline]
            pub const fn field104(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field104)
            }

            #[inline]
            pub const fn has_field100(&self) -> bool {
                (self.has_fields[0] & 32768) != 0
            }

            #[inline]
            pub const fn field100(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field100)
            }

            #[inline]
            pub const fn has_field101(&self) -> bool {
                (self.has_fields[0] & 65536) != 0
            }

            #[inline]
            pub const fn field101(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field101)
            }

            #[inline]
            pub const fn has_field102(&self) -> bool {
                self.field102.is_present()
            }

            #[inline]
            pub fn field102(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field102.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field102)
            }

            #[inline]
            pub const fn has_field103(&self) -> bool {
                self.field103.is_present()
            }

            #[inline]
            pub fn field103(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field103.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field103)
            }

            #[inline]
            pub const fn has_field29(&self) -> bool {
                (self.has_fields[0] & 131072) != 0
            }

            #[inline]
            pub const fn field29(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field29)
            }

            #[inline]
            pub const fn has_field30(&self) -> bool {
                (self.has_fields[0] & 262144) != 0
            }

            #[inline]
            pub const fn field30(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field30)
            }

            #[inline]
            pub const fn has_field60(&self) -> bool {
                (self.has_fields[0] & 524288) != 0
            }

            #[inline]
            pub const fn field60(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field60)
            }

            #[inline]
            pub const fn has_field271(&self) -> bool {
                (self.has_fields[0] & 1048576) != 0
            }

            #[inline]
            pub const fn field271(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field271)
            }

            #[inline]
            pub const fn has_field272(&self) -> bool {
                (self.has_fields[0] & 2097152) != 0
            }

            #[inline]
            pub const fn field272(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field272)
            }

            #[inline]
            pub const fn has_field150(&self) -> bool {
                (self.has_fields[0] & 4194304) != 0
            }

            #[inline]
            pub const fn field150(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field150)
            }

            #[inline]
            pub const fn has_field23(&self) -> bool {
                (self.has_fields[0] & 8388608) != 0
            }

            #[inline]
            pub const fn field23(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field23)
            }

            #[inline]
            pub const fn has_field24(&self) -> bool {
                (self.has_fields[0] & 16777216) != 0
            }

            #[inline]
            pub const fn field24(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field24)
            }

            #[inline]
            pub const fn has_field25(&self) -> bool {
                (self.has_fields[0] & 33554432) != 0
            }

            #[inline]
            pub const fn field25(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field25)
            }

            #[inline]
            pub const fn has_field15(&self) -> bool {
                self.field15.is_present()
            }

            #[inline]
            pub fn field15(&self) -> Result<Option<crate::benchmarks::proto2::GoogleMessage1SubMessageView<'a>>, ::roto::runtime::DecodeError> {
                if !self.field15.is_present() {
                    return Ok(None);
                }
                crate::benchmarks::proto2::GoogleMessage1SubMessageView::from_bytes(self.field15.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_field78(&self) -> bool {
                (self.has_fields[0] & 67108864) != 0
            }

            #[inline]
            pub const fn field78(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field78)
            }

            #[inline]
            pub const fn has_field67(&self) -> bool {
                (self.has_fields[0] & 134217728) != 0
            }

            #[inline]
            pub const fn field67(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field67)
            }

            #[inline]
            pub const fn has_field68(&self) -> bool {
                (self.has_fields[0] & 268435456) != 0
            }

            #[inline]
            pub const fn field68(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field68)
            }

            #[inline]
            pub const fn has_field128(&self) -> bool {
                (self.has_fields[0] & 536870912) != 0
            }

            #[inline]
            pub const fn field128(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field128)
            }

            #[inline]
            pub const fn has_field129(&self) -> bool {
                self.field129.is_present()
            }

            #[inline]
            pub fn field129(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field129.is_present() {
                    return Ok("xxxxxxxxxxxxxxxxxxxxx");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field129)
            }

            #[inline]
            pub const fn has_field131(&self) -> bool {
                (self.has_fields[0] & 1073741824) != 0
            }

            #[inline]
            pub const fn field131(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field131)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for GoogleMessage1View<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct GoogleMessage1Builder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> GoogleMessage1Builder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &GoogleMessage1View<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoogleMessage1View::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &GoogleMessage1View<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    18 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    80 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    81 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    2 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    280 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    22 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 2048;
                        (prev_set[0] & 2048) == 0
                    }
                    59 => {
                        self.set_fields[0] |= 4096;
                        (prev_set[0] & 4096) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 16384;
                        (prev_set[0] & 16384) == 0
                    }
                    130 => {
                        self.set_fields[0] |= 32768;
                        (prev_set[0] & 32768) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 65536;
                        (prev_set[0] & 65536) == 0
                    }
                    17 => {
                        self.set_fields[0] |= 131072;
                        (prev_set[0] & 131072) == 0
                    }
                    13 => {
                        self.set_fields[0] |= 262144;
                        (prev_set[0] & 262144) == 0
                    }
                    14 => {
                        self.set_fields[0] |= 524288;
                        (prev_set[0] & 524288) == 0
                    }
                    104 => {
                        self.set_fields[0] |= 1048576;
                        (prev_set[0] & 1048576) == 0
                    }
                    100 => {
                        self.set_fields[0] |= 2097152;
                        (prev_set[0] & 2097152) == 0
                    }
                    101 => {
                        self.set_fields[0] |= 4194304;
                        (prev_set[0] & 4194304) == 0
                    }
                    102 => {
                        self.set_fields[0] |= 8388608;
                        (prev_set[0] & 8388608) == 0
                    }
                    103 => {
                        self.set_fields[0] |= 16777216;
                        (prev_set[0] & 16777216) == 0
                    }
                    29 => {
                        self.set_fields[0] |= 33554432;
                        (prev_set[0] & 33554432) == 0
                    }
                    30 => {
                        self.set_fields[0] |= 67108864;
                        (prev_set[0] & 67108864) == 0
                    }
                    60 => {
                        self.set_fields[0] |= 134217728;
                        (prev_set[0] & 134217728) == 0
                    }
                    271 => {
                        self.set_fields[0] |= 268435456;
                        (prev_set[0] & 268435456) == 0
                    }
                    272 => {
                        self.set_fields[0] |= 536870912;
                        (prev_set[0] & 536870912) == 0
                    }
                    150 => {
                        self.set_fields[0] |= 1073741824;
                        (prev_set[0] & 1073741824) == 0
                    }
                    23 => {
                        self.set_fields[0] |= 2147483648;
                        (prev_set[0] & 2147483648) == 0
                    }
                    24 => {
                        self.set_fields[0] |= 4294967296;
                        (prev_set[0] & 4294967296) == 0
                    }
                    25 => {
                        self.set_fields[0] |= 8589934592;
                        (prev_set[0] & 8589934592) == 0
                    }
                    15 => {
                        self.set_fields[0] |= 17179869184;
                        (prev_set[0] & 17179869184) == 0
                    }
                    78 => {
                        self.set_fields[0] |= 34359738368;
                        (prev_set[0] & 34359738368) == 0
                    }
                    67 => {
                        self.set_fields[0] |= 68719476736;
                        (prev_set[0] & 68719476736) == 0
                    }
                    68 => {
                        self.set_fields[0] |= 137438953472;
                        (prev_set[0] & 137438953472) == 0
                    }
                    128 => {
                        self.set_fields[0] |= 274877906944;
                        (prev_set[0] & 274877906944) == 0
                    }
                    129 => {
                        self.set_fields[0] |= 549755813888;
                        (prev_set[0] & 549755813888) == 0
                    }
                    131 => {
                        self.set_fields[0] |= 1099511627776;
                        (prev_set[0] & 1099511627776) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_field1(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field9(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field18(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(18, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field80(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(80, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field81(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(81, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field2(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field3(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field280(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(280, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field6(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field22(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field4(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field5(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 2048;
                let _ = self.writer.write_fixed64(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_packed_field5(&mut self, vals: &[u64]) -> &mut Self {
                self.set_fields[0] |= 2048;
                let end_pos = self.writer.cursor();
                for &val in vals.iter().rev() {
                    let _ = self.writer.write_fixed64(val);
                }
                let packed_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(packed_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field59(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(59, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field7(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field16(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 16384;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field130(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 32768;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(130, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field12(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 65536;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field17(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 131072;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field13(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 262144;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field14(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 524288;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field104(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1048576;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(104, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field100(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2097152;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(100, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field101(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4194304;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(101, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field102(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8388608;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(102, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field103(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16777216;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(103, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field29(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 33554432;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(29, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field30(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 67108864;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(30, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field60(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 134217728;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(60, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field271(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 268435456;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(271, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field272(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 536870912;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(272, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field150(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1073741824;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(150, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field23(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2147483648;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field24(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4294967296;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(24, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field25(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 8589934592;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(25, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field15<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::benchmarks::proto2::GoogleMessage1SubMessageBuilder<'_>),
            {
                self.set_fields[0] |= 17179869184;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::benchmarks::proto2::GoogleMessage1SubMessageBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field78(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 34359738368;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(78, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field67(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 68719476736;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(67, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field68(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 137438953472;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(68, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field128(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 274877906944;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(128, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field129(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 549755813888;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(129, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field131(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1099511627776;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(131, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct GoogleMessage1SubMessageView<'a> {
            buf: &'a [u8],
            field1: i32,
            field2: i32,
            field3: i32,
            field15: ::roto::runtime::FieldMarker,
            field12: bool,
            field13: i64,
            field14: i64,
            field16: i32,
            field19: i32,
            field20: bool,
            field28: bool,
            field21: u64,
            field22: i32,
            field23: bool,
            field206: bool,
            field203: u32,
            field204: i32,
            field205: ::roto::runtime::FieldMarker,
            field207: u64,
            field300: u64,
            has_fields: [u32; 1],
        }

        impl<'a> GoogleMessage1SubMessageView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 15, 12, 13, 14, 16, 19, 20, 28, 21, 22, 23, 206, 203, 204, 205, 207, 300];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field1: i32 = 0i32;
                let mut field2: i32 = 0i32;
                let mut field3: i32 = 0i32;
                let mut field15 = ::roto::runtime::FieldMarker::none();
                let mut field12: bool = true;
                let mut field13: i64 = 0i64;
                let mut field14: i64 = 0i64;
                let mut field16: i32 = 0i32;
                let mut field19: i32 = 2i32;
                let mut field20: bool = true;
                let mut field28: bool = true;
                let mut field21: u64 = 0u64;
                let mut field22: i32 = 0i32;
                let mut field23: bool = false;
                let mut field206: bool = false;
                let mut field203: u32 = 0u32;
                let mut field204: i32 = 0i32;
                let mut field205 = ::roto::runtime::FieldMarker::none();
                let mut field207: u64 = 0u64;
                let mut field300: u64 = 0u64;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            field1 = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            field2 = field.val as i32;
                            has_fields[0] |= 2;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            field3 = field.val as i32;
                            has_fields[0] |= 4;
                        }
                        (15, ::roto::runtime::WireType::LengthDelimited) => field15 = field.marker,
                        (12, ::roto::runtime::WireType::Varint) => {
                            field12 = field.val != 0;
                            has_fields[0] |= 8;
                        }
                        (13, ::roto::runtime::WireType::Varint) => {
                            field13 = field.val as i64;
                            has_fields[0] |= 16;
                        }
                        (14, ::roto::runtime::WireType::Varint) => {
                            field14 = field.val as i64;
                            has_fields[0] |= 32;
                        }
                        (16, ::roto::runtime::WireType::Varint) => {
                            field16 = field.val as i32;
                            has_fields[0] |= 64;
                        }
                        (19, ::roto::runtime::WireType::Varint) => {
                            field19 = field.val as i32;
                            has_fields[0] |= 128;
                        }
                        (20, ::roto::runtime::WireType::Varint) => {
                            field20 = field.val != 0;
                            has_fields[0] |= 256;
                        }
                        (28, ::roto::runtime::WireType::Varint) => {
                            field28 = field.val != 0;
                            has_fields[0] |= 512;
                        }
                        (21, ::roto::runtime::WireType::Fixed64) => {
                            field21 = field.val;
                            has_fields[0] |= 1024;
                        }
                        (22, ::roto::runtime::WireType::Varint) => {
                            field22 = field.val as i32;
                            has_fields[0] |= 2048;
                        }
                        (23, ::roto::runtime::WireType::Varint) => {
                            field23 = field.val != 0;
                            has_fields[0] |= 4096;
                        }
                        (206, ::roto::runtime::WireType::Varint) => {
                            field206 = field.val != 0;
                            has_fields[0] |= 8192;
                        }
                        (203, ::roto::runtime::WireType::Fixed32) => {
                            field203 = field.val as u32;
                            has_fields[0] |= 16384;
                        }
                        (204, ::roto::runtime::WireType::Varint) => {
                            field204 = field.val as i32;
                            has_fields[0] |= 32768;
                        }
                        (205, ::roto::runtime::WireType::LengthDelimited) => field205 = field.marker,
                        (207, ::roto::runtime::WireType::Varint) => {
                            field207 = field.val;
                            has_fields[0] |= 65536;
                        }
                        (300, ::roto::runtime::WireType::Varint) => {
                            field300 = field.val;
                            has_fields[0] |= 131072;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    field1,
                    field2,
                    field3,
                    field15,
                    field12,
                    field13,
                    field14,
                    field16,
                    field19,
                    field20,
                    field28,
                    field21,
                    field22,
                    field23,
                    field206,
                    field203,
                    field204,
                    field205,
                    field207,
                    field300,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_field1(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn field1(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field1)
            }

            #[inline]
            pub const fn has_field2(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn field2(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field2)
            }

            #[inline]
            pub const fn has_field3(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn field3(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field3)
            }

            #[inline]
            pub const fn has_field15(&self) -> bool {
                self.field15.is_present()
            }

            #[inline]
            pub fn field15(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field15.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field15)
            }

            #[inline]
            pub const fn has_field12(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn field12(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field12)
            }

            #[inline]
            pub const fn has_field13(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn field13(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field13)
            }

            #[inline]
            pub const fn has_field14(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn field14(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field14)
            }

            #[inline]
            pub const fn has_field16(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn field16(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field16)
            }

            #[inline]
            pub const fn has_field19(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn field19(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field19)
            }

            #[inline]
            pub const fn has_field20(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn field20(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field20)
            }

            #[inline]
            pub const fn has_field28(&self) -> bool {
                (self.has_fields[0] & 512) != 0
            }

            #[inline]
            pub const fn field28(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field28)
            }

            #[inline]
            pub const fn has_field21(&self) -> bool {
                (self.has_fields[0] & 1024) != 0
            }

            #[inline]
            pub const fn field21(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field21)
            }

            #[inline]
            pub const fn has_field22(&self) -> bool {
                (self.has_fields[0] & 2048) != 0
            }

            #[inline]
            pub const fn field22(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field22)
            }

            #[inline]
            pub const fn has_field23(&self) -> bool {
                (self.has_fields[0] & 4096) != 0
            }

            #[inline]
            pub const fn field23(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field23)
            }

            #[inline]
            pub const fn has_field206(&self) -> bool {
                (self.has_fields[0] & 8192) != 0
            }

            #[inline]
            pub const fn field206(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field206)
            }

            #[inline]
            pub const fn has_field203(&self) -> bool {
                (self.has_fields[0] & 16384) != 0
            }

            #[inline]
            pub const fn field203(&self) -> Result<u32, ::roto::runtime::DecodeError> {
                Ok(self.field203)
            }

            #[inline]
            pub const fn has_field204(&self) -> bool {
                (self.has_fields[0] & 32768) != 0
            }

            #[inline]
            pub const fn field204(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field204)
            }

            #[inline]
            pub const fn has_field205(&self) -> bool {
                self.field205.is_present()
            }

            #[inline]
            pub fn field205(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field205.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field205)
            }

            #[inline]
            pub const fn has_field207(&self) -> bool {
                (self.has_fields[0] & 65536) != 0
            }

            #[inline]
            pub const fn field207(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field207)
            }

            #[inline]
            pub const fn has_field300(&self) -> bool {
                (self.has_fields[0] & 131072) != 0
            }

            #[inline]
            pub const fn field300(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field300)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for GoogleMessage1SubMessageView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct GoogleMessage1SubMessageBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> GoogleMessage1SubMessageBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &GoogleMessage1SubMessageView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoogleMessage1SubMessageView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &GoogleMessage1SubMessageView<'_>) -> &mut Self {
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
                    15 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    13 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    14 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    19 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    20 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    28 => {
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
                    23 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    206 => {
                        self.set_fields[0] |= 16384;
                        (prev_set[0] & 16384) == 0
                    }
                    203 => {
                        self.set_fields[0] |= 32768;
                        (prev_set[0] & 32768) == 0
                    }
                    204 => {
                        self.set_fields[0] |= 65536;
                        (prev_set[0] & 65536) == 0
                    }
                    205 => {
                        self.set_fields[0] |= 131072;
                        (prev_set[0] & 131072) == 0
                    }
                    207 => {
                        self.set_fields[0] |= 262144;
                        (prev_set[0] & 262144) == 0
                    }
                    300 => {
                        self.set_fields[0] |= 524288;
                        (prev_set[0] & 524288) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_field1(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field2(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field3(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field15(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field12(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field13(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field14(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field16(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field19(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(19, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field20(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field28(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(28, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field21(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 2048;
                let _ = self.writer.write_fixed64(val);
                let _ = self.writer.write_tag(21, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_field22(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field23(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field206(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16384;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(206, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field203(&mut self, val: u32) -> &mut Self {
                self.set_fields[0] |= 32768;
                let _ = self.writer.write_fixed32(val);
                let _ = self.writer.write_tag(203, ::roto::runtime::WireType::Fixed32);
                self
            }

            #[inline]
            pub fn write_field204(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 65536;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(204, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field205(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 131072;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(205, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field207(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 262144;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(207, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field300(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 524288;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(300, ::roto::runtime::WireType::Varint);
                self
            }
        }
    }

    pub mod proto3 {
        #[derive(Clone, Copy, Debug)]
        pub struct GoogleMessage1View<'a> {
            buf: &'a [u8],
            field1: ::roto::runtime::FieldMarker,
            field9: ::roto::runtime::FieldMarker,
            field18: ::roto::runtime::FieldMarker,
            field80: bool,
            field81: bool,
            field2: i32,
            field3: i32,
            field280: i32,
            field6: i32,
            field22: i64,
            field4: ::roto::runtime::FieldMarker,
            field5: ::roto::runtime::FieldMarker,
            field59: bool,
            field7: ::roto::runtime::FieldMarker,
            field16: i32,
            field130: i32,
            field12: bool,
            field17: bool,
            field13: bool,
            field14: bool,
            field104: i32,
            field100: i32,
            field101: i32,
            field102: ::roto::runtime::FieldMarker,
            field103: ::roto::runtime::FieldMarker,
            field29: i32,
            field30: bool,
            field60: i32,
            field271: i32,
            field272: i32,
            field150: i32,
            field23: i32,
            field24: bool,
            field25: i32,
            field15: ::roto::runtime::FieldMarker,
            field78: bool,
            field67: i32,
            field68: i32,
            field128: i32,
            field129: ::roto::runtime::FieldMarker,
            field131: i32,
            has_fields: [u32; 1],
        }

        impl<'a> GoogleMessage1View<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 9, 18, 80, 81, 2, 3, 280, 6, 22, 4, 5, 59, 7, 16, 130, 12, 17, 13, 14, 104, 100, 101, 102, 103, 29, 30, 60, 271, 272, 150, 23, 24, 25, 15, 78, 67, 68, 128, 129, 131];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field1 = ::roto::runtime::FieldMarker::none();
                let mut field9 = ::roto::runtime::FieldMarker::none();
                let mut field18 = ::roto::runtime::FieldMarker::none();
                let mut field80: bool = false;
                let mut field81: bool = false;
                let mut field2: i32 = 0i32;
                let mut field3: i32 = 0i32;
                let mut field280: i32 = 0i32;
                let mut field6: i32 = 0i32;
                let mut field22: i64 = 0i64;
                let mut field4 = ::roto::runtime::FieldMarker::none();
                let mut field5 = ::roto::runtime::FieldMarker::none();
                let mut field59: bool = false;
                let mut field7 = ::roto::runtime::FieldMarker::none();
                let mut field16: i32 = 0i32;
                let mut field130: i32 = 0i32;
                let mut field12: bool = false;
                let mut field17: bool = false;
                let mut field13: bool = false;
                let mut field14: bool = false;
                let mut field104: i32 = 0i32;
                let mut field100: i32 = 0i32;
                let mut field101: i32 = 0i32;
                let mut field102 = ::roto::runtime::FieldMarker::none();
                let mut field103 = ::roto::runtime::FieldMarker::none();
                let mut field29: i32 = 0i32;
                let mut field30: bool = false;
                let mut field60: i32 = 0i32;
                let mut field271: i32 = 0i32;
                let mut field272: i32 = 0i32;
                let mut field150: i32 = 0i32;
                let mut field23: i32 = 0i32;
                let mut field24: bool = false;
                let mut field25: i32 = 0i32;
                let mut field15 = ::roto::runtime::FieldMarker::none();
                let mut field78: bool = false;
                let mut field67: i32 = 0i32;
                let mut field68: i32 = 0i32;
                let mut field128: i32 = 0i32;
                let mut field129 = ::roto::runtime::FieldMarker::none();
                let mut field131: i32 = 0i32;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::LengthDelimited) => field1 = field.marker,
                        (9, ::roto::runtime::WireType::LengthDelimited) => field9 = field.marker,
                        (18, ::roto::runtime::WireType::LengthDelimited) => field18 = field.marker,
                        (80, ::roto::runtime::WireType::Varint) => {
                            field80 = field.val != 0;
                            has_fields[0] |= 1;
                        }
                        (81, ::roto::runtime::WireType::Varint) => {
                            field81 = field.val != 0;
                            has_fields[0] |= 2;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            field2 = field.val as i32;
                            has_fields[0] |= 4;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            field3 = field.val as i32;
                            has_fields[0] |= 8;
                        }
                        (280, ::roto::runtime::WireType::Varint) => {
                            field280 = field.val as i32;
                            has_fields[0] |= 16;
                        }
                        (6, ::roto::runtime::WireType::Varint) => {
                            field6 = field.val as i32;
                            has_fields[0] |= 32;
                        }
                        (22, ::roto::runtime::WireType::Varint) => {
                            field22 = field.val as i64;
                            has_fields[0] |= 64;
                        }
                        (4, ::roto::runtime::WireType::LengthDelimited) => field4 = field.marker,
                        (5, _) => field5.extend(field.raw_span),
                        (59, ::roto::runtime::WireType::Varint) => {
                            field59 = field.val != 0;
                            has_fields[0] |= 128;
                        }
                        (7, ::roto::runtime::WireType::LengthDelimited) => field7 = field.marker,
                        (16, ::roto::runtime::WireType::Varint) => {
                            field16 = field.val as i32;
                            has_fields[0] |= 256;
                        }
                        (130, ::roto::runtime::WireType::Varint) => {
                            field130 = field.val as i32;
                            has_fields[0] |= 512;
                        }
                        (12, ::roto::runtime::WireType::Varint) => {
                            field12 = field.val != 0;
                            has_fields[0] |= 1024;
                        }
                        (17, ::roto::runtime::WireType::Varint) => {
                            field17 = field.val != 0;
                            has_fields[0] |= 2048;
                        }
                        (13, ::roto::runtime::WireType::Varint) => {
                            field13 = field.val != 0;
                            has_fields[0] |= 4096;
                        }
                        (14, ::roto::runtime::WireType::Varint) => {
                            field14 = field.val != 0;
                            has_fields[0] |= 8192;
                        }
                        (104, ::roto::runtime::WireType::Varint) => {
                            field104 = field.val as i32;
                            has_fields[0] |= 16384;
                        }
                        (100, ::roto::runtime::WireType::Varint) => {
                            field100 = field.val as i32;
                            has_fields[0] |= 32768;
                        }
                        (101, ::roto::runtime::WireType::Varint) => {
                            field101 = field.val as i32;
                            has_fields[0] |= 65536;
                        }
                        (102, ::roto::runtime::WireType::LengthDelimited) => field102 = field.marker,
                        (103, ::roto::runtime::WireType::LengthDelimited) => field103 = field.marker,
                        (29, ::roto::runtime::WireType::Varint) => {
                            field29 = field.val as i32;
                            has_fields[0] |= 131072;
                        }
                        (30, ::roto::runtime::WireType::Varint) => {
                            field30 = field.val != 0;
                            has_fields[0] |= 262144;
                        }
                        (60, ::roto::runtime::WireType::Varint) => {
                            field60 = field.val as i32;
                            has_fields[0] |= 524288;
                        }
                        (271, ::roto::runtime::WireType::Varint) => {
                            field271 = field.val as i32;
                            has_fields[0] |= 1048576;
                        }
                        (272, ::roto::runtime::WireType::Varint) => {
                            field272 = field.val as i32;
                            has_fields[0] |= 2097152;
                        }
                        (150, ::roto::runtime::WireType::Varint) => {
                            field150 = field.val as i32;
                            has_fields[0] |= 4194304;
                        }
                        (23, ::roto::runtime::WireType::Varint) => {
                            field23 = field.val as i32;
                            has_fields[0] |= 8388608;
                        }
                        (24, ::roto::runtime::WireType::Varint) => {
                            field24 = field.val != 0;
                            has_fields[0] |= 16777216;
                        }
                        (25, ::roto::runtime::WireType::Varint) => {
                            field25 = field.val as i32;
                            has_fields[0] |= 33554432;
                        }
                        (15, ::roto::runtime::WireType::LengthDelimited) => field15 = field.marker,
                        (78, ::roto::runtime::WireType::Varint) => {
                            field78 = field.val != 0;
                            has_fields[0] |= 67108864;
                        }
                        (67, ::roto::runtime::WireType::Varint) => {
                            field67 = field.val as i32;
                            has_fields[0] |= 134217728;
                        }
                        (68, ::roto::runtime::WireType::Varint) => {
                            field68 = field.val as i32;
                            has_fields[0] |= 268435456;
                        }
                        (128, ::roto::runtime::WireType::Varint) => {
                            field128 = field.val as i32;
                            has_fields[0] |= 536870912;
                        }
                        (129, ::roto::runtime::WireType::LengthDelimited) => field129 = field.marker,
                        (131, ::roto::runtime::WireType::Varint) => {
                            field131 = field.val as i32;
                            has_fields[0] |= 1073741824;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    field1,
                    field9,
                    field18,
                    field80,
                    field81,
                    field2,
                    field3,
                    field280,
                    field6,
                    field22,
                    field4,
                    field5,
                    field59,
                    field7,
                    field16,
                    field130,
                    field12,
                    field17,
                    field13,
                    field14,
                    field104,
                    field100,
                    field101,
                    field102,
                    field103,
                    field29,
                    field30,
                    field60,
                    field271,
                    field272,
                    field150,
                    field23,
                    field24,
                    field25,
                    field15,
                    field78,
                    field67,
                    field68,
                    field128,
                    field129,
                    field131,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_field1(&self) -> bool {
                self.field1.is_present()
            }

            #[inline]
            pub fn field1(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field1.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field1)
            }

            #[inline]
            pub const fn has_field9(&self) -> bool {
                self.field9.is_present()
            }

            #[inline]
            pub fn field9(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field9.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field9)
            }

            #[inline]
            pub const fn has_field18(&self) -> bool {
                self.field18.is_present()
            }

            #[inline]
            pub fn field18(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field18.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field18)
            }

            #[inline]
            pub const fn has_field80(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn field80(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field80)
            }

            #[inline]
            pub const fn has_field81(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn field81(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field81)
            }

            #[inline]
            pub const fn has_field2(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn field2(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field2)
            }

            #[inline]
            pub const fn has_field3(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn field3(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field3)
            }

            #[inline]
            pub const fn has_field280(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn field280(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field280)
            }

            #[inline]
            pub const fn has_field6(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn field6(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field6)
            }

            #[inline]
            pub const fn has_field22(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn field22(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field22)
            }

            #[inline]
            pub const fn has_field4(&self) -> bool {
                self.field4.is_present()
            }

            #[inline]
            pub fn field4(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field4.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field4)
            }

            #[inline]
            pub const fn field5(&self) -> ::roto::runtime::RepeatedFixed64Iter<'a, u64> {
                ::roto::runtime::RepeatedFixed64Iter::new(self.field5.slice(self.buf), 5)
            }

            #[inline]
            pub const fn has_field59(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn field59(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field59)
            }

            #[inline]
            pub const fn has_field7(&self) -> bool {
                self.field7.is_present()
            }

            #[inline]
            pub fn field7(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field7.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field7)
            }

            #[inline]
            pub const fn has_field16(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn field16(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field16)
            }

            #[inline]
            pub const fn has_field130(&self) -> bool {
                (self.has_fields[0] & 512) != 0
            }

            #[inline]
            pub const fn field130(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field130)
            }

            #[inline]
            pub const fn has_field12(&self) -> bool {
                (self.has_fields[0] & 1024) != 0
            }

            #[inline]
            pub const fn field12(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field12)
            }

            #[inline]
            pub const fn has_field17(&self) -> bool {
                (self.has_fields[0] & 2048) != 0
            }

            #[inline]
            pub const fn field17(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field17)
            }

            #[inline]
            pub const fn has_field13(&self) -> bool {
                (self.has_fields[0] & 4096) != 0
            }

            #[inline]
            pub const fn field13(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field13)
            }

            #[inline]
            pub const fn has_field14(&self) -> bool {
                (self.has_fields[0] & 8192) != 0
            }

            #[inline]
            pub const fn field14(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field14)
            }

            #[inline]
            pub const fn has_field104(&self) -> bool {
                (self.has_fields[0] & 16384) != 0
            }

            #[inline]
            pub const fn field104(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field104)
            }

            #[inline]
            pub const fn has_field100(&self) -> bool {
                (self.has_fields[0] & 32768) != 0
            }

            #[inline]
            pub const fn field100(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field100)
            }

            #[inline]
            pub const fn has_field101(&self) -> bool {
                (self.has_fields[0] & 65536) != 0
            }

            #[inline]
            pub const fn field101(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field101)
            }

            #[inline]
            pub const fn has_field102(&self) -> bool {
                self.field102.is_present()
            }

            #[inline]
            pub fn field102(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field102.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field102)
            }

            #[inline]
            pub const fn has_field103(&self) -> bool {
                self.field103.is_present()
            }

            #[inline]
            pub fn field103(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field103.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field103)
            }

            #[inline]
            pub const fn has_field29(&self) -> bool {
                (self.has_fields[0] & 131072) != 0
            }

            #[inline]
            pub const fn field29(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field29)
            }

            #[inline]
            pub const fn has_field30(&self) -> bool {
                (self.has_fields[0] & 262144) != 0
            }

            #[inline]
            pub const fn field30(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field30)
            }

            #[inline]
            pub const fn has_field60(&self) -> bool {
                (self.has_fields[0] & 524288) != 0
            }

            #[inline]
            pub const fn field60(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field60)
            }

            #[inline]
            pub const fn has_field271(&self) -> bool {
                (self.has_fields[0] & 1048576) != 0
            }

            #[inline]
            pub const fn field271(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field271)
            }

            #[inline]
            pub const fn has_field272(&self) -> bool {
                (self.has_fields[0] & 2097152) != 0
            }

            #[inline]
            pub const fn field272(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field272)
            }

            #[inline]
            pub const fn has_field150(&self) -> bool {
                (self.has_fields[0] & 4194304) != 0
            }

            #[inline]
            pub const fn field150(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field150)
            }

            #[inline]
            pub const fn has_field23(&self) -> bool {
                (self.has_fields[0] & 8388608) != 0
            }

            #[inline]
            pub const fn field23(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field23)
            }

            #[inline]
            pub const fn has_field24(&self) -> bool {
                (self.has_fields[0] & 16777216) != 0
            }

            #[inline]
            pub const fn field24(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field24)
            }

            #[inline]
            pub const fn has_field25(&self) -> bool {
                (self.has_fields[0] & 33554432) != 0
            }

            #[inline]
            pub const fn field25(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field25)
            }

            #[inline]
            pub const fn has_field15(&self) -> bool {
                self.field15.is_present()
            }

            #[inline]
            pub fn field15(&self) -> Result<Option<crate::benchmarks::proto3::GoogleMessage1SubMessageView<'a>>, ::roto::runtime::DecodeError> {
                if !self.field15.is_present() {
                    return Ok(None);
                }
                crate::benchmarks::proto3::GoogleMessage1SubMessageView::from_bytes(self.field15.slice(self.buf)).map(Some)
            }

            #[inline]
            pub const fn has_field78(&self) -> bool {
                (self.has_fields[0] & 67108864) != 0
            }

            #[inline]
            pub const fn field78(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field78)
            }

            #[inline]
            pub const fn has_field67(&self) -> bool {
                (self.has_fields[0] & 134217728) != 0
            }

            #[inline]
            pub const fn field67(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field67)
            }

            #[inline]
            pub const fn has_field68(&self) -> bool {
                (self.has_fields[0] & 268435456) != 0
            }

            #[inline]
            pub const fn field68(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field68)
            }

            #[inline]
            pub const fn has_field128(&self) -> bool {
                (self.has_fields[0] & 536870912) != 0
            }

            #[inline]
            pub const fn field128(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field128)
            }

            #[inline]
            pub const fn has_field129(&self) -> bool {
                self.field129.is_present()
            }

            #[inline]
            pub fn field129(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field129.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field129)
            }

            #[inline]
            pub const fn has_field131(&self) -> bool {
                (self.has_fields[0] & 1073741824) != 0
            }

            #[inline]
            pub const fn field131(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field131)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for GoogleMessage1View<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct GoogleMessage1Builder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> GoogleMessage1Builder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &GoogleMessage1View<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoogleMessage1View::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &GoogleMessage1View<'_>) -> &mut Self {
                let prev_set = self.set_fields;
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
                    1 => {
                        self.set_fields[0] |= 1;
                        (prev_set[0] & 1) == 0
                    }
                    9 => {
                        self.set_fields[0] |= 2;
                        (prev_set[0] & 2) == 0
                    }
                    18 => {
                        self.set_fields[0] |= 4;
                        (prev_set[0] & 4) == 0
                    }
                    80 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    81 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    2 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    3 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    280 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    6 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    22 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    4 => {
                        self.set_fields[0] |= 1024;
                        (prev_set[0] & 1024) == 0
                    }
                    5 => {
                        self.set_fields[0] |= 2048;
                        (prev_set[0] & 2048) == 0
                    }
                    59 => {
                        self.set_fields[0] |= 4096;
                        (prev_set[0] & 4096) == 0
                    }
                    7 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 16384;
                        (prev_set[0] & 16384) == 0
                    }
                    130 => {
                        self.set_fields[0] |= 32768;
                        (prev_set[0] & 32768) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 65536;
                        (prev_set[0] & 65536) == 0
                    }
                    17 => {
                        self.set_fields[0] |= 131072;
                        (prev_set[0] & 131072) == 0
                    }
                    13 => {
                        self.set_fields[0] |= 262144;
                        (prev_set[0] & 262144) == 0
                    }
                    14 => {
                        self.set_fields[0] |= 524288;
                        (prev_set[0] & 524288) == 0
                    }
                    104 => {
                        self.set_fields[0] |= 1048576;
                        (prev_set[0] & 1048576) == 0
                    }
                    100 => {
                        self.set_fields[0] |= 2097152;
                        (prev_set[0] & 2097152) == 0
                    }
                    101 => {
                        self.set_fields[0] |= 4194304;
                        (prev_set[0] & 4194304) == 0
                    }
                    102 => {
                        self.set_fields[0] |= 8388608;
                        (prev_set[0] & 8388608) == 0
                    }
                    103 => {
                        self.set_fields[0] |= 16777216;
                        (prev_set[0] & 16777216) == 0
                    }
                    29 => {
                        self.set_fields[0] |= 33554432;
                        (prev_set[0] & 33554432) == 0
                    }
                    30 => {
                        self.set_fields[0] |= 67108864;
                        (prev_set[0] & 67108864) == 0
                    }
                    60 => {
                        self.set_fields[0] |= 134217728;
                        (prev_set[0] & 134217728) == 0
                    }
                    271 => {
                        self.set_fields[0] |= 268435456;
                        (prev_set[0] & 268435456) == 0
                    }
                    272 => {
                        self.set_fields[0] |= 536870912;
                        (prev_set[0] & 536870912) == 0
                    }
                    150 => {
                        self.set_fields[0] |= 1073741824;
                        (prev_set[0] & 1073741824) == 0
                    }
                    23 => {
                        self.set_fields[0] |= 2147483648;
                        (prev_set[0] & 2147483648) == 0
                    }
                    24 => {
                        self.set_fields[0] |= 4294967296;
                        (prev_set[0] & 4294967296) == 0
                    }
                    25 => {
                        self.set_fields[0] |= 8589934592;
                        (prev_set[0] & 8589934592) == 0
                    }
                    15 => {
                        self.set_fields[0] |= 17179869184;
                        (prev_set[0] & 17179869184) == 0
                    }
                    78 => {
                        self.set_fields[0] |= 34359738368;
                        (prev_set[0] & 34359738368) == 0
                    }
                    67 => {
                        self.set_fields[0] |= 68719476736;
                        (prev_set[0] & 68719476736) == 0
                    }
                    68 => {
                        self.set_fields[0] |= 137438953472;
                        (prev_set[0] & 137438953472) == 0
                    }
                    128 => {
                        self.set_fields[0] |= 274877906944;
                        (prev_set[0] & 274877906944) == 0
                    }
                    129 => {
                        self.set_fields[0] |= 549755813888;
                        (prev_set[0] & 549755813888) == 0
                    }
                    131 => {
                        self.set_fields[0] |= 1099511627776;
                        (prev_set[0] & 1099511627776) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_field1(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field9(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(9, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field18(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(18, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field80(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(80, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field81(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(81, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field2(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field3(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field280(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(280, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field6(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field22(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field4(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field5(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 2048;
                let _ = self.writer.write_fixed64(val);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_packed_field5(&mut self, vals: &[u64]) -> &mut Self {
                self.set_fields[0] |= 2048;
                let end_pos = self.writer.cursor();
                for &val in vals.iter().rev() {
                    let _ = self.writer.write_fixed64(val);
                }
                let packed_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(packed_len as u64);
                let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field59(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(59, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field7(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field16(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 16384;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field130(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 32768;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(130, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field12(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 65536;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field17(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 131072;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(17, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field13(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 262144;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field14(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 524288;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field104(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1048576;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(104, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field100(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2097152;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(100, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field101(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4194304;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(101, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field102(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8388608;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(102, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field103(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 16777216;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(103, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field29(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 33554432;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(29, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field30(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 67108864;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(30, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field60(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 134217728;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(60, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field271(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 268435456;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(271, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field272(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 536870912;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(272, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field150(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1073741824;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(150, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field23(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2147483648;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field24(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 4294967296;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(24, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field25(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 8589934592;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(25, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field15<F>(&mut self, f: F) -> &mut Self
            where
                F: FnOnce(&mut crate::benchmarks::proto3::GoogleMessage1SubMessageBuilder<'_>),
            {
                self.set_fields[0] |= 17179869184;
                let end_pos = self.writer.cursor();
                self.writer.with_sub_writer(|sub_writer| {
                    let mut sub_builder = crate::benchmarks::proto3::GoogleMessage1SubMessageBuilder::from_writer(sub_writer);
                    f(&mut sub_builder);
                    sub_builder.writer
                });
                let nested_len = end_pos - self.writer.cursor();
                let _ = self.writer.write_varint(nested_len as u64);
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field78(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 34359738368;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(78, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field67(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 68719476736;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(67, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field68(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 137438953472;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(68, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field128(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 274877906944;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(128, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field129(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 549755813888;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(129, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field131(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1099511627776;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(131, ::roto::runtime::WireType::Varint);
                self
            }
        }

        #[derive(Clone, Copy, Debug)]
        pub struct GoogleMessage1SubMessageView<'a> {
            buf: &'a [u8],
            field1: i32,
            field2: i32,
            field3: i32,
            field15: ::roto::runtime::FieldMarker,
            field12: bool,
            field13: i64,
            field14: i64,
            field16: i32,
            field19: i32,
            field20: bool,
            field28: bool,
            field21: u64,
            field22: i32,
            field23: bool,
            field206: bool,
            field203: u32,
            field204: i32,
            field205: ::roto::runtime::FieldMarker,
            field207: u64,
            field300: u64,
            has_fields: [u32; 1],
        }

        impl<'a> GoogleMessage1SubMessageView<'a> {
            pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 15, 12, 13, 14, 16, 19, 20, 28, 21, 22, 23, 206, 203, 204, 205, 207, 300];

            pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                let mut field1: i32 = 0i32;
                let mut field2: i32 = 0i32;
                let mut field3: i32 = 0i32;
                let mut field15 = ::roto::runtime::FieldMarker::none();
                let mut field12: bool = false;
                let mut field13: i64 = 0i64;
                let mut field14: i64 = 0i64;
                let mut field16: i32 = 0i32;
                let mut field19: i32 = 0i32;
                let mut field20: bool = false;
                let mut field28: bool = false;
                let mut field21: u64 = 0u64;
                let mut field22: i32 = 0i32;
                let mut field23: bool = false;
                let mut field206: bool = false;
                let mut field203: u32 = 0u32;
                let mut field204: i32 = 0i32;
                let mut field205 = ::roto::runtime::FieldMarker::none();
                let mut field207: u64 = 0u64;
                let mut field300: u64 = 0u64;
                let mut has_fields = [0u32; 1];

                for item in ::roto::runtime::FieldIter::new(buf) {
                    let field = item?;
                    match (field.tag, field.wire_type) {
                        (1, ::roto::runtime::WireType::Varint) => {
                            field1 = field.val as i32;
                            has_fields[0] |= 1;
                        }
                        (2, ::roto::runtime::WireType::Varint) => {
                            field2 = field.val as i32;
                            has_fields[0] |= 2;
                        }
                        (3, ::roto::runtime::WireType::Varint) => {
                            field3 = field.val as i32;
                            has_fields[0] |= 4;
                        }
                        (15, ::roto::runtime::WireType::LengthDelimited) => field15 = field.marker,
                        (12, ::roto::runtime::WireType::Varint) => {
                            field12 = field.val != 0;
                            has_fields[0] |= 8;
                        }
                        (13, ::roto::runtime::WireType::Varint) => {
                            field13 = field.val as i64;
                            has_fields[0] |= 16;
                        }
                        (14, ::roto::runtime::WireType::Varint) => {
                            field14 = field.val as i64;
                            has_fields[0] |= 32;
                        }
                        (16, ::roto::runtime::WireType::Varint) => {
                            field16 = field.val as i32;
                            has_fields[0] |= 64;
                        }
                        (19, ::roto::runtime::WireType::Varint) => {
                            field19 = field.val as i32;
                            has_fields[0] |= 128;
                        }
                        (20, ::roto::runtime::WireType::Varint) => {
                            field20 = field.val != 0;
                            has_fields[0] |= 256;
                        }
                        (28, ::roto::runtime::WireType::Varint) => {
                            field28 = field.val != 0;
                            has_fields[0] |= 512;
                        }
                        (21, ::roto::runtime::WireType::Fixed64) => {
                            field21 = field.val;
                            has_fields[0] |= 1024;
                        }
                        (22, ::roto::runtime::WireType::Varint) => {
                            field22 = field.val as i32;
                            has_fields[0] |= 2048;
                        }
                        (23, ::roto::runtime::WireType::Varint) => {
                            field23 = field.val != 0;
                            has_fields[0] |= 4096;
                        }
                        (206, ::roto::runtime::WireType::Varint) => {
                            field206 = field.val != 0;
                            has_fields[0] |= 8192;
                        }
                        (203, ::roto::runtime::WireType::Fixed32) => {
                            field203 = field.val as u32;
                            has_fields[0] |= 16384;
                        }
                        (204, ::roto::runtime::WireType::Varint) => {
                            field204 = field.val as i32;
                            has_fields[0] |= 32768;
                        }
                        (205, ::roto::runtime::WireType::LengthDelimited) => field205 = field.marker,
                        (207, ::roto::runtime::WireType::Varint) => {
                            field207 = field.val;
                            has_fields[0] |= 65536;
                        }
                        (300, ::roto::runtime::WireType::Varint) => {
                            field300 = field.val;
                            has_fields[0] |= 131072;
                        }
                        _ => {}
                    }
                }

                Ok(Self {
                    buf,
                    field1,
                    field2,
                    field3,
                    field15,
                    field12,
                    field13,
                    field14,
                    field16,
                    field19,
                    field20,
                    field28,
                    field21,
                    field22,
                    field23,
                    field206,
                    field203,
                    field204,
                    field205,
                    field207,
                    field300,
                    has_fields,
                })
            }

            #[inline]
            pub const fn as_bytes(&self) -> &'a [u8] {
                self.buf
            }

            #[inline]
            pub const fn has_field1(&self) -> bool {
                (self.has_fields[0] & 1) != 0
            }

            #[inline]
            pub const fn field1(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field1)
            }

            #[inline]
            pub const fn has_field2(&self) -> bool {
                (self.has_fields[0] & 2) != 0
            }

            #[inline]
            pub const fn field2(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field2)
            }

            #[inline]
            pub const fn has_field3(&self) -> bool {
                (self.has_fields[0] & 4) != 0
            }

            #[inline]
            pub const fn field3(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field3)
            }

            #[inline]
            pub const fn has_field15(&self) -> bool {
                self.field15.is_present()
            }

            #[inline]
            pub fn field15(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field15.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field15)
            }

            #[inline]
            pub const fn has_field12(&self) -> bool {
                (self.has_fields[0] & 8) != 0
            }

            #[inline]
            pub const fn field12(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field12)
            }

            #[inline]
            pub const fn has_field13(&self) -> bool {
                (self.has_fields[0] & 16) != 0
            }

            #[inline]
            pub const fn field13(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field13)
            }

            #[inline]
            pub const fn has_field14(&self) -> bool {
                (self.has_fields[0] & 32) != 0
            }

            #[inline]
            pub const fn field14(&self) -> Result<i64, ::roto::runtime::DecodeError> {
                Ok(self.field14)
            }

            #[inline]
            pub const fn has_field16(&self) -> bool {
                (self.has_fields[0] & 64) != 0
            }

            #[inline]
            pub const fn field16(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field16)
            }

            #[inline]
            pub const fn has_field19(&self) -> bool {
                (self.has_fields[0] & 128) != 0
            }

            #[inline]
            pub const fn field19(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field19)
            }

            #[inline]
            pub const fn has_field20(&self) -> bool {
                (self.has_fields[0] & 256) != 0
            }

            #[inline]
            pub const fn field20(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field20)
            }

            #[inline]
            pub const fn has_field28(&self) -> bool {
                (self.has_fields[0] & 512) != 0
            }

            #[inline]
            pub const fn field28(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field28)
            }

            #[inline]
            pub const fn has_field21(&self) -> bool {
                (self.has_fields[0] & 1024) != 0
            }

            #[inline]
            pub const fn field21(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field21)
            }

            #[inline]
            pub const fn has_field22(&self) -> bool {
                (self.has_fields[0] & 2048) != 0
            }

            #[inline]
            pub const fn field22(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field22)
            }

            #[inline]
            pub const fn has_field23(&self) -> bool {
                (self.has_fields[0] & 4096) != 0
            }

            #[inline]
            pub const fn field23(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field23)
            }

            #[inline]
            pub const fn has_field206(&self) -> bool {
                (self.has_fields[0] & 8192) != 0
            }

            #[inline]
            pub const fn field206(&self) -> Result<bool, ::roto::runtime::DecodeError> {
                Ok(self.field206)
            }

            #[inline]
            pub const fn has_field203(&self) -> bool {
                (self.has_fields[0] & 16384) != 0
            }

            #[inline]
            pub const fn field203(&self) -> Result<u32, ::roto::runtime::DecodeError> {
                Ok(self.field203)
            }

            #[inline]
            pub const fn has_field204(&self) -> bool {
                (self.has_fields[0] & 32768) != 0
            }

            #[inline]
            pub const fn field204(&self) -> Result<i32, ::roto::runtime::DecodeError> {
                Ok(self.field204)
            }

            #[inline]
            pub const fn has_field205(&self) -> bool {
                self.field205.is_present()
            }

            #[inline]
            pub fn field205(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
                if !self.field205.is_present() {
                    return Ok("");
                }
                ::roto::runtime::decode_str_at(self.buf, self.field205)
            }

            #[inline]
            pub const fn has_field207(&self) -> bool {
                (self.has_fields[0] & 65536) != 0
            }

            #[inline]
            pub const fn field207(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field207)
            }

            #[inline]
            pub const fn has_field300(&self) -> bool {
                (self.has_fields[0] & 131072) != 0
            }

            #[inline]
            pub const fn field300(&self) -> Result<u64, ::roto::runtime::DecodeError> {
                Ok(self.field300)
            }

            #[inline]
            pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
                ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
            }
        }

        impl<'a> ::roto::runtime::MessageView<'a> for GoogleMessage1SubMessageView<'a> {
            #[inline]
            fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
                Self::from_bytes(buf)
            }
        }

        pub struct GoogleMessage1SubMessageBuilder<'b> {
            pub writer: ::roto::runtime::BackwardWriter<'b>,
            set_fields: [u64; 1],
        }

        impl<'b> GoogleMessage1SubMessageBuilder<'b> {
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
            pub fn copy_unknown_fields(&mut self, view: &GoogleMessage1SubMessageView<'_>) -> &mut Self {
                let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoogleMessage1SubMessageView::KNOWN_TAGS.contains(&tag));
                self
            }

            #[inline]
            pub fn copy_unset_fields(&mut self, view: &GoogleMessage1SubMessageView<'_>) -> &mut Self {
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
                    15 => {
                        self.set_fields[0] |= 8;
                        (prev_set[0] & 8) == 0
                    }
                    12 => {
                        self.set_fields[0] |= 16;
                        (prev_set[0] & 16) == 0
                    }
                    13 => {
                        self.set_fields[0] |= 32;
                        (prev_set[0] & 32) == 0
                    }
                    14 => {
                        self.set_fields[0] |= 64;
                        (prev_set[0] & 64) == 0
                    }
                    16 => {
                        self.set_fields[0] |= 128;
                        (prev_set[0] & 128) == 0
                    }
                    19 => {
                        self.set_fields[0] |= 256;
                        (prev_set[0] & 256) == 0
                    }
                    20 => {
                        self.set_fields[0] |= 512;
                        (prev_set[0] & 512) == 0
                    }
                    28 => {
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
                    23 => {
                        self.set_fields[0] |= 8192;
                        (prev_set[0] & 8192) == 0
                    }
                    206 => {
                        self.set_fields[0] |= 16384;
                        (prev_set[0] & 16384) == 0
                    }
                    203 => {
                        self.set_fields[0] |= 32768;
                        (prev_set[0] & 32768) == 0
                    }
                    204 => {
                        self.set_fields[0] |= 65536;
                        (prev_set[0] & 65536) == 0
                    }
                    205 => {
                        self.set_fields[0] |= 131072;
                        (prev_set[0] & 131072) == 0
                    }
                    207 => {
                        self.set_fields[0] |= 262144;
                        (prev_set[0] & 262144) == 0
                    }
                    300 => {
                        self.set_fields[0] |= 524288;
                        (prev_set[0] & 524288) == 0
                    }
                    _ => true,
                });
                self
            }

            #[inline]
            pub fn write_field1(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 1;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field2(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 2;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field3(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field15(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 8;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(15, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field12(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(12, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field13(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 32;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(13, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field14(&mut self, val: i64) -> &mut Self {
                self.set_fields[0] |= 64;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(14, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field16(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 128;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(16, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field19(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 256;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(19, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field20(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 512;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(20, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field28(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 1024;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(28, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field21(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 2048;
                let _ = self.writer.write_fixed64(val);
                let _ = self.writer.write_tag(21, ::roto::runtime::WireType::Fixed64);
                self
            }

            #[inline]
            pub fn write_field22(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 4096;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(22, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field23(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 8192;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(23, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field206(&mut self, val: bool) -> &mut Self {
                self.set_fields[0] |= 16384;
                let _ = self.writer.write_bool(val);
                let _ = self.writer.write_tag(206, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field203(&mut self, val: u32) -> &mut Self {
                self.set_fields[0] |= 32768;
                let _ = self.writer.write_fixed32(val);
                let _ = self.writer.write_tag(203, ::roto::runtime::WireType::Fixed32);
                self
            }

            #[inline]
            pub fn write_field204(&mut self, val: i32) -> &mut Self {
                self.set_fields[0] |= 65536;
                let _ = self.writer.write_varint(val as u64);
                let _ = self.writer.write_tag(204, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field205(&mut self, val: &str) -> &mut Self {
                self.set_fields[0] |= 131072;
                let _ = self.writer.write_length_delimited(val.as_bytes());
                let _ = self.writer.write_tag(205, ::roto::runtime::WireType::LengthDelimited);
                self
            }

            #[inline]
            pub fn write_field207(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 262144;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(207, ::roto::runtime::WireType::Varint);
                self
            }

            #[inline]
            pub fn write_field300(&mut self, val: u64) -> &mut Self {
                self.set_fields[0] |= 524288;
                let _ = self.writer.write_varint(val);
                let _ = self.writer.write_tag(300, ::roto::runtime::WireType::Varint);
                self
            }
        }
    }
}

pub mod perftest_data {
    #[derive(Clone, Copy, Debug)]
    pub struct Test1View<'a> {
        buf: &'a [u8],
        value: i32,
        has_fields: [u32; 1],
    }

    impl<'a> Test1View<'a> {
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

    impl<'a> ::roto::runtime::MessageView<'a> for Test1View<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct Test1Builder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> Test1Builder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &Test1View<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !Test1View::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &Test1View<'_>) -> &mut Self {
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
    pub struct TestRepeatedBoolView<'a> {
        buf: &'a [u8],
        values: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestRepeatedBoolView<'a> {
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
        pub const fn values(&self) -> ::roto::runtime::RepeatedVarintIter<'a, bool> {
            ::roto::runtime::RepeatedVarintIter::new(self.values.slice(self.buf), 1)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestRepeatedBoolView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestRepeatedBoolBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestRepeatedBoolBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestRepeatedBoolView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestRepeatedBoolView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestRepeatedBoolView<'_>) -> &mut Self {
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
        pub fn write_values(&mut self, val: bool) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_bool(val);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
            self
        }

        #[inline]
        pub fn write_value(&mut self, val: bool) -> &mut Self {
            self.write_values(val)
        }

        #[inline]
        pub fn write_packed_values(&mut self, vals: &[bool]) -> &mut Self {
            self.set_fields[0] |= 1;
            let end_pos = self.writer.cursor();
            for &val in vals.iter().rev() {
                let _ = self.writer.write_bool(val);
            }
            let packed_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(packed_len as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct TestRepeatedPackedInt32View<'a> {
        buf: &'a [u8],
        values: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestRepeatedPackedInt32View<'a> {
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
        pub const fn values(&self) -> ::roto::runtime::RepeatedVarintIter<'a, i32> {
            ::roto::runtime::RepeatedVarintIter::new(self.values.slice(self.buf), 1)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestRepeatedPackedInt32View<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestRepeatedPackedInt32Builder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestRepeatedPackedInt32Builder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestRepeatedPackedInt32View<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestRepeatedPackedInt32View::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestRepeatedPackedInt32View<'_>) -> &mut Self {
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
        pub fn write_values(&mut self, val: i32) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_varint(val as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
            self
        }

        #[inline]
        pub fn write_value(&mut self, val: i32) -> &mut Self {
            self.write_values(val)
        }

        #[inline]
        pub fn write_packed_values(&mut self, vals: &[i32]) -> &mut Self {
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
    }

    #[derive(Clone, Copy, Debug)]
    pub struct TestRepeatedMessagesView<'a> {
        buf: &'a [u8],
        messages1: ::roto::runtime::FieldMarker,
        messages2: ::roto::runtime::FieldMarker,
        messages3: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestRepeatedMessagesView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut messages1 = ::roto::runtime::FieldMarker::none();
            let mut messages2 = ::roto::runtime::FieldMarker::none();
            let mut messages3 = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, _) => messages1.extend(field.raw_span),
                    (2, _) => messages2.extend(field.raw_span),
                    (3, _) => messages3.extend(field.raw_span),
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                messages1,
                messages2,
                messages3,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn messages1(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedMessagesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.messages1.slice(self.buf), 1)
        }

        #[inline]
        pub const fn messages2(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedMessagesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.messages2.slice(self.buf), 2)
        }

        #[inline]
        pub const fn messages3(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedMessagesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.messages3.slice(self.buf), 3)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestRepeatedMessagesView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestRepeatedMessagesBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestRepeatedMessagesBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestRepeatedMessagesView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestRepeatedMessagesView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestRepeatedMessagesView<'_>) -> &mut Self {
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
        pub fn write_messages1<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 1;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_messages2<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 2;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_messages3<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 4;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedMessagesBuilder::from_writer(sub_writer);
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
    pub struct TestOptionalMessagesView<'a> {
        buf: &'a [u8],
        message1: ::roto::runtime::FieldMarker,
        message2: ::roto::runtime::FieldMarker,
        message3: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestOptionalMessagesView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut message1 = ::roto::runtime::FieldMarker::none();
            let mut message2 = ::roto::runtime::FieldMarker::none();
            let mut message3 = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => message1 = field.marker,
                    (2, ::roto::runtime::WireType::LengthDelimited) => message2 = field.marker,
                    (3, ::roto::runtime::WireType::LengthDelimited) => message3 = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                message1,
                message2,
                message3,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_message1(&self) -> bool {
            self.message1.is_present()
        }

        #[inline]
        pub fn message1(&self) -> Result<Option<crate::perftest_data::TestOptionalMessagesView<'a>>, ::roto::runtime::DecodeError> {
            if !self.message1.is_present() {
                return Ok(None);
            }
            crate::perftest_data::TestOptionalMessagesView::from_bytes(self.message1.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_message2(&self) -> bool {
            self.message2.is_present()
        }

        #[inline]
        pub fn message2(&self) -> Result<Option<crate::perftest_data::TestOptionalMessagesView<'a>>, ::roto::runtime::DecodeError> {
            if !self.message2.is_present() {
                return Ok(None);
            }
            crate::perftest_data::TestOptionalMessagesView::from_bytes(self.message2.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_message3(&self) -> bool {
            self.message3.is_present()
        }

        #[inline]
        pub fn message3(&self) -> Result<Option<crate::perftest_data::TestOptionalMessagesView<'a>>, ::roto::runtime::DecodeError> {
            if !self.message3.is_present() {
                return Ok(None);
            }
            crate::perftest_data::TestOptionalMessagesView::from_bytes(self.message3.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestOptionalMessagesView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestOptionalMessagesBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestOptionalMessagesBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestOptionalMessagesView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestOptionalMessagesView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestOptionalMessagesView<'_>) -> &mut Self {
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
        pub fn write_message1<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestOptionalMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 1;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestOptionalMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_message2<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestOptionalMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 2;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestOptionalMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_message3<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestOptionalMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 4;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestOptionalMessagesBuilder::from_writer(sub_writer);
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
    pub struct TestStringsView<'a> {
        buf: &'a [u8],
        s1: ::roto::runtime::FieldMarker,
        s2: ::roto::runtime::FieldMarker,
        s3: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestStringsView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut s1 = ::roto::runtime::FieldMarker::none();
            let mut s2 = ::roto::runtime::FieldMarker::none();
            let mut s3 = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => s1 = field.marker,
                    (2, ::roto::runtime::WireType::LengthDelimited) => s2 = field.marker,
                    (3, ::roto::runtime::WireType::LengthDelimited) => s3 = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                s1,
                s2,
                s3,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_s1(&self) -> bool {
            self.s1.is_present()
        }

        #[inline]
        pub fn s1(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.s1.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.s1)
        }

        #[inline]
        pub const fn has_s2(&self) -> bool {
            self.s2.is_present()
        }

        #[inline]
        pub fn s2(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.s2.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.s2)
        }

        #[inline]
        pub const fn has_s3(&self) -> bool {
            self.s3.is_present()
        }

        #[inline]
        pub fn s3(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
            if !self.s3.is_present() {
                return Ok("");
            }
            ::roto::runtime::decode_str_at(self.buf, self.s3)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestStringsView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestStringsBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestStringsBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestStringsView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestStringsView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestStringsView<'_>) -> &mut Self {
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
        pub fn write_s1(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_s2(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_s3(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 4;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct TestBytesView<'a> {
        buf: &'a [u8],
        b1: ::roto::runtime::FieldMarker,
    }

    impl<'a> TestBytesView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut b1 = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                #[allow(clippy::single_match)]
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => b1 = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                b1,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_b1(&self) -> bool {
            self.b1.is_present()
        }

        #[inline]
        pub fn b1(&self) -> Result<&'a [u8], ::roto::runtime::DecodeError> {
            if !self.b1.is_present() {
                return Ok(&b""[..]);
            }
            ::roto::runtime::decode_bytes_at(self.buf, self.b1)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for TestBytesView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct TestBytesBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> TestBytesBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &TestBytesView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !TestBytesView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &TestBytesView<'_>) -> &mut Self {
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
        pub fn write_b1(&mut self, val: &[u8]) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_length_delimited(val);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct PerftestDataView<'a> {
        buf: &'a [u8],
        test1: ::roto::runtime::FieldMarker,
        test_repeated_bool: ::roto::runtime::FieldMarker,
        test_repeated_messages: ::roto::runtime::FieldMarker,
        test_optional_messages: ::roto::runtime::FieldMarker,
        test_strings: ::roto::runtime::FieldMarker,
        test_repeated_packed_int32: ::roto::runtime::FieldMarker,
        test_small_bytearrays: ::roto::runtime::FieldMarker,
        test_large_bytearrays: ::roto::runtime::FieldMarker,
    }

    impl<'a> PerftestDataView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut test1 = ::roto::runtime::FieldMarker::none();
            let mut test_repeated_bool = ::roto::runtime::FieldMarker::none();
            let mut test_repeated_messages = ::roto::runtime::FieldMarker::none();
            let mut test_optional_messages = ::roto::runtime::FieldMarker::none();
            let mut test_strings = ::roto::runtime::FieldMarker::none();
            let mut test_repeated_packed_int32 = ::roto::runtime::FieldMarker::none();
            let mut test_small_bytearrays = ::roto::runtime::FieldMarker::none();
            let mut test_large_bytearrays = ::roto::runtime::FieldMarker::none();

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, _) => test1.extend(field.raw_span),
                    (2, _) => test_repeated_bool.extend(field.raw_span),
                    (3, _) => test_repeated_messages.extend(field.raw_span),
                    (4, _) => test_optional_messages.extend(field.raw_span),
                    (5, _) => test_strings.extend(field.raw_span),
                    (6, _) => test_repeated_packed_int32.extend(field.raw_span),
                    (7, _) => test_small_bytearrays.extend(field.raw_span),
                    (8, _) => test_large_bytearrays.extend(field.raw_span),
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                test1,
                test_repeated_bool,
                test_repeated_messages,
                test_optional_messages,
                test_strings,
                test_repeated_packed_int32,
                test_small_bytearrays,
                test_large_bytearrays,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn test1(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::Test1View<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test1.slice(self.buf), 1)
        }

        #[inline]
        pub const fn test_repeated_bool(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedBoolView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_repeated_bool.slice(self.buf), 2)
        }

        #[inline]
        pub const fn test_repeated_messages(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedMessagesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_repeated_messages.slice(self.buf), 3)
        }

        #[inline]
        pub const fn test_optional_messages(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestOptionalMessagesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_optional_messages.slice(self.buf), 4)
        }

        #[inline]
        pub const fn test_strings(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestStringsView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_strings.slice(self.buf), 5)
        }

        #[inline]
        pub const fn test_repeated_packed_int32(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestRepeatedPackedInt32View<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_repeated_packed_int32.slice(self.buf), 6)
        }

        #[inline]
        pub const fn test_small_bytearrays(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestBytesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_small_bytearrays.slice(self.buf), 7)
        }

        #[inline]
        pub const fn test_large_bytearrays(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::perftest_data::TestBytesView<'a>> {
            ::roto::runtime::RepeatedMessageIter::new(self.test_large_bytearrays.slice(self.buf), 8)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for PerftestDataView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct PerftestDataBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> PerftestDataBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &PerftestDataView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !PerftestDataView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &PerftestDataView<'_>) -> &mut Self {
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
        pub fn write_test1<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::Test1Builder<'_>),
        {
            self.set_fields[0] |= 1;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::Test1Builder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_repeated_bool<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedBoolBuilder<'_>),
        {
            self.set_fields[0] |= 2;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedBoolBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_repeated_messages<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 4;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_repeated_message<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedMessagesBuilder<'_>),
        {
            self.write_test_repeated_messages(f)
        }

        #[inline]
        pub fn write_test_optional_messages<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestOptionalMessagesBuilder<'_>),
        {
            self.set_fields[0] |= 8;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestOptionalMessagesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_optional_message<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestOptionalMessagesBuilder<'_>),
        {
            self.write_test_optional_messages(f)
        }

        #[inline]
        pub fn write_test_strings<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestStringsBuilder<'_>),
        {
            self.set_fields[0] |= 16;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestStringsBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_string<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestStringsBuilder<'_>),
        {
            self.write_test_strings(f)
        }

        #[inline]
        pub fn write_test_repeated_packed_int32<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestRepeatedPackedInt32Builder<'_>),
        {
            self.set_fields[0] |= 32;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestRepeatedPackedInt32Builder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_small_bytearrays<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestBytesBuilder<'_>),
        {
            self.set_fields[0] |= 64;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestBytesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_small_bytearray<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestBytesBuilder<'_>),
        {
            self.write_test_small_bytearrays(f)
        }

        #[inline]
        pub fn write_test_large_bytearrays<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestBytesBuilder<'_>),
        {
            self.set_fields[0] |= 128;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::perftest_data::TestBytesBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_test_large_bytearray<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::perftest_data::TestBytesBuilder<'_>),
        {
            self.write_test_large_bytearrays(f)
        }
    }
}
