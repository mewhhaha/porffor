use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::process;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum ObservationCategory {
    PathRewriteEntrypoint,
    DirectPathPredicate,
    SourceTextPredicate,
    HarnessHelperReduction,
}

impl ObservationCategory {
    fn as_str(self) -> &'static str {
        match self {
            Self::PathRewriteEntrypoint => "path-rewrite-entrypoint",
            Self::DirectPathPredicate => "direct-path-predicate",
            Self::SourceTextPredicate => "source-text-predicate",
            Self::HarnessHelperReduction => "harness-helper-reduction",
        }
    }

    fn source_order(self) -> u8 {
        match self {
            Self::PathRewriteEntrypoint => 0,
            Self::DirectPathPredicate => 1,
            Self::SourceTextPredicate => 2,
            Self::HarnessHelperReduction => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StringLiteralKind {
    Normal,
    Byte,
    Raw,
    RawByte,
    CString,
    RawCString,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenKind {
    Identifier,
    StringLiteral(StringLiteralKind),
    CharacterLiteral,
    Punctuation(u8),
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ByteSpan {
    start: usize,
    end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
    span: ByteSpan,
    line: usize,
}

impl Token {
    fn text<'source>(self, source: &'source str) -> &'source str {
        &source[self.span.start..self.span.end]
    }

    fn punctuation(self) -> Option<u8> {
        match self.kind {
            TokenKind::Punctuation(value) => Some(value),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct ScanError(String);

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ObservationIdentity {
    EnclosingDeclaration,
    RewriteEntrypoint(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingObservation {
    category: ObservationCategory,
    span: ByteSpan,
    evidence_override: Option<String>,
    source_offset: usize,
    line: usize,
    identity: ObservationIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AnchorRegion {
    span: ByteSpan,
    name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PathMatchRegion {
    match_index: usize,
    scrutinee_end_index: usize,
    body_close_index: usize,
    evidence: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PathMacroRegion {
    macro_index: usize,
    argument_end_index: usize,
    close_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RenderedObservation {
    stable_key: String,
    line: usize,
    anchor: String,
    category: ObservationCategory,
    evidence: String,
}

fn is_identifier_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic()
}

fn is_identifier_continue(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric()
}

fn normal_string_end(source: &str, start: usize, quote_offset: usize) -> Result<usize, ScanError> {
    let bytes = source.as_bytes();
    let mut cursor = quote_offset + 1;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => {
                cursor += 1;
                if cursor < bytes.len() {
                    cursor += 1;
                }
            }
            b'"' => return Ok(cursor + 1),
            _ => cursor += 1,
        }
    }
    Err(ScanError(format!(
        "unterminated string literal beginning at byte {start}"
    )))
}

fn raw_string_end(
    source: &str,
    start: usize,
    raw_prefix_offset: usize,
) -> Result<Option<usize>, ScanError> {
    let bytes = source.as_bytes();
    let mut cursor = raw_prefix_offset + 1;
    let mut hashes = 0;
    while cursor < bytes.len() && bytes[cursor] == b'#' {
        hashes += 1;
        cursor += 1;
    }
    if cursor >= bytes.len() || bytes[cursor] != b'"' {
        return Ok(None);
    }
    cursor += 1;
    while cursor < bytes.len() {
        if bytes[cursor] != b'"' {
            cursor += 1;
            continue;
        }
        let hashes_end = cursor + 1 + hashes;
        if hashes_end <= bytes.len()
            && bytes[cursor + 1..hashes_end]
                .iter()
                .all(|byte| *byte == b'#')
        {
            return Ok(Some(hashes_end));
        }
        cursor += 1;
    }
    Err(ScanError(format!(
        "unterminated raw string literal beginning at byte {start}"
    )))
}

fn character_literal_end(source: &str, quote_offset: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let body_start = quote_offset + 1;
    if body_start >= bytes.len() || matches!(bytes[body_start], b'\n' | b'\r' | b'\'') {
        return None;
    }

    let body_end = if bytes[body_start] == b'\\' {
        let escape = *bytes.get(body_start + 1)?;
        match escape {
            b'x' => body_start.checked_add(4)?,
            b'u' => {
                if bytes.get(body_start + 2) != Some(&b'{') {
                    return None;
                }
                let close = bytes[body_start + 3..]
                    .iter()
                    .position(|byte| *byte == b'}')?;
                body_start + 4 + close
            }
            _ => body_start + 2,
        }
    } else {
        let character = source[body_start..].chars().next()?;
        body_start + character.len_utf8()
    };
    (bytes.get(body_end) == Some(&b'\'')).then_some(body_end + 1)
}

fn lex(source: &str) -> Result<Vec<Token>, ScanError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut cursor = 0;
    let mut line = 1;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b' ' | b'\t' | b'\r' => {
                cursor += 1;
                continue;
            }
            b'\n' => {
                cursor += 1;
                line += 1;
                continue;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'/') => {
                cursor += 2;
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                continue;
            }
            b'/' if bytes.get(cursor + 1) == Some(&b'*') => {
                let comment_start = cursor;
                cursor += 2;
                let mut depth = 1usize;
                while cursor < bytes.len() && depth != 0 {
                    if bytes.get(cursor..cursor + 2) == Some(b"/*") {
                        depth += 1;
                        cursor += 2;
                    } else if bytes.get(cursor..cursor + 2) == Some(b"*/") {
                        depth -= 1;
                        cursor += 2;
                    } else {
                        if bytes[cursor] == b'\n' {
                            line += 1;
                        }
                        cursor += 1;
                    }
                }
                if depth != 0 {
                    return Err(ScanError(format!(
                        "unterminated block comment beginning at byte {comment_start}"
                    )));
                }
                continue;
            }
            _ => {}
        }

        let start = cursor;
        let token_line = line;
        let (kind, end) = if bytes[cursor] == b'b' && bytes.get(cursor + 1) == Some(&b'r') {
            match raw_string_end(source, start, cursor + 1)? {
                Some(end) => (TokenKind::StringLiteral(StringLiteralKind::RawByte), end),
                None => lex_identifier(source, cursor),
            }
        } else if bytes[cursor] == b'c' && bytes.get(cursor + 1) == Some(&b'r') {
            match raw_string_end(source, start, cursor + 1)? {
                Some(end) => (TokenKind::StringLiteral(StringLiteralKind::RawCString), end),
                None => lex_identifier(source, cursor),
            }
        } else if bytes[cursor] == b'r' {
            match raw_string_end(source, start, cursor)? {
                Some(end) => (TokenKind::StringLiteral(StringLiteralKind::Raw), end),
                None => lex_identifier(source, cursor),
            }
        } else if bytes[cursor] == b'b' && bytes.get(cursor + 1) == Some(&b'"') {
            (
                TokenKind::StringLiteral(StringLiteralKind::Byte),
                normal_string_end(source, start, cursor + 1)?,
            )
        } else if bytes[cursor] == b'c' && bytes.get(cursor + 1) == Some(&b'"') {
            (
                TokenKind::StringLiteral(StringLiteralKind::CString),
                normal_string_end(source, start, cursor + 1)?,
            )
        } else if bytes[cursor] == b'"' {
            (
                TokenKind::StringLiteral(StringLiteralKind::Normal),
                normal_string_end(source, start, cursor)?,
            )
        } else if bytes[cursor] == b'b' && bytes.get(cursor + 1) == Some(&b'\'') {
            let Some(end) = character_literal_end(source, cursor + 1) else {
                return Err(ScanError(format!(
                    "invalid byte character literal beginning at byte {start}"
                )));
            };
            (TokenKind::CharacterLiteral, end)
        } else if bytes[cursor] == b'\'' {
            match character_literal_end(source, cursor) {
                Some(end) => (TokenKind::CharacterLiteral, end),
                None => (TokenKind::Punctuation(b'\''), cursor + 1),
            }
        } else if is_identifier_start(bytes[cursor]) {
            lex_identifier(source, cursor)
        } else if bytes[cursor].is_ascii_digit() {
            (TokenKind::Other, numeric_literal_end(source, cursor))
        } else if bytes[cursor].is_ascii_punctuation() {
            (TokenKind::Punctuation(bytes[cursor]), cursor + 1)
        } else {
            let character = source[cursor..]
                .chars()
                .next()
                .expect("cursor is within source");
            (TokenKind::Other, cursor + character.len_utf8())
        };

        line += source[start..end]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count();
        tokens.push(Token {
            kind,
            span: ByteSpan { start, end },
            line: token_line,
        });
        cursor = end;
    }

    Ok(tokens)
}

fn lex_identifier(source: &str, start: usize) -> (TokenKind, usize) {
    let bytes = source.as_bytes();
    let mut cursor = start + 1;
    if bytes[start] == b'r'
        && bytes.get(cursor) == Some(&b'#')
        && bytes
            .get(cursor + 1)
            .is_some_and(|byte| is_identifier_start(*byte))
    {
        cursor += 1;
    }
    while cursor < bytes.len() && is_identifier_continue(bytes[cursor]) {
        cursor += 1;
    }
    (TokenKind::Identifier, cursor)
}

fn numeric_literal_end(source: &str, start: usize) -> usize {
    let bytes = source.as_bytes();
    let mut cursor = start;
    let radix = if bytes.get(start..start + 2) == Some(b"0x") {
        cursor += 2;
        16
    } else if bytes.get(start..start + 2) == Some(b"0o") {
        cursor += 2;
        8
    } else if bytes.get(start..start + 2) == Some(b"0b") {
        cursor += 2;
        2
    } else {
        10
    };
    while cursor < bytes.len()
        && (bytes[cursor] == b'_'
            || match radix {
                16 => bytes[cursor].is_ascii_hexdigit(),
                10 => bytes[cursor].is_ascii_digit(),
                8 => matches!(bytes[cursor], b'0'..=b'7'),
                2 => matches!(bytes[cursor], b'0' | b'1'),
                _ => unreachable!(),
            })
    {
        cursor += 1;
    }
    if radix == 10
        && bytes.get(cursor) == Some(&b'.')
        && bytes.get(cursor + 1).is_some_and(u8::is_ascii_digit)
    {
        cursor += 1;
        while cursor < bytes.len() && (bytes[cursor].is_ascii_digit() || bytes[cursor] == b'_') {
            cursor += 1;
        }
    }
    if radix == 10 && matches!(bytes.get(cursor), Some(b'e' | b'E')) {
        let exponent_start = cursor;
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        let digits_start = cursor;
        while cursor < bytes.len() && (bytes[cursor].is_ascii_digit() || bytes[cursor] == b'_') {
            cursor += 1;
        }
        if cursor == digits_start {
            cursor = exponent_start;
        }
    }
    const SUFFIXES: [&[u8]; 14] = [
        b"usize", b"isize", b"u128", b"i128", b"u64", b"i64", b"u32", b"i32", b"f64", b"f32",
        b"u16", b"i16", b"u8", b"i8",
    ];
    if let Some(suffix) = SUFFIXES.iter().find(|suffix| {
        bytes.get(cursor..cursor + suffix.len()) == Some(**suffix)
            && bytes
                .get(cursor + suffix.len())
                .map_or(true, |byte| !is_identifier_continue(*byte))
    }) {
        cursor += suffix.len();
    }
    cursor
}

fn production_token_count(tokens: &[Token], source: &str) -> usize {
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        let cfg_test_attribute = index >= 7
            && tokens[index - 7].punctuation() == Some(b'#')
            && tokens[index - 6].punctuation() == Some(b'[')
            && tokens[index - 5].text(source) == "cfg"
            && tokens[index - 4].punctuation() == Some(b'(')
            && tokens[index - 3].text(source) == "test"
            && tokens[index - 2].punctuation() == Some(b')')
            && tokens[index - 1].punctuation() == Some(b']');
        if brace_depth == 0
            && cfg_test_attribute
            && token.text(source) == "mod"
            && tokens
                .get(index + 1)
                .is_some_and(|next| next.text(source) == "tests")
            && tokens
                .get(index + 2)
                .is_some_and(|next| next.punctuation() == Some(b'{'))
        {
            return index;
        }
        match token.punctuation() {
            Some(b'{') => brace_depth += 1,
            Some(b'}') => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }
    tokens.len()
}

fn delimiter_pairs(tokens: &[Token]) -> Result<Vec<Option<usize>>, ScanError> {
    let mut pairs = vec![None; tokens.len()];
    let mut stack: Vec<(u8, usize)> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token.punctuation() {
            Some(open @ (b'(' | b'[' | b'{')) => stack.push((open, index)),
            Some(close @ (b')' | b']' | b'}')) => {
                let Some((open, open_index)) = stack.pop() else {
                    return Err(ScanError(format!(
                        "unmatched closing delimiter at line {}",
                        token.line
                    )));
                };
                let expected = match open {
                    b'(' => b')',
                    b'[' => b']',
                    b'{' => b'}',
                    _ => unreachable!(),
                };
                if close != expected {
                    return Err(ScanError(format!(
                        "mismatched delimiter at line {}",
                        token.line
                    )));
                }
                pairs[open_index] = Some(index);
                pairs[index] = Some(open_index);
            }
            _ => {}
        }
    }
    if let Some((_, index)) = stack.last() {
        return Err(ScanError(format!(
            "unclosed delimiter at line {}",
            tokens[*index].line
        )));
    }
    Ok(pairs)
}

fn is_standalone_identifier(tokens: &[Token], index: usize) -> bool {
    let member_qualified = index >= 1
        && tokens[index - 1].punctuation() == Some(b'.')
        && !(index >= 2 && tokens[index - 2].punctuation() == Some(b'.'));
    let namespace_qualified = index >= 2
        && tokens[index - 1].punctuation() == Some(b':')
        && tokens[index - 2].punctuation() == Some(b':');
    !member_qualified && !namespace_qualified
}

fn exact_field_receiver_start(
    tokens: &[Token],
    source: &str,
    dot_index: usize,
    root: &str,
    field: Option<&str>,
) -> Option<usize> {
    if field.is_none()
        && dot_index >= 1
        && tokens[dot_index - 1].text(source) == root
        && is_standalone_identifier(tokens, dot_index - 1)
    {
        return Some(dot_index - 1);
    }
    let field = field?;
    if dot_index >= 3
        && tokens[dot_index - 3].text(source) == root
        && tokens[dot_index - 2].punctuation() == Some(b'.')
        && tokens[dot_index - 1].text(source) == field
        && is_standalone_identifier(tokens, dot_index - 3)
    {
        return Some(dot_index - 3);
    }
    None
}

fn source_receiver_root(
    tokens: &[Token],
    source: &str,
    pairs: &[Option<usize>],
    dot_index: usize,
) -> Option<usize> {
    for (root, field) in [
        ("source", None),
        ("original_source", None),
        ("case", Some("original_source")),
    ] {
        if let Some(start) = exact_field_receiver_start(tokens, source, dot_index, root, field) {
            return Some(start);
        }
    }

    let receiver_close = dot_index.checked_sub(1)?;
    if tokens[receiver_close].punctuation() != Some(b')') {
        return None;
    }
    let call_open = pairs[receiver_close]?;
    let prior_method = call_open.checked_sub(1)?;
    let prior_dot = prior_method.checked_sub(1)?;
    if tokens[prior_method].kind != TokenKind::Identifier
        || tokens[prior_dot].punctuation() != Some(b'.')
    {
        return None;
    }
    source_receiver_root(tokens, source, pairs, prior_dot)
}

fn call_close(tokens: &[Token], pairs: &[Option<usize>], method_index: usize) -> Option<usize> {
    let open_index = method_index + 1;
    (tokens.get(open_index)?.punctuation() == Some(b'('))
        .then(|| pairs[open_index])
        .flatten()
}

fn is_rewrite_callee(identifier: &str) -> bool {
    identifier.starts_with("rewrite_") || identifier == "wasm_aot_rewrite_skips_test_typed_array"
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SelectorArgument {
    Case,
    CasePath,
    Path,
}

fn sole_selector_argument(tokens: &[Token], source: &str) -> Option<SelectorArgument> {
    let mut arguments = tokens
        .last()
        .is_some_and(|token| token.punctuation() == Some(b','))
        .then(|| &tokens[..tokens.len() - 1])
        .unwrap_or(tokens);
    loop {
        if arguments.len() < 2
            || arguments[0].punctuation() != Some(b'(')
            || arguments.last()?.punctuation() != Some(b')')
        {
            break;
        }
        let mut depth = 0usize;
        let mut encloses_all_arguments = true;
        for (index, token) in arguments.iter().enumerate() {
            match token.punctuation() {
                Some(b'(') => depth += 1,
                Some(b')') => depth = depth.checked_sub(1)?,
                _ => {}
            }
            if depth == 0 && index + 1 != arguments.len() {
                encloses_all_arguments = false;
                break;
            }
        }
        if !encloses_all_arguments || depth != 0 {
            break;
        }
        arguments = &arguments[1..arguments.len() - 1];
    }
    match arguments {
        [case] if case.text(source) == "case" => Some(SelectorArgument::Case),
        [path] if path.text(source) == "path" => Some(SelectorArgument::Path),
        [reference, case, dot, path]
            if reference.punctuation() == Some(b'&')
                && case.text(source) == "case"
                && dot.punctuation() == Some(b'.')
                && path.text(source) == "path" =>
        {
            Some(SelectorArgument::CasePath)
        }
        _ => None,
    }
}

fn path_selector_chain_end(
    tokens: &[Token],
    source: &str,
    pairs: &[Option<usize>],
    mut close_index: usize,
) -> usize {
    loop {
        let dot_index = close_index + 1;
        let method_index = close_index + 2;
        if tokens.get(dot_index).and_then(|token| token.punctuation()) != Some(b'.') {
            return close_index;
        }
        let Some(method) = tokens.get(method_index) else {
            return close_index;
        };
        if !matches!(
            method.text(source),
            "next" | "unwrap_or" | "unwrap_or_default"
        ) {
            return close_index;
        }
        let Some(next_close) = call_close(tokens, pairs, method_index) else {
            return close_index;
        };
        close_index = next_close;
    }
}

const FREE_PATH_SELECTORS: [&str; 5] = [
    "typed_array_literal_method_and_file",
    "dataview_constructor_file",
    "dataview_method_for_path",
    "supported_wasm_aot_shared_array_buffer_metadata_case",
    "supported_wasm_aot_atomics_shared_array_buffer_case",
];

const PATH_ALIAS_MATCHES: [(&str, &str); 4] = [
    ("rewrite_iterator_filter_case", "leaf"),
    ("rewrite_iterator_flat_map_case", "leaf"),
    ("rewrite_dataview_constructor_case", "file"),
    ("dataview_method_range_info", "method"),
];

const NON_TEST_SOURCE_DECLARATIONS: [&str; 2] =
    ["fnv1a", "rewrite_array_iteration_resizable_buffer_case"];

fn path_receiver_at(tokens: &[Token], source: &str, index: usize) -> Option<(usize, usize)> {
    if tokens.get(index)?.text(source) == "path" && is_standalone_identifier(tokens, index) {
        return Some((index, index + 1));
    }
    if tokens[index].text(source) == "case"
        && is_standalone_identifier(tokens, index)
        && tokens.get(index + 1)?.punctuation() == Some(b'.')
        && tokens.get(index + 2)?.text(source) == "path"
    {
        return Some((index, index + 3));
    }
    None
}

fn find_match_scrutinee_end(
    tokens: &[Token],
    pairs: &[Option<usize>],
    start: usize,
) -> Option<usize> {
    let mut cursor = start;
    while cursor < tokens.len() {
        match tokens[cursor].punctuation() {
            Some(b'(' | b'[') => cursor = pairs[cursor]? + 1,
            Some(b'{') => return cursor.checked_sub(1),
            _ => cursor += 1,
        }
    }
    None
}

fn match_arm_pattern_spans(
    tokens: &[Token],
    pairs: &[Option<usize>],
    body_open_index: usize,
) -> Option<Vec<(usize, usize)>> {
    let body_close_index = pairs[body_open_index]?;
    let mut patterns = Vec::new();
    let mut cursor = body_open_index + 1;

    while cursor < body_close_index {
        while cursor < body_close_index && tokens[cursor].punctuation() == Some(b',') {
            cursor += 1;
        }
        if cursor >= body_close_index {
            break;
        }

        let pattern_start = cursor;
        let arrow_index = loop {
            if cursor + 1 >= body_close_index {
                return None;
            }
            if tokens[cursor].punctuation() == Some(b'=')
                && tokens[cursor + 1].punctuation() == Some(b'>')
            {
                break cursor;
            }
            match tokens[cursor].punctuation() {
                Some(b',') => return None,
                Some(b'(' | b'[' | b'{') => cursor = pairs[cursor]? + 1,
                _ => cursor += 1,
            }
        };
        let pattern_end = arrow_index.checked_sub(1)?;
        patterns.push((pattern_start, pattern_end));
        cursor = arrow_index + 2;

        if tokens
            .get(cursor)
            .and_then(|body_start| body_start.punctuation())
            == Some(b'{')
        {
            cursor = pairs[cursor]? + 1;
            if tokens.get(cursor).and_then(|comma| comma.punctuation()) == Some(b',') {
                cursor += 1;
            }
            continue;
        }

        while cursor < body_close_index {
            if cursor + 1 < body_close_index
                && tokens[cursor].punctuation() == Some(b'=')
                && tokens[cursor + 1].punctuation() == Some(b'>')
            {
                return None;
            }
            match tokens[cursor].punctuation() {
                Some(b',') => {
                    cursor += 1;
                    break;
                }
                Some(b'(' | b'[' | b'{') => cursor = pairs[cursor]? + 1,
                _ => cursor += 1,
            }
        }
    }

    Some(patterns)
}

fn path_match_regions(
    source: &str,
    tokens: &[Token],
    pairs: &[Option<usize>],
    anchors: &[AnchorRegion],
) -> Result<Vec<PathMatchRegion>, ScanError> {
    let mut regions = Vec::new();
    for (match_index, token) in tokens.iter().enumerate() {
        if token.text(source) != "match" {
            continue;
        }
        let direct_path = path_receiver_at(tokens, source, match_index + 1).is_some();
        let alias = tokens.get(match_index + 1).map(|token| token.text(source));
        let anchor = anchor_name_at(anchors, token.span.start).unwrap_or("module");
        let admitted_alias = alias.is_some_and(|alias| {
            PATH_ALIAS_MATCHES
                .iter()
                .any(|candidate| *candidate == (anchor, alias))
        });
        if !direct_path && !admitted_alias {
            continue;
        }
        let scrutinee_end_index = find_match_scrutinee_end(tokens, pairs, match_index + 1)
            .ok_or_else(|| {
                ScanError(format!(
                    "cannot find execution-selector match body at line {}",
                    token.line
                ))
            })?;
        let body_open_index = scrutinee_end_index + 1;
        if tokens
            .get(body_open_index)
            .and_then(|body_open| body_open.punctuation())
            != Some(b'{')
        {
            return Err(ScanError(format!(
                "execution-selector match has no braced body at line {}",
                token.line
            )));
        }
        let body_close_index = pairs[body_open_index].ok_or_else(|| {
            ScanError(format!(
                "execution-selector match has an unclosed body at line {}",
                token.line
            ))
        })?;
        let patterns =
            match_arm_pattern_spans(tokens, pairs, body_open_index).ok_or_else(|| {
                ScanError(format!(
                    "unsupported execution-selector match arm grammar at line {}",
                    token.line
                ))
            })?;

        let mut evidence =
            source[token.span.start..tokens[scrutinee_end_index].span.end].to_owned();
        evidence.push_str(" {");
        for (pattern_start, pattern_end) in patterns {
            evidence.push_str("\n  ");
            evidence
                .push_str(&source[tokens[pattern_start].span.start..tokens[pattern_end].span.end]);
            evidence.push_str(" => <body>");
        }
        evidence.push_str("\n}");
        regions.push(PathMatchRegion {
            match_index,
            scrutinee_end_index,
            body_close_index,
            evidence,
        });
    }
    Ok(regions)
}

fn path_macro_regions(
    source: &str,
    tokens: &[Token],
    pairs: &[Option<usize>],
) -> Result<Vec<PathMacroRegion>, ScanError> {
    let mut regions = Vec::new();
    for (macro_index, token) in tokens.iter().enumerate() {
        if token.text(source) != "matches"
            || !is_standalone_identifier(tokens, macro_index)
            || tokens
                .get(macro_index + 1)
                .and_then(|bang| bang.punctuation())
                != Some(b'!')
            || tokens
                .get(macro_index + 2)
                .and_then(|open| open.punctuation())
                != Some(b'(')
        {
            continue;
        }
        let close_index = pairs[macro_index + 2].ok_or_else(|| {
            ScanError(format!(
                "execution-selector matches! macro is unclosed at line {}",
                token.line
            ))
        })?;
        let argument_start = macro_index + 3;
        let Some((_, mut after_argument)) = path_receiver_at(tokens, source, argument_start) else {
            continue;
        };
        if tokens.get(after_argument).and_then(|dot| dot.punctuation()) == Some(b'.')
            && tokens
                .get(after_argument + 1)
                .is_some_and(|method| method.text(source) == "as_str")
        {
            let method_close = call_close(tokens, pairs, after_argument + 1).ok_or_else(|| {
                ScanError(format!(
                    "execution-selector matches! method is unclosed at line {}",
                    token.line
                ))
            })?;
            after_argument = method_close + 1;
        }
        if tokens
            .get(after_argument)
            .and_then(|comma| comma.punctuation())
            != Some(b',')
        {
            return Err(ScanError(format!(
                "execution-selector matches! has no pattern separator at line {}",
                token.line
            )));
        }
        let built_ins_literal = tokens[after_argument + 1..close_index]
            .iter()
            .any(|argument| {
                argument.kind == TokenKind::StringLiteral(StringLiteralKind::Normal)
                    && argument.text(source).starts_with("\"built-ins/")
            });
        if built_ins_literal {
            regions.push(PathMacroRegion {
                macro_index,
                argument_end_index: after_argument - 1,
                close_index,
            });
        }
    }
    Ok(regions)
}

fn declaration_end(
    tokens: &[Token],
    pairs: &[Option<usize>],
    start: usize,
    keyword: &str,
) -> Option<usize> {
    let semicolon_terminated = matches!(keyword, "const" | "type");
    let mut cursor = start + 2;
    while cursor < tokens.len() {
        match tokens[cursor].punctuation() {
            Some(b';') => return Some(cursor),
            Some(b'{' | b'(' | b'[') if semicolon_terminated => {
                cursor = pairs[cursor]? + 1;
            }
            Some(b'(' | b'[') => cursor = pairs[cursor]? + 1,
            Some(b'{') => return pairs[cursor],
            _ => cursor += 1,
        }
    }
    None
}

fn declaration_anchors(
    tokens: &[Token],
    source: &str,
    pairs: &[Option<usize>],
) -> Vec<AnchorRegion> {
    let mut regions = Vec::new();
    let mut brace_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == TokenKind::Identifier {
            let keyword = token.text(source);
            let declares_function = keyword == "fn";
            let declares_top_level_item = brace_depth == 0
                && matches!(
                    keyword,
                    "const" | "static" | "struct" | "enum" | "union" | "trait" | "type"
                );
            let const_generic = keyword == "const"
                && index > 0
                && matches!(tokens[index - 1].punctuation(), Some(b'<' | b','));
            let const_function = keyword == "const"
                && tokens
                    .get(index + 1)
                    .is_some_and(|next| next.text(source) == "fn");
            let static_lifetime =
                keyword == "static" && index >= 1 && tokens[index - 1].punctuation() == Some(b'\'');
            if (declares_function || declares_top_level_item)
                && !const_generic
                && !const_function
                && !static_lifetime
            {
                if let Some(name) = tokens.get(index + 1) {
                    if name.kind == TokenKind::Identifier {
                        if let Some(end_index) = declaration_end(tokens, pairs, index, keyword) {
                            regions.push(AnchorRegion {
                                span: ByteSpan {
                                    start: token.span.start,
                                    end: tokens[end_index].span.end,
                                },
                                name: name.text(source).to_owned(),
                            });
                        }
                    }
                }
            }
        }
        match token.punctuation() {
            Some(b'{') => brace_depth += 1,
            Some(b'}') => brace_depth = brace_depth.saturating_sub(1),
            _ => {}
        }
    }
    regions
}

fn anchor_name_at<'regions>(
    regions: &'regions [AnchorRegion],
    offset: usize,
) -> Option<&'regions str> {
    regions
        .iter()
        .filter(|region| region.span.start <= offset && offset < region.span.end)
        .max_by_key(|region| region.span.start)
        .map(|region| region.name.as_str())
}

fn anchor_at(regions: &[AnchorRegion], offset: usize) -> String {
    anchor_name_at(regions, offset).map_or_else(|| "module".to_owned(), str::to_owned)
}

fn add_declaration_observation(
    observations: &mut Vec<PendingObservation>,
    category: ObservationCategory,
    tokens: &[Token],
    start_index: usize,
    end_index: usize,
) {
    observations.push(PendingObservation {
        category,
        span: ByteSpan {
            start: tokens[start_index].span.start,
            end: tokens[end_index].span.end,
        },
        evidence_override: None,
        source_offset: tokens[start_index].span.start,
        line: tokens[start_index].line,
        identity: ObservationIdentity::EnclosingDeclaration,
    });
}

fn lexical_observations(
    source: &str,
    tokens: &[Token],
    pairs: &[Option<usize>],
    anchors: &[AnchorRegion],
) -> Result<Vec<PendingObservation>, ScanError> {
    let mut observations = Vec::new();
    let path_matches = path_match_regions(source, tokens, pairs, anchors)?;
    let path_macros = path_macro_regions(source, tokens, pairs)?;
    for path_match in &path_matches {
        let match_token = tokens[path_match.match_index];
        observations.push(PendingObservation {
            category: ObservationCategory::DirectPathPredicate,
            span: ByteSpan {
                start: match_token.span.start,
                end: tokens[path_match.body_close_index].span.end,
            },
            evidence_override: Some(path_match.evidence.clone()),
            source_offset: match_token.span.start,
            line: match_token.line,
            identity: ObservationIdentity::EnclosingDeclaration,
        });
    }
    for path_macro in &path_macros {
        add_declaration_observation(
            &mut observations,
            ObservationCategory::DirectPathPredicate,
            tokens,
            path_macro.macro_index,
            path_macro.close_index,
        );
    }

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != TokenKind::Identifier {
            continue;
        }
        let text = token.text(source);

        if is_rewrite_callee(text) && is_standalone_identifier(tokens, index) {
            if tokens
                .get(index + 1)
                .is_some_and(|next| next.punctuation() == Some(b'('))
            {
                if let Some(close_index) = pairs[index + 1] {
                    let arguments = &tokens[index + 2..close_index];
                    if matches!(
                        sole_selector_argument(arguments, source),
                        Some(SelectorArgument::Case | SelectorArgument::CasePath)
                    ) {
                        observations.push(PendingObservation {
                            category: ObservationCategory::PathRewriteEntrypoint,
                            span: ByteSpan {
                                start: token.span.start,
                                end: tokens[close_index].span.end,
                            },
                            evidence_override: None,
                            source_offset: token.span.start,
                            line: token.line,
                            identity: ObservationIdentity::RewriteEntrypoint(text.to_owned()),
                        });
                    }
                }
            }
        }
        if FREE_PATH_SELECTORS.contains(&text)
            && is_standalone_identifier(tokens, index)
            && tokens
                .get(index + 1)
                .is_some_and(|next| next.punctuation() == Some(b'('))
        {
            if let Some(close_index) = pairs[index + 1] {
                let arguments = &tokens[index + 2..close_index];
                if matches!(
                    sole_selector_argument(arguments, source),
                    Some(SelectorArgument::CasePath | SelectorArgument::Path)
                ) {
                    let selector_end = if tokens
                        .get(close_index + 1)
                        .and_then(|question| question.punctuation())
                        == Some(b'?')
                    {
                        close_index + 1
                    } else {
                        close_index
                    };
                    add_declaration_observation(
                        &mut observations,
                        ObservationCategory::DirectPathPredicate,
                        tokens,
                        index,
                        selector_end,
                    );
                }
            }
        }

        if matches!(
            text,
            "starts_with"
                | "ends_with"
                | "contains"
                | "as_str"
                | "as_bytes"
                | "split"
                | "strip_prefix"
                | "rsplit"
        ) && index >= 1
            && tokens[index - 1].punctuation() == Some(b'.')
        {
            let local_path_start =
                exact_field_receiver_start(tokens, source, index - 1, "path", None);
            let case_path_start =
                exact_field_receiver_start(tokens, source, index - 1, "case", Some("path"));
            let start_index = local_path_start.or(case_path_start);
            if let (Some(start_index), Some(close_index)) =
                (start_index, call_close(tokens, pairs, index))
            {
                let arguments = &tokens[index + 2..close_index];
                let empty_arguments = arguments.is_empty();
                let slash_argument = arguments.len() == 1
                    && arguments[0].kind == TokenKind::CharacterLiteral
                    && arguments[0].text(source) == "'/'";
                let built_ins_argument = arguments.first().is_some_and(|argument| {
                    argument.kind == TokenKind::StringLiteral(StringLiteralKind::Normal)
                        && argument.text(source).starts_with("\"built-ins/")
                }) && matches!(arguments.len(), 1 | 2)
                    && (arguments.len() == 1 || arguments[1].punctuation() == Some(b','));
                let admitted = match text {
                    "as_str" => empty_arguments,
                    "as_bytes" => case_path_start.is_some() && empty_arguments,
                    "split" => case_path_start.is_some() && slash_argument,
                    "strip_prefix" => built_ins_argument,
                    "rsplit" => slash_argument,
                    _ => true,
                };
                let inside_path_match = path_matches.iter().any(|path_match| {
                    path_match.match_index < start_index
                        && close_index <= path_match.scrutinee_end_index
                });
                let inside_path_macro = path_macros.iter().any(|path_macro| {
                    path_macro.macro_index < start_index
                        && close_index <= path_macro.argument_end_index
                });
                if admitted && !inside_path_match && !inside_path_macro {
                    let mut selector_end = if text == "rsplit" {
                        path_selector_chain_end(tokens, source, pairs, close_index)
                    } else {
                        close_index
                    };
                    if tokens
                        .get(selector_end + 1)
                        .and_then(|question| question.punctuation())
                        == Some(b'?')
                    {
                        selector_end += 1;
                    }
                    add_declaration_observation(
                        &mut observations,
                        ObservationCategory::DirectPathPredicate,
                        tokens,
                        start_index,
                        selector_end,
                    );
                }
            }
        }

        if matches!(
            text,
            "contains" | "replace" | "replacen" | "as_bytes" | "fold" | "count"
        ) && index >= 1
            && tokens[index - 1].punctuation() == Some(b'.')
        {
            let start_index = source_receiver_root(tokens, source, pairs, index - 1);
            if let (Some(start_index), Some(close_index)) =
                (start_index, call_close(tokens, pairs, index))
            {
                let anchor = anchor_name_at(anchors, token.span.start).unwrap_or("module");
                if !NON_TEST_SOURCE_DECLARATIONS.contains(&anchor) {
                    add_declaration_observation(
                        &mut observations,
                        ObservationCategory::SourceTextPredicate,
                        tokens,
                        start_index,
                        close_index,
                    );
                }
            }
        }

        if text == "source"
            && is_standalone_identifier(tokens, index)
            && tokens.get(index + 1).and_then(|open| open.punctuation()) == Some(b'[')
        {
            if let Some(close_index) = pairs[index + 1] {
                let range = &tokens[index + 2..close_index];
                if range.len() == 4
                    && range[0].text(source) == "identifier_start"
                    && range[1].punctuation() == Some(b'.')
                    && range[2].punctuation() == Some(b'.')
                    && range[3].text(source) == "idx"
                {
                    add_declaration_observation(
                        &mut observations,
                        ObservationCategory::SourceTextPredicate,
                        tokens,
                        index,
                        close_index,
                    );
                }
            }
        }

        if let Some((start_index, after_receiver)) = path_receiver_at(tokens, source, index) {
            let Some(first_operator) = tokens.get(after_receiver) else {
                continue;
            };
            let Some(second_equals) = tokens.get(after_receiver + 1) else {
                continue;
            };
            let Some(literal) = tokens.get(after_receiver + 2) else {
                continue;
            };
            if matches!(first_operator.punctuation(), Some(b'=' | b'!'))
                && second_equals.punctuation() == Some(b'=')
                && literal.kind == TokenKind::StringLiteral(StringLiteralKind::Normal)
                && literal.text(source).starts_with("\"built-ins/")
                && !path_matches.iter().any(|path_match| {
                    path_match.match_index < start_index
                        && after_receiver + 2 <= path_match.scrutinee_end_index
                })
            {
                add_declaration_observation(
                    &mut observations,
                    ObservationCategory::DirectPathPredicate,
                    tokens,
                    start_index,
                    after_receiver + 2,
                );
            }
        }
    }
    Ok(observations)
}

const HARNESS_HELPER_MARKERS: [&str; 6] = [
    "prelude.contents",
    "used_preludes",
    "helper used",
    "assert.sameValue = function",
    "assert.throws = function",
    "skips_test_typed_array",
];

fn harness_helper_observations(source: &str, production_end: usize) -> Vec<PendingObservation> {
    let mut observations = Vec::new();
    let mut line_start = 0usize;
    let mut line_number = 1usize;
    while line_start < production_end {
        let next_newline = source[line_start..production_end]
            .find('\n')
            .map(|offset| line_start + offset);
        let line_end = next_newline.unwrap_or(production_end);
        let line = &source[line_start..line_end];
        let marker_offset = HARNESS_HELPER_MARKERS
            .iter()
            .filter_map(|marker| line.find(marker))
            .min();
        if let Some(marker_offset) = marker_offset {
            observations.push(PendingObservation {
                category: ObservationCategory::HarnessHelperReduction,
                span: ByteSpan {
                    start: line_start,
                    end: line_end,
                },
                evidence_override: None,
                source_offset: line_start + marker_offset,
                line: line_number,
                identity: ObservationIdentity::EnclosingDeclaration,
            });
        }
        let Some(newline) = next_newline else {
            break;
        };
        line_start = newline + 1;
        line_number += 1;
    }
    observations
}

fn escape_evidence(evidence: &str) -> String {
    let mut escaped = String::with_capacity(evidence.len());
    for character in evidence.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(character),
        }
    }
    escaped
}

