use std::collections::BTreeMap;

/// A parsed JSON value produced by the zero-dependency recursive-descent parser.
#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    /// Parses a complete JSON document from `input`.
    pub fn parse(input: &str) -> Result<Self, String> {
        parse_json(input)
    }

    /// Returns `true` if this value is `JsonValue::Null`.
    #[inline]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the boolean value if this is `JsonValue::Bool`.
    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Returns the `f64` value if this is `JsonValue::Number`.
    #[inline]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Returns the `i64` value if this is an integral `JsonValue::Number`.
    #[inline]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(n) if n.is_finite() && n.fract() == 0.0 => {
                if *n >= (i64::MIN as f64) && *n < (i64::MAX as f64) {
                    Some(*n as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the string slice if this is `JsonValue::String`.
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns the slice of elements if this is `JsonValue::Array`.
    #[inline]
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(arr) => Some(arr.as_slice()),
            _ => None,
        }
    }

    /// Returns the key-value map if this is `JsonValue::Object`.
    #[inline]
    pub fn as_object(&self) -> Option<&BTreeMap<String, JsonValue>> {
        match self {
            Self::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Looks up `key` if this value is a `JsonValue::Object`.
    #[inline]
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object()?.get(key)
    }
}

/// Parses a JSON string into a [`JsonValue`].
pub fn parse_json(input: &str) -> Result<JsonValue, String> {
    let mut parser = Parser::new(input);
    parser.skip_whitespace();
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if !parser.is_eof() {
        return Err(parser.error("unexpected trailing characters after JSON value"));
    }
    Ok(value)
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    #[inline]
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline]
    fn next_byte(&mut self) -> Option<u8> {
        let b = self.peek()?;
        self.pos += 1;
        Some(b)
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn error(&self, msg: &str) -> String {
        format!("JSON parse error at byte {}: {msg}", self.pos)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        let b = self
            .peek()
            .ok_or_else(|| self.error("unexpected end of input"))?;
        match b {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string().map(JsonValue::String),
            b't' => self.parse_literal(b"true", JsonValue::Bool(true)),
            b'f' => self.parse_literal(b"false", JsonValue::Bool(false)),
            b'n' => self.parse_literal(b"null", JsonValue::Null),
            b'-' | b'0'..=b'9' => self.parse_number(),
            _ => Err(self.error(&format!("unexpected character '{}'", b as char))),
        }
    }

    fn parse_literal(&mut self, expected: &[u8], value: JsonValue) -> Result<JsonValue, String> {
        if self.bytes[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            Ok(value)
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // consume '{'
        self.skip_whitespace();

        let mut map = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(map));
        }

        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected string key in object"));
            }
            let key = self.parse_string()?;

            self.skip_whitespace();
            if self.next_byte() != Some(b':') {
                return Err(self.error("expected ':' after object key"));
            }

            let val = self.parse_value()?;
            map.insert(key, val);

            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b'}') => break,
                _ => return Err(self.error("expected ',' or '}' in object")),
            }
        }

        Ok(JsonValue::Object(map))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.pos += 1; // consume '['
        self.skip_whitespace();

        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }

        loop {
            let val = self.parse_value()?;
            items.push(val);

            self.skip_whitespace();
            match self.next_byte() {
                Some(b',') => continue,
                Some(b']') => break,
                _ => return Err(self.error("expected ',' or ']' in array")),
            }
        }

        Ok(JsonValue::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        if self.next_byte() != Some(b'"') {
            return Err(self.error("expected '\"' to start string"));
        }

        let mut out = String::new();
        while !self.is_eof() {
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1;
                return Ok(out);
            }
            if b == b'\\' {
                self.pos += 1;
                let esc = self
                    .next_byte()
                    .ok_or_else(|| self.error("unterminated escape sequence in string"))?;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let code_unit = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&code_unit) {
                            // High surrogate: must be followed by \uXXXX low surrogate
                            if self.next_byte() != Some(b'\\') || self.next_byte() != Some(b'u') {
                                return Err(
                                    self.error("unpaired UTF-16 high surrogate in string escape")
                                );
                            }
                            let low = self.parse_hex4()?;
                            if !(0xDC00..=0xDFFF).contains(&low) {
                                return Err(
                                    self.error("invalid UTF-16 low surrogate in surrogate pair")
                                );
                            }
                            let combined = 0x10000
                                + (((code_unit as u32 - 0xD800) << 10) | (low as u32 - 0xDC00));
                            let ch = char::from_u32(combined)
                                .ok_or_else(|| self.error("invalid Unicode scalar value"))?;
                            out.push(ch);
                        } else if (0xDC00..=0xDFFF).contains(&code_unit) {
                            return Err(self.error("unpaired UTF-16 low surrogate in string"));
                        } else {
                            let ch = char::from_u32(code_unit as u32)
                                .ok_or_else(|| self.error("invalid Unicode escape"))?;
                            out.push(ch);
                        }
                    }
                    _ => {
                        return Err(
                            self.error(&format!("invalid escape character '\\{}'", esc as char))
                        );
                    }
                }
            } else if b < 0x20 {
                return Err(self.error("unescaped control character in string"));
            } else {
                // Advance by one UTF-8 char directly from the underlying &str slice
                let rest = &self.input[self.pos..];
                let ch = rest
                    .chars()
                    .next()
                    .ok_or_else(|| self.error("unexpected end of string"))?;
                self.pos += ch.len_utf8();
                out.push(ch);
            }
        }

        Err(self.error("unterminated string literal"))
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        let mut val: u16 = 0;
        for _ in 0..4 {
            let b = self
                .next_byte()
                .ok_or_else(|| self.error("incomplete \\uXXXX escape"))?;
            let digit = match b {
                b'0'..=b'9' => (b - b'0') as u16,
                b'a'..=b'f' => (b - b'a' + 10) as u16,
                b'A'..=b'F' => (b - b'A' + 10) as u16,
                _ => return Err(self.error("invalid hex digit in \\uXXXX escape")),
            };
            val = (val << 4) | digit;
        }
        Ok(val)
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;

        if self.peek() == Some(b'-') {
            self.pos += 1;
        }

        match self.peek() {
            Some(b'0') => {
                self.pos += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.error("leading zeros are not permitted in JSON numbers"));
                }
            }
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.error("expected digit in number")),
        }

        if self.peek() == Some(b'.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("expected digit after decimal point"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("expected digit in exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }

        let num_str = &self.input[start..self.pos];
        let num = num_str
            .parse::<f64>()
            .map_err(|e| self.error(&format!("invalid number '{num_str}': {e}")))?;
        Ok(JsonValue::Number(num))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_primitives_arrays_and_nested_objects() {
        let raw = r#"
        {
            "null_val": null,
            "bool_true": true,
            "bool_false": false,
            "int_num": 42,
            "neg_num": -17,
            "float_num": 3.125e2,
            "arr": [1, "two", false, null, {"nested": "ok"}]
        }
        "#;

        let val = parse_json(raw).expect("valid JSON should parse");
        assert!(val.get("null_val").unwrap().is_null());
        assert_eq!(val.get("bool_true").unwrap().as_bool(), Some(true));
        assert_eq!(val.get("bool_false").unwrap().as_bool(), Some(false));
        assert_eq!(val.get("int_num").unwrap().as_i64(), Some(42));
        assert_eq!(val.get("neg_num").unwrap().as_i64(), Some(-17));
        assert_eq!(val.get("float_num").unwrap().as_f64(), Some(312.5));

        let arr = val.get("arr").unwrap().as_array().unwrap();
        assert_eq!(arr.len(), 5);
        assert_eq!(arr[0].as_i64(), Some(1));
        assert_eq!(arr[1].as_str(), Some("two"));
        assert_eq!(arr[2].as_bool(), Some(false));
        assert!(arr[3].is_null());
        assert_eq!(arr[4].get("nested").unwrap().as_str(), Some("ok"));
    }

    #[test]
    fn parses_all_string_escapes_and_surrogate_pairs() {
        let raw = r#""quote:\" slash:\\ fwd:\/ bs:\b ff:\f nl:\n cr:\r tab:\t hex:\u0041\u03b1 emoji:\uD83D\uDE80""#;
        let val = parse_json(raw).expect("escaped string should parse");
        assert_eq!(
            val.as_str().unwrap(),
            "quote:\" slash:\\ fwd:/ bs:\u{0008} ff:\u{000C} nl:\n cr:\r tab:\t hex:A\u{03b1} emoji:🚀"
        );
    }

    #[test]
    fn rejects_malformed_json_inputs() {
        assert!(parse_json("").is_err());
        assert!(parse_json("{\"a\": 1} trailing").is_err());
        assert!(parse_json("{\"a\": 1,}").is_err());
        assert!(parse_json("[1, 2,]").is_err());
        assert!(parse_json("01").is_err());
        assert!(parse_json("1.").is_err());
        assert!(parse_json("1e").is_err());
        assert!(parse_json("\"unterminated").is_err());
        assert!(parse_json("\"bad_escape:\\x\"").is_err());
        assert!(parse_json("\"unpaired_high:\\uD800\"").is_err());
        assert!(parse_json("\"unpaired_low:\\uDC00\"").is_err());
        assert!(parse_json("\"bad_pair:\\uD800\\u0041\"").is_err());
    }
}
