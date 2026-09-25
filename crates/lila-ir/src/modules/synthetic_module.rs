//! The default-export Synthetic Module Records the host creates for
//! non-JavaScript module types, realized ahead of time: `ParseJSONModule`
//! (`type: "json"`) and `CreateTextModule` (`type: "text"`).
//!
//! A text module is `CreateDefaultExportSyntheticModule(source)`: its only
//! export, `default`, is the module's source text as a String. It is realized
//! as `export default "<literal>";`, where the literal spells every UTF-16 code
//! unit of the text with an escape or as itself, so the bound value is exactly
//! the String the host decoded.
//!
//! A JSON module is the Synthetic Module Record `CreateDefaultExportSyntheticModule`
//! builds around `? Call(%JSON.parse%, undefined, « source »)`: it requests no
//! modules, has no top-level await, and exports exactly one name, `default`.
//! `ParseJSONModule` runs while the host loads the module, so invalid JSON is
//! a load failure — the importer's graph never links and `import()` rejects —
//! and no user code can run between the parse and the first read of the value.
//!
//! lila validates the text against the JSON grammar here and realizes the
//! record as a Source Text Module whose whole body is
//! `export default (<literal>);`, with `<literal>` an ECMAScript array/object
//! literal spelling of the parsed value. That is not an approximation of the
//! synthetic record; every observation agrees:
//!
//! * The namespace exports only `default`, the module requests nothing and it
//!   is never asynchronous, exactly as `CreateDefaultExportSyntheticModule`
//!   specifies.
//! * `JSON.parse` builds its result with `OrdinaryObjectCreate(%Object.prototype%)`,
//!   `ArrayCreate` and `CreateDataProperty`; object and array literals use the
//!   same intrinsics and the same operations, so neither can call user code
//!   (no inherited setter, no `Array.prototype` hook) and building the value at
//!   evaluation instead of at load time is unobservable. The one literal form
//!   that is *not* a plain `CreateDataProperty` — a non-computed `__proto__`
//!   key, which sets `[[Prototype]]` — is spelled as a computed key.
//! * JSON numbers and strings are spelled verbatim. Every JSON number is an
//!   ECMAScript numeric literal (optionally negated) denoting the same
//!   `StringToNumber` value, and every JSON string is an ECMAScript string
//!   literal with the same code units: JSON forbids raw control characters,
//!   and its escapes (`\" \\ \/ \b \f \n \r \t \uXXXX`) are all valid strict
//!   ECMAScript escapes with the same meaning.
//! * Duplicate keys keep the first key's position and the last value, as both
//!   `CreateDataProperty` sequences do.
//!
//! What differs is the `default` binding's state before evaluation — the
//! synthetic record initializes it to `undefined`, `export default` leaves it
//! uninitialized — and nothing can read it then: a JSON module has no
//! dependencies, so it finishes evaluating before any importer runs, and a
//! deferred namespace evaluates it before its first property read.

/// Why `ParseJSONModule` threw its `SyntaxError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonModuleSyntaxError {
    /// Byte offset into the module's text.
    pub offset: usize,
    /// What the JSON grammar expected there.
    pub reason: &'static str,
}

impl core::fmt::Display for JsonModuleSyntaxError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "invalid JSON module text at byte {}: {}",
            self.offset, self.reason
        )
    }
}

impl std::error::Error for JsonModuleSyntaxError {}

/// `CreateTextModule`: the Module-goal source text that realizes the
/// Synthetic Module Record whose `default` export is `text`.
pub(super) fn synthesize_text_module_source(text: &str) -> String {
    let mut source = String::with_capacity(text.len() + 20);
    source.push_str("export default ");
    super::namespace::push_js_string_literal(&mut source, text);
    source.push_str(";\n");
    source
}

