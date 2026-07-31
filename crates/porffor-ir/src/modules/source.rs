//! Removing module-goal-only syntax from a module's source text.
//!
//! [`link`] links a graph by concatenating every unit's body, in evaluation
//! order, into one Script-goal source that the ordinary single-script pipeline
//! lowers. The only thing standing between a module body and that pipeline is
//! the module-goal-only syntax itself: `import` declarations, which declare
//! bindings rather than execute, and the `export` modifier, which decorates a
//! declaration without changing what it does.
//!
//! [`link`]: super::link
//!
//! This scanner deletes exactly that syntax. Unlike the byte scanner it
//! replaces, it is a real JavaScript lexical scanner: it tracks comments,
//! string literals, template literals (including nested `${}` substitutions)
//! and regular-expression literals, and it only treats `import`/`export` as
//! module syntax at nesting depth zero and never immediately after `.`. So
//! neither `const s = "export const x = 1;"` nor `obj.export` is touched.
//!
//! Deleted bytes are replaced with spaces and every line terminator inside the
//! deleted range is kept, so the stripped text has the same byte length and the
//! same line structure as the original. Positions therefore stay comparable
//! with the record's spans.

/// Module syntax the linker cannot express as Script text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StripError {
    /// Human-readable reason, already phrased as a diagnostic message body.
    pub(crate) reason: String,
}

impl StripError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Deletes every top-level `import` declaration and every `export` modifier.
///
/// # Errors
/// Returns [`StripError`] for module syntax this stage cannot express, and for
/// source the scanner cannot lex (an unterminated string or comment).
pub(crate) fn strip_module_syntax(source: &str) -> Result<String, StripError> {
    let mut scanner = Scanner::new(source);
    scanner.run()?;
    Ok(scanner.finish())
}

/// What a `/` means at the current position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlashMeaning {
    /// The previous significant token can end an expression, so `/` divides.
    Divide,
    /// The previous significant token cannot end an expression, so `/` opens a
    /// regular-expression literal.
    Regexp,
}

