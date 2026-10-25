use std::collections::{BTreeMap, BTreeSet};

use google_protobuf_roto_bootstrap::google::protobuf::{
    compiler::CodeGeneratorRequestView,
    field_descriptor_proto::{Label as FieldLabel, Type as FieldType},
    DescriptorProtoView, FileDescriptorProtoView, MethodDescriptorProtoView, MethodOptionsView,
};
use roto_runtime::{decode_bytes_at, decode_str_at, FieldIter, WireType};

/// Extension field number for `google.api.http` on `google.protobuf.MethodOptions`
/// (`extend google.protobuf.MethodOptions { HttpRule http = 72295728; }`).
pub const GOOGLE_API_HTTP_TAG: u32 = 72_295_728;

const HTTP_RULE_SELECTOR_TAG: u32 = 1;
const HTTP_RULE_GET_TAG: u32 = 2;
const HTTP_RULE_PUT_TAG: u32 = 3;
const HTTP_RULE_POST_TAG: u32 = 4;
const HTTP_RULE_DELETE_TAG: u32 = 5;
const HTTP_RULE_PATCH_TAG: u32 = 6;
const HTTP_RULE_BODY_TAG: u32 = 7;
const HTTP_RULE_CUSTOM_TAG: u32 = 8;
const HTTP_RULE_ADDITIONAL_BINDINGS_TAG: u32 = 11;
const HTTP_RULE_RESPONSE_BODY_TAG: u32 = 12;

const CUSTOM_HTTP_PATTERN_KIND_TAG: u32 = 1;
const CUSTOM_HTTP_PATTERN_PATH_TAG: u32 = 2;

/// HTTP method and URL path template specified by a `google.api.HttpRule` `pattern` oneof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HttpPattern {
    Get(String),
    Put(String),
    Post(String),
    Delete(String),
    Patch(String),
    Custom { kind: String, path: String },
}

impl HttpPattern {
    /// Returns the HTTP method verb string (e.g. `"GET"`, `"POST"`, or the custom `kind`).
    #[inline]
    pub fn method_name(&self) -> &str {
        match self {
            Self::Get(_) => "GET",
            Self::Put(_) => "PUT",
            Self::Post(_) => "POST",
            Self::Delete(_) => "DELETE",
            Self::Patch(_) => "PATCH",
            Self::Custom { kind, .. } => kind.as_str(),
        }
    }

    /// Returns the raw URL path template string.
    #[inline]
    pub fn path(&self) -> &str {
        match self {
            Self::Get(p)
            | Self::Put(p)
            | Self::Post(p)
            | Self::Delete(p)
            | Self::Patch(p)
            | Self::Custom { path: p, .. } => p.as_str(),
        }
    }
}

/// Decoded `google.api.HttpRule` annotation attached to an RPC method.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HttpRule {
    /// Selects a method to which this rule applies.
    pub selector: String,
    /// HTTP method and path template pattern (`oneof pattern`).
    pub pattern: Option<HttpPattern>,
    /// The name of the request field whose value is mapped to the HTTP request body,
    /// `"*"` for mapping all request fields not captured by the path pattern to the body,
    /// or `""` for not mapping any part of the request to the body.
    pub body: String,
    /// Optional name of the response field whose value is mapped to the HTTP response body.
    pub response_body: String,
    /// Additional HTTP bindings for the selector. Must not contain nested `additional_bindings`.
    pub additional_bindings: Vec<HttpRule>,
}

/// Extracts `(google.api.http)` (`tag = 72295728`) from `method.options()` if present.
pub fn extract_method_http_rule(
    method: &MethodDescriptorProtoView<'_>,
) -> Result<Option<HttpRule>, String> {
    let method_name = method
        .name()
        .map_err(|e| format!("invalid MethodDescriptorProto.name: {e}"))?;
    let Some(opts) = method.options().map_err(|e| {
        format!("invalid MethodDescriptorProto.options in method '{method_name}': {e}")
    })?
    else {
        return Ok(None);
    };
    extract_http_rule_from_options(&opts)
        .map_err(|e| format!("invalid google.api.http option on method '{method_name}': {e}"))
}

/// Extracts `(google.api.http)` (`tag = 72295728`) from a [`MethodOptionsView`].
pub fn extract_http_rule_from_options(
    options: &MethodOptionsView<'_>,
) -> Result<Option<HttpRule>, String> {
    extract_http_rule_from_options_bytes(options.as_bytes())
}

/// Extracts `(google.api.http)` (`tag = 72295728`) from raw serialized `MethodOptions` bytes.
pub fn extract_http_rule_from_options_bytes(buf: &[u8]) -> Result<Option<HttpRule>, String> {
    let mut rule: Option<HttpRule> = None;

    for item in FieldIter::new(buf) {
        let field = item.map_err(|e| format!("corrupt MethodOptions wire bytes: {e}"))?;
        if field.tag == GOOGLE_API_HTTP_TAG {
            if field.wire_type != WireType::LengthDelimited {
                return Err(format!(
                    "google.api.http extension (tag {GOOGLE_API_HTTP_TAG}) must have LengthDelimited wire type, got {:?}",
                    field.wire_type
                ));
            }
            let payload = decode_bytes_at(buf, field.marker)
                .map_err(|e| format!("corrupt google.api.http extension payload: {e}"))?;
            let target = rule.get_or_insert_with(HttpRule::default);
            merge_http_rule(payload, target, false)?;
        }
    }

    Ok(rule)
}

/// Decodes a standalone `google.api.HttpRule` message from raw protobuf wire bytes.
pub fn decode_http_rule(buf: &[u8]) -> Result<HttpRule, String> {
    let mut rule = HttpRule::default();
    merge_http_rule(buf, &mut rule, false)?;
    Ok(rule)
}

fn merge_http_rule(
    buf: &[u8],
    rule: &mut HttpRule,
    is_additional_binding: bool,
) -> Result<(), String> {
    for item in FieldIter::new(buf) {
        let field = item.map_err(|e| format!("corrupt HttpRule wire bytes: {e}"))?;
        match field.tag {
            HTTP_RULE_SELECTOR_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.selector (tag 1) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.selector: {e}"))?;
                rule.selector = s.to_string();
            }
            HTTP_RULE_GET_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.get (tag 2) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.get: {e}"))?;
                rule.pattern = Some(HttpPattern::Get(s.to_string()));
            }
            HTTP_RULE_PUT_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.put (tag 3) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.put: {e}"))?;
                rule.pattern = Some(HttpPattern::Put(s.to_string()));
            }
            HTTP_RULE_POST_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.post (tag 4) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.post: {e}"))?;
                rule.pattern = Some(HttpPattern::Post(s.to_string()));
            }
            HTTP_RULE_DELETE_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.delete (tag 5) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.delete: {e}"))?;
                rule.pattern = Some(HttpPattern::Delete(s.to_string()));
            }
            HTTP_RULE_PATCH_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.patch (tag 6) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.patch: {e}"))?;
                rule.pattern = Some(HttpPattern::Patch(s.to_string()));
            }
            HTTP_RULE_BODY_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.body (tag 7) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.body: {e}"))?;
                rule.body = s.to_string();
            }
            HTTP_RULE_CUSTOM_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.custom (tag 8) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let custom_bytes = decode_bytes_at(buf, field.marker)
                    .map_err(|e| format!("corrupt HttpRule.custom submessage: {e}"))?;
                let (kind, path) = decode_custom_http_pattern(custom_bytes)?;
                rule.pattern = Some(HttpPattern::Custom { kind, path });
            }
            HTTP_RULE_ADDITIONAL_BINDINGS_TAG => {
                if is_additional_binding {
                    return Err(
                        "google.api.http additional_bindings cannot contain nested additional_bindings"
                            .to_string(),
                    );
                }
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.additional_bindings (tag 11) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let sub_bytes = decode_bytes_at(buf, field.marker)
                    .map_err(|e| format!("corrupt HttpRule.additional_bindings submessage: {e}"))?;
                let mut child = HttpRule::default();
                merge_http_rule(sub_bytes, &mut child, true)?;
                rule.additional_bindings.push(child);
            }
            HTTP_RULE_RESPONSE_BODY_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "HttpRule.response_body (tag 12) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in HttpRule.response_body: {e}"))?;
                rule.response_body = s.to_string();
            }
            _ => {}
        }
    }

    Ok(())
}