fn render_observations(
    source: &str,
    anchors: &[AnchorRegion],
    mut pending: Vec<PendingObservation>,
) -> Vec<RenderedObservation> {
    pending.sort_by_key(|observation| {
        (
            observation.source_offset,
            observation.span.end,
            observation.category.source_order(),
        )
    });
    let mut ordinals: HashMap<(ObservationCategory, String), usize> = HashMap::new();
    pending
        .into_iter()
        .map(|observation| {
            let anchor = anchor_at(anchors, observation.source_offset);
            let key_owner = match &observation.identity {
                ObservationIdentity::EnclosingDeclaration => anchor.clone(),
                ObservationIdentity::RewriteEntrypoint(identity) => identity.clone(),
            };
            let ordinal = ordinals
                .entry((observation.category, key_owner.clone()))
                .and_modify(|count| *count += 1)
                .or_insert(1);
            RenderedObservation {
                stable_key: format!(
                    "{}/{}/{:03}",
                    observation.category.as_str(),
                    key_owner,
                    ordinal
                ),
                line: observation.line,
                anchor,
                category: observation.category,
                evidence: escape_evidence(
                    observation
                        .evidence_override
                        .as_deref()
                        .unwrap_or_else(|| &source[observation.span.start..observation.span.end]),
                ),
            }
        })
        .collect()
}