struct Scanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    /// Byte ranges to blank out, in ascending order and non-overlapping.
    deletions: Vec<(usize, usize)>,
    /// Nesting depth of `(`, `[` and `{`. Module declarations only exist at 0.
    depth: usize,
    /// One entry per open template substitution, holding the `depth` *inside*
    /// it, so the `}` that closes it is told apart from an ordinary `}`.
    template_stack: Vec<usize>,
    slash: SlashMeaning,
    /// The previous significant token was `.`, so the next identifier is a
    /// property name (`obj.export`) rather than a keyword.
    previous_was_dot: bool,
    index: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            deletions: Vec::new(),
            depth: 0,
            template_stack: Vec::new(),
            slash: SlashMeaning::Regexp,
            previous_was_dot: false,
            index: 0,
        }
    }

    /// Rebuilds the source with every deleted range blanked.
    ///
    /// Deleted ranges are produced in ascending order and never overlap, so one
    /// forward pass suffices. Line terminators inside a deleted range survive,
    /// which is what keeps line numbers stable; every other character becomes a
    /// space, which keeps byte offsets stable.
    fn finish(self) -> String {
        let mut rebuilt = String::with_capacity(self.source.len());
        let mut cursor = 0usize;
        for (start, end) in &self.deletions {
            rebuilt.push_str(&self.source[cursor..*start]);
            for ch in self.source[*start..*end].chars() {
                if matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}') {
                    rebuilt.push(ch);
                } else {
                    rebuilt.push(' ');
                }
            }
            cursor = *end;
        }
        rebuilt.push_str(&self.source[cursor..]);
        rebuilt
    }

    fn run(&mut self) -> Result<(), StripError> {
        while self.index < self.bytes.len() {
            let byte = self.bytes[self.index];
            match byte {
                b'/' if self.bytes.get(self.index + 1) == Some(&b'/') => self.skip_line_comment(),
                b'/' if self.bytes.get(self.index + 1) == Some(&b'*') => {
                    self.skip_block_comment()?
                }
                b'/' if self.slash == SlashMeaning::Regexp => {
                    self.skip_regexp()?;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'\'' | b'"' => {
                    self.skip_string(byte)?;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'`' => {
                    self.enter_template()?;
                }
                b'(' | b'[' => {
                    self.depth += 1;
                    self.index += 1;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                }
                b'{' => {
                    self.depth += 1;
                    self.index += 1;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                }
                b')' | b']' => {
                    self.depth = self.depth.saturating_sub(1);
                    self.index += 1;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'}' => {
                    // A `}` that closes a template substitution resumes the
                    // template body rather than ordinary source. The stack
                    // holds the depth *inside* the substitution, so the match
                    // is against the current depth before unwinding it.
                    if self
                        .template_stack
                        .last()
                        .is_some_and(|open_depth| *open_depth == self.depth)
                    {
                        self.template_stack.pop();
                        self.depth = self.depth.saturating_sub(1);
                        self.index += 1;
                        self.resume_template()?;
                        continue;
                    }
                    self.depth = self.depth.saturating_sub(1);
                    self.index += 1;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'.' => {
                    self.index += 1;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = true;
                }
                byte if is_identifier_start_byte(byte) => self.scan_word()?,
                byte if byte.is_ascii_digit() => {
                    self.skip_number();
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                byte if byte.is_ascii_whitespace() => self.index += 1,
                _ => {
                    // Any other punctuator. `++`/`--` end an expression; every
                    // other operator opens one.
                    let two = self.source.get(self.index..self.index + 2);
                    self.slash = if two == Some("++") || two == Some("--") {
                        SlashMeaning::Divide
                    } else {
                        SlashMeaning::Regexp
                    };
                    self.previous_was_dot = false;
                    self.index += self.char_len();
                }
            }
        }
        Ok(())
    }

    fn char_len(&self) -> usize {
        self.source[self.index..]
            .chars()
            .next()
            .map_or(1, char::len_utf8)
    }

    fn scan_word(&mut self) -> Result<(), StripError> {
        let start = self.index;
        while self
            .bytes
            .get(self.index)
            .copied()
            .is_some_and(is_identifier_part_byte)
        {
            self.index += 1;
        }
        let word = &self.source[start..self.index];
        let module_position = self.depth == 0 && self.template_stack.is_empty();
        if module_position && !self.previous_was_dot {
            match word {
                "import" => {
                    let after = self.peek_significant();
                    // `import(` is a dynamic import and `import.meta` is an
                    // expression; neither is a declaration.
                    if after != Some(b'(') && after != Some(b'.') {
                        let end = self.scan_import_declaration()?;
                        self.deletions.push((start, end));
                        self.index = end;
                        self.slash = SlashMeaning::Regexp;
                        self.previous_was_dot = false;
                        return Ok(());
                    }
                }
                "export" => {
                    let end = self.scan_export_prefix()?;
                    self.deletions.push((start, end));
                    self.index = end;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                    return Ok(());
                }
                _ => {}
            }
        }
        // `return`, `typeof`, `case` and friends cannot end an expression, so a
        // `/` after them opens a regular expression. Only a value-shaped word
        // puts the scanner into divide context.
        self.slash = match word {
            "this" | "super" | "true" | "false" | "null" => SlashMeaning::Divide,
            word if is_reserved_word(word) => SlashMeaning::Regexp,
            _ => SlashMeaning::Divide,
        };
        self.previous_was_dot = false;
        Ok(())
    }

    /// Byte offset one past the end of the `import` declaration starting at
    /// `self.index` (which points just past the `import` keyword).
    fn scan_import_declaration(&mut self) -> Result<usize, StripError> {
        let mut cursor = self.index;
        // Everything up to the module specifier is binding syntax: skip to the
        // first string literal, which is always the specifier.
        loop {
            cursor = self.skip_trivia_from(cursor)?;
            let Some(byte) = self.bytes.get(cursor).copied() else {
                return Err(StripError::new(
                    "import declaration has no module specifier",
                ));
            };
            if byte == b'\'' || byte == b'"' {
                cursor = self.string_end(cursor, byte)?;
                break;
            }
            cursor += self.char_len_at(cursor);
        }
        // Optional `with { ... }` / `assert { ... }` attributes clause.
        let after_specifier = self.skip_trivia_from(cursor)?;
        if self.word_at(after_specifier, "with") || self.word_at(after_specifier, "assert") {
            let mut attributes = after_specifier;
            while self
                .bytes
                .get(attributes)
                .copied()
                .is_some_and(is_identifier_part_byte)
            {
                attributes += 1;
            }
            attributes = self.skip_trivia_from(attributes)?;
            if self.bytes.get(attributes) == Some(&b'{') {
                cursor = self.balanced_brace_end(attributes)?;
            }
        }
        self.consume_optional_semicolon(cursor)
    }

    /// Byte offset one past the end of the part of an `export` declaration the
    /// linker deletes.
    ///
    /// For `export { ... }` and `export * from "m"` that is the whole
    /// declaration. For `export <declaration>` it is only the keyword, so the
    /// declaration itself stays and runs exactly as written.
    fn scan_export_prefix(&mut self) -> Result<usize, StripError> {
        let after_keyword = self.index;
        let cursor = self.skip_trivia_from(after_keyword)?;
        match self.bytes.get(cursor).copied() {
            Some(b'{') => {
                let mut end = self.balanced_brace_end(cursor)?;
                let after_list = self.skip_trivia_from(end)?;
                if self.word_at(after_list, "from") {
                    let mut from = after_list + "from".len();
                    from = self.skip_trivia_from(from)?;
                    let Some(quote) = self.bytes.get(from).copied() else {
                        return Err(StripError::new("export ... from has no module specifier"));
                    };
                    if quote != b'\'' && quote != b'"' {
                        return Err(StripError::new("export ... from has no module specifier"));
                    }
                    end = self.string_end(from, quote)?;
                } else {
                    end = after_list;
                }
                self.consume_optional_semicolon(end)
            }
            Some(b'*') => {
                let mut end = cursor + 1;
                loop {
                    end = self.skip_trivia_from(end)?;
                    let Some(byte) = self.bytes.get(end).copied() else {
                        return Err(StripError::new("export * has no module specifier"));
                    };
                    if byte == b'\'' || byte == b'"' {
                        end = self.string_end(end, byte)?;
                        break;
                    }
                    end += self.char_len_at(end);
                }
                self.consume_optional_semicolon(end)
            }
            _ if self.word_at(cursor, "default") => Err(StripError::new(
                "`export default` is not linked yet; name the export instead",
            )),
            Some(_) => Ok(after_keyword),
            None => Err(StripError::new("`export` at end of source")),
        }
    }

    fn consume_optional_semicolon(&self, end: usize) -> Result<usize, StripError> {
        let after = self.skip_trivia_from(end)?;
        if self.bytes.get(after) == Some(&b';') {
            return Ok(after + 1);
        }
        Ok(end)
    }

    fn word_at(&self, index: usize, word: &str) -> bool {
        let Some(slice) = self.source.get(index..index + word.len()) else {
            return false;
        };
        if slice != word {
            return false;
        }
        !self
            .bytes
            .get(index + word.len())
            .copied()
            .is_some_and(is_identifier_part_byte)
    }

    fn char_len_at(&self, index: usize) -> usize {
        self.source[index..]
            .chars()
            .next()
            .map_or(1, char::len_utf8)
    }

    /// Skips whitespace and comments starting at `index`.
    fn skip_trivia_from(&self, mut index: usize) -> Result<usize, StripError> {
        loop {
            match self.bytes.get(index).copied() {
                Some(byte) if byte.is_ascii_whitespace() => index += 1,
                Some(b'/') if self.bytes.get(index + 1) == Some(&b'/') => {
                    while self
                        .bytes
                        .get(index)
                        .copied()
                        .is_some_and(|byte| byte != b'\n')
                    {
                        index += 1;
                    }
                }
                Some(b'/') if self.bytes.get(index + 1) == Some(&b'*') => {
                    let mut end = index + 2;
                    loop {
                        if end + 1 >= self.bytes.len() {
                            return Err(StripError::new("unterminated block comment"));
                        }
                        if self.bytes[end] == b'*' && self.bytes[end + 1] == b'/' {
                            end += 2;
                            break;
                        }
                        end += 1;
                    }
                    index = end;
                }
                Some(byte) if !byte.is_ascii() => {
                    let ch = self.source[index..].chars().next().unwrap_or(' ');
                    if ch.is_whitespace() {
                        index += ch.len_utf8();
                    } else {
                        return Ok(index);
                    }
                }
                _ => return Ok(index),
            }
        }
    }

    /// First non-whitespace, non-comment byte at or after `self.index`.
    fn peek_significant(&self) -> Option<u8> {
        let index = self.skip_trivia_from(self.index).ok()?;
        self.bytes.get(index).copied()
    }

    fn string_end(&self, start: usize, quote: u8) -> Result<usize, StripError> {
        let mut index = start + 1;
        while index < self.bytes.len() {
            match self.bytes[index] {
                b'\\' => index += 1 + self.char_len_at((index + 1).min(self.bytes.len())),
                byte if byte == quote => return Ok(index + 1),
                _ => index += self.char_len_at(index),
            }
        }
        Err(StripError::new("unterminated string literal"))
    }

    fn balanced_brace_end(&self, start: usize) -> Result<usize, StripError> {
        let mut index = start;
        let mut depth = 0usize;
        while index < self.bytes.len() {
            match self.bytes[index] {
                b'{' => {
                    depth += 1;
                    index += 1;
                }
                b'}' => {
                    depth -= 1;
                    index += 1;
                    if depth == 0 {
                        return Ok(index);
                    }
                }
                quote @ (b'\'' | b'"') => index = self.string_end(index, quote)?,
                _ => index += self.char_len_at(index),
            }
        }
        Err(StripError::new("unbalanced braces in module declaration"))
    }

    fn skip_line_comment(&mut self) {
        while self
            .bytes
            .get(self.index)
            .copied()
            .is_some_and(|byte| byte != b'\n')
        {
            self.index += 1;
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), StripError> {
        let mut index = self.index + 2;
        loop {
            if index + 1 >= self.bytes.len() {
                return Err(StripError::new("unterminated block comment"));
            }
            if self.bytes[index] == b'*' && self.bytes[index + 1] == b'/' {
                self.index = index + 2;
                return Ok(());
            }
            index += 1;
        }
    }

    fn skip_string(&mut self, quote: u8) -> Result<(), StripError> {
        self.index = self.string_end(self.index, quote)?;
        Ok(())
    }

    fn skip_number(&mut self) {
        while self.bytes.get(self.index).copied().is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_' || byte == b'$'
        }) {
            self.index += 1;
        }
    }

    fn skip_regexp(&mut self) -> Result<(), StripError> {
        let mut index = self.index + 1;
        let mut in_class = false;
        loop {
            let Some(byte) = self.bytes.get(index).copied() else {
                return Err(StripError::new("unterminated regular expression literal"));
            };
            match byte {
                b'\\' => index += 1 + self.char_len_at((index + 1).min(self.bytes.len())),
                b'[' => {
                    in_class = true;
                    index += 1;
                }
                b']' => {
                    in_class = false;
                    index += 1;
                }
                b'/' if !in_class => {
                    index += 1;
                    break;
                }
                b'\n' => return Err(StripError::new("unterminated regular expression literal")),
                _ => index += self.char_len_at(index),
            }
        }
        while self
            .bytes
            .get(index)
            .copied()
            .is_some_and(is_identifier_part_byte)
        {
            index += 1;
        }
        self.index = index;
        Ok(())
    }

    /// Consumes a template literal starting at the backtick under the cursor,
    /// stopping either after its closing backtick or inside a `${`
    /// substitution (which is ordinary source and must keep being scanned).
    fn enter_template(&mut self) -> Result<(), StripError> {
        self.index += 1;
        self.scan_template_body()
    }

    /// Continues a template body after the `}` that closed a substitution.
    fn resume_template(&mut self) -> Result<(), StripError> {
        self.scan_template_body()
    }

    fn scan_template_body(&mut self) -> Result<(), StripError> {
        while let Some(byte) = self.bytes.get(self.index).copied() {
            match byte {
                b'\\' => {
                    self.index += 1;
                    self.index += self.char_len_at(self.index.min(self.bytes.len()));
                }
                b'`' => {
                    self.index += 1;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                    return Ok(());
                }
                b'$' if self.bytes.get(self.index + 1) == Some(&b'{') => {
                    self.index += 2;
                    self.depth += 1;
                    self.template_stack.push(self.depth);
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                    return Ok(());
                }
                _ => self.index += self.char_len_at(self.index),
            }
        }
        Err(StripError::new("unterminated template literal"))
    }
}

fn is_identifier_start_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$' || !byte.is_ascii()
}

fn is_identifier_part_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || !byte.is_ascii()
}

fn is_reserved_word(word: &str) -> bool {
    matches!(
        word,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "switch"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(source: &str) -> String {
        strip_module_syntax(source).expect("source should strip")
    }

    #[test]
    fn export_modifier_is_deleted_and_the_declaration_stays() {
        assert_eq!(
            strip("export const value = 41;"),
            "       const value = 41;"
        );
    }

    #[test]
    fn import_declaration_is_deleted_whole() {
        let stripped = strip("import { value } from \"./a.mjs\";\nprint(value);\n");
        assert_eq!(stripped.trim_start(), "\nprint(value);\n".trim_start());
        assert!(stripped.ends_with("print(value);\n"));
    }

    #[test]
    fn line_structure_and_length_are_preserved() {
        let source = "import { a } from \"m\";\nexport const b = a;\n";
        let stripped = strip(source);
        assert_eq!(stripped.len(), source.len());
        assert_eq!(
            stripped.lines().count(),
            source.lines().count(),
            "line count must survive stripping"
        );
    }

    #[test]
    fn export_inside_a_string_literal_is_left_alone() {
        let source = "const s = \"export const x = 1;\";";
        assert_eq!(strip(source), source);
    }

    #[test]
    fn export_as_a_property_name_is_left_alone() {
        let source = "const o = { export: 1 };\no.export;";
        assert_eq!(strip(source), source);
    }

    #[test]
    fn export_inside_a_template_substitution_is_left_alone() {
        let source = "const t = `${ 1 } export`;";
        assert_eq!(strip(source), source);
    }

    #[test]
    fn regexp_literal_containing_a_quote_does_not_open_a_string() {
        let source = "const r = /'/;\nexport const x = 1;";
        assert_eq!(strip(source), "const r = /'/;\n       const x = 1;");
    }

    #[test]
    fn export_list_clause_is_deleted() {
        let source = "const x = 1;\nexport { x };\n";
        let stripped = strip(source);
        assert_eq!(stripped.len(), source.len());
        assert!(!stripped.contains("export"));
        assert!(stripped.contains("const x = 1;"));
    }

    #[test]
    fn export_default_is_reported_rather_than_mangled() {
        let error = strip_module_syntax("export default 1;").expect_err("must be reported");
        assert!(error.reason.contains("export default"));
    }

    #[test]
    fn dynamic_import_call_is_not_a_declaration() {
        let source = "import(\"m\").then(f);";
        assert_eq!(strip(source), source);
    }

    #[test]
    fn import_meta_is_not_a_declaration() {
        let source = "print(import.meta.url);";
        assert_eq!(strip(source), source);
    }

    #[test]
    fn import_with_attributes_is_deleted_whole() {
        let source = "import a from \"m\" with { type: \"json\" };\n";
        let stripped = strip(source);
        assert_eq!(stripped.trim(), "");
    }
}