fn decode_custom_http_pattern(buf: &[u8]) -> Result<(String, String), String> {
    let mut kind = String::new();
    let mut path = String::new();

    for item in FieldIter::new(buf) {
        let field = item.map_err(|e| format!("corrupt CustomHttpPattern wire bytes: {e}"))?;
        match field.tag {
            CUSTOM_HTTP_PATTERN_KIND_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "CustomHttpPattern.kind (tag 1) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in CustomHttpPattern.kind: {e}"))?;
                kind = s.to_string();
            }
            CUSTOM_HTTP_PATTERN_PATH_TAG => {
                if field.wire_type != WireType::LengthDelimited {
                    return Err(format!(
                        "CustomHttpPattern.path (tag 2) must be LengthDelimited, got {:?}",
                        field.wire_type
                    ));
                }
                let s = decode_str_at(buf, field.marker)
                    .map_err(|e| format!("invalid UTF-8 in CustomHttpPattern.path: {e}"))?;
                path = s.to_string();
            }
            _ => {}
        }
    }

    Ok((kind, path))
}

/// A segment within a variable's sub-template (e.g. `organizations/*/users/**`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubSegment {
    /// Exact literal segment (e.g. `"organizations"`).
    Literal(String),
    /// Single-segment wildcard (`*`).
    SingleWildcard,
    /// Multi-segment wildcard (`**`), only valid as the final segment of the entire path template.
    MultiWildcard,
}

/// A variable binding within a URL path template (e.g. `{user.id}` or `{name=v1/users/*}`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathVariable {
    /// Dotted field path segments (e.g. `["user", "id"]`).
    pub field_path: Vec<String>,
    /// Sub-template segments matched by this variable (defaults to `[SubSegment::SingleWildcard]`).
    pub sub_template: Vec<SubSegment>,
}

impl PathVariable {
    /// Returns the dotted field path string (e.g. `"user.id"`).
    #[inline]
    pub fn dotted_path(&self) -> String {
        self.field_path.join(".")
    }
}

/// A top-level element of a parsed URL path template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PathSegment {
    /// Literal URL path segment (e.g. `"v1"`, `"users"`).
    Literal(String),
    /// Single-segment unnamed wildcard (`*`).
    SingleWildcard,
    /// Multi-segment unnamed wildcard (`**`).
    MultiWildcard,
    /// Bound variable segment (`{field.path}` or `{field.path=sub_template}`).
    Variable(PathVariable),
}

/// Structured representation of a parsed `google.api.http` URL path template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedPathTemplate {
    /// Original raw template string.
    pub raw: String,
    /// Ordered top-level path segments.
    pub segments: Vec<PathSegment>,
    /// Optional trailing custom verb (e.g. `Some("cancel")` for `"/v1/items/{id}:cancel"`).
    pub verb: Option<String>,
}