/// Performs `ParseJSONModule`'s grammar check and returns the Module-goal
/// source text that realizes its Synthetic Module Record.
pub(super) fn synthesize_json_module_source(json: &str) -> Result<String, JsonModuleSyntaxError> {
    let mut emitter = JsonLiteralEmitter {
        input: json.as_bytes(),
        index: 0,
        output: String::with_capacity(json.len() + 24),
    };
    emitter.output.push_str("export default (");
    emitter.skip_whitespace();
    emitter.value()?;
    emitter.skip_whitespace();
    if emitter.index != emitter.input.len() {
        return Err(emitter.error("end of JSON text"));
    }
    emitter.output.push_str(");\n");
    Ok(emitter.output)
}

/// A recursive-descent recognizer for the JSON text grammar (ECMA-404, as
/// `JSON.parse` step 2 requires) that copies each accepted token into
/// `output` as its ECMAScript literal spelling.
struct JsonLiteralEmitter<'a> {
    input: &'a [u8],
    index: usize,
    output: String,
}

impl JsonLiteralEmitter<'_> {
    fn error(&self, reason: &'static str) -> JsonModuleSyntaxError {
        JsonModuleSyntaxError {
            offset: self.index,
            reason,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.index).copied()
    }

    /// JSON whitespace is exactly tab, line feed, carriage return and space.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.index += 1;
        }
    }

    /// Appends `self.input[start..self.index]`, which the caller has already
    /// recognized as a complete token.
    fn copy_token(&mut self, start: usize) {
        let token = core::str::from_utf8(&self.input[start..self.index])
            .expect("a JSON token starts and ends on UTF-8 boundaries");
        self.output.push_str(token);
    }

    fn value(&mut self) -> Result<(), JsonModuleSyntaxError> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => {
                let start = self.index;
                self.string()?;
                self.copy_token(start);
                Ok(())
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(b't') => self.keyword("true"),
            Some(b'f') => self.keyword("false"),
            Some(b'n') => self.keyword("null"),
            _ => Err(self.error("a JSON value")),
        }
    }

    fn keyword(&mut self, keyword: &'static str) -> Result<(), JsonModuleSyntaxError> {
        if !self.input[self.index..].starts_with(keyword.as_bytes()) {
            return Err(self.error("true, false or null"));
        }
        self.index += keyword.len();
        self.output.push_str(keyword);
        Ok(())
    }

    fn digits(&mut self) -> Result<(), JsonModuleSyntaxError> {
        if !matches!(self.peek(), Some(b'0'..=b'9')) {
            return Err(self.error("a decimal digit"));
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
        Ok(())
    }

    fn number(&mut self) -> Result<(), JsonModuleSyntaxError> {
        let start = self.index;
        if self.peek() == Some(b'-') {
            self.index += 1;
        }
        match self.peek() {
            // A leading zero stands alone: `01` is not a JSON number.
            Some(b'0') => self.index += 1,
            Some(b'1'..=b'9') => self.digits()?,
            _ => return Err(self.error("a decimal digit")),
        }
        if self.peek() == Some(b'.') {
            self.index += 1;
            self.digits()?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            self.digits()?;
        }
        self.copy_token(start);
        Ok(())
    }

    /// Recognizes one JSON string, leaving `self.index` after its closing
    /// quote. Returns whether it contained an escape sequence.
    fn string(&mut self) -> Result<bool, JsonModuleSyntaxError> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.index += 1;
        let mut escaped = false;
        loop {
            match self.peek() {
                None => return Err(self.error("a closing quotation mark")),
                Some(b'"') => {
                    self.index += 1;
                    return Ok(escaped);
                }
                Some(b'\\') => {
                    escaped = true;
                    self.index += 1;
                    match self.peek() {
                        Some(b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't') => {
                            self.index += 1;
                        }
                        Some(b'u') => {
                            self.index += 1;
                            for _ in 0..4 {
                                if !self.peek().is_some_and(|byte| byte.is_ascii_hexdigit()) {
                                    return Err(self.error("four hexadecimal digits"));
                                }
                                self.index += 1;
                            }
                        }
                        _ => return Err(self.error("a JSON escape sequence")),
                    }
                }
                Some(0x00..=0x1F) => {
                    return Err(self.error("an escaped control character"));
                }
                Some(_) => self.index += 1,
            }
        }
    }

    fn array(&mut self) -> Result<(), JsonModuleSyntaxError> {
        self.index += 1;
        self.output.push('[');
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.index += 1;
            self.output.push(']');
            return Ok(());
        }
        loop {
            self.skip_whitespace();
            self.value()?;
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.index += 1;
                    self.output.push(',');
                }
                Some(b']') => {
                    self.index += 1;
                    self.output.push(']');
                    return Ok(());
                }
                _ => return Err(self.error("`,` or `]`")),
            }
        }
    }

    fn object(&mut self) -> Result<(), JsonModuleSyntaxError> {
        self.index += 1;
        self.output.push('{');
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.index += 1;
            self.output.push('}');
            return Ok(());
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some(b'"') {
                return Err(self.error("a string property name"));
            }
            let start = self.index;
            let escaped = self.string()?;
            // `__proto__: value` in an object literal sets [[Prototype]]
            // rather than creating a property (13.2.5.5), and it is decided on
            // the key's StringValue, escapes included. A computed key is always
            // a plain CreateDataProperty, so any key that is or might decode to
            // `__proto__` is spelled computed.
            let computed = escaped || &self.input[start..self.index] == b"\"__proto__\"";
            if computed {
                self.output.push('[');
            }
            self.copy_token(start);
            if computed {
                self.output.push(']');
            }
            self.skip_whitespace();
            if self.peek() != Some(b':') {
                return Err(self.error("`:`"));
            }
            self.index += 1;
            self.output.push(':');
            self.skip_whitespace();
            self.value()?;
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.index += 1;
                    self.output.push(',');
                }
                Some(b'}') => {
                    self.index += 1;
                    self.output.push('}');
                    return Ok(());
                }
                _ => return Err(self.error("`,` or `}`")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{synthesize_json_module_source, synthesize_text_module_source};

    #[test]
    fn text_is_spelled_as_one_string_literal_of_the_same_code_units() {
        for (text, literal) in [
            ("", "\"\""),
            ("a string value\n", "\"a string value\\n\""),
            (
                "\"quoted\" \\ export default 1;",
                "\"\\\"quoted\\\" \\\\ export default 1;\"",
            ),
            ("\u{2028}\u{1F600}", "\"\\u2028\\uD83D\\uDE00\""),
        ] {
            assert_eq!(
                synthesize_text_module_source(text),
                format!("export default {literal};\n"),
                "{text:?}"
            );
        }
    }

    #[test]
    fn values_are_spelled_as_equivalent_literals() {
        for (json, body) in [
            ("262", "262"),
            (" \t\r\n-0.5e+3 ", "-0.5e+3"),
            ("\"a\\u0062\\/\"", "\"a\\u0062\\/\""),
            (
                "[1, [], {}, null, true, false]",
                "[1,[],{},null,true,false]",
            ),
            ("{\"a\": 1, \"a\": 2}", "{\"a\":1,\"a\":2}"),
            ("{\"__proto__\": []}", "{[\"__proto__\"]:[]}"),
            ("{\"\\u005f_proto__\": 1}", "{[\"\\u005f_proto__\"]:1}"),
            ("\"\u{2028}\"", "\"\u{2028}\""),
        ] {
            assert_eq!(
                synthesize_json_module_source(json).as_deref(),
                Ok(format!("export default ({body});\n").as_str()),
                "{json}"
            );
        }
    }

    #[test]
    fn text_outside_the_json_grammar_is_a_syntax_error() {
        for json in [
            "",
            "{",
            "[1,]",
            "{\"a\":1,}",
            "01",
            "1.",
            ".5",
            "+1",
            "-",
            "1e",
            "NaN",
            "undefined",
            "'single'",
            "\"\\x41\"",
            "\"\\u12\"",
            "\"raw\ttab\"",
            "{a: 1}",
            "1 2",
            "\u{a0}1",
            "// comment\n1",
        ] {
            assert!(synthesize_json_module_source(json).is_err(), "{json:?}");
        }
    }
}
