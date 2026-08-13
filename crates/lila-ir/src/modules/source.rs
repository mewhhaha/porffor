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
//! Deleted bytes are replaced with the same number of space *bytes* and every
//! line terminator inside the deleted range is kept, so the stripped text has
//! the same byte length and line count as the original. Replacements carry the
//! same invariant in a closed type. Later tokens therefore keep their byte
//! offsets and line numbers, including the distinction between one CRLF
//! sequence and separate CR/LF sequences, although a replacement may move a
//! sequence within its own span and therefore does not promise column fidelity.

use crate::{
    MergedName, DEFAULT_BINDING_ASSIGN, DEFAULT_BINDING_LET, DEFAULT_BINDING_VAR, DEFAULT_KEYWORD,
    EXPORT_KEYWORD,
};

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

/// One source edit, whose kind exhaustively determines how its span is rebuilt.
struct SourceEdit {
    start: usize,
    end: usize,
    kind: ModuleSyntaxEdit,
}

enum ModuleSyntaxEdit {
    /// Keep line-terminator sequences; replace every other source byte with a
    /// space.
    Blank,
    /// Replace with text proved stable against this edit's source span.
    Replace(SpanStableReplacement),
}

/// Replacement text with the byte width and ordered ECMAScript
/// LineTerminatorSequences of the source span it erases.
///
/// There is deliberately no raw-string constructor. The only constructor
/// receives the erased source slice and reserves its terminator sequences
/// before it admits generated text, so neither length nor line structure can be
/// forgotten at a call site.
struct SpanStableReplacement(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpanStableReplacementError {
    InvalidSpan,
    GeneratedLineTerminator,
    DoesNotFit,
}

impl SourceEdit {
    fn blank(source: &str, start: usize, end: usize) -> Result<Self, StripError> {
        source.get(start..end).ok_or_else(|| {
            StripError::new(format!(
                "module-syntax edit {start}..{end} is not a span of this source text"
            ))
        })?;
        Ok(Self {
            start,
            end,
            kind: ModuleSyntaxEdit::Blank,
        })
    }

    fn replace_around_padding(
        source: &str,
        start: usize,
        end: usize,
        before_padding: &str,
        after_padding: &str,
    ) -> Result<Self, SpanStableReplacementError> {
        let erased = source
            .get(start..end)
            .ok_or(SpanStableReplacementError::InvalidSpan)?;
        let suffix = source
            .get(end..)
            .ok_or(SpanStableReplacementError::InvalidSpan)?;
        let replacement =
            SpanStableReplacement::around_padding(erased, suffix, before_padding, after_padding)?;
        Ok(Self {
            start,
            end,
            kind: ModuleSyntaxEdit::Replace(replacement),
        })
    }
}

impl SpanStableReplacement {
    fn around_padding(
        erased: &str,
        suffix: &str,
        before_padding: &str,
        after_padding: &str,
    ) -> Result<Self, SpanStableReplacementError> {
        if contains_ecmascript_line_terminator(before_padding)
            || contains_ecmascript_line_terminator(after_padding)
        {
            return Err(SpanStableReplacementError::GeneratedLineTerminator);
        }

        let mut terminators = Vec::new();
        let mut cursor = 0usize;
        while cursor < erased.len() {
            if let Some(sequence) = ecmascript_line_terminator_sequence_at(erased, cursor) {
                terminators.push(sequence);
                cursor += sequence.len();
            } else {
                cursor += erased[cursor..].chars().next().map_or(1, char::len_utf8);
            }
        }
        // Relocating separate CR and LF sequences next to one another would
        // turn them into one CRLF sequence. Each internal pair had at least one
        // non-terminator byte between it in `erased`. The edit-boundary pair
        // below had `default` between the erased CR and the untouched suffix
        // LF. Reserving one of those displaced bytes as a barrier therefore
        // cannot make an otherwise-fitting rewrite overflow.
        let internal_barriers = terminators
            .windows(2)
            .filter(|pair| pair[0] == "\r" && pair[1] == "\n")
            .count();
        let trailing_barrier = terminators.last() == Some(&"\r") && suffix.starts_with('\n');
        let terminator_width = terminators
            .iter()
            .map(|sequence| sequence.len())
            .sum::<usize>();
        let Some(generated_width) = before_padding.len().checked_add(after_padding.len()) else {
            return Err(SpanStableReplacementError::DoesNotFit);
        };
        let Some(required_width) = generated_width
            .checked_add(terminator_width)
            .and_then(|width| width.checked_add(internal_barriers))
            .and_then(|width| width.checked_add(if trailing_barrier { 1 } else { 0 }))
        else {
            return Err(SpanStableReplacementError::DoesNotFit);
        };
        let Some(padding) = erased.len().checked_sub(required_width) else {
            return Err(SpanStableReplacementError::DoesNotFit);
        };

        let mut replacement = String::with_capacity(erased.len());
        replacement.push_str(before_padding);
        replacement.extend(core::iter::repeat_n(' ', padding));
        replacement.push_str(after_padding);
        for (index, sequence) in terminators.iter().enumerate() {
            if index != 0 && terminators[index - 1] == "\r" && *sequence == "\n" {
                replacement.push(' ');
            }
            replacement.push_str(sequence);
        }
        if trailing_barrier {
            replacement.push(' ');
        }

        debug_assert_eq!(replacement.len(), erased.len());
        let mut replacement_with_suffix = String::with_capacity(replacement.len() + suffix.len());
        replacement_with_suffix.push_str(&replacement);
        replacement_with_suffix.push_str(suffix);
        let mut erased_with_suffix = String::with_capacity(erased.len() + suffix.len());
        erased_with_suffix.push_str(erased);
        erased_with_suffix.push_str(suffix);
        debug_assert_eq!(
            collect_ecmascript_line_terminator_sequences(&replacement_with_suffix),
            collect_ecmascript_line_terminator_sequences(&erased_with_suffix)
        );
        Ok(Self(replacement))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

/// The ECMAScript LineTerminatorSequence beginning at this byte offset.
///
/// CRLF is returned as one sequence; a standalone CR or LF is returned as one
/// sequence of its own. All scanner paths use this helper so line comments and
/// span-stable replacements cannot disagree about the lexical line boundary.
fn ecmascript_line_terminator_sequence_at(source: &str, index: usize) -> Option<&str> {
    let remaining = source.get(index..)?;
    for sequence in ["\r\n", "\r", "\n", "\u{2028}", "\u{2029}"] {
        if remaining.starts_with(sequence) {
            return remaining.get(..sequence.len());
        }
    }
    None
}

fn contains_ecmascript_line_terminator(source: &str) -> bool {
    let mut index = 0usize;
    while index < source.len() {
        if ecmascript_line_terminator_sequence_at(source, index).is_some() {
            return true;
        }
        index += source[index..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

fn collect_ecmascript_line_terminator_sequences(source: &str) -> Vec<&str> {
    let mut sequences = Vec::new();
    let mut index = 0usize;
    while index < source.len() {
        if let Some(sequence) = ecmascript_line_terminator_sequence_at(source, index) {
            sequences.push(sequence);
            index += sequence.len();
        } else {
            index += source[index..].chars().next().map_or(1, char::len_utf8);
        }
    }
    sequences
}

/// What the scanner does with the `export default` keyword pair, decided by the
/// record rather than re-derived from the text.
///
/// Once line terminators are reserved, the two keywords guarantee 13 bytes for
/// generated code even in the narrowest split pair, `export\ndefault`. That is
/// invariant B1, and const assertion V2 in `crate::binding_names` holds
/// `MergedName::anonymous_default` to it at compile time using the very
/// constants this scanner matches on — [`EXPORT_KEYWORD`], [`DEFAULT_KEYWORD`],
/// [`DEFAULT_BINDING_LET`] and [`DEFAULT_BINDING_ASSIGN`]. The runtime check in
/// `Scanner::rewrite_default_keywords` still verifies the actual source span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DefaultExportRewrite<'a> {
    /// No `export default` in this unit; one found anyway is a disagreement
    /// between the record and the text, and is reported.
    None,
    /// The declaration binds its own name, so only the keywords are deleted.
    DeleteKeywords,
    /// The declaration binds nothing spellable, so the keywords become
    /// `var <name> =` or `let <name> =`.
    Bind {
        /// Merged-scope name to declare.
        ///
        /// A [`MergedName`], so the only thing that can be written here is a
        /// name of the scope the declaration lands in. A `[[LocalName]]` — in
        /// particular the `*default*` this rewrite exists to replace — is
        /// `E0308`.
        name: &'a MergedName,
        /// Use `var` rather than `let`, for a hoistable declaration.
        hoisted: bool,
    },
}

/// Deletes every top-level `import` declaration and every `export` modifier,
/// and rewrites `export default` as `default_export` directs.
///
/// # Errors
/// Returns [`StripError`] for module syntax this stage cannot express, and for
/// source the scanner cannot lex (an unterminated string or comment).
pub(crate) fn strip_module_syntax(
    source: &str,
    default_export: DefaultExportRewrite<'_>,
) -> Result<String, StripError> {
    let mut scanner = Scanner::new(source, default_export);
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
    default_export: DefaultExportRewrite<'a>,
    /// Byte ranges to rewrite, in ascending order and non-overlapping.
    edits: Vec<SourceEdit>,
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
    fn new(source: &'a str, default_export: DefaultExportRewrite<'a>) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            default_export,
            edits: Vec::new(),
            depth: 0,
            template_stack: Vec::new(),
            slash: SlashMeaning::Regexp,
            previous_was_dot: false,
            index: 0,
        }
    }

    /// Rebuilds the source with every rewritten range blanked or replaced.
    ///
    /// Rewritten ranges are produced in ascending order and never overlap, so
    /// one forward pass suffices. A blank emits one space per source *byte* and
    /// retains line-terminator sequences; a replacement already proves the
    /// same byte length and ordered sequence list as the range it covers.
    fn finish(self) -> String {
        let mut rebuilt = String::with_capacity(self.source.len());
        let mut cursor = 0usize;
        for edit in &self.edits {
            rebuilt.push_str(&self.source[cursor..edit.start]);
            match &edit.kind {
                ModuleSyntaxEdit::Replace(replacement) => {
                    rebuilt.push_str(replacement.as_str());
                }
                ModuleSyntaxEdit::Blank => {
                    let erased = &self.source[edit.start..edit.end];
                    let mut index = 0usize;
                    while index < erased.len() {
                        if let Some(sequence) =
                            ecmascript_line_terminator_sequence_at(erased, index)
                        {
                            rebuilt.push_str(sequence);
                            index += sequence.len();
                        } else {
                            let width = erased[index..].chars().next().map_or(1, char::len_utf8);
                            rebuilt.extend(core::iter::repeat_n(' ', width));
                            index += width;
                        }
                    }
                }
            }
            cursor = edit.end;
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
                        self.edits.push(SourceEdit::blank(self.source, start, end)?);
                        self.index = end;
                        self.slash = SlashMeaning::Regexp;
                        self.previous_was_dot = false;
                        return Ok(());
                    }
                }
                EXPORT_KEYWORD => {
                    let edit = self.scan_export_prefix(start)?;
                    self.index = edit.end;
                    self.edits.push(edit);
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

    /// Edit for the part of an `export` declaration the linker rewrites.
    ///
    /// For `export { ... }` and `export * from "m"` that is the whole
    /// declaration, blanked. For `export <declaration>` it is only the keyword,
    /// so the declaration itself stays and runs exactly as written. For
    /// `export default` it is both keywords, either blanked or replaced by a
    /// declaration head — see [`DefaultExportRewrite`].
    ///
    /// `start` is the offset of the `export` keyword, which is where a
    /// replacement has to begin.
    fn scan_export_prefix(&mut self, start: usize) -> Result<SourceEdit, StripError> {
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
                SourceEdit::blank(self.source, start, self.consume_optional_semicolon(end)?)
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
                SourceEdit::blank(self.source, start, self.consume_optional_semicolon(end)?)
            }
            _ if self.word_at(cursor, DEFAULT_KEYWORD) => {
                self.rewrite_default_keywords(start, cursor + DEFAULT_KEYWORD.len())
            }
            Some(_) => SourceEdit::blank(self.source, start, after_keyword),
            None => Err(StripError::new("`export` at end of source")),
        }
    }

    /// Rewrite for the `export default` keyword pair spanning `start..end`.
    ///
    /// The named forms need nothing but the keywords gone: what follows is
    /// already a `FunctionDeclaration` or `ClassDeclaration` that binds the
    /// export entry's `[[LocalName]]`. The anonymous forms have no such name,
    /// so the keywords become the head of a declaration of the minted one, and
    /// the rest of the text — the function, the class or the expression — stays
    /// exactly where it was as that declaration's initializer.
    fn rewrite_default_keywords(&self, start: usize, end: usize) -> Result<SourceEdit, StripError> {
        let (name, hoisted) = match self.default_export {
            DefaultExportRewrite::None => {
                return Err(StripError::new(
                    "`export default` in a module whose record has no default export",
                ));
            }
            DefaultExportRewrite::DeleteKeywords => {
                return SourceEdit::blank(self.source, start, end);
            }
            DefaultExportRewrite::Bind { name, hoisted } => (name, hoisted),
        };
        let keyword = if hoisted {
            DEFAULT_BINDING_VAR
        } else {
            DEFAULT_BINDING_LET
        };
        let name = name.as_str();
        let width = end.saturating_sub(start);
        let before_padding = format!("{keyword}{name}");
        SourceEdit::replace_around_padding(
            self.source,
            start,
            end,
            &before_padding,
            DEFAULT_BINDING_ASSIGN,
        )
        .map_err(|error| match error {
            SpanStableReplacementError::DoesNotFit => StripError::new(format!(
                "`export default` binding `{name}` does not fit in the {width} bytes it replaces \
                 after preserving its line terminators"
            )),
            SpanStableReplacementError::InvalidSpan => StripError::new(format!(
                "`export default` span {start}..{end} is not a span of this module's source text"
            )),
            SpanStableReplacementError::GeneratedLineTerminator => StripError::new(
                "generated `export default` declaration head contains a line terminator",
            ),
        })
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
                    while index < self.source.len()
                        && ecmascript_line_terminator_sequence_at(self.source, index).is_none()
                    {
                        index += self.char_len_at(index);
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
        while self.index < self.source.len()
            && ecmascript_line_terminator_sequence_at(self.source, self.index).is_none()
        {
            self.index += self.char_len();
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

    use crate::{LocalName, MAX_LINKABLE_MODULE_UNIT_ID};

    fn strip(source: &str) -> String {
        strip_module_syntax(source, DefaultExportRewrite::None).expect("source should strip")
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

    /// Blanking is measured in bytes, not Unicode scalar values. A single
    /// space for `π` or `☿` would move the marker that follows the erased
    /// declarations and invalidate every later span.
    #[test]
    fn non_ascii_module_syntax_is_blanked_without_moving_later_bytes() {
        let source = "import { π as value } from \"☿\";\n\
                      export { value as \"☿\" };\n\
                      const after_unicode_module_syntax = 1;";
        let stripped = strip(source);

        assert_eq!(stripped.len(), source.len());
        assert_eq!(
            stripped.find("after_unicode_module_syntax"),
            source.find("after_unicode_module_syntax")
        );
        assert_eq!(
            collect_ecmascript_line_terminator_sequences(&stripped),
            collect_ecmascript_line_terminator_sequences(source)
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

    /// A record that says the unit has no default export disagrees with text
    /// that does: reported, never guessed at.
    #[test]
    fn export_default_without_a_record_entry_is_reported() {
        let error = strip_module_syntax("export default 1;", DefaultExportRewrite::None)
            .expect_err("must be reported");
        assert!(error.reason.contains("export default"), "{}", error.reason);
    }

    /// The anonymous form becomes a declaration of the minted name, in place
    /// and without moving a byte of the initializer.
    #[test]
    fn anonymous_export_default_becomes_a_declaration_of_the_minted_name() {
        let source = "export default 42;\nprint(1);\n";
        let stripped = strip_module_syntax(
            source,
            DefaultExportRewrite::Bind {
                name: &LocalName::AnonymousDefault.merged_in(0),
                hoisted: false,
            },
        )
        .expect("source should strip");
        assert_eq!(stripped.len(), source.len());
        assert_eq!(stripped, "let $d0$     = 42;\nprint(1);\n");
    }

    /// The grammar permits line terminators between `export` and `default`.
    /// Reserve every terminator byte before fitting the widest binding name,
    /// including CRLF as one sequence and the two Unicode forms.
    #[test]
    fn split_anonymous_defaults_preserve_bytes_and_ordered_line_sequences_at_the_cap() {
        let name = LocalName::AnonymousDefault.merged_in(MAX_LINKABLE_MODULE_UNIT_ID);
        assert_eq!(name.as_str(), "$d9999$");

        let forms = [
            ("42", false, "let $d9999$"),
            ("function () {}", true, "var $d9999$"),
        ];
        for trivia in [
            "\n",
            "\r\n",
            "\u{2028}",
            "\u{2029}",
            "/*☿\r\n\u{2028}π\u{2029}*/",
        ] {
            for (initializer, hoisted, declaration) in forms {
                let source =
                    format!("export{trivia}default {initializer};\nconst after_split_default = 1;");
                let stripped = strip_module_syntax(
                    &source,
                    DefaultExportRewrite::Bind {
                        name: &name,
                        hoisted,
                    },
                )
                .expect("split default should strip");

                assert_eq!(stripped.len(), source.len(), "{trivia:?} {initializer}");
                assert_eq!(
                    stripped.find("after_split_default"),
                    source.find("after_split_default"),
                    "{trivia:?} {initializer}"
                );
                assert!(stripped.starts_with(declaration), "got {stripped}");
                let initializer_offset = stripped
                    .find(initializer)
                    .expect("initializer should stay present");
                let terminators = collect_ecmascript_line_terminator_sequences(trivia).concat();
                assert!(
                    stripped[..initializer_offset].ends_with(&format!("={terminators} ")),
                    "got {stripped}"
                );
                assert_eq!(
                    collect_ecmascript_line_terminator_sequences(&stripped),
                    collect_ecmascript_line_terminator_sequences(&source),
                    "got {stripped}"
                );
            }
        }
    }

    /// A standalone CR and a later standalone LF are two line-terminator
    /// sequences. Relocating their raw code points adjacently would silently
    /// collapse them into one CRLF sequence and move every later line number.
    #[test]
    fn relocated_separate_cr_and_lf_sequences_keep_a_non_terminator_barrier() {
        let source = "export/*\rseparate\n*/default 42;\nconst after = 1;";
        let stripped = strip_module_syntax(
            source,
            DefaultExportRewrite::Bind {
                name: &LocalName::AnonymousDefault.merged_in(0),
                hoisted: false,
            },
        )
        .expect("separated CR and LF should strip");

        assert_eq!(stripped.len(), source.len());
        assert_eq!(stripped.find("after"), source.find("after"));
        assert_eq!(
            collect_ecmascript_line_terminator_sequences(&stripped),
            vec!["\r", "\n", "\n"]
        );
        assert_eq!(
            collect_ecmascript_line_terminator_sequences(&stripped),
            collect_ecmascript_line_terminator_sequences(source)
        );
        assert!(stripped.contains("=\r \n 42;"), "got {stripped}");
    }

    /// The untouched suffix participates in line-sequence grouping too. A
    /// relocated standalone CR at the end of the edit must not fuse with an LF
    /// that begins the initializer suffix.
    #[test]
    fn relocated_cr_keeps_a_barrier_before_an_untouched_suffix_lf_at_the_cap() {
        let source = "export\rdefault\n42;\nconst after_boundary = 1;";
        let stripped = strip_module_syntax(
            source,
            DefaultExportRewrite::Bind {
                name: &LocalName::AnonymousDefault.merged_in(MAX_LINKABLE_MODULE_UNIT_ID),
                hoisted: false,
            },
        )
        .expect("edit-boundary CR and LF should stay separate");

        assert_eq!(stripped.len(), source.len());
        assert_eq!(
            stripped.find("after_boundary"),
            source.find("after_boundary")
        );
        assert_eq!(
            collect_ecmascript_line_terminator_sequences(&stripped),
            vec!["\r", "\n", "\n"]
        );
        assert_eq!(
            collect_ecmascript_line_terminator_sequences(&stripped),
            collect_ecmascript_line_terminator_sequences(source)
        );
        assert!(
            stripped.starts_with("let $d9999$=\r \n42;"),
            "got {stripped}"
        );
    }

    /// Both the lookahead scanner inside an export and the top-level scanner
    /// must end `//` at every ECMAScript LineTerminatorSequence, not only LF.
    #[test]
    fn line_comments_end_at_cr_ls_and_ps_around_a_split_default() {
        for terminator in ["\r", "\u{2028}", "\u{2029}"] {
            let source = format!(
                "// leading{terminator}export// between{terminator}default 42;\nconst after = 1;"
            );
            let stripped = strip_module_syntax(
                &source,
                DefaultExportRewrite::Bind {
                    name: &LocalName::AnonymousDefault.merged_in(0),
                    hoisted: false,
                },
            )
            .expect("line comments should stop at an ECMAScript line terminator");

            assert_eq!(stripped.len(), source.len(), "{terminator:?}");
            assert_eq!(stripped.find("after"), source.find("after"));
            assert!(stripped.contains("let $d0$"), "got {stripped}");
            assert_eq!(
                collect_ecmascript_line_terminator_sequences(&stripped),
                collect_ecmascript_line_terminator_sequences(&source),
                "got {stripped}"
            );
        }
    }

    #[test]
    fn a_generated_replacement_cannot_add_a_line_terminator() {
        assert!(matches!(
            SpanStableReplacement::around_padding("export default", "", "let $d0$\n", "="),
            Err(SpanStableReplacementError::GeneratedLineTerminator)
        ));
    }

    /// A hoistable anonymous default keeps being initialized before the body
    /// runs, which `let` would have replaced with a TDZ.
    #[test]
    fn a_hoistable_anonymous_default_is_declared_with_var() {
        let stripped = strip_module_syntax(
            "export default function () {}",
            DefaultExportRewrite::Bind {
                name: &LocalName::AnonymousDefault.merged_in(3),
                hoisted: true,
            },
        )
        .expect("source should strip");
        assert_eq!(stripped, "var $d3$     = function () {}");
    }

    /// `export default function f() {}` already binds `f`, so the keywords are
    /// simply deleted and the declaration stays a declaration.
    #[test]
    fn a_named_export_default_only_loses_its_keywords() {
        let source = "export default function f() {}\n";
        let stripped = strip_module_syntax(source, DefaultExportRewrite::DeleteKeywords)
            .expect("source should strip");
        assert_eq!(stripped.len(), source.len());
        assert_eq!(stripped, "               function f() {}\n");
    }

    /// A name too long for the keywords it replaces is reported rather than
    /// emitted at the wrong length.
    ///
    /// Reaching this needs a unit id past
    /// [`MAX_LINKABLE_MODULE_UNIT_ID`](crate::MAX_LINKABLE_MODULE_UNIT_ID),
    /// which `build_graph` refuses to mint (ledger R3) and which const
    /// assertion V2 pins the format side of. The check stays because the *span*
    /// is data — `export  default` with two spaces is wider than the minimum,
    /// and a hypothetical narrower one would be caught here rather than
    /// silently emitted at the wrong length.
    #[test]
    fn a_default_binding_that_does_not_fit_is_reported() {
        let over_cap = LocalName::AnonymousDefault.merged_in(1_234_567_890);
        let error = strip_module_syntax(
            "export default 1;",
            DefaultExportRewrite::Bind {
                name: &over_cap,
                hoisted: false,
            },
        )
        .expect_err("must be reported");
        assert!(error.reason.contains("does not fit"), "{}", error.reason);
    }

    #[test]
    fn export_star_from_is_deleted_whole() {
        let source = "export * from \"./m.mjs\";\nprint(1);\n";
        let stripped = strip(source);
        assert_eq!(stripped.len(), source.len());
        assert!(!stripped.contains("export"), "got {stripped}");
        assert!(stripped.ends_with("print(1);\n"));
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