impl ParsedPathTemplate {
    /// Parses a `google.api.http` URL path template into a [`ParsedPathTemplate`].
    pub fn parse(template: &str) -> Result<Self, String> {
        if !template.starts_with('/') {
            return Err(format!(
                "HTTP path template '{template}' must start with '/'"
            ));
        }

        // First validate brace balance and find any top-level `:` custom verb outside `{...}`.
        let mut brace_depth = 0usize;
        let mut top_level_colon_idx: Option<usize> = None;

        for (idx, ch) in template.char_indices() {
            match ch {
                '{' => {
                    if brace_depth > 0 {
                        return Err(format!(
                            "nested '{{' in HTTP path template '{template}' at byte {idx}"
                        ));
                    }
                    brace_depth += 1;
                }
                '}' => {
                    if brace_depth == 0 {
                        return Err(format!(
                            "unmatched '}}' in HTTP path template '{template}' at byte {idx}"
                        ));
                    }
                    brace_depth -= 1;
                }
                '?' | '#' => {
                    return Err(format!(
                        "invalid character '{ch}' in HTTP path template '{template}'"
                    ));
                }
                ':' if brace_depth == 0 => {
                    if top_level_colon_idx.is_some() {
                        return Err(format!(
                            "multiple top-level ':' custom verbs in HTTP path template '{template}'"
                        ));
                    }
                    top_level_colon_idx = Some(idx);
                }
                _ if ch.is_whitespace() => {
                    return Err(format!(
                        "whitespace is not allowed in HTTP path template '{template}'"
                    ));
                }
                _ => {}
            }
        }

        if brace_depth != 0 {
            return Err(format!("unclosed '{{' in HTTP path template '{template}'"));
        }

        let (path_part, verb) = if let Some(colon_idx) = top_level_colon_idx {
            let before = &template[..colon_idx];
            let after = &template[colon_idx + 1..];
            if after.is_empty() {
                return Err(format!(
                    "empty custom verb after ':' in HTTP path template '{template}'"
                ));
            }
            if after.contains('/') || after.contains('{') || after.contains('}') {
                return Err(format!(
                    "custom verb ':{after}' must appear at the very end of HTTP path template '{template}'"
                ));
            }
            if before.ends_with('/') {
                return Err(format!(
                    "custom verb ':{after}' cannot immediately follow '/' in HTTP path template '{template}'"
                ));
            }
            validate_literal_chars(after, template)?;
            (before, Some(after.to_string()))
        } else {
            (template, None)
        };

        let mut segments = Vec::new();
        if path_part != "/" {
            let without_leading_slash = &path_part[1..];
            if without_leading_slash.ends_with('/') {
                return Err(format!(
                    "HTTP path template '{template}' cannot have a trailing '/'"
                ));
            }

            let raw_segments = split_top_level_slashes(without_leading_slash);
            for raw_seg in raw_segments {
                if raw_seg.is_empty() {
                    return Err(format!(
                        "empty path segment in HTTP path template '{template}'"
                    ));
                }
                segments.push(parse_top_level_segment(raw_seg, template)?);
            }
        }

        // Enforce that `**` may only appear as the very last segment of the entire path template
        // and that no variable field_path is bound more than once.
        let mut seen_vars = BTreeSet::new();
        let num_segments = segments.len();
        for (seg_idx, seg) in segments.iter().enumerate() {
            let is_last_top_seg = seg_idx + 1 == num_segments;
            match seg {
                PathSegment::MultiWildcard if !is_last_top_seg => {
                    return Err(format!(
                        "unbounded wildcard '**' may only appear as the last segment in HTTP path template '{template}'"
                    ));
                }
                PathSegment::Variable(var) => {
                    let dotted = var.dotted_path();
                    if !seen_vars.insert(dotted.clone()) {
                        return Err(format!(
                            "duplicate variable binding '{{{dotted}}}' in HTTP path template '{template}'"
                        ));
                    }
                    let num_subs = var.sub_template.len();
                    for (sub_idx, sub) in var.sub_template.iter().enumerate() {
                        let is_last_sub = is_last_top_seg && (sub_idx + 1 == num_subs);
                        if matches!(sub, SubSegment::MultiWildcard) && !is_last_sub {
                            return Err(format!(
                                "unbounded wildcard '**' in variable '{{{dotted}}}' may only appear as the last segment in HTTP path template '{template}'"
                            ));
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            raw: template.to_string(),
            segments,
            verb,
        })
    }

    /// Iterates over all bound [`PathVariable`]s in the path template.
    pub fn variables(&self) -> impl Iterator<Item = &PathVariable> {
        self.segments.iter().filter_map(|seg| match seg {
            PathSegment::Variable(v) => Some(v),
            _ => None,
        })
    }

    /// Returns `true` if the path template binds at least one variable.
    #[inline]
    pub fn has_variables(&self) -> bool {
        self.variables().next().is_some()
    }
}

fn split_top_level_slashes(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;

    for (idx, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            '/' if depth == 0 => {
                parts.push(&s[start..idx]);
                start = idx + 1;
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

fn parse_top_level_segment(raw_seg: &str, full_template: &str) -> Result<PathSegment, String> {
    if raw_seg.starts_with('{') || raw_seg.ends_with('}') {
        if !(raw_seg.starts_with('{') && raw_seg.ends_with('}')) {
            return Err(format!(
                "variable binding '{raw_seg}' must span the entire path segment in HTTP path template '{full_template}'"
            ));
        }
        let inner = &raw_seg[1..raw_seg.len() - 1];
        if inner.contains('{') || inner.contains('}') {
            return Err(format!(
                "invalid braces in variable segment '{raw_seg}' in HTTP path template '{full_template}'"
            ));
        }
        let (raw_field_path, raw_sub_template) = match inner.split_once('=') {
            Some((f, sub)) => (f, Some(sub)),
            None => (inner, None),
        };
        if raw_field_path.is_empty() {
            return Err(format!(
                "empty field path in variable '{raw_seg}' in HTTP path template '{full_template}'"
            ));
        }

        let mut field_path = Vec::new();
        for ident in raw_field_path.split('.') {
            if !is_valid_proto_ident(ident) {
                return Err(format!(
                    "invalid field identifier '{ident}' in variable '{raw_seg}' in HTTP path template '{full_template}'"
                ));
            }
            field_path.push(ident.to_string());
        }

        let sub_template = match raw_sub_template {
            None => vec![SubSegment::SingleWildcard],
            Some(sub_str) => {
                if sub_str.is_empty() {
                    return Err(format!(
                        "empty sub-template after '=' in variable '{raw_seg}' in HTTP path template '{full_template}'"
                    ));
                }
                if sub_str.starts_with('/') || sub_str.ends_with('/') {
                    return Err(format!(
                        "sub-template '{sub_str}' cannot start or end with '/' in HTTP path template '{full_template}'"
                    ));
                }
                let mut subs = Vec::new();
                for part in sub_str.split('/') {
                    if part.is_empty() {
                        return Err(format!(
                            "empty segment in sub-template '{sub_str}' in HTTP path template '{full_template}'"
                        ));
                    }
                    if part.contains('=') || part.contains(':') {
                        return Err(format!(
                            "invalid character in sub-template segment '{part}' in HTTP path template '{full_template}'"
                        ));
                    }
                    match part {
                        "*" => subs.push(SubSegment::SingleWildcard),
                        "**" => subs.push(SubSegment::MultiWildcard),
                        lit => {
                            validate_literal_chars(lit, full_template)?;
                            subs.push(SubSegment::Literal(lit.to_string()));
                        }
                    }
                }
                subs
            }
        };

        return Ok(PathSegment::Variable(PathVariable {
            field_path,
            sub_template,
        }));
    }

    match raw_seg {
        "*" => Ok(PathSegment::SingleWildcard),
        "**" => Ok(PathSegment::MultiWildcard),
        lit => {
            validate_literal_chars(lit, full_template)?;
            Ok(PathSegment::Literal(lit.to_string()))
        }
    }
}

fn is_valid_proto_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn validate_literal_chars(lit: &str, full_template: &str) -> Result<(), String> {
    if lit.is_empty() {
        return Err(format!(
            "empty literal in HTTP path template '{full_template}'"
        ));
    }
    if lit
        .chars()
        .any(|c| matches!(c, '{' | '}' | '*' | '=' | '?' | '#' | ':') || c.is_whitespace())
    {
        return Err(format!(
            "invalid literal segment '{lit}' in HTTP path template '{full_template}'"
        ));
    }
    Ok(())
}

/// Metadata for a single resolved Protobuf field step in a path variable, `body`, or `response_body`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedFieldStep {
    /// Proto field name (e.g. `"user"` or `"id"`).
    pub name: String,
    /// Protobuf field tag number (`1..=536_870_911`).
    pub tag: u32,
    /// Protobuf field type (`TypeInt32`, `TypeString`, `TypeMessage`, `TypeEnum`, etc.).
    pub proto_type: FieldType,
    /// Wire type used when encoding an unpacked single occurrence of this field.
    pub wire_type: WireType,
    /// Whether the field has `LABEL_REPEATED`.
    pub is_repeated: bool,
    /// Canonical fully-qualified Protobuf type name (e.g. `".acme.v1.User"`) for message/enum fields,
    /// or `""` for primitive scalar/string/bytes fields.
    pub type_name: String,
}

/// A path variable validated against the RPC input message descriptor hierarchy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPathVariable {
    /// Parsed path variable (`field_path` and `sub_template`).
    pub variable: PathVariable,
    /// Resolved field chain from the top-level input message down to the leaf field (non-empty).
    pub steps: Vec<ResolvedFieldStep>,
}

impl ValidatedPathVariable {
    /// Returns the terminal leaf field step of this path variable.
    #[inline]
    pub fn leaf(&self) -> &ResolvedFieldStep {
        self.steps
            .last()
            .expect("ValidatedPathVariable always has at least one step")
    }
}

/// Validated `body` mapping for a `google.api.HttpRule`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidatedBody {
    /// `body: ""` — no HTTP request body is mapped.
    Empty,
    /// `body: "*"` — the entire HTTP request body maps to the RPC input message.
    Wildcard,
    /// `body: "<field>"` — the HTTP request body maps to a specific top-level field of the input message.
    Field(ResolvedFieldStep),
}

/// Validated `response_body` mapping for a `google.api.HttpRule`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValidatedResponseBody {
    /// `response_body: ""` — the entire RPC output message is serialized as the HTTP response body.
    Empty,
    /// `response_body: "<field>"` — only the specified top-level field of the output message is returned.
    Field(ResolvedFieldStep),
}

/// A single validated HTTP binding (either the primary `HttpRule` or an entry in `additional_bindings`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedHttpBinding {
    /// Selector string from `HttpRule`.
    pub selector: String,
    /// Canonical fully-qualified RPC input message name (e.g. `".acme.v1.GetShelfRequest"`).
    pub input_fq: String,
    /// Canonical fully-qualified RPC output message name (e.g. `".acme.v1.Shelf"`).
    pub output_fq: String,
    /// HTTP method and raw path pattern (`GET`, `PUT`, `POST`, `DELETE`, `PATCH`, or `Custom`).
    pub pattern: HttpPattern,
    /// Parsed URL path template.
    pub path_template: ParsedPathTemplate,
    /// Validated path variables in order of appearance in `path_template`.
    pub path_variables: Vec<ValidatedPathVariable>,
    /// Validated `body` specification (`""`, `"*"`, or top-level input field).
    pub body: ValidatedBody,
    /// Validated `response_body` specification (`""` or top-level output field).
    pub response_body: ValidatedResponseBody,
}

/// Validated `google.api.HttpRule` (including all `additional_bindings`) for an RPC method.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedHttpRule {
    /// Primary HTTP binding.
    pub primary: ValidatedHttpBinding,
    /// Validated `additional_bindings`.
    pub additional_bindings: Vec<ValidatedHttpBinding>,
}

impl ValidatedHttpRule {
    /// Iterates over the primary binding followed by all `additional_bindings`.
    pub fn all_bindings(&self) -> impl Iterator<Item = &ValidatedHttpBinding> {
        std::iter::once(&self.primary).chain(self.additional_bindings.iter())
    }
}

#[derive(Clone, Copy, Debug)]
struct IndexedMessage<'a> {
    descriptor: DescriptorProtoView<'a>,
    is_map_entry: bool,
}

/// Lookup table indexing all message and enum descriptors in a `CodeGeneratorRequestView`.
#[derive(Clone, Debug, Default)]
pub struct MessageIndex<'a> {
    messages: BTreeMap<String, IndexedMessage<'a>>,
    enums: BTreeMap<String, Vec<(String, i32)>>,
}

