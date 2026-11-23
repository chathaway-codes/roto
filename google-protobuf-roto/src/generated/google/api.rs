#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ClientLibraryOrganization {
    ClientLibraryOrganizationUnspecified = 0,
    Cloud = 1,
    Ads = 2,
    Photos = 3,
    StreetView = 4,
    Shopping = 5,
    Geo = 6,
    GenerativeAi = 7,
}

impl ClientLibraryOrganization {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::ClientLibraryOrganizationUnspecified),
            1 => Some(Self::Cloud),
            2 => Some(Self::Ads),
            3 => Some(Self::Photos),
            4 => Some(Self::StreetView),
            5 => Some(Self::Shopping),
            6 => Some(Self::Geo),
            7 => Some(Self::GenerativeAi),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for ClientLibraryOrganization {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for ClientLibraryOrganization {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::ClientLibraryOrganizationUnspecified)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ClientLibraryDestination {
    ClientLibraryDestinationUnspecified = 0,
    Github = 10,
    PackageManager = 20,
}

impl ClientLibraryDestination {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::ClientLibraryDestinationUnspecified),
            10 => Some(Self::Github),
            20 => Some(Self::PackageManager),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for ClientLibraryDestination {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for ClientLibraryDestination {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::ClientLibraryDestinationUnspecified)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FlowControlLimitExceededBehaviorProto {
    UnsetBehavior = 0,
    ThrowException = 1,
    Block = 2,
    Ignore = 3,
}

impl FlowControlLimitExceededBehaviorProto {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::UnsetBehavior),
            1 => Some(Self::ThrowException),
            2 => Some(Self::Block),
            3 => Some(Self::Ignore),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for FlowControlLimitExceededBehaviorProto {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for FlowControlLimitExceededBehaviorProto {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::UnsetBehavior)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommonLanguageSettingsView<'a> {
    buf: &'a [u8],
    reference_docs_uri: ::roto::runtime::FieldMarker,
    destinations: ::roto::runtime::FieldMarker,
    selective_gapic_generation: ::roto::runtime::FieldMarker,
}

impl<'a> CommonLanguageSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut reference_docs_uri = ::roto::runtime::FieldMarker::none();
        let mut destinations = ::roto::runtime::FieldMarker::none();
        let mut selective_gapic_generation = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => reference_docs_uri = field.marker,
                (2, _) => destinations.extend(field.raw_span),
                (3, ::roto::runtime::WireType::LengthDelimited) => selective_gapic_generation = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            reference_docs_uri,
            destinations,
            selective_gapic_generation,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_reference_docs_uri(&self) -> bool {
        self.reference_docs_uri.is_present()
    }

    #[inline]
    pub fn reference_docs_uri(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.reference_docs_uri.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.reference_docs_uri)
    }

    #[inline]
    pub const fn destinations(&self) -> ::roto::runtime::RepeatedVarintIter<'a, crate::google::api::ClientLibraryDestination> {
        ::roto::runtime::RepeatedVarintIter::new(self.destinations.slice(self.buf), 2)
    }

    #[inline]
    pub const fn has_selective_gapic_generation(&self) -> bool {
        self.selective_gapic_generation.is_present()
    }

    #[inline]
    pub fn selective_gapic_generation(&self) -> Result<Option<crate::google::api::SelectiveGapicGenerationView<'a>>, ::roto::runtime::DecodeError> {
        if !self.selective_gapic_generation.is_present() {
            return Ok(None);
        }
        crate::google::api::SelectiveGapicGenerationView::from_bytes(self.selective_gapic_generation.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CommonLanguageSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CommonLanguageSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CommonLanguageSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &CommonLanguageSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CommonLanguageSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CommonLanguageSettingsView<'_>) -> &mut Self {
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
    pub fn write_reference_docs_uri(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_destinations(&mut self, val: crate::google::api::ClientLibraryDestination) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_destination(&mut self, val: crate::google::api::ClientLibraryDestination) -> &mut Self {
        self.write_destinations(val)
    }

    #[inline]
    pub fn write_packed_destinations(&mut self, vals: &[crate::google::api::ClientLibraryDestination]) -> &mut Self {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val.as_i32() as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_selective_gapic_generation<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::SelectiveGapicGenerationBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::SelectiveGapicGenerationBuilder::from_writer(sub_writer);
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
pub struct ClientLibrarySettingsView<'a> {
    buf: &'a [u8],
    version: ::roto::runtime::FieldMarker,
    launch_stage: crate::google::api::LaunchStage,
    rest_numeric_enums: bool,
    java_settings: ::roto::runtime::FieldMarker,
    cpp_settings: ::roto::runtime::FieldMarker,
    php_settings: ::roto::runtime::FieldMarker,
    python_settings: ::roto::runtime::FieldMarker,
    node_settings: ::roto::runtime::FieldMarker,
    dotnet_settings: ::roto::runtime::FieldMarker,
    ruby_settings: ::roto::runtime::FieldMarker,
    go_settings: ::roto::runtime::FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> ClientLibrarySettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 21, 22, 23, 24, 25, 26, 27, 28];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut version = ::roto::runtime::FieldMarker::none();
        let mut launch_stage: crate::google::api::LaunchStage = crate::google::api::LaunchStage::LaunchStageUnspecified;
        let mut rest_numeric_enums: bool = false;
        let mut java_settings = ::roto::runtime::FieldMarker::none();
        let mut cpp_settings = ::roto::runtime::FieldMarker::none();
        let mut php_settings = ::roto::runtime::FieldMarker::none();
        let mut python_settings = ::roto::runtime::FieldMarker::none();
        let mut node_settings = ::roto::runtime::FieldMarker::none();
        let mut dotnet_settings = ::roto::runtime::FieldMarker::none();
        let mut ruby_settings = ::roto::runtime::FieldMarker::none();
        let mut go_settings = ::roto::runtime::FieldMarker::none();
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => version = field.marker,
                (2, ::roto::runtime::WireType::Varint) => {
                    launch_stage = <crate::google::api::LaunchStage as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 1;
                }
                (3, ::roto::runtime::WireType::Varint) => {
                    rest_numeric_enums = field.val != 0;
                    has_fields[0] |= 2;
                }
                (21, ::roto::runtime::WireType::LengthDelimited) => java_settings = field.marker,
                (22, ::roto::runtime::WireType::LengthDelimited) => cpp_settings = field.marker,
                (23, ::roto::runtime::WireType::LengthDelimited) => php_settings = field.marker,
                (24, ::roto::runtime::WireType::LengthDelimited) => python_settings = field.marker,
                (25, ::roto::runtime::WireType::LengthDelimited) => node_settings = field.marker,
                (26, ::roto::runtime::WireType::LengthDelimited) => dotnet_settings = field.marker,
                (27, ::roto::runtime::WireType::LengthDelimited) => ruby_settings = field.marker,
                (28, ::roto::runtime::WireType::LengthDelimited) => go_settings = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            version,
            launch_stage,
            rest_numeric_enums,
            java_settings,
            cpp_settings,
            php_settings,
            python_settings,
            node_settings,
            dotnet_settings,
            ruby_settings,
            go_settings,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
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
    pub const fn has_launch_stage(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn launch_stage(&self) -> Result<crate::google::api::LaunchStage, ::roto::runtime::DecodeError> {
        Ok(self.launch_stage)
    }

    #[inline]
    pub const fn has_rest_numeric_enums(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn rest_numeric_enums(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.rest_numeric_enums)
    }

    #[inline]
    pub const fn has_java_settings(&self) -> bool {
        self.java_settings.is_present()
    }

    #[inline]
    pub fn java_settings(&self) -> Result<Option<crate::google::api::JavaSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.java_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::JavaSettingsView::from_bytes(self.java_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_cpp_settings(&self) -> bool {
        self.cpp_settings.is_present()
    }

    #[inline]
    pub fn cpp_settings(&self) -> Result<Option<crate::google::api::CppSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.cpp_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::CppSettingsView::from_bytes(self.cpp_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_php_settings(&self) -> bool {
        self.php_settings.is_present()
    }

    #[inline]
    pub fn php_settings(&self) -> Result<Option<crate::google::api::PhpSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.php_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::PhpSettingsView::from_bytes(self.php_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_python_settings(&self) -> bool {
        self.python_settings.is_present()
    }

    #[inline]
    pub fn python_settings(&self) -> Result<Option<crate::google::api::PythonSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.python_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::PythonSettingsView::from_bytes(self.python_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_node_settings(&self) -> bool {
        self.node_settings.is_present()
    }

    #[inline]
    pub fn node_settings(&self) -> Result<Option<crate::google::api::NodeSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.node_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::NodeSettingsView::from_bytes(self.node_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_dotnet_settings(&self) -> bool {
        self.dotnet_settings.is_present()
    }

    #[inline]
    pub fn dotnet_settings(&self) -> Result<Option<crate::google::api::DotnetSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.dotnet_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::DotnetSettingsView::from_bytes(self.dotnet_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_ruby_settings(&self) -> bool {
        self.ruby_settings.is_present()
    }

    #[inline]
    pub fn ruby_settings(&self) -> Result<Option<crate::google::api::RubySettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.ruby_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::RubySettingsView::from_bytes(self.ruby_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_go_settings(&self) -> bool {
        self.go_settings.is_present()
    }

    #[inline]
    pub fn go_settings(&self) -> Result<Option<crate::google::api::GoSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.go_settings.is_present() {
            return Ok(None);
        }
        crate::google::api::GoSettingsView::from_bytes(self.go_settings.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for ClientLibrarySettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ClientLibrarySettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> ClientLibrarySettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &ClientLibrarySettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ClientLibrarySettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &ClientLibrarySettingsView<'_>) -> &mut Self {
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
            21 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            22 => {
                self.set_fields[0] |= 16;
                (prev_set[0] & 16) == 0
            }
            23 => {
                self.set_fields[0] |= 32;
                (prev_set[0] & 32) == 0
            }
            24 => {
                self.set_fields[0] |= 64;
                (prev_set[0] & 64) == 0
            }
            25 => {
                self.set_fields[0] |= 128;
                (prev_set[0] & 128) == 0
            }
            26 => {
                self.set_fields[0] |= 256;
                (prev_set[0] & 256) == 0
            }
            27 => {
                self.set_fields[0] |= 512;
                (prev_set[0] & 512) == 0
            }
            28 => {
                self.set_fields[0] |= 1024;
                (prev_set[0] & 1024) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_version(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_launch_stage(&mut self, val: crate::google::api::LaunchStage) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_rest_numeric_enums(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_java_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::JavaSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 8;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::JavaSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(21, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_cpp_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CppSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 16;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CppSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(22, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_php_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::PhpSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 32;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::PhpSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(23, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_python_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::PythonSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 64;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::PythonSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(24, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_node_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::NodeSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 128;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::NodeSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(25, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_dotnet_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::DotnetSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 256;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::DotnetSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(26, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_ruby_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::RubySettingsBuilder<'_>),
    {
        self.set_fields[0] |= 512;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::RubySettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(27, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_go_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::GoSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1024;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::GoSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(28, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PublishingView<'a> {
    buf: &'a [u8],
    method_settings: ::roto::runtime::FieldMarker,
    new_issue_uri: ::roto::runtime::FieldMarker,
    documentation_uri: ::roto::runtime::FieldMarker,
    api_short_name: ::roto::runtime::FieldMarker,
    github_label: ::roto::runtime::FieldMarker,
    codeowner_github_teams: ::roto::runtime::FieldMarker,
    doc_tag_prefix: ::roto::runtime::FieldMarker,
    organization: crate::google::api::ClientLibraryOrganization,
    library_settings: ::roto::runtime::FieldMarker,
    proto_reference_documentation_uri: ::roto::runtime::FieldMarker,
    rest_reference_documentation_uri: ::roto::runtime::FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> PublishingView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[2, 101, 102, 103, 104, 105, 106, 107, 109, 110, 111];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut method_settings = ::roto::runtime::FieldMarker::none();
        let mut new_issue_uri = ::roto::runtime::FieldMarker::none();
        let mut documentation_uri = ::roto::runtime::FieldMarker::none();
        let mut api_short_name = ::roto::runtime::FieldMarker::none();
        let mut github_label = ::roto::runtime::FieldMarker::none();
        let mut codeowner_github_teams = ::roto::runtime::FieldMarker::none();
        let mut doc_tag_prefix = ::roto::runtime::FieldMarker::none();
        let mut organization: crate::google::api::ClientLibraryOrganization = crate::google::api::ClientLibraryOrganization::ClientLibraryOrganizationUnspecified;
        let mut library_settings = ::roto::runtime::FieldMarker::none();
        let mut proto_reference_documentation_uri = ::roto::runtime::FieldMarker::none();
        let mut rest_reference_documentation_uri = ::roto::runtime::FieldMarker::none();
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (2, _) => method_settings.extend(field.raw_span),
                (101, ::roto::runtime::WireType::LengthDelimited) => new_issue_uri = field.marker,
                (102, ::roto::runtime::WireType::LengthDelimited) => documentation_uri = field.marker,
                (103, ::roto::runtime::WireType::LengthDelimited) => api_short_name = field.marker,
                (104, ::roto::runtime::WireType::LengthDelimited) => github_label = field.marker,
                (105, _) => codeowner_github_teams.extend(field.raw_span),
                (106, ::roto::runtime::WireType::LengthDelimited) => doc_tag_prefix = field.marker,
                (107, ::roto::runtime::WireType::Varint) => {
                    organization = <crate::google::api::ClientLibraryOrganization as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 1;
                }
                (109, _) => library_settings.extend(field.raw_span),
                (110, ::roto::runtime::WireType::LengthDelimited) => proto_reference_documentation_uri = field.marker,
                (111, ::roto::runtime::WireType::LengthDelimited) => rest_reference_documentation_uri = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            method_settings,
            new_issue_uri,
            documentation_uri,
            api_short_name,
            github_label,
            codeowner_github_teams,
            doc_tag_prefix,
            organization,
            library_settings,
            proto_reference_documentation_uri,
            rest_reference_documentation_uri,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn method_settings(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::MethodSettingsView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.method_settings.slice(self.buf), 2)
    }

    #[inline]
    pub const fn has_new_issue_uri(&self) -> bool {
        self.new_issue_uri.is_present()
    }

    #[inline]
    pub fn new_issue_uri(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.new_issue_uri.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.new_issue_uri)
    }

    #[inline]
    pub const fn has_documentation_uri(&self) -> bool {
        self.documentation_uri.is_present()
    }

    #[inline]
    pub fn documentation_uri(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.documentation_uri.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.documentation_uri)
    }

    #[inline]
    pub const fn has_api_short_name(&self) -> bool {
        self.api_short_name.is_present()
    }

    #[inline]
    pub fn api_short_name(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.api_short_name.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.api_short_name)
    }

    #[inline]
    pub const fn has_github_label(&self) -> bool {
        self.github_label.is_present()
    }

    #[inline]
    pub fn github_label(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.github_label.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.github_label)
    }

    #[inline]
    pub const fn codeowner_github_teams(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.codeowner_github_teams.slice(self.buf), 105)
    }

    #[inline]
    pub const fn has_doc_tag_prefix(&self) -> bool {
        self.doc_tag_prefix.is_present()
    }

    #[inline]
    pub fn doc_tag_prefix(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.doc_tag_prefix.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.doc_tag_prefix)
    }

    #[inline]
    pub const fn has_organization(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn organization(&self) -> Result<crate::google::api::ClientLibraryOrganization, ::roto::runtime::DecodeError> {
        Ok(self.organization)
    }

    #[inline]
    pub const fn library_settings(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::ClientLibrarySettingsView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.library_settings.slice(self.buf), 109)
    }

    #[inline]
    pub const fn has_proto_reference_documentation_uri(&self) -> bool {
        self.proto_reference_documentation_uri.is_present()
    }

    #[inline]
    pub fn proto_reference_documentation_uri(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.proto_reference_documentation_uri.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.proto_reference_documentation_uri)
    }

    #[inline]
    pub const fn has_rest_reference_documentation_uri(&self) -> bool {
        self.rest_reference_documentation_uri.is_present()
    }

    #[inline]
    pub fn rest_reference_documentation_uri(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.rest_reference_documentation_uri.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.rest_reference_documentation_uri)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for PublishingView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct PublishingBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> PublishingBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &PublishingView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !PublishingView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &PublishingView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            2 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            101 => {
                self.set_fields[0] |= 2;
                (prev_set[0] & 2) == 0
            }
            102 => {
                self.set_fields[0] |= 4;
                (prev_set[0] & 4) == 0
            }
            103 => {
                self.set_fields[0] |= 8;
                (prev_set[0] & 8) == 0
            }
            104 => {
                self.set_fields[0] |= 16;
                (prev_set[0] & 16) == 0
            }
            105 => {
                self.set_fields[0] |= 32;
                (prev_set[0] & 32) == 0
            }
            106 => {
                self.set_fields[0] |= 64;
                (prev_set[0] & 64) == 0
            }
            107 => {
                self.set_fields[0] |= 128;
                (prev_set[0] & 128) == 0
            }
            109 => {
                self.set_fields[0] |= 256;
                (prev_set[0] & 256) == 0
            }
            110 => {
                self.set_fields[0] |= 512;
                (prev_set[0] & 512) == 0
            }
            111 => {
                self.set_fields[0] |= 1024;
                (prev_set[0] & 1024) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_method_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::MethodSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::MethodSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_method_setting<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::MethodSettingsBuilder<'_>),
    {
        self.write_method_settings(f)
    }

    #[inline]
    pub fn write_new_issue_uri(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(101, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_documentation_uri(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(102, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_api_short_name(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(103, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_github_label(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(104, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_codeowner_github_teams(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 32;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(105, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_codeowner_github_team(&mut self, val: &str) -> &mut Self {
        self.write_codeowner_github_teams(val)
    }

    #[inline]
    pub fn write_doc_tag_prefix(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 64;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(106, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_organization(&mut self, val: crate::google::api::ClientLibraryOrganization) -> &mut Self {
        self.set_fields[0] |= 128;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(107, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_library_settings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::ClientLibrarySettingsBuilder<'_>),
    {
        self.set_fields[0] |= 256;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::ClientLibrarySettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(109, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_library_setting<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::ClientLibrarySettingsBuilder<'_>),
    {
        self.write_library_settings(f)
    }

    #[inline]
    pub fn write_proto_reference_documentation_uri(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 512;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(110, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_rest_reference_documentation_uri(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1024;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(111, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct JavaSettingsView<'a> {
    buf: &'a [u8],
    library_package: ::roto::runtime::FieldMarker,
    service_class_names: ::roto::runtime::FieldMarker,
    common: ::roto::runtime::FieldMarker,
}

impl<'a> JavaSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut library_package = ::roto::runtime::FieldMarker::none();
        let mut service_class_names = ::roto::runtime::FieldMarker::none();
        let mut common = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => library_package = field.marker,
                (2, _) => service_class_names.extend(field.raw_span),
                (3, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            library_package,
            service_class_names,
            common,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_library_package(&self) -> bool {
        self.library_package.is_present()
    }

    #[inline]
    pub fn library_package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.library_package.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.library_package)
    }

    #[inline]
    pub const fn service_class_names(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::java_settings::ServiceClassNamesEntryView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.service_class_names.slice(self.buf), 2)
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for JavaSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct JavaSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> JavaSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &JavaSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !JavaSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &JavaSettingsView<'_>) -> &mut Self {
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
    pub fn write_library_package(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_service_class_names<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::java_settings::ServiceClassNamesEntryBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::java_settings::ServiceClassNamesEntryBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_service_class_name<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::java_settings::ServiceClassNamesEntryBuilder<'_>),
    {
        self.write_service_class_names(f)
    }

    #[inline]
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

pub mod java_settings {
    #[derive(Clone, Copy, Debug)]
    pub struct ServiceClassNamesEntryView<'a> {
        buf: &'a [u8],
        key: ::roto::runtime::FieldMarker,
        value: ::roto::runtime::FieldMarker,
    }

    impl<'a> ServiceClassNamesEntryView<'a> {
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

    impl<'a> ::roto::runtime::MessageView<'a> for ServiceClassNamesEntryView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct ServiceClassNamesEntryBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> ServiceClassNamesEntryBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &ServiceClassNamesEntryView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ServiceClassNamesEntryView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &ServiceClassNamesEntryView<'_>) -> &mut Self {
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
        pub fn write_value(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CppSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
}

impl<'a> CppSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            #[allow(clippy::single_match)]
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CppSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CppSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CppSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &CppSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CppSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CppSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
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
pub struct PhpSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
    library_package: ::roto::runtime::FieldMarker,
}

impl<'a> PhpSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();
        let mut library_package = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => library_package = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
            library_package,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_library_package(&self) -> bool {
        self.library_package.is_present()
    }

    #[inline]
    pub fn library_package(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.library_package.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.library_package)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for PhpSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct PhpSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> PhpSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &PhpSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !PhpSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &PhpSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_library_package(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PythonSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
    experimental_features: ::roto::runtime::FieldMarker,
}

impl<'a> PythonSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();
        let mut experimental_features = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => experimental_features = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
            experimental_features,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_experimental_features(&self) -> bool {
        self.experimental_features.is_present()
    }

    #[inline]
    pub fn experimental_features(&self) -> Result<Option<crate::google::api::python_settings::ExperimentalFeaturesView<'a>>, ::roto::runtime::DecodeError> {
        if !self.experimental_features.is_present() {
            return Ok(None);
        }
        crate::google::api::python_settings::ExperimentalFeaturesView::from_bytes(self.experimental_features.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for PythonSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct PythonSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> PythonSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &PythonSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !PythonSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &PythonSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_experimental_features<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::python_settings::ExperimentalFeaturesBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::python_settings::ExperimentalFeaturesBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

pub mod python_settings {
    #[derive(Clone, Copy, Debug)]
    pub struct ExperimentalFeaturesView<'a> {
        buf: &'a [u8],
        rest_async_io_enabled: bool,
        protobuf_pythonic_types_enabled: bool,
        unversioned_package_disabled: bool,
        has_fields: [u32; 1],
    }

    impl<'a> ExperimentalFeaturesView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut rest_async_io_enabled: bool = false;
            let mut protobuf_pythonic_types_enabled: bool = false;
            let mut unversioned_package_disabled: bool = false;
            let mut has_fields = [0u32; 1];

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::Varint) => {
                        rest_async_io_enabled = field.val != 0;
                        has_fields[0] |= 1;
                    }
                    (2, ::roto::runtime::WireType::Varint) => {
                        protobuf_pythonic_types_enabled = field.val != 0;
                        has_fields[0] |= 2;
                    }
                    (3, ::roto::runtime::WireType::Varint) => {
                        unversioned_package_disabled = field.val != 0;
                        has_fields[0] |= 4;
                    }
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                rest_async_io_enabled,
                protobuf_pythonic_types_enabled,
                unversioned_package_disabled,
                has_fields,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_rest_async_io_enabled(&self) -> bool {
            (self.has_fields[0] & 1) != 0
        }

        #[inline]
        pub const fn rest_async_io_enabled(&self) -> Result<bool, ::roto::runtime::DecodeError> {
            Ok(self.rest_async_io_enabled)
        }

        #[inline]
        pub const fn has_protobuf_pythonic_types_enabled(&self) -> bool {
            (self.has_fields[0] & 2) != 0
        }

        #[inline]
        pub const fn protobuf_pythonic_types_enabled(&self) -> Result<bool, ::roto::runtime::DecodeError> {
            Ok(self.protobuf_pythonic_types_enabled)
        }

        #[inline]
        pub const fn has_unversioned_package_disabled(&self) -> bool {
            (self.has_fields[0] & 4) != 0
        }

        #[inline]
        pub const fn unversioned_package_disabled(&self) -> Result<bool, ::roto::runtime::DecodeError> {
            Ok(self.unversioned_package_disabled)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for ExperimentalFeaturesView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct ExperimentalFeaturesBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> ExperimentalFeaturesBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &ExperimentalFeaturesView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ExperimentalFeaturesView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &ExperimentalFeaturesView<'_>) -> &mut Self {
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
        pub fn write_rest_async_io_enabled(&mut self, val: bool) -> &mut Self {
            self.set_fields[0] |= 1;
            let _ = self.writer.write_bool(val);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
            self
        }

        #[inline]
        pub fn write_protobuf_pythonic_types_enabled(&mut self, val: bool) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_bool(val);
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
            self
        }

        #[inline]
        pub fn write_unversioned_package_disabled(&mut self, val: bool) -> &mut Self {
            self.set_fields[0] |= 4;
            let _ = self.writer.write_bool(val);
            let _ = self.writer.write_tag(3, ::roto::runtime::WireType::Varint);
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct NodeSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
}

impl<'a> NodeSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            #[allow(clippy::single_match)]
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for NodeSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct NodeSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> NodeSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &NodeSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !NodeSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &NodeSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
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
pub struct DotnetSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
    renamed_services: ::roto::runtime::FieldMarker,
    renamed_resources: ::roto::runtime::FieldMarker,
    ignored_resources: ::roto::runtime::FieldMarker,
    forced_namespace_aliases: ::roto::runtime::FieldMarker,
    handwritten_signatures: ::roto::runtime::FieldMarker,
}

impl<'a> DotnetSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();
        let mut renamed_services = ::roto::runtime::FieldMarker::none();
        let mut renamed_resources = ::roto::runtime::FieldMarker::none();
        let mut ignored_resources = ::roto::runtime::FieldMarker::none();
        let mut forced_namespace_aliases = ::roto::runtime::FieldMarker::none();
        let mut handwritten_signatures = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                (2, _) => renamed_services.extend(field.raw_span),
                (3, _) => renamed_resources.extend(field.raw_span),
                (4, _) => ignored_resources.extend(field.raw_span),
                (5, _) => forced_namespace_aliases.extend(field.raw_span),
                (6, _) => handwritten_signatures.extend(field.raw_span),
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
            renamed_services,
            renamed_resources,
            ignored_resources,
            forced_namespace_aliases,
            handwritten_signatures,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn renamed_services(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::dotnet_settings::RenamedServicesEntryView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.renamed_services.slice(self.buf), 2)
    }

    #[inline]
    pub const fn renamed_resources(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::dotnet_settings::RenamedResourcesEntryView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.renamed_resources.slice(self.buf), 3)
    }

    #[inline]
    pub const fn ignored_resources(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.ignored_resources.slice(self.buf), 4)
    }

    #[inline]
    pub const fn forced_namespace_aliases(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.forced_namespace_aliases.slice(self.buf), 5)
    }

    #[inline]
    pub const fn handwritten_signatures(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.handwritten_signatures.slice(self.buf), 6)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for DotnetSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct DotnetSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> DotnetSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &DotnetSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !DotnetSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &DotnetSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_renamed_services<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::dotnet_settings::RenamedServicesEntryBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::dotnet_settings::RenamedServicesEntryBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_renamed_service<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::dotnet_settings::RenamedServicesEntryBuilder<'_>),
    {
        self.write_renamed_services(f)
    }

    #[inline]
    pub fn write_renamed_resources<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::dotnet_settings::RenamedResourcesEntryBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::dotnet_settings::RenamedResourcesEntryBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_renamed_resource<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::dotnet_settings::RenamedResourcesEntryBuilder<'_>),
    {
        self.write_renamed_resources(f)
    }

    #[inline]
    pub fn write_ignored_resources(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_ignored_resource(&mut self, val: &str) -> &mut Self {
        self.write_ignored_resources(val)
    }

    #[inline]
    pub fn write_forced_namespace_aliases(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_forced_namespace_aliase(&mut self, val: &str) -> &mut Self {
        self.write_forced_namespace_aliases(val)
    }

    #[inline]
    pub fn write_handwritten_signatures(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 32;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_handwritten_signature(&mut self, val: &str) -> &mut Self {
        self.write_handwritten_signatures(val)
    }
}

pub mod dotnet_settings {
    #[derive(Clone, Copy, Debug)]
    pub struct RenamedServicesEntryView<'a> {
        buf: &'a [u8],
        key: ::roto::runtime::FieldMarker,
        value: ::roto::runtime::FieldMarker,
    }

    impl<'a> RenamedServicesEntryView<'a> {
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

    impl<'a> ::roto::runtime::MessageView<'a> for RenamedServicesEntryView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct RenamedServicesEntryBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> RenamedServicesEntryBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &RenamedServicesEntryView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !RenamedServicesEntryView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &RenamedServicesEntryView<'_>) -> &mut Self {
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
        pub fn write_value(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct RenamedResourcesEntryView<'a> {
        buf: &'a [u8],
        key: ::roto::runtime::FieldMarker,
        value: ::roto::runtime::FieldMarker,
    }

    impl<'a> RenamedResourcesEntryView<'a> {
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

    impl<'a> ::roto::runtime::MessageView<'a> for RenamedResourcesEntryView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct RenamedResourcesEntryBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> RenamedResourcesEntryBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &RenamedResourcesEntryView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !RenamedResourcesEntryView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &RenamedResourcesEntryView<'_>) -> &mut Self {
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
        pub fn write_value(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RubySettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
}

impl<'a> RubySettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            #[allow(clippy::single_match)]
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for RubySettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct RubySettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> RubySettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &RubySettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !RubySettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &RubySettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
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
pub struct GoSettingsView<'a> {
    buf: &'a [u8],
    common: ::roto::runtime::FieldMarker,
    renamed_services: ::roto::runtime::FieldMarker,
}

impl<'a> GoSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut common = ::roto::runtime::FieldMarker::none();
        let mut renamed_services = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => common = field.marker,
                (2, _) => renamed_services.extend(field.raw_span),
                _ => {}
            }
        }

        Ok(Self {
            buf,
            common,
            renamed_services,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_common(&self) -> bool {
        self.common.is_present()
    }

    #[inline]
    pub fn common(&self) -> Result<Option<crate::google::api::CommonLanguageSettingsView<'a>>, ::roto::runtime::DecodeError> {
        if !self.common.is_present() {
            return Ok(None);
        }
        crate::google::api::CommonLanguageSettingsView::from_bytes(self.common.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn renamed_services(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::go_settings::RenamedServicesEntryView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.renamed_services.slice(self.buf), 2)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for GoSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct GoSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> GoSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &GoSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !GoSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &GoSettingsView<'_>) -> &mut Self {
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
    pub fn write_common<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CommonLanguageSettingsBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CommonLanguageSettingsBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_renamed_services<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::go_settings::RenamedServicesEntryBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::go_settings::RenamedServicesEntryBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_renamed_service<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::go_settings::RenamedServicesEntryBuilder<'_>),
    {
        self.write_renamed_services(f)
    }
}

pub mod go_settings {
    #[derive(Clone, Copy, Debug)]
    pub struct RenamedServicesEntryView<'a> {
        buf: &'a [u8],
        key: ::roto::runtime::FieldMarker,
        value: ::roto::runtime::FieldMarker,
    }

    impl<'a> RenamedServicesEntryView<'a> {
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

    impl<'a> ::roto::runtime::MessageView<'a> for RenamedServicesEntryView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct RenamedServicesEntryBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> RenamedServicesEntryBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &RenamedServicesEntryView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !RenamedServicesEntryView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &RenamedServicesEntryView<'_>) -> &mut Self {
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
        pub fn write_value(&mut self, val: &str) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_length_delimited(val.as_bytes());
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MethodSettingsView<'a> {
    buf: &'a [u8],
    selector: ::roto::runtime::FieldMarker,
    long_running: ::roto::runtime::FieldMarker,
    auto_populated_fields: ::roto::runtime::FieldMarker,
    batching: ::roto::runtime::FieldMarker,
}

impl<'a> MethodSettingsView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut selector = ::roto::runtime::FieldMarker::none();
        let mut long_running = ::roto::runtime::FieldMarker::none();
        let mut auto_populated_fields = ::roto::runtime::FieldMarker::none();
        let mut batching = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => selector = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => long_running = field.marker,
                (3, _) => auto_populated_fields.extend(field.raw_span),
                (4, ::roto::runtime::WireType::LengthDelimited) => batching = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            selector,
            long_running,
            auto_populated_fields,
            batching,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_selector(&self) -> bool {
        self.selector.is_present()
    }

    #[inline]
    pub fn selector(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.selector.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.selector)
    }

    #[inline]
    pub const fn has_long_running(&self) -> bool {
        self.long_running.is_present()
    }

    #[inline]
    pub fn long_running(&self) -> Result<Option<crate::google::api::method_settings::LongRunningView<'a>>, ::roto::runtime::DecodeError> {
        if !self.long_running.is_present() {
            return Ok(None);
        }
        crate::google::api::method_settings::LongRunningView::from_bytes(self.long_running.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn auto_populated_fields(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.auto_populated_fields.slice(self.buf), 3)
    }

    #[inline]
    pub const fn has_batching(&self) -> bool {
        self.batching.is_present()
    }

    #[inline]
    pub fn batching(&self) -> Result<Option<crate::google::api::BatchingConfigProtoView<'a>>, ::roto::runtime::DecodeError> {
        if !self.batching.is_present() {
            return Ok(None);
        }
        crate::google::api::BatchingConfigProtoView::from_bytes(self.batching.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for MethodSettingsView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct MethodSettingsBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> MethodSettingsBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &MethodSettingsView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !MethodSettingsView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &MethodSettingsView<'_>) -> &mut Self {
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
    pub fn write_selector(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_long_running<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::method_settings::LongRunningBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::method_settings::LongRunningBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_auto_populated_fields(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_auto_populated_field(&mut self, val: &str) -> &mut Self {
        self.write_auto_populated_fields(val)
    }

    #[inline]
    pub fn write_batching<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::BatchingConfigProtoBuilder<'_>),
    {
        self.set_fields[0] |= 8;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::BatchingConfigProtoBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

pub mod method_settings {
    #[derive(Clone, Copy, Debug)]
    pub struct LongRunningView<'a> {
        buf: &'a [u8],
        initial_poll_delay: ::roto::runtime::FieldMarker,
        poll_delay_multiplier: f32,
        max_poll_delay: ::roto::runtime::FieldMarker,
        total_poll_timeout: ::roto::runtime::FieldMarker,
        has_fields: [u32; 1],
    }

    impl<'a> LongRunningView<'a> {
        pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4];

        pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            let mut initial_poll_delay = ::roto::runtime::FieldMarker::none();
            let mut poll_delay_multiplier: f32 = 0.0f32;
            let mut max_poll_delay = ::roto::runtime::FieldMarker::none();
            let mut total_poll_timeout = ::roto::runtime::FieldMarker::none();
            let mut has_fields = [0u32; 1];

            for item in ::roto::runtime::FieldIter::new(buf) {
                let field = item?;
                match (field.tag, field.wire_type) {
                    (1, ::roto::runtime::WireType::LengthDelimited) => initial_poll_delay = field.marker,
                    (2, ::roto::runtime::WireType::Fixed32) => {
                        poll_delay_multiplier = f32::from_bits(field.val as u32);
                        has_fields[0] |= 1;
                    }
                    (3, ::roto::runtime::WireType::LengthDelimited) => max_poll_delay = field.marker,
                    (4, ::roto::runtime::WireType::LengthDelimited) => total_poll_timeout = field.marker,
                    _ => {}
                }
            }

            Ok(Self {
                buf,
                initial_poll_delay,
                poll_delay_multiplier,
                max_poll_delay,
                total_poll_timeout,
                has_fields,
            })
        }

        #[inline]
        pub const fn as_bytes(&self) -> &'a [u8] {
            self.buf
        }

        #[inline]
        pub const fn has_initial_poll_delay(&self) -> bool {
            self.initial_poll_delay.is_present()
        }

        #[inline]
        pub fn initial_poll_delay(&self) -> Result<Option<crate::google::protobuf::DurationView<'a>>, ::roto::runtime::DecodeError> {
            if !self.initial_poll_delay.is_present() {
                return Ok(None);
            }
            crate::google::protobuf::DurationView::from_bytes(self.initial_poll_delay.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_poll_delay_multiplier(&self) -> bool {
            (self.has_fields[0] & 1) != 0
        }

        #[inline]
        pub const fn poll_delay_multiplier(&self) -> Result<f32, ::roto::runtime::DecodeError> {
            Ok(self.poll_delay_multiplier)
        }

        #[inline]
        pub const fn has_max_poll_delay(&self) -> bool {
            self.max_poll_delay.is_present()
        }

        #[inline]
        pub fn max_poll_delay(&self) -> Result<Option<crate::google::protobuf::DurationView<'a>>, ::roto::runtime::DecodeError> {
            if !self.max_poll_delay.is_present() {
                return Ok(None);
            }
            crate::google::protobuf::DurationView::from_bytes(self.max_poll_delay.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn has_total_poll_timeout(&self) -> bool {
            self.total_poll_timeout.is_present()
        }

        #[inline]
        pub fn total_poll_timeout(&self) -> Result<Option<crate::google::protobuf::DurationView<'a>>, ::roto::runtime::DecodeError> {
            if !self.total_poll_timeout.is_present() {
                return Ok(None);
            }
            crate::google::protobuf::DurationView::from_bytes(self.total_poll_timeout.slice(self.buf)).map(Some)
        }

        #[inline]
        pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
            ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
        }
    }

    impl<'a> ::roto::runtime::MessageView<'a> for LongRunningView<'a> {
        #[inline]
        fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
            Self::from_bytes(buf)
        }
    }

    pub struct LongRunningBuilder<'b> {
        pub writer: ::roto::runtime::BackwardWriter<'b>,
        set_fields: [u64; 1],
    }

    impl<'b> LongRunningBuilder<'b> {
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
        pub fn copy_unknown_fields(&mut self, view: &LongRunningView<'_>) -> &mut Self {
            let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !LongRunningView::KNOWN_TAGS.contains(&tag));
            self
        }

        #[inline]
        pub fn copy_unset_fields(&mut self, view: &LongRunningView<'_>) -> &mut Self {
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
        pub fn write_initial_poll_delay<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::google::protobuf::DurationBuilder<'_>),
        {
            self.set_fields[0] |= 1;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::google::protobuf::DurationBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_poll_delay_multiplier(&mut self, val: f32) -> &mut Self {
            self.set_fields[0] |= 2;
            let _ = self.writer.write_f32(val);
            let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Fixed32);
            self
        }

        #[inline]
        pub fn write_max_poll_delay<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::google::protobuf::DurationBuilder<'_>),
        {
            self.set_fields[0] |= 4;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::google::protobuf::DurationBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
            self
        }

        #[inline]
        pub fn write_total_poll_timeout<F>(&mut self, f: F) -> &mut Self
        where
            F: FnOnce(&mut crate::google::protobuf::DurationBuilder<'_>),
        {
            self.set_fields[0] |= 8;
            let end_pos = self.writer.cursor();
            self.writer.with_sub_writer(|sub_writer| {
                let mut sub_builder = crate::google::protobuf::DurationBuilder::from_writer(sub_writer);
                f(&mut sub_builder);
                sub_builder.writer
            });
            let nested_len = end_pos - self.writer.cursor();
            let _ = self.writer.write_varint(nested_len as u64);
            let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
            self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SelectiveGapicGenerationView<'a> {
    buf: &'a [u8],
    methods: ::roto::runtime::FieldMarker,
    generate_omitted_as_internal: bool,
    has_fields: [u32; 1],
}

impl<'a> SelectiveGapicGenerationView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut methods = ::roto::runtime::FieldMarker::none();
        let mut generate_omitted_as_internal: bool = false;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, _) => methods.extend(field.raw_span),
                (2, ::roto::runtime::WireType::Varint) => {
                    generate_omitted_as_internal = field.val != 0;
                    has_fields[0] |= 1;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            methods,
            generate_omitted_as_internal,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn methods(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.methods.slice(self.buf), 1)
    }

    #[inline]
    pub const fn has_generate_omitted_as_internal(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn generate_omitted_as_internal(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.generate_omitted_as_internal)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for SelectiveGapicGenerationView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct SelectiveGapicGenerationBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> SelectiveGapicGenerationBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &SelectiveGapicGenerationView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !SelectiveGapicGenerationView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &SelectiveGapicGenerationView<'_>) -> &mut Self {
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
    pub fn write_methods(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_method(&mut self, val: &str) -> &mut Self {
        self.write_methods(val)
    }

    #[inline]
    pub fn write_generate_omitted_as_internal(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BatchingConfigProtoView<'a> {
    buf: &'a [u8],
    thresholds: ::roto::runtime::FieldMarker,
    batch_descriptor: ::roto::runtime::FieldMarker,
}

impl<'a> BatchingConfigProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut thresholds = ::roto::runtime::FieldMarker::none();
        let mut batch_descriptor = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => thresholds = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => batch_descriptor = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            thresholds,
            batch_descriptor,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_thresholds(&self) -> bool {
        self.thresholds.is_present()
    }

    #[inline]
    pub fn thresholds(&self) -> Result<Option<crate::google::api::BatchingSettingsProtoView<'a>>, ::roto::runtime::DecodeError> {
        if !self.thresholds.is_present() {
            return Ok(None);
        }
        crate::google::api::BatchingSettingsProtoView::from_bytes(self.thresholds.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_batch_descriptor(&self) -> bool {
        self.batch_descriptor.is_present()
    }

    #[inline]
    pub fn batch_descriptor(&self) -> Result<Option<crate::google::api::BatchingDescriptorProtoView<'a>>, ::roto::runtime::DecodeError> {
        if !self.batch_descriptor.is_present() {
            return Ok(None);
        }
        crate::google::api::BatchingDescriptorProtoView::from_bytes(self.batch_descriptor.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for BatchingConfigProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct BatchingConfigProtoBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> BatchingConfigProtoBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &BatchingConfigProtoView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BatchingConfigProtoView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &BatchingConfigProtoView<'_>) -> &mut Self {
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
    pub fn write_thresholds<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::BatchingSettingsProtoBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::BatchingSettingsProtoBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_batch_descriptor<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::BatchingDescriptorProtoBuilder<'_>),
    {
        self.set_fields[0] |= 2;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::BatchingDescriptorProtoBuilder::from_writer(sub_writer);
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
pub struct BatchingSettingsProtoView<'a> {
    buf: &'a [u8],
    element_count_threshold: i32,
    request_byte_threshold: i64,
    delay_threshold: ::roto::runtime::FieldMarker,
    element_count_limit: i32,
    request_byte_limit: i32,
    flow_control_element_limit: i32,
    flow_control_byte_limit: i32,
    flow_control_limit_exceeded_behavior: crate::google::api::FlowControlLimitExceededBehaviorProto,
    has_fields: [u32; 1],
}

impl<'a> BatchingSettingsProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 7, 8];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut element_count_threshold: i32 = 0i32;
        let mut request_byte_threshold: i64 = 0i64;
        let mut delay_threshold = ::roto::runtime::FieldMarker::none();
        let mut element_count_limit: i32 = 0i32;
        let mut request_byte_limit: i32 = 0i32;
        let mut flow_control_element_limit: i32 = 0i32;
        let mut flow_control_byte_limit: i32 = 0i32;
        let mut flow_control_limit_exceeded_behavior: crate::google::api::FlowControlLimitExceededBehaviorProto = crate::google::api::FlowControlLimitExceededBehaviorProto::UnsetBehavior;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::Varint) => {
                    element_count_threshold = field.val as i32;
                    has_fields[0] |= 1;
                }
                (2, ::roto::runtime::WireType::Varint) => {
                    request_byte_threshold = field.val as i64;
                    has_fields[0] |= 2;
                }
                (3, ::roto::runtime::WireType::LengthDelimited) => delay_threshold = field.marker,
                (4, ::roto::runtime::WireType::Varint) => {
                    element_count_limit = field.val as i32;
                    has_fields[0] |= 4;
                }
                (5, ::roto::runtime::WireType::Varint) => {
                    request_byte_limit = field.val as i32;
                    has_fields[0] |= 8;
                }
                (6, ::roto::runtime::WireType::Varint) => {
                    flow_control_element_limit = field.val as i32;
                    has_fields[0] |= 16;
                }
                (7, ::roto::runtime::WireType::Varint) => {
                    flow_control_byte_limit = field.val as i32;
                    has_fields[0] |= 32;
                }
                (8, ::roto::runtime::WireType::Varint) => {
                    flow_control_limit_exceeded_behavior = <crate::google::api::FlowControlLimitExceededBehaviorProto as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 64;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            element_count_threshold,
            request_byte_threshold,
            delay_threshold,
            element_count_limit,
            request_byte_limit,
            flow_control_element_limit,
            flow_control_byte_limit,
            flow_control_limit_exceeded_behavior,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_element_count_threshold(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn element_count_threshold(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.element_count_threshold)
    }

    #[inline]
    pub const fn has_request_byte_threshold(&self) -> bool {
        (self.has_fields[0] & 2) != 0
    }

    #[inline]
    pub const fn request_byte_threshold(&self) -> Result<i64, ::roto::runtime::DecodeError> {
        Ok(self.request_byte_threshold)
    }

    #[inline]
    pub const fn has_delay_threshold(&self) -> bool {
        self.delay_threshold.is_present()
    }

    #[inline]
    pub fn delay_threshold(&self) -> Result<Option<crate::google::protobuf::DurationView<'a>>, ::roto::runtime::DecodeError> {
        if !self.delay_threshold.is_present() {
            return Ok(None);
        }
        crate::google::protobuf::DurationView::from_bytes(self.delay_threshold.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_element_count_limit(&self) -> bool {
        (self.has_fields[0] & 4) != 0
    }

    #[inline]
    pub const fn element_count_limit(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.element_count_limit)
    }

    #[inline]
    pub const fn has_request_byte_limit(&self) -> bool {
        (self.has_fields[0] & 8) != 0
    }

    #[inline]
    pub const fn request_byte_limit(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.request_byte_limit)
    }

    #[inline]
    pub const fn has_flow_control_element_limit(&self) -> bool {
        (self.has_fields[0] & 16) != 0
    }

    #[inline]
    pub const fn flow_control_element_limit(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.flow_control_element_limit)
    }

    #[inline]
    pub const fn has_flow_control_byte_limit(&self) -> bool {
        (self.has_fields[0] & 32) != 0
    }

    #[inline]
    pub const fn flow_control_byte_limit(&self) -> Result<i32, ::roto::runtime::DecodeError> {
        Ok(self.flow_control_byte_limit)
    }

    #[inline]
    pub const fn has_flow_control_limit_exceeded_behavior(&self) -> bool {
        (self.has_fields[0] & 64) != 0
    }

    #[inline]
    pub const fn flow_control_limit_exceeded_behavior(&self) -> Result<crate::google::api::FlowControlLimitExceededBehaviorProto, ::roto::runtime::DecodeError> {
        Ok(self.flow_control_limit_exceeded_behavior)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for BatchingSettingsProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct BatchingSettingsProtoBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> BatchingSettingsProtoBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &BatchingSettingsProtoView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BatchingSettingsProtoView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &BatchingSettingsProtoView<'_>) -> &mut Self {
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
    pub fn write_element_count_threshold(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_request_byte_threshold(&mut self, val: i64) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_delay_threshold<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::protobuf::DurationBuilder<'_>),
    {
        self.set_fields[0] |= 4;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::protobuf::DurationBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_element_count_limit(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_request_byte_limit(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_flow_control_element_limit(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 32;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_flow_control_byte_limit(&mut self, val: i32) -> &mut Self {
        self.set_fields[0] |= 64;
        let _ = self.writer.write_varint(val as u64);
        let _ = self.writer.write_tag(7, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_flow_control_limit_exceeded_behavior(&mut self, val: crate::google::api::FlowControlLimitExceededBehaviorProto) -> &mut Self {
        self.set_fields[0] |= 128;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(8, ::roto::runtime::WireType::Varint);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BatchingDescriptorProtoView<'a> {
    buf: &'a [u8],
    batched_field: ::roto::runtime::FieldMarker,
    discriminator_fields: ::roto::runtime::FieldMarker,
    subresponse_field: ::roto::runtime::FieldMarker,
}

impl<'a> BatchingDescriptorProtoView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut batched_field = ::roto::runtime::FieldMarker::none();
        let mut discriminator_fields = ::roto::runtime::FieldMarker::none();
        let mut subresponse_field = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => batched_field = field.marker,
                (2, _) => discriminator_fields.extend(field.raw_span),
                (3, ::roto::runtime::WireType::LengthDelimited) => subresponse_field = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            batched_field,
            discriminator_fields,
            subresponse_field,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_batched_field(&self) -> bool {
        self.batched_field.is_present()
    }

    #[inline]
    pub fn batched_field(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.batched_field.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.batched_field)
    }

    #[inline]
    pub const fn discriminator_fields(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.discriminator_fields.slice(self.buf), 2)
    }

    #[inline]
    pub const fn has_subresponse_field(&self) -> bool {
        self.subresponse_field.is_present()
    }

    #[inline]
    pub fn subresponse_field(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.subresponse_field.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.subresponse_field)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for BatchingDescriptorProtoView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct BatchingDescriptorProtoBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> BatchingDescriptorProtoBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &BatchingDescriptorProtoView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !BatchingDescriptorProtoView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &BatchingDescriptorProtoView<'_>) -> &mut Self {
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
    pub fn write_batched_field(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_discriminator_fields(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_discriminator_field(&mut self, val: &str) -> &mut Self {
        self.write_discriminator_fields(val)
    }

    #[inline]
    pub fn write_subresponse_field(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FieldBehavior {
    FieldBehaviorUnspecified = 0,
    Optional = 1,
    Required = 2,
    OutputOnly = 3,
    InputOnly = 4,
    Immutable = 5,
    UnorderedList = 6,
    NonEmptyDefault = 7,
    Identifier = 8,
}

impl FieldBehavior {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::FieldBehaviorUnspecified),
            1 => Some(Self::Optional),
            2 => Some(Self::Required),
            3 => Some(Self::OutputOnly),
            4 => Some(Self::InputOnly),
            5 => Some(Self::Immutable),
            6 => Some(Self::UnorderedList),
            7 => Some(Self::NonEmptyDefault),
            8 => Some(Self::Identifier),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for FieldBehavior {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for FieldBehavior {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::FieldBehaviorUnspecified)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HttpView<'a> {
    buf: &'a [u8],
    rules: ::roto::runtime::FieldMarker,
    fully_decode_reserved_expansion: bool,
    has_fields: [u32; 1],
}

impl<'a> HttpView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut rules = ::roto::runtime::FieldMarker::none();
        let mut fully_decode_reserved_expansion: bool = false;
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, _) => rules.extend(field.raw_span),
                (2, ::roto::runtime::WireType::Varint) => {
                    fully_decode_reserved_expansion = field.val != 0;
                    has_fields[0] |= 1;
                }
                _ => {}
            }
        }

        Ok(Self {
            buf,
            rules,
            fully_decode_reserved_expansion,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn rules(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::HttpRuleView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.rules.slice(self.buf), 1)
    }

    #[inline]
    pub const fn has_fully_decode_reserved_expansion(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn fully_decode_reserved_expansion(&self) -> Result<bool, ::roto::runtime::DecodeError> {
        Ok(self.fully_decode_reserved_expansion)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for HttpView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct HttpBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> HttpBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &HttpView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !HttpView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &HttpView<'_>) -> &mut Self {
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
    pub fn write_rules<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::HttpRuleBuilder<'_>),
    {
        self.set_fields[0] |= 1;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::HttpRuleBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_rule<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::HttpRuleBuilder<'_>),
    {
        self.write_rules(f)
    }

    #[inline]
    pub fn write_fully_decode_reserved_expansion(&mut self, val: bool) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_bool(val);
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::Varint);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HttpRuleView<'a> {
    buf: &'a [u8],
    selector: ::roto::runtime::FieldMarker,
    get: ::roto::runtime::FieldMarker,
    put: ::roto::runtime::FieldMarker,
    post: ::roto::runtime::FieldMarker,
    delete: ::roto::runtime::FieldMarker,
    patch: ::roto::runtime::FieldMarker,
    custom: ::roto::runtime::FieldMarker,
    body: ::roto::runtime::FieldMarker,
    response_body: ::roto::runtime::FieldMarker,
    additional_bindings: ::roto::runtime::FieldMarker,
    oneof_0: u32,
}

impl<'a> HttpRuleView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 8, 7, 12, 11];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut selector = ::roto::runtime::FieldMarker::none();
        let mut get = ::roto::runtime::FieldMarker::none();
        let mut put = ::roto::runtime::FieldMarker::none();
        let mut post = ::roto::runtime::FieldMarker::none();
        let mut delete = ::roto::runtime::FieldMarker::none();
        let mut patch = ::roto::runtime::FieldMarker::none();
        let mut custom = ::roto::runtime::FieldMarker::none();
        let mut body = ::roto::runtime::FieldMarker::none();
        let mut response_body = ::roto::runtime::FieldMarker::none();
        let mut additional_bindings = ::roto::runtime::FieldMarker::none();
        let mut oneof_0: u32 = 0;

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => selector = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => {
                    get = field.marker;
                    oneof_0 = 2;
                }
                (3, ::roto::runtime::WireType::LengthDelimited) => {
                    put = field.marker;
                    oneof_0 = 3;
                }
                (4, ::roto::runtime::WireType::LengthDelimited) => {
                    post = field.marker;
                    oneof_0 = 4;
                }
                (5, ::roto::runtime::WireType::LengthDelimited) => {
                    delete = field.marker;
                    oneof_0 = 5;
                }
                (6, ::roto::runtime::WireType::LengthDelimited) => {
                    patch = field.marker;
                    oneof_0 = 6;
                }
                (8, ::roto::runtime::WireType::LengthDelimited) => {
                    custom = field.marker;
                    oneof_0 = 8;
                }
                (7, ::roto::runtime::WireType::LengthDelimited) => body = field.marker,
                (12, ::roto::runtime::WireType::LengthDelimited) => response_body = field.marker,
                (11, _) => additional_bindings.extend(field.raw_span),
                _ => {}
            }
        }

        Ok(Self {
            buf,
            selector,
            get,
            put,
            post,
            delete,
            patch,
            custom,
            body,
            response_body,
            additional_bindings,
            oneof_0,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_selector(&self) -> bool {
        self.selector.is_present()
    }

    #[inline]
    pub fn selector(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.selector.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.selector)
    }

    #[inline]
    pub const fn has_get(&self) -> bool {
        self.oneof_0 == 2
    }

    #[inline]
    pub fn get(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 2 {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.get)
    }

    #[inline]
    pub const fn has_put(&self) -> bool {
        self.oneof_0 == 3
    }

    #[inline]
    pub fn put(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 3 {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.put)
    }

    #[inline]
    pub const fn has_post(&self) -> bool {
        self.oneof_0 == 4
    }

    #[inline]
    pub fn post(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 4 {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.post)
    }

    #[inline]
    pub const fn has_delete(&self) -> bool {
        self.oneof_0 == 5
    }

    #[inline]
    pub fn delete(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 5 {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.delete)
    }

    #[inline]
    pub const fn has_patch(&self) -> bool {
        self.oneof_0 == 6
    }

    #[inline]
    pub fn patch(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 6 {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.patch)
    }

    #[inline]
    pub const fn has_custom(&self) -> bool {
        self.oneof_0 == 8
    }

    #[inline]
    pub fn custom(&self) -> Result<Option<crate::google::api::CustomHttpPatternView<'a>>, ::roto::runtime::DecodeError> {
        if self.oneof_0 != 8 {
            return Ok(None);
        }
        crate::google::api::CustomHttpPatternView::from_bytes(self.custom.slice(self.buf)).map(Some)
    }

    #[inline]
    pub const fn has_body(&self) -> bool {
        self.body.is_present()
    }

    #[inline]
    pub fn body(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.body.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.body)
    }

    #[inline]
    pub const fn has_response_body(&self) -> bool {
        self.response_body.is_present()
    }

    #[inline]
    pub fn response_body(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.response_body.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.response_body)
    }

    #[inline]
    pub const fn additional_bindings(&self) -> ::roto::runtime::RepeatedMessageIter<'a, crate::google::api::HttpRuleView<'a>> {
        ::roto::runtime::RepeatedMessageIter::new(self.additional_bindings.slice(self.buf), 11)
    }

    #[inline]
    pub fn pattern(&self) -> Result<Option<http_rule::Pattern<'a>>, ::roto::runtime::DecodeError> {
        match self.oneof_0 {
            2 => Ok(Some(http_rule::Pattern::Get(::roto::runtime::decode_str_at(self.buf, self.get)?))),
            3 => Ok(Some(http_rule::Pattern::Put(::roto::runtime::decode_str_at(self.buf, self.put)?))),
            4 => Ok(Some(http_rule::Pattern::Post(::roto::runtime::decode_str_at(self.buf, self.post)?))),
            5 => Ok(Some(http_rule::Pattern::Delete(::roto::runtime::decode_str_at(self.buf, self.delete)?))),
            6 => Ok(Some(http_rule::Pattern::Patch(::roto::runtime::decode_str_at(self.buf, self.patch)?))),
            8 => Ok(Some(http_rule::Pattern::Custom(crate::google::api::CustomHttpPatternView::from_bytes(self.custom.slice(self.buf))?))),
            _ => Ok(None),
        }
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for HttpRuleView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct HttpRuleBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> HttpRuleBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &HttpRuleView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !HttpRuleView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &HttpRuleView<'_>) -> &mut Self {
        let prev_set = self.set_fields;
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| match tag {
            1 => {
                self.set_fields[0] |= 1;
                (prev_set[0] & 1) == 0
            }
            2 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 2) == 0
            }
            3 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 4) == 0
            }
            4 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 8) == 0
            }
            5 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 16) == 0
            }
            6 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 32) == 0
            }
            8 => {
                self.set_fields[0] |= 126;
                (prev_set[0] & 64) == 0
            }
            7 => {
                self.set_fields[0] |= 128;
                (prev_set[0] & 128) == 0
            }
            12 => {
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
    pub fn write_selector(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_get(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 126;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_put(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 126;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_post(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 126;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_delete(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 126;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_patch(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 126;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_custom<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::CustomHttpPatternBuilder<'_>),
    {
        self.set_fields[0] |= 126;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::CustomHttpPatternBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(8, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_body(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 128;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(7, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_response_body(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 256;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(12, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_additional_bindings<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::HttpRuleBuilder<'_>),
    {
        self.set_fields[0] |= 512;
        let end_pos = self.writer.cursor();
        self.writer.with_sub_writer(|sub_writer| {
            let mut sub_builder = crate::google::api::HttpRuleBuilder::from_writer(sub_writer);
            f(&mut sub_builder);
            sub_builder.writer
        });
        let nested_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(nested_len as u64);
        let _ = self.writer.write_tag(11, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_additional_binding<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut crate::google::api::HttpRuleBuilder<'_>),
    {
        self.write_additional_bindings(f)
    }
}

pub mod http_rule {
    #[derive(Clone, Copy, Debug)]
    pub enum Pattern<'a> {
        Get(&'a str),
        Put(&'a str),
        Post(&'a str),
        Delete(&'a str),
        Patch(&'a str),
        Custom(crate::google::api::CustomHttpPatternView<'a>),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CustomHttpPatternView<'a> {
    buf: &'a [u8],
    kind: ::roto::runtime::FieldMarker,
    path: ::roto::runtime::FieldMarker,
}

impl<'a> CustomHttpPatternView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut kind = ::roto::runtime::FieldMarker::none();
        let mut path = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => kind = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => path = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            kind,
            path,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
    }

    #[inline]
    pub const fn has_kind(&self) -> bool {
        self.kind.is_present()
    }

    #[inline]
    pub fn kind(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.kind.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.kind)
    }

    #[inline]
    pub const fn has_path(&self) -> bool {
        self.path.is_present()
    }

    #[inline]
    pub fn path(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.path.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.path)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for CustomHttpPatternView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct CustomHttpPatternBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> CustomHttpPatternBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &CustomHttpPatternView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !CustomHttpPatternView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &CustomHttpPatternView<'_>) -> &mut Self {
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
    pub fn write_kind(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_path(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LaunchStage {
    LaunchStageUnspecified = 0,
    Unimplemented = 6,
    Prelaunch = 7,
    EarlyAccess = 1,
    Alpha = 2,
    Beta = 3,
    Ga = 4,
    Deprecated = 5,
}

impl LaunchStage {
    #[inline]
    pub const fn from_i32(val: i32) -> Option<Self> {
        match val {
            0 => Some(Self::LaunchStageUnspecified),
            6 => Some(Self::Unimplemented),
            7 => Some(Self::Prelaunch),
            1 => Some(Self::EarlyAccess),
            2 => Some(Self::Alpha),
            3 => Some(Self::Beta),
            4 => Some(Self::Ga),
            5 => Some(Self::Deprecated),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

impl core::convert::TryFrom<i32> for LaunchStage {
    type Error = i32;

    #[inline]
    fn try_from(val: i32) -> Result<Self, Self::Error> {
        Self::from_i32(val).ok_or(val)
    }
}

impl ::roto::runtime::VarintValue for LaunchStage {
    #[inline]
    fn from_raw_varint(val: u64) -> Self {
        Self::from_i32(val as i32).unwrap_or(Self::LaunchStageUnspecified)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceDescriptorView<'a> {
    buf: &'a [u8],
    r#type: ::roto::runtime::FieldMarker,
    pattern: ::roto::runtime::FieldMarker,
    name_field: ::roto::runtime::FieldMarker,
    history: crate::google::api::resource_descriptor::History,
    plural: ::roto::runtime::FieldMarker,
    singular: ::roto::runtime::FieldMarker,
    style: ::roto::runtime::FieldMarker,
    has_fields: [u32; 1],
}

impl<'a> ResourceDescriptorView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2, 3, 4, 5, 6, 10];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut r#type = ::roto::runtime::FieldMarker::none();
        let mut pattern = ::roto::runtime::FieldMarker::none();
        let mut name_field = ::roto::runtime::FieldMarker::none();
        let mut history: crate::google::api::resource_descriptor::History = crate::google::api::resource_descriptor::History::HistoryUnspecified;
        let mut plural = ::roto::runtime::FieldMarker::none();
        let mut singular = ::roto::runtime::FieldMarker::none();
        let mut style = ::roto::runtime::FieldMarker::none();
        let mut has_fields = [0u32; 1];

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => r#type = field.marker,
                (2, _) => pattern.extend(field.raw_span),
                (3, ::roto::runtime::WireType::LengthDelimited) => name_field = field.marker,
                (4, ::roto::runtime::WireType::Varint) => {
                    history = <crate::google::api::resource_descriptor::History as ::roto::runtime::VarintValue>::from_raw_varint(field.val);
                    has_fields[0] |= 1;
                }
                (5, ::roto::runtime::WireType::LengthDelimited) => plural = field.marker,
                (6, ::roto::runtime::WireType::LengthDelimited) => singular = field.marker,
                (10, _) => style.extend(field.raw_span),
                _ => {}
            }
        }

        Ok(Self {
            buf,
            r#type,
            pattern,
            name_field,
            history,
            plural,
            singular,
            style,
            has_fields,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
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
    pub const fn pattern(&self) -> ::roto::runtime::RepeatedStrIter<'a> {
        ::roto::runtime::RepeatedStrIter::new(self.pattern.slice(self.buf), 2)
    }

    #[inline]
    pub const fn has_name_field(&self) -> bool {
        self.name_field.is_present()
    }

    #[inline]
    pub fn name_field(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.name_field.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.name_field)
    }

    #[inline]
    pub const fn has_history(&self) -> bool {
        (self.has_fields[0] & 1) != 0
    }

    #[inline]
    pub const fn history(&self) -> Result<crate::google::api::resource_descriptor::History, ::roto::runtime::DecodeError> {
        Ok(self.history)
    }

    #[inline]
    pub const fn has_plural(&self) -> bool {
        self.plural.is_present()
    }

    #[inline]
    pub fn plural(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.plural.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.plural)
    }

    #[inline]
    pub const fn has_singular(&self) -> bool {
        self.singular.is_present()
    }

    #[inline]
    pub fn singular(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.singular.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.singular)
    }

    #[inline]
    pub const fn style(&self) -> ::roto::runtime::RepeatedVarintIter<'a, crate::google::api::resource_descriptor::Style> {
        ::roto::runtime::RepeatedVarintIter::new(self.style.slice(self.buf), 10)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for ResourceDescriptorView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ResourceDescriptorBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> ResourceDescriptorBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &ResourceDescriptorView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ResourceDescriptorView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &ResourceDescriptorView<'_>) -> &mut Self {
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
            10 => {
                self.set_fields[0] |= 64;
                (prev_set[0] & 64) == 0
            }
            _ => true,
        });
        self
    }

    #[inline]
    pub fn write_type(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_pattern(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_name_field(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 4;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(3, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_history(&mut self, val: crate::google::api::resource_descriptor::History) -> &mut Self {
        self.set_fields[0] |= 8;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(4, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_plural(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 16;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(5, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_singular(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 32;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(6, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_style(&mut self, val: crate::google::api::resource_descriptor::Style) -> &mut Self {
        self.set_fields[0] |= 64;
        let _ = self.writer.write_varint(val.as_i32() as u64);
        let _ = self.writer.write_tag(10, ::roto::runtime::WireType::Varint);
        self
    }

    #[inline]
    pub fn write_packed_style(&mut self, vals: &[crate::google::api::resource_descriptor::Style]) -> &mut Self {
        self.set_fields[0] |= 64;
        let end_pos = self.writer.cursor();
        for &val in vals.iter().rev() {
            let _ = self.writer.write_varint(val.as_i32() as u64);
        }
        let packed_len = end_pos - self.writer.cursor();
        let _ = self.writer.write_varint(packed_len as u64);
        let _ = self.writer.write_tag(10, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}

pub mod resource_descriptor {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum History {
        HistoryUnspecified = 0,
        OriginallySinglePattern = 1,
        FutureMultiPattern = 2,
    }

    impl History {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::HistoryUnspecified),
                1 => Some(Self::OriginallySinglePattern),
                2 => Some(Self::FutureMultiPattern),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for History {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for History {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::HistoryUnspecified)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    #[repr(i32)]
    pub enum Style {
        StyleUnspecified = 0,
        DeclarativeFriendly = 1,
    }

    impl Style {
        #[inline]
        pub const fn from_i32(val: i32) -> Option<Self> {
            match val {
                0 => Some(Self::StyleUnspecified),
                1 => Some(Self::DeclarativeFriendly),
                _ => None,
            }
        }

        #[inline]
        pub const fn as_i32(self) -> i32 {
            self as i32
        }
    }

    impl core::convert::TryFrom<i32> for Style {
        type Error = i32;

        #[inline]
        fn try_from(val: i32) -> Result<Self, Self::Error> {
            Self::from_i32(val).ok_or(val)
        }
    }

    impl ::roto::runtime::VarintValue for Style {
        #[inline]
        fn from_raw_varint(val: u64) -> Self {
            Self::from_i32(val as i32).unwrap_or(Self::StyleUnspecified)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ResourceReferenceView<'a> {
    buf: &'a [u8],
    r#type: ::roto::runtime::FieldMarker,
    child_type: ::roto::runtime::FieldMarker,
}

impl<'a> ResourceReferenceView<'a> {
    pub const KNOWN_TAGS: &'static [u32] = &[1, 2];

    pub fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        let mut r#type = ::roto::runtime::FieldMarker::none();
        let mut child_type = ::roto::runtime::FieldMarker::none();

        for item in ::roto::runtime::FieldIter::new(buf) {
            let field = item?;
            match (field.tag, field.wire_type) {
                (1, ::roto::runtime::WireType::LengthDelimited) => r#type = field.marker,
                (2, ::roto::runtime::WireType::LengthDelimited) => child_type = field.marker,
                _ => {}
            }
        }

        Ok(Self {
            buf,
            r#type,
            child_type,
        })
    }

    #[inline]
    pub const fn as_bytes(&self) -> &'a [u8] {
        self.buf
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
    pub const fn has_child_type(&self) -> bool {
        self.child_type.is_present()
    }

    #[inline]
    pub fn child_type(&self) -> Result<&'a str, ::roto::runtime::DecodeError> {
        if !self.child_type.is_present() {
            return Ok("");
        }
        ::roto::runtime::decode_str_at(self.buf, self.child_type)
    }

    #[inline]
    pub const fn unknown_fields(&self) -> ::roto::runtime::UnknownFieldsIter<'a> {
        ::roto::runtime::UnknownFieldsIter::new(self.buf, Self::KNOWN_TAGS)
    }
}

impl<'a> ::roto::runtime::MessageView<'a> for ResourceReferenceView<'a> {
    #[inline]
    fn from_bytes(buf: &'a [u8]) -> Result<Self, ::roto::runtime::DecodeError> {
        Self::from_bytes(buf)
    }
}

pub struct ResourceReferenceBuilder<'b> {
    pub writer: ::roto::runtime::BackwardWriter<'b>,
    set_fields: [u64; 1],
}

impl<'b> ResourceReferenceBuilder<'b> {
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
    pub fn copy_unknown_fields(&mut self, view: &ResourceReferenceView<'_>) -> &mut Self {
        let _ = self.writer.copy_filtered_fields(view.as_bytes(), |tag| !ResourceReferenceView::KNOWN_TAGS.contains(&tag));
        self
    }

    #[inline]
    pub fn copy_unset_fields(&mut self, view: &ResourceReferenceView<'_>) -> &mut Self {
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
    pub fn write_type(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 1;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(1, ::roto::runtime::WireType::LengthDelimited);
        self
    }

    #[inline]
    pub fn write_child_type(&mut self, val: &str) -> &mut Self {
        self.set_fields[0] |= 2;
        let _ = self.writer.write_length_delimited(val.as_bytes());
        let _ = self.writer.write_tag(2, ::roto::runtime::WireType::LengthDelimited);
        self
    }
}