fn scan_source(source: &str) -> Result<Vec<RenderedObservation>, ScanError> {
    let all_tokens = lex(source)?;
    let production_count = production_token_count(&all_tokens, source);
    let production_end = all_tokens
        .get(production_count)
        .map_or(source.len(), |token| token.span.start);
    let tokens = &all_tokens[..production_count];
    let pairs = delimiter_pairs(tokens)?;
    let anchors = declaration_anchors(tokens, source, &pairs);
    let mut observations = lexical_observations(source, tokens, &pairs, &anchors)?;
    observations.extend(harness_helper_observations(source, production_end));
    Ok(render_observations(source, &anchors, observations))
}

fn run(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let observations =
        scan_source(&source).map_err(|error| format!("cannot scan {}: {error}", path.display()))?;
    for observation in observations {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            observation.stable_key,
            observation.line,
            observation.anchor,
            observation.category.as_str(),
            observation.evidence
        );
    }
    Ok(())
}

fn main() {
    let mut arguments = env::args_os();
    let program = arguments
        .next()
        .and_then(|value| Path::new(&value).file_name().map(|name| name.to_owned()))
        .unwrap_or_else(|| "audit-test262-shortcuts".into());
    let Some(path) = arguments.next() else {
        eprintln!("usage: {} SOURCE", Path::new(&program).display());
        process::exit(2);
    };
    if arguments.next().is_some() {
        eprintln!("usage: {} SOURCE", Path::new(&program).display());
        process::exit(2);
    }
    if let Err(error) = run(Path::new(&path)) {
        eprintln!("audit-test262-shortcuts: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(source: &str) -> Vec<RenderedObservation> {
        scan_source(source).expect("fixture should scan")
    }

    #[test]
    fn detects_multiline_calls_and_same_line_multiplicity() {
        let source = r#"
fn production(case: &TestCase, path: &str, source: &str) {
    let _ = case
        .path
        .ends_with("built-ins/one.js");
    let _ = source.contains("first") || source.contains("second");
    let _ = rewrite_example(
        &case.path,
    );
    match path.rsplit('/').next().unwrap_or_default() { _ => {} }
}
"#;
        let observations = rows(source);
        assert_eq!(
            observations
                .iter()
                .map(|observation| observation.category)
                .collect::<Vec<_>>(),
            vec![
                ObservationCategory::DirectPathPredicate,
                ObservationCategory::SourceTextPredicate,
                ObservationCategory::SourceTextPredicate,
                ObservationCategory::PathRewriteEntrypoint,
                ObservationCategory::DirectPathPredicate,
            ]
        );
        assert_eq!(
            observations[0].evidence,
            "case\\n        .path\\n        .ends_with(\"built-ins/one.js\")"
        );
        assert_eq!(
            observations[1].stable_key,
            "source-text-predicate/production/001"
        );
        assert_eq!(
            observations[2].stable_key,
            "source-text-predicate/production/002"
        );
        assert_eq!(
            observations[3].stable_key,
            "path-rewrite-entrypoint/rewrite_example/001"
        );
    }

    #[test]
    fn excludes_member_suffixes_comments_literals_and_tests() {
        let source = r##"
fn production(path: &str) {
    let _ = failure.test_path.starts_with("built-ins/failure.js");
    let _ = artifact.path.contains("built-ins/artifact.js");
    let _ = "path.ends_with(\"built-ins/string.js\")";
    let _ = r#"case.path.contains("built-ins/raw.js")"#;
    /* outer /* path.contains("built-ins/comment.js") */ comment */
    let _: &'static str = if path.contains("built-ins/real.js") { "" } else { "" };
}
#[cfg(test)]
mod tests {
    fn ignored(case: &TestCase) {
        let _ = rewrite_ignored(&case.path);
    }
}
"##;
        let observations = rows(source);
        assert_eq!(observations.len(), 1);
        assert_eq!(
            observations[0].evidence,
            "path.contains(\"built-ins/real.js\")"
        );
    }

    #[test]
    fn scans_modules_named_tests_without_the_test_configuration_boundary() {
        let source = r#"
mod tests {
    fn production(path: &str) {
        let _ = path.contains("built-ins/production.js");
    }
}
"#;
        let observations = rows(source);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].anchor, "production");
    }

    #[test]
    fn detects_exact_path_and_source_forms() {
        let source = r#"
fn predicates(case: &TestCase, path: &str, source: &str, original_source: &str) {
    let _ = path == "built-ins/one.js";
    let _ = case.path == "built-ins/two.js";
    let _ = case.path != "built-ins/three.js";
    let _ = matches!(case.path.as_str(), "built-ins/four.js" | "built-ins/five.js");
    let _ = path == r"built-ins/raw-is-not-admitted.js";
    let _ = case.original_source.contains("one");
    let _ = original_source.replace("two", "three");
    let _ = source.replace("four", "five");
    let _ = source.replacen("six", "seven", 1);
    let _ = case
        .original_source
        .bytes()
        .fold(0usize, |count, byte| count + usize::from(byte));
    let _ = case.original_source.matches("eight").count();
}
"#;
        let observations = rows(source);
        assert_eq!(observations.len(), 10);
        assert_eq!(
            observations
                .iter()
                .filter(|observation| {
                    observation.category == ObservationCategory::DirectPathPredicate
                })
                .count(),
            4
        );
        assert_eq!(
            observations
                .iter()
                .filter(|observation| {
                    observation.category == ObservationCategory::SourceTextPredicate
                })
                .count(),
            6
        );
        assert!(observations
            .iter()
            .any(|observation| observation.evidence.contains(".bytes()\\n        .fold(")));
        assert!(observations.iter().any(|observation| {
            observation
                .evidence
                .contains("original_source.matches(\"eight\").count()")
        }));
        assert!(observations.iter().any(|observation| {
            observation
                .evidence
                .starts_with("matches!(case.path.as_str()")
                && observation.evidence.contains("built-ins/five.js")
        }));
    }

    #[test]
    fn traces_chained_source_calls_and_fingerprints_the_full_chain() {
        let source = r#"
fn chained(source: &str) {
    let _ = source.replace("first", "one").replacen("second", "two", 1);
}
"#;
        let original = rows(source);
        assert_eq!(original.len(), 2);
        assert_eq!(original[0].evidence, "source.replace(\"first\", \"one\")");
        assert_eq!(
            original[1].evidence,
            "source.replace(\"first\", \"one\").replacen(\"second\", \"two\", 1)"
        );
        assert_eq!(original[1].stable_key, "source-text-predicate/chained/002");

        let changed = rows(&source.replace("\"second\"", "\"changed\""));
        assert_eq!(original[0].evidence, changed[0].evidence);
        assert_ne!(original[1].evidence, changed[1].evidence);
        assert!(changed[1].evidence.contains("changed"));
    }

    #[test]
    fn detects_closed_selector_grammar_without_namespace_false_positives() {
        let source = r#"
fn selectors(case: &TestCase, path: &str, source: &str) {
    let _ = rewrite_whole_case((case));
    let _ = typed_array_retired_rewrite_source_matches_vendored_case(case);
    let _ = module::rewrite_namespaced(case);
    let _ = typed_array_literal_method_and_file(&case.path);
    let _ = case.path.as_bytes();
    let _ = case.path.split('/');
    let _ = path.strip_prefix("built-ins/Iterator/");
    let _ = path.rsplit('/').next().unwrap_or(path);
    let _ = matches!(path, "built-ins/one.js" | "built-ins/two.js");
    let _bytes = source.as_bytes();
    let _identifier = &source[identifier_start..idx];
    let _ = module::path.contains("built-ins/qualified.js");
}
"#;
        let observations = rows(source);
        assert_eq!(observations.len(), 9);
        assert_eq!(
            observations
                .iter()
                .filter(|observation| {
                    observation.category == ObservationCategory::DirectPathPredicate
                })
                .count(),
            6
        );
        assert!(observations.iter().any(|observation| {
            observation.stable_key == "path-rewrite-entrypoint/rewrite_whole_case/001"
        }));
        assert!(observations.iter().any(|observation| {
            observation.evidence == "path.rsplit('/').next().unwrap_or(path)"
        }));
        assert!(!observations
            .iter()
            .any(|observation| observation.evidence.contains("qualified")));
    }

    #[test]
    fn match_selector_evidence_tracks_patterns_but_not_replacement_bodies() {
        let source = r#"
fn selected(case: &TestCase) -> usize {
    match case.path.as_str() {
        "built-ins/one.js" | "built-ins/two.js" => { 1 }
        _ => 2,
    }
}
"#;
        let original = rows(source);
        assert_eq!(original.len(), 1);
        assert!(original[0].evidence.contains("built-ins/one.js"));
        assert!(original[0].evidence.contains("built-ins/two.js"));
        assert!(original[0].evidence.contains("_ => <body>"));

        let body_changed = rows(&source.replace("{ 1 }", "{ 3 }"));
        assert_eq!(original[0].evidence, body_changed[0].evidence);
        let selector_changed = rows(&source.replace("built-ins/two.js", "built-ins/three.js"));
        assert_ne!(original[0].evidence, selector_changed[0].evidence);
    }

    #[test]
    fn fingerprints_path_derived_alias_match_patterns() {
        let source = r#"
fn rewrite_iterator_filter_case(path: &str) -> Option<usize> {
    let leaf = path.strip_prefix("built-ins/Iterator/prototype/filter/")?;
    match leaf {
        "one.js" => Some(1),
        _ => None,
    }
}
"#;
        let observations = rows(source);
        assert_eq!(observations.len(), 2);
        assert_eq!(
            observations[0].evidence,
            "path.strip_prefix(\"built-ins/Iterator/prototype/filter/\")?"
        );
        assert!(observations[1].evidence.contains("\"one.js\" => <body>"));
        assert!(observations[1].evidence.contains("_ => <body>"));
    }

    #[test]
    fn rejects_selector_match_grammar_that_could_hide_following_patterns() {
        let source = r#"
fn selected(path: &str, condition: bool) -> usize {
    match path {
        "built-ins/one.js" => if condition { 1 } else { 2 }
        _ => 3,
    }
}
"#;
        let error = scan_source(source).expect_err("unsupported selector grammar must fail closed");
        assert!(error
            .to_string()
            .contains("unsupported execution-selector match arm grammar"));
    }

    #[test]
    fn rejects_closure_parameter_commas_that_can_masquerade_as_arm_separators() {
        let source = r#"
fn selected(path: &str) {
    match path {
        "built-ins/one.js" => |left, right| left + right,
        _ => |_| 0,
    }
}
"#;
        let error = scan_source(source).expect_err("closure comma must fail closed");
        assert!(error
            .to_string()
            .contains("unsupported execution-selector match arm grammar"));
    }

    #[test]
    fn rejects_turbofish_commas_that_can_masquerade_as_arm_separators() {
        let source = r#"
fn selected(path: &str) {
    match path {
        "built-ins/one.js" => value::<Left, Right>(),
        _ => 0,
    }
}
"#;
        let error = scan_source(source).expect_err("turbofish comma must fail closed");
        assert!(error
            .to_string()
            .contains("unsupported execution-selector match arm grammar"));
    }

    #[test]
    fn preserves_harness_marker_line_semantics() {
        let source = r##"
const PRELUDE: &str = r#"
assert.sameValue = function (actual, expected) {
    return actual === expected; // used_preludes is a second marker
}
"#;
fn production() {
    let _ = prelude.contents;
}
"##;
        let observations = rows(source);
        let harness = observations
            .iter()
            .filter(|observation| {
                observation.category == ObservationCategory::HarnessHelperReduction
            })
            .collect::<Vec<_>>();
        assert_eq!(harness.len(), 3);
        assert_eq!(harness[0].anchor, "PRELUDE");
        assert_eq!(
            harness[0].evidence,
            "assert.sameValue = function (actual, expected) {"
        );
        assert_eq!(harness[1].anchor, "PRELUDE");
        assert_eq!(harness[2].anchor, "production");
    }

    #[test]
    fn function_anchor_survives_const_generics_and_lifetimes() {
        let source = r#"
const fn generic<const N: usize>(path: &'static str) {
    const LOCAL: &str = "local const must not replace the function anchor";
    let _: [u8; N] = [0; N];
    let _ = LOCAL;
    let _ = path.as_str();
}
"#;
        let observations = rows(source);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].anchor, "generic");
    }

    #[test]
    fn numeric_literals_stop_before_range_and_member_tokens() {
        let source = "fn scan(path: &str) { let _ = 1..path.contains(\"range end\"); }";
        let tokens = lex(source).expect("fixture should lex");
        assert!(tokens.windows(4).any(|window| {
            window[0].text(source) == "1"
                && window[1].punctuation() == Some(b'.')
                && window[2].punctuation() == Some(b'.')
                && window[3].text(source) == "path"
        }));
        let observations = rows(source);
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].evidence, "path.contains(\"range end\")");
    }

    #[test]
    fn escapes_every_tsv_control_character() {
        assert_eq!(escape_evidence("a\\b\tc\rd\ne"), "a\\\\b\\tc\\rd\\ne");
    }
}