impl<'a> MessageIndex<'a> {
    /// Creates an empty [`MessageIndex`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Indexes all `FileDescriptorProtoView` entries in `request.proto_file()`.
    pub fn build(request: &CodeGeneratorRequestView<'a>) -> Result<Self, String> {
        Self::from_request(request)
    }

    /// Indexes all `FileDescriptorProtoView` entries in `request.proto_file()`.
    pub fn from_request(request: &CodeGeneratorRequestView<'a>) -> Result<Self, String> {
        let mut index = Self::new();
        for file_res in request.proto_file() {
            let file =
                file_res.map_err(|e| format!("invalid FileDescriptorProto in proto_file: {e}"))?;
            index.insert_file(&file)?;
        }
        Ok(index)
    }

    /// Indexes an iterator of `FileDescriptorProtoView` descriptors.
    pub fn from_files<I>(files: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = FileDescriptorProtoView<'a>>,
    {
        let mut index = Self::new();
        for file in files {
            index.insert_file(&file)?;
        }
        Ok(index)
    }

    /// Returns the `(variant_name, variant_number)` pairs for a canonical enum type name (e.g. `".acme.v1.ShelfTheme"`).
    pub fn enum_variants(&self, fq_enum_name: &str) -> Option<&[(String, i32)]> {
        self.enums.get(fq_enum_name).map(Vec::as_slice)
    }

    /// Indexes all messages and enums in `file`.
    pub fn insert_file(&mut self, file: &FileDescriptorProtoView<'a>) -> Result<(), String> {
        let file_name = file
            .name()
            .map_err(|e| format!("invalid FileDescriptorProto.name: {e}"))?;
        let pkg = file
            .package()
            .map_err(|e| format!("invalid FileDescriptorProto.package in '{file_name}': {e}"))?
            .trim()
            .trim_matches('.');
        let base_prefix = if pkg.is_empty() {
            String::new()
        } else {
            format!(".{pkg}")
        };

        for en_res in file.enum_type() {
            let en =
                en_res.map_err(|e| format!("invalid EnumDescriptorProto in '{file_name}': {e}"))?;
            self.insert_enum(&en, &base_prefix, file_name)?;
        }

        for msg_res in file.message_type() {
            let msg =
                msg_res.map_err(|e| format!("invalid DescriptorProto in '{file_name}': {e}"))?;
            self.insert_message(&msg, &base_prefix, file_name)?;
        }

        Ok(())
    }

    fn insert_enum(
        &mut self,
        en: &google_protobuf_roto_bootstrap::google::protobuf::EnumDescriptorProtoView<'a>,
        parent_prefix: &str,
        file_name: &str,
    ) -> Result<(), String> {
        let en_name = en
            .name()
            .map_err(|e| format!("invalid EnumDescriptorProto.name in '{file_name}': {e}"))?;
        let fq_enum = format!("{parent_prefix}.{en_name}");
        let mut variants = Vec::new();
        for val_res in en.value() {
            let val = val_res
                .map_err(|e| format!("invalid EnumValueDescriptorProto in '{fq_enum}': {e}"))?;
            let v_name = val
                .name()
                .map_err(|e| format!("invalid EnumValueDescriptorProto.name in '{fq_enum}': {e}"))?
                .to_string();
            let v_num = val.number().map_err(|e| {
                format!("invalid EnumValueDescriptorProto.number in '{fq_enum}.{v_name}': {e}")
            })?;
            variants.push((v_name, v_num));
        }
        self.enums.insert(fq_enum, variants);
        Ok(())
    }

    fn insert_message(
        &mut self,
        msg: &DescriptorProtoView<'a>,
        parent_prefix: &str,
        file_name: &str,
    ) -> Result<(), String> {
        let name = msg
            .name()
            .map_err(|e| format!("invalid DescriptorProto.name in '{file_name}': {e}"))?;
        let fq_name = format!("{parent_prefix}.{name}");
        let is_map_entry = match msg
            .options()
            .map_err(|e| format!("invalid MessageOptions in '{fq_name}': {e}"))?
        {
            Some(opts) => opts
                .map_entry()
                .map_err(|e| format!("invalid MessageOptions.map_entry in '{fq_name}': {e}"))?,
            None => false,
        };

        self.messages.insert(
            fq_name.clone(),
            IndexedMessage {
                descriptor: *msg,
                is_map_entry,
            },
        );

        for en_res in msg.enum_type() {
            let en = en_res
                .map_err(|e| format!("invalid nested EnumDescriptorProto in '{fq_name}': {e}"))?;
            self.insert_enum(&en, &fq_name, file_name)?;
        }

        for nested_res in msg.nested_type() {
            let nested = nested_res
                .map_err(|e| format!("invalid nested DescriptorProto in '{fq_name}': {e}"))?;
            self.insert_message(&nested, &fq_name, file_name)?;
        }

        Ok(())
    }

    /// Resolves a relative or fully-qualified message `type_name` from `current_scope`
    /// to its canonical `.pkg.Msg` name and [`DescriptorProtoView`].
    pub fn resolve_message(
        &self,
        current_scope: &str,
        type_name: &str,
    ) -> Option<(String, DescriptorProtoView<'a>, bool)> {
        let fq = self.resolve_type_name(current_scope, type_name)?;
        let indexed = self.messages.get(&fq)?;
        Some((fq, indexed.descriptor, indexed.is_map_entry))
    }

    /// Resolves a relative or fully-qualified message or enum `type_name` from `current_scope`
    /// to its canonical `.pkg.Type` name.
    pub fn resolve_type_name(&self, current_scope: &str, type_name: &str) -> Option<String> {
        let trimmed = type_name.trim();
        if trimmed.is_empty() {
            return None;
        }
        if trimmed.starts_with('.') {
            if self.messages.contains_key(trimmed) || self.enums.contains_key(trimmed) {
                return Some(trimmed.to_string());
            }
            return None;
        }

        let mut scope = current_scope.trim().trim_matches('.').to_string();
        loop {
            let candidate = if scope.is_empty() {
                format!(".{trimmed}")
            } else {
                format!(".{scope}.{trimmed}")
            };
            if self.messages.contains_key(&candidate) || self.enums.contains_key(&candidate) {
                return Some(candidate);
            }
            if scope.is_empty() {
                break;
            }
            match scope.rfind('.') {
                Some(idx) => scope.truncate(idx),
                None => scope.clear(),
            }
        }
        None
    }

    /// Resolves all top-level fields of the message identified by `fq_message_name`.
    pub fn top_level_fields(
        &self,
        fq_message_name: &str,
    ) -> Result<Vec<ResolvedFieldStep>, String> {
        let indexed = self
            .messages
            .get(fq_message_name)
            .ok_or_else(|| format!("unknown message descriptor '{fq_message_name}'"))?;
        let mut fields = Vec::new();
        for field_res in indexed.descriptor.field() {
            let field = field_res
                .map_err(|e| format!("invalid FieldDescriptorProto in '{fq_message_name}': {e}"))?;
            fields.push(self.build_field_step(fq_message_name, &field)?);
        }
        Ok(fields)
    }

    /// Returns all leaf field step chains reachable from `fq_message_name` that can be bound via
    /// HTTP query parameters (singular or repeated scalar/enum/string/bytes leaves, including
    /// those nested inside non-repeated, non-map submessages).
    pub fn query_bindable_fields(
        &self,
        fq_message_name: &str,
    ) -> Result<Vec<Vec<ResolvedFieldStep>>, String> {
        let mut out = Vec::new();
        let mut visited = BTreeSet::new();
        visited.insert(fq_message_name.to_string());
        let mut prefix = Vec::new();
        self.collect_query_fields(fq_message_name, &mut prefix, &mut visited, &mut out)?;
        Ok(out)
    }

    fn collect_query_fields(
        &self,
        curr_msg_fq: &str,
        prefix: &mut Vec<ResolvedFieldStep>,
        visited: &mut BTreeSet<String>,
        out: &mut Vec<Vec<ResolvedFieldStep>>,
    ) -> Result<(), String> {
        for step in self.top_level_fields(curr_msg_fq)? {
            if is_valid_path_leaf_type(step.proto_type) {
                let mut chain = prefix.clone();
                chain.push(step);
                out.push(chain);
            } else if step.proto_type == FieldType::TypeMessage && !step.is_repeated {
                if let Some((next_fq, _, is_map_entry)) =
                    self.resolve_message(curr_msg_fq, &step.type_name)
                {
                    if !is_map_entry && visited.insert(next_fq.clone()) {
                        prefix.push(step);
                        self.collect_query_fields(&next_fq, prefix, visited, out)?;
                        prefix.pop();
                        visited.remove(&next_fq);
                    }
                }
            }
        }
        Ok(())
    }

    fn build_field_step(
        &self,
        enclosing_msg_fq: &str,
        field: &google_protobuf_roto_bootstrap::google::protobuf::FieldDescriptorProtoView<'_>,
    ) -> Result<ResolvedFieldStep, String> {
        let name = field
            .name()
            .map_err(|e| format!("invalid FieldDescriptorProto.name in '{enclosing_msg_fq}': {e}"))?
            .to_string();
        let number = field.number().map_err(|e| {
            format!("invalid FieldDescriptorProto.number for '{enclosing_msg_fq}.{name}': {e}")
        })?;
        if number <= 0 {
            return Err(format!(
                "invalid field tag number {number} on '{enclosing_msg_fq}.{name}'"
            ));
        }
        let label = field.label().map_err(|e| {
            format!("invalid FieldDescriptorProto.label for '{enclosing_msg_fq}.{name}': {e}")
        })?;
        let proto_type = field.r#type().map_err(|e| {
            format!("invalid FieldDescriptorProto.type for '{enclosing_msg_fq}.{name}': {e}")
        })?;
        let raw_type_name = field.type_name().map_err(|e| {
            format!("invalid FieldDescriptorProto.type_name for '{enclosing_msg_fq}.{name}': {e}")
        })?;

        let type_name = if matches!(
            proto_type,
            FieldType::TypeMessage | FieldType::TypeEnum | FieldType::TypeGroup
        ) || !raw_type_name.is_empty()
        {
            self.resolve_type_name(enclosing_msg_fq, raw_type_name)
                .unwrap_or_else(|| {
                    if raw_type_name.starts_with('.') {
                        raw_type_name.to_string()
                    } else {
                        format!(".{raw_type_name}")
                    }
                })
        } else {
            String::new()
        };

        Ok(ResolvedFieldStep {
            name,
            tag: number as u32,
            proto_type,
            wire_type: wire_type_for_field_type(proto_type),
            is_repeated: label == FieldLabel::LabelRepeated,
            type_name,
        })
    }
}

/// Returns the unpacked Protobuf [`WireType`] corresponding to `field_type`.
pub const fn wire_type_for_field_type(field_type: FieldType) -> WireType {
    match field_type {
        FieldType::TypeInt32
        | FieldType::TypeInt64
        | FieldType::TypeUint32
        | FieldType::TypeUint64
        | FieldType::TypeSint32
        | FieldType::TypeSint64
        | FieldType::TypeBool
        | FieldType::TypeEnum => WireType::Varint,
        FieldType::TypeDouble | FieldType::TypeFixed64 | FieldType::TypeSfixed64 => {
            WireType::Fixed64
        }
        FieldType::TypeFloat | FieldType::TypeFixed32 | FieldType::TypeSfixed32 => {
            WireType::Fixed32
        }
        FieldType::TypeString | FieldType::TypeBytes | FieldType::TypeMessage => {
            WireType::LengthDelimited
        }
        FieldType::TypeGroup => WireType::StartGroup,
    }
}

/// Returns `true` if `field_type` can be bound by a URL path variable leaf (scalar, string, bytes, or enum).
pub const fn is_valid_path_leaf_type(field_type: FieldType) -> bool {
    !matches!(field_type, FieldType::TypeMessage | FieldType::TypeGroup)
}

/// Extracts and validates `(google.api.http)` on `method` (if present) against `index`.
pub fn extract_and_validate_method_http_rule(
    method: &MethodDescriptorProtoView<'_>,
    proto_scope: &str,
    index: &MessageIndex<'_>,
) -> Result<Option<ValidatedHttpRule>, String> {
    let Some(rule) = extract_method_http_rule(method)? else {
        return Ok(None);
    };
    validate_http_rule_for_method(&rule, method, proto_scope, index).map(Some)
}

/// Validates a decoded [`HttpRule`] against an RPC [`MethodDescriptorProtoView`] and [`MessageIndex`].
pub fn validate_http_rule_for_method(
    rule: &HttpRule,
    method: &MethodDescriptorProtoView<'_>,
    proto_scope: &str,
    index: &MessageIndex<'_>,
) -> Result<ValidatedHttpRule, String> {
    let method_name = method
        .name()
        .map_err(|e| format!("invalid MethodDescriptorProto.name: {e}"))?;

    let client_streaming = method.client_streaming().map_err(|e| {
        format!("invalid MethodDescriptorProto.client_streaming in '{method_name}': {e}")
    })?;
    let server_streaming = method.server_streaming().map_err(|e| {
        format!("invalid MethodDescriptorProto.server_streaming in '{method_name}': {e}")
    })?;
    if client_streaming || server_streaming {
        return Err(format!(
            "google.api.http annotation is not supported on streaming method '{method_name}'"
        ));
    }

    let raw_input_type = method
        .input_type()
        .map_err(|e| format!("invalid MethodDescriptorProto.input_type in '{method_name}': {e}"))?;
    let (input_fq, input_desc, _) = index
        .resolve_message(proto_scope, raw_input_type)
        .ok_or_else(|| {
            format!("unresolved input message type '{raw_input_type}' for method '{method_name}'")
        })?;

    let raw_output_type = method.output_type().map_err(|e| {
        format!("invalid MethodDescriptorProto.output_type in '{method_name}': {e}")
    })?;
    let (output_fq, output_desc, _) = index
        .resolve_message(proto_scope, raw_output_type)
        .ok_or_else(|| {
            format!("unresolved output message type '{raw_output_type}' for method '{method_name}'")
        })?;

    let primary = validate_single_binding(
        rule,
        method_name,
        &input_fq,
        &input_desc,
        &output_fq,
        &output_desc,
        index,
    )?;

    let mut additional_bindings = Vec::with_capacity(rule.additional_bindings.len());
    for (idx, binding) in rule.additional_bindings.iter().enumerate() {
        if !binding.additional_bindings.is_empty() {
            return Err(format!(
                "google.api.http additional_bindings[{idx}] on method '{method_name}' cannot contain nested additional_bindings"
            ));
        }
        let validated = validate_single_binding(
            binding,
            method_name,
            &input_fq,
            &input_desc,
            &output_fq,
            &output_desc,
            index,
        )?;
        additional_bindings.push(validated);
    }

    Ok(ValidatedHttpRule {
        primary,
        additional_bindings,
    })
}

fn validate_single_binding(
    rule: &HttpRule,
    method_name: &str,
    input_fq: &str,
    input_desc: &DescriptorProtoView<'_>,
    output_fq: &str,
    output_desc: &DescriptorProtoView<'_>,
    index: &MessageIndex<'_>,
) -> Result<ValidatedHttpBinding, String> {
    let pattern = rule.pattern.clone().ok_or_else(|| {
        format!(
            "google.api.http rule on method '{method_name}' is missing an HTTP pattern (get, put, post, delete, patch, or custom)"
        )
    })?;

    if let HttpPattern::Custom { ref kind, .. } = pattern {
        if kind.trim().is_empty() {
            return Err(format!(
                "google.api.http CustomHttpPattern.kind on method '{method_name}' cannot be empty"
            ));
        }
    }

    let path_template = ParsedPathTemplate::parse(pattern.path()).map_err(|e| {
        format!("invalid path template in google.api.http on method '{method_name}': {e}")
    })?;

    let mut path_variables = Vec::new();
    for var in path_template.variables() {
        let steps =
            resolve_path_variable_steps(&var.field_path, method_name, input_fq, input_desc, index)?;
        path_variables.push(ValidatedPathVariable {
            variable: var.clone(),
            steps,
        });
    }

    let body = match rule.body.as_str() {
        "" => ValidatedBody::Empty,
        "*" => ValidatedBody::Wildcard,
        body_field => {
            if path_variables.iter().any(|pv| {
                pv.variable.field_path.len() == 1 && pv.variable.field_path[0] == body_field
            }) {
                return Err(format!(
                    "google.api.http body field '{body_field}' on method '{method_name}' overlaps with bound path variable '{{{body_field}}}'"
                ));
            }
            let step = find_top_level_field(input_fq, input_desc, body_field, index).ok_or_else(|| {
                format!(
                    "google.api.http body field '{body_field}' on method '{method_name}' does not exist in input message '{input_fq}'"
                )
            })?;
            ValidatedBody::Field(step)
        }
    };

    let response_body = match rule.response_body.as_str() {
        "" => ValidatedResponseBody::Empty,
        resp_field => {
            let step = find_top_level_field(output_fq, output_desc, resp_field, index).ok_or_else(|| {
                format!(
                    "google.api.http response_body field '{resp_field}' on method '{method_name}' does not exist in output message '{output_fq}'"
                )
            })?;
            ValidatedResponseBody::Field(step)
        }
    };

    Ok(ValidatedHttpBinding {
        selector: rule.selector.clone(),
        input_fq: input_fq.to_string(),
        output_fq: output_fq.to_string(),
        pattern,
        path_template,
        path_variables,
        body,
        response_body,
    })
}

fn find_top_level_field(
    msg_fq: &str,
    msg_desc: &DescriptorProtoView<'_>,
    field_name: &str,
    index: &MessageIndex<'_>,
) -> Option<ResolvedFieldStep> {
    for field_res in msg_desc.field() {
        let field = field_res.ok()?;
        if field.name().ok()? == field_name {
            return index.build_field_step(msg_fq, &field).ok();
        }
    }
    None
}

fn resolve_path_variable_steps(
    field_path: &[String],
    method_name: &str,
    root_msg_fq: &str,
    root_msg_desc: &DescriptorProtoView<'_>,
    index: &MessageIndex<'_>,
) -> Result<Vec<ResolvedFieldStep>, String> {
    let dotted = field_path.join(".");
    let mut curr_msg_fq = root_msg_fq.to_string();
    let mut curr_msg_desc = *root_msg_desc;
    let mut steps = Vec::with_capacity(field_path.len());

    for (idx, seg_name) in field_path.iter().enumerate() {
        let is_leaf = idx + 1 == field_path.len();
        let step = find_top_level_field(&curr_msg_fq, &curr_msg_desc, seg_name, index)
            .ok_or_else(|| {
                format!(
                    "path variable '{{{dotted}}}' on method '{method_name}' references unknown field '{seg_name}' on message '{curr_msg_fq}'"
                )
            })?;

        if step.is_repeated {
            return Err(format!(
                "path variable '{{{dotted}}}' on method '{method_name}' cannot bind repeated field '{curr_msg_fq}.{seg_name}'"
            ));
        }

        if is_leaf {
            if !is_valid_path_leaf_type(step.proto_type) {
                return Err(format!(
                    "path variable '{{{dotted}}}' on method '{method_name}' leaf field '{curr_msg_fq}.{seg_name}' must be a scalar, string, bytes, or enum, got {:?}",
                    step.proto_type
                ));
            }
            steps.push(step);
        } else {
            if step.proto_type != FieldType::TypeMessage {
                return Err(format!(
                    "path variable '{{{dotted}}}' on method '{method_name}' intermediate field '{curr_msg_fq}.{seg_name}' must be a message, got {:?}",
                    step.proto_type
                ));
            }
            let (next_fq, next_desc, is_map_entry) = index
                .resolve_message(&curr_msg_fq, &step.type_name)
                .ok_or_else(|| {
                    format!(
                        "path variable '{{{dotted}}}' on method '{method_name}' intermediate field '{curr_msg_fq}.{seg_name}' has unresolved message type '{}'",
                        step.type_name
                    )
                })?;
            if is_map_entry {
                return Err(format!(
                    "path variable '{{{dotted}}}' on method '{method_name}' intermediate field '{curr_msg_fq}.{seg_name}' cannot traverse map field"
                ));
            }
            steps.push(step);
            curr_msg_fq = next_fq;
            curr_msg_desc = next_desc;
        }
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::process::Command;

    use google_protobuf_roto_bootstrap::google::protobuf::FileDescriptorSetView;
    use roto_runtime::BackwardWriter;

    fn encode_len_delimited_field(writer: &mut BackwardWriter<'_>, tag: u32, payload: &[u8]) {
        writer.write_bytes(payload).unwrap();
        writer.write_varint(payload.len() as u64).unwrap();
        writer.write_tag(tag, WireType::LengthDelimited).unwrap();
    }

    fn encode_string_field(writer: &mut BackwardWriter<'_>, tag: u32, val: &str) {
        encode_len_delimited_field(writer, tag, val.as_bytes());
    }

    fn build_http_rule_bytes(
        selector: &str,
        pattern: Option<&HttpPattern>,
        body: &str,
        response_body: &str,
        additional: &[Vec<u8>],
    ) -> Vec<u8> {
        let mut buf = vec![0u8; 4096];
        let mut w = BackwardWriter::new(&mut buf);

        for add_bytes in additional.iter().rev() {
            encode_len_delimited_field(&mut w, HTTP_RULE_ADDITIONAL_BINDINGS_TAG, add_bytes);
        }
        if !response_body.is_empty() {
            encode_string_field(&mut w, HTTP_RULE_RESPONSE_BODY_TAG, response_body);
        }
        if !body.is_empty() {
            encode_string_field(&mut w, HTTP_RULE_BODY_TAG, body);
        }
        if let Some(pat) = pattern {
            match pat {
                HttpPattern::Get(p) => encode_string_field(&mut w, HTTP_RULE_GET_TAG, p),
                HttpPattern::Put(p) => encode_string_field(&mut w, HTTP_RULE_PUT_TAG, p),
                HttpPattern::Post(p) => encode_string_field(&mut w, HTTP_RULE_POST_TAG, p),
                HttpPattern::Delete(p) => encode_string_field(&mut w, HTTP_RULE_DELETE_TAG, p),
                HttpPattern::Patch(p) => encode_string_field(&mut w, HTTP_RULE_PATCH_TAG, p),
                HttpPattern::Custom { kind, path } => {
                    let mut cbuf = [0u8; 512];
                    let mut cw = BackwardWriter::new(&mut cbuf);
                    encode_string_field(&mut cw, CUSTOM_HTTP_PATTERN_PATH_TAG, path);
                    encode_string_field(&mut cw, CUSTOM_HTTP_PATTERN_KIND_TAG, kind);
                    let cslice = cw.finish().unwrap();
                    encode_len_delimited_field(&mut w, HTTP_RULE_CUSTOM_TAG, cslice);
                }
            }
        }
        if !selector.is_empty() {
            encode_string_field(&mut w, HTTP_RULE_SELECTOR_TAG, selector);
        }

        w.finish().unwrap().to_vec()
    }

    fn wrap_in_method_options(http_rule_bytes: &[u8]) -> Vec<u8> {
        let mut buf = vec![0u8; 4096];
        let mut w = BackwardWriter::new(&mut buf);
        encode_len_delimited_field(&mut w, GOOGLE_API_HTTP_TAG, http_rule_bytes);
        // Also write standard MethodOptions.deprecated = false (tag 33) before it
        w.write_varint(0).unwrap();
        w.write_tag(33, WireType::Varint).unwrap();
        w.finish().unwrap().to_vec()
    }

    #[test]
    fn extracts_all_http_rule_verbs_custom_body_response_body_and_additional_bindings() {
        let verbs = [
            HttpPattern::Get("/v1/items/{id}".to_string()),
            HttpPattern::Put("/v1/items/{id}".to_string()),
            HttpPattern::Post("/v1/items".to_string()),
            HttpPattern::Delete("/v1/items/{id}".to_string()),
            HttpPattern::Patch("/v1/items/{id}".to_string()),
            HttpPattern::Custom {
                kind: "LOCK".to_string(),
                path: "/v1/items/{id}:lock".to_string(),
            },
        ];

        for pat in &verbs {
            let add1 = build_http_rule_bytes(
                "",
                Some(&HttpPattern::Get("/v2/items/{id}".to_string())),
                "",
                "item",
                &[],
            );
            let rule_bytes = build_http_rule_bytes(
                "acme.v1.ItemService.Mutate",
                Some(pat),
                "*",
                "result",
                &[add1],
            );
            let opts_bytes = wrap_in_method_options(&rule_bytes);
            let opts_view = MethodOptionsView::from_bytes(&opts_bytes).unwrap();

            let extracted = extract_http_rule_from_options(&opts_view)
                .expect("should decode HttpRule")
                .expect("HttpRule should be present");

            assert_eq!(extracted.selector, "acme.v1.ItemService.Mutate");
            assert_eq!(extracted.pattern.as_ref(), Some(pat));
            assert_eq!(extracted.body, "*");
            assert_eq!(extracted.response_body, "result");
            assert_eq!(extracted.additional_bindings.len(), 1);
            assert_eq!(
                extracted.additional_bindings[0].pattern,
                Some(HttpPattern::Get("/v2/items/{id}".to_string()))
            );
            assert_eq!(extracted.additional_bindings[0].response_body, "item");
        }
    }

    #[test]
    fn rejects_nested_additional_bindings_and_wrong_wire_types() {
        let leaf_binding = build_http_rule_bytes(
            "",
            Some(&HttpPattern::Get("/v3/items".to_string())),
            "",
            "",
            &[],
        );
        let nested_binding = build_http_rule_bytes(
            "",
            Some(&HttpPattern::Get("/v2/items".to_string())),
            "",
            "",
            &[leaf_binding],
        );
        let root_rule = build_http_rule_bytes(
            "",
            Some(&HttpPattern::Get("/v1/items".to_string())),
            "",
            "",
            &[nested_binding],
        );
        let opts_bytes = wrap_in_method_options(&root_rule);

        let err = extract_http_rule_from_options_bytes(&opts_bytes).unwrap_err();
        assert!(
            err.contains("additional_bindings cannot contain nested additional_bindings"),
            "unexpected error: {err}"
        );

        // Wrong wire type on GOOGLE_API_HTTP_TAG
        let mut bad_buf = [0u8; 64];
        let mut w = BackwardWriter::new(&mut bad_buf);
        w.write_varint(123).unwrap();
        w.write_tag(GOOGLE_API_HTTP_TAG, WireType::Varint).unwrap();
        let bad_bytes = w.finish().unwrap();
        let err_wt = extract_http_rule_from_options_bytes(bad_bytes).unwrap_err();
        assert!(err_wt.contains("must have LengthDelimited wire type"));
    }

    #[test]
    fn parses_simple_nested_subtemplate_wildcard_and_custom_verb_path_templates() {
        // 1. Root path
        let root = ParsedPathTemplate::parse("/").unwrap();
        assert!(root.segments.is_empty());
        assert_eq!(root.verb, None);

        // 2. Nested field path + explicit `=*` + `:cancel` custom verb
        let tpl1 = ParsedPathTemplate::parse("/v1/users/{user.id}/messages/{message_id=*}:cancel")
            .unwrap();
        assert_eq!(tpl1.verb.as_deref(), Some("cancel"));
        assert_eq!(
            tpl1.segments,
            vec![
                PathSegment::Literal("v1".to_string()),
                PathSegment::Literal("users".to_string()),
                PathSegment::Variable(PathVariable {
                    field_path: vec!["user".to_string(), "id".to_string()],
                    sub_template: vec![SubSegment::SingleWildcard],
                }),
                PathSegment::Literal("messages".to_string()),
                PathSegment::Variable(PathVariable {
                    field_path: vec!["message_id".to_string()],
                    sub_template: vec![SubSegment::SingleWildcard],
                }),
            ]
        );

        // 3. Multi-segment sub-template with `*` and trailing `**`
        let tpl2 = ParsedPathTemplate::parse("/v1/{name=organizations/*/users/**}").unwrap();
        assert_eq!(tpl2.verb, None);
        assert_eq!(
            tpl2.segments,
            vec![
                PathSegment::Literal("v1".to_string()),
                PathSegment::Variable(PathVariable {
                    field_path: vec!["name".to_string()],
                    sub_template: vec![
                        SubSegment::Literal("organizations".to_string()),
                        SubSegment::SingleWildcard,
                        SubSegment::Literal("users".to_string()),
                        SubSegment::MultiWildcard,
                    ],
                }),
            ]
        );

        // 4. Deeply nested `{a.b.c}` and `{path=**}` with custom verb
        let tpl3 = ParsedPathTemplate::parse("/files/{meta.owner.id}/{path=**}:download").unwrap();
        assert_eq!(tpl3.verb.as_deref(), Some("download"));
        assert_eq!(tpl3.variables().count(), 2);
    }

    #[test]
    fn rejects_malformed_path_templates() {
        let invalid_cases = [
            ("", "must start with '/'"),
            ("v1/users", "must start with '/'"),
            ("/v1//users", "empty path segment"),
            ("/v1/users/", "cannot have a trailing '/'"),
            ("/v1/{unclosed", "unclosed '{'"),
            ("/v1/unmatched}", "unmatched '}'"),
            ("/v1/{nested={b}}", "nested '{'"),
            ("/v1/{}", "empty field path"),
            ("/v1/{123}", "invalid field identifier"),
            ("/v1/{a..b}", "invalid field identifier"),
            ("/v1/{id=}", "empty sub-template"),
            ("/v1/{id=/a}", "cannot start or end with '/'"),
            ("/v1/{path=**}/tail", "may only appear as the last segment"),
            ("/v1/{path=**/tail}", "may only appear as the last segment"),
            ("/v1/**/tail", "may only appear as the last segment"),
            ("/v1/{id}/items/{id}", "duplicate variable binding"),
            ("/v1/items:", "empty custom verb"),
            ("/v1/items:a:b", "multiple top-level ':'"),
            ("/v1/items:a/b", "must appear at the very end"),
            ("/v1/items/:cancel", "cannot immediately follow '/'"),
            ("/v1/prefix{id}", "must span the entire path segment"),
        ];

        for (raw, expected_err) in invalid_cases {
            let err = ParsedPathTemplate::parse(raw)
                .expect_err(&format!("expected '{raw}' to be rejected"));
            assert!(
                err.contains(expected_err),
                "for template '{raw}', expected error containing '{expected_err}', got: '{err}'"
            );
        }
    }

    #[test]
    fn compiles_bundled_google_api_annotations_with_protoc_and_validates_descriptors() {
        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.parent().expect("workspace root");
        let bundled_proto_dir = workspace_root.join("google-protobuf-roto/proto");

        let tmp_dir =
            std::env::temp_dir().join(format!("roto_http_annotation_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp_dir);
        std::fs::create_dir_all(&tmp_dir).unwrap();

        let service_proto = r#"syntax = "proto3";
package acme.shelf.v1;

import "google/api/annotations.proto";

enum ShelfTheme {
  SHELF_THEME_UNSPECIFIED = 0;
  SHELF_THEME_DARK = 1;
}

message Shelf {
  string id = 1;
  string title = 2;
  ShelfTheme theme = 3;
}

message Book {
  string id = 1;
  string author = 2;
}

message GetShelfRequest {
  Shelf shelf = 1;
  repeated string tags = 2;
  map<string, string> labels = 3;
}

message UpdateBookRequest {
  string shelf_id = 1;
  Book book = 2;
  string extra_note = 3;
}

message UpdateBookResponse {
  Book book = 1;
  string status = 2;
}

service ShelfService {
  rpc GetShelf(GetShelfRequest) returns (Shelf) {
    option (google.api.http) = {
      get: "/v1/shelves/{shelf.id}/themes/{shelf.theme}"
      additional_bindings {
        get: "/v1/{shelf.id=organizations/*/shelves/**}:inspect"
      }
    };
  }

  rpc UpdateBook(UpdateBookRequest) returns (UpdateBookResponse) {
    option (google.api.http) = {
      patch: "/v1/shelves/{shelf_id}/books/{book.id}"
      body: "book"
      response_body: "book"
      additional_bindings {
        put: "/v1/shelves/{shelf_id}/books/{book.id}"
        body: "*"
      }
      additional_bindings {
        custom: {
          kind: "MERGE"
          path: "/v1/shelves/{shelf_id}:merge"
        }
        body: "extra_note"
        response_body: "status"
      }
    };
  }

  rpc WatchShelves(GetShelfRequest) returns (stream Shelf) {
    option (google.api.http) = {
      get: "/v1/shelves/{shelf.id}:watch"
    };
  }
}
"#;
        let proto_path = tmp_dir.join("shelf.proto");
        std::fs::write(&proto_path, service_proto).unwrap();

        let out = Command::new("protoc")
            .arg(format!("-I{}", tmp_dir.display()))
            .arg(format!("-I{}", bundled_proto_dir.display()))
            .arg("--include_imports")
            .arg("--descriptor_set_out=/dev/stdout")
            .arg("shelf.proto")
            .output()
            .expect("failed to execute protoc");
        let _ = std::fs::remove_dir_all(&tmp_dir);

        assert!(
            out.status.success(),
            "protoc failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        let fds = FileDescriptorSetView::from_bytes(&out.stdout).unwrap();
        let files: Vec<FileDescriptorProtoView<'_>> = fds.file().map(|f| f.unwrap()).collect();
        let index = MessageIndex::from_files(files.iter().copied()).unwrap();

        let shelf_file = files
            .iter()
            .find(|f| f.name().unwrap() == "shelf.proto")
            .unwrap();
        let svc = shelf_file.service().next().unwrap().unwrap();
        let methods: Vec<MethodDescriptorProtoView<'_>> =
            svc.method().map(|m| m.unwrap()).collect();

        // 1. Validate GetShelf
        let get_shelf = &methods[0];
        let v_get = extract_and_validate_method_http_rule(get_shelf, ".acme.shelf.v1", &index)
            .expect("GetShelf http rule should be valid")
            .expect("GetShelf has http rule");

        assert_eq!(v_get.primary.pattern.method_name(), "GET");
        assert_eq!(v_get.primary.path_variables.len(), 2);
        assert_eq!(v_get.primary.path_variables[0].steps.len(), 2);
        assert_eq!(v_get.primary.path_variables[0].steps[0].name, "shelf");
        assert_eq!(v_get.primary.path_variables[0].steps[0].tag, 1);
        assert_eq!(
            v_get.primary.path_variables[0].steps[0].proto_type,
            FieldType::TypeMessage
        );
        assert_eq!(v_get.primary.path_variables[0].leaf().name, "id");
        assert_eq!(v_get.primary.path_variables[0].leaf().tag, 1);
        assert_eq!(
            v_get.primary.path_variables[0].leaf().proto_type,
            FieldType::TypeString
        );
        assert_eq!(v_get.primary.path_variables[1].leaf().name, "theme");
        assert_eq!(
            v_get.primary.path_variables[1].leaf().proto_type,
            FieldType::TypeEnum
        );
        assert_eq!(v_get.primary.body, ValidatedBody::Empty);
        assert_eq!(v_get.primary.response_body, ValidatedResponseBody::Empty);
        assert_eq!(v_get.additional_bindings.len(), 1);
        assert_eq!(
            v_get.additional_bindings[0].path_template.verb.as_deref(),
            Some("inspect")
        );

        // 2. Validate UpdateBook (with body: "book", response_body: "book", and 2 additional_bindings)
        let update_book = &methods[1];
        let v_upd = extract_and_validate_method_http_rule(update_book, ".acme.shelf.v1", &index)
            .expect("UpdateBook http rule should be valid")
            .expect("UpdateBook has http rule");

        assert_eq!(v_upd.primary.pattern.method_name(), "PATCH");
        match &v_upd.primary.body {
            ValidatedBody::Field(step) => {
                assert_eq!(step.name, "book");
                assert_eq!(step.tag, 2);
                assert_eq!(step.proto_type, FieldType::TypeMessage);
            }
            other => panic!("expected ValidatedBody::Field, got {other:?}"),
        }
        match &v_upd.primary.response_body {
            ValidatedResponseBody::Field(step) => {
                assert_eq!(step.name, "book");
                assert_eq!(step.tag, 1);
            }
            other => panic!("expected ValidatedResponseBody::Field, got {other:?}"),
        }
        assert_eq!(v_upd.additional_bindings.len(), 2);
        assert_eq!(v_upd.additional_bindings[0].body, ValidatedBody::Wildcard);
        assert_eq!(v_upd.additional_bindings[1].pattern.method_name(), "MERGE");

        // 3. Reject streaming method WatchShelves
        let watch_shelves = &methods[2];
        let err_stream =
            extract_and_validate_method_http_rule(watch_shelves, ".acme.shelf.v1", &index)
                .unwrap_err();
        assert!(err_stream.contains("not supported on streaming method 'WatchShelves'"));

        // 4. Reject invalid field paths, repeated fields, map fields, message leaves, and bad body/response_body
        let bad_rules = [
            (
                HttpRule {
                    pattern: Some(HttpPattern::Get("/v1/{nonexistent}".to_string())),
                    ..Default::default()
                },
                "references unknown field 'nonexistent'",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Get("/v1/{tags}".to_string())),
                    ..Default::default()
                },
                "cannot bind repeated field",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Get("/v1/{labels.key}".to_string())),
                    ..Default::default()
                },
                "cannot bind repeated field",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Get("/v1/{shelf}".to_string())),
                    ..Default::default()
                },
                "leaf field '.acme.shelf.v1.GetShelfRequest.shelf' must be a scalar, string, bytes, or enum",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Get("/v1/{shelf.id.sub}".to_string())),
                    ..Default::default()
                },
                "intermediate field '.acme.shelf.v1.Shelf.id' must be a message",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Post("/v1/shelves/{shelf.id}".to_string())),
                    body: "missing_body_field".to_string(),
                    ..Default::default()
                },
                "body field 'missing_body_field' on method 'GetShelf' does not exist",
            ),
            (
                HttpRule {
                    pattern: Some(HttpPattern::Post("/v1/shelves/{shelf.id}".to_string())),
                    response_body: "missing_resp_field".to_string(),
                    ..Default::default()
                },
                "response_body field 'missing_resp_field' on method 'GetShelf' does not exist",
            ),
        ];

        for (bad_rule, expected_msg) in bad_rules {
            let err = validate_http_rule_for_method(&bad_rule, get_shelf, ".acme.shelf.v1", &index)
                .unwrap_err();
            assert!(
                err.contains(expected_msg),
                "expected error containing '{expected_msg}', got: '{err}'"
            );
        }

        // Also verify body overlapping with leaf path variable on UpdateBook
        let overlap_rule = HttpRule {
            pattern: Some(HttpPattern::Post("/v1/shelves/{shelf_id}".to_string())),
            body: "shelf_id".to_string(),
            ..Default::default()
        };
        let overlap_err =
            validate_http_rule_for_method(&overlap_rule, update_book, ".acme.shelf.v1", &index)
                .unwrap_err();
        assert!(overlap_err.contains("overlaps with bound path variable '{shelf_id}'"));
    }
}
