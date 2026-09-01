use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/modules/dynamic.rs");

const RUN_FINGERPRINT: &str = concat!(
    r#"whileself.index<self.bytes.len(){letbyte=self.bytes[self.index];matchbyte{"#,
    r#"b'/'ifself.bytes.get(self.index+1)==Some(&b'/')=>self.skip_line_comment(),"#,
    r#"b'/'ifself.bytes.get(self.index+1)==Some(&b'*')=>{self.skip_block_comment()?;}"#,
    r#"b'/'=>match&self.slash{SlashMeaning::Regexp=>{self.skip_regexp()?;"#,
    r#"self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}"#,
    r#"SlashMeaning::Divide=>{self.slash=SlashMeaning::Regexp;"#,
    r#"self.previous_was_dot=false;self.index+=self.char_len_at(self.index);}},"#,
    r#"b'\''|b'"'=>{self.index=self.string_end(self.index,byte)?;"#,
    r#"self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}"#,
    r#"b'`'=>{self.index+=1;self.scan_template_body()?;}"#,
    r#"b'('|b'['|b'{'=>{self.depth+=1;self.index+=1;"#,
    r#"self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;}"#,
    r#"b')'|b']'=>{self.depth=self.depth.saturating_sub(1);self.index+=1;"#,
    r#"self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}"#,
    r#"b'}'=>{letcloses_substitution=self.template_stack.last()"#,
    r#".is_some_and(|open_depth|*open_depth==self.depth);"#,
    r#"self.depth=self.depth.saturating_sub(1);self.index+=1;"#,
    r#"ifcloses_substitution{self.template_stack.pop();self.scan_template_body()?;continue;}"#,
    r#"self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}"#,
    r#"b'.'=>{self.index+=1;self.slash=SlashMeaning::Regexp;self.previous_was_dot=true;}"#,
    r#"byteif!byte.is_ascii()&&is_js_whitespace(self.char_at(self.index))=>{"#,
    r#"self.index+=self.char_len_at(self.index);}"#,
    r#"byteifis_identifier_start_byte(byte)=>self.scan_word(),"#,
    r#"byteifbyte.is_ascii_digit()=>{self.skip_number();"#,
    r#"self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}"#,
    r#"byteifbyte.is_ascii_whitespace()=>self.index+=1,_=>{"#,
    r#"lettwo=self.source.get(self.index..self.index+2);"#,
    r#"self.slash=iftwo==Some("++")||two==Some("--"){"#,
    r#"SlashMeaning::Divide}else{SlashMeaning::Regexp};self.previous_was_dot=false;"#,
    r#"self.index+=self.char_len_at(self.index);}}}Ok(self.sites)}"#,
);

const SCAN_WORD_FINGERPRINT: &str = concat!(
    r#"letstart=self.index;whileletSome(character)=self.source[self.index..].chars().next(){"#,
    r#"ifcharacter.is_ascii(){if!is_identifier_part_byte(characterasu8){break;}"#,
    r#"self.index+=1;}else{ifis_js_whitespace(character){break;}"#,
    r#"self.index+=character.len_utf8();}}letword=&self.source[start..self.index];"#,
    r#"ifword=="import"&&!self.previous_was_dot{ifself.peek_significant()==Some(b'('){"#,
    r#"self.sites.push(ImportCallSite{start,end:self.index,phase:ImportPhaseIr::Evaluation,});}"#,
    r#"elseifletSome((phase,end))=self.peek_phased_call(){"#,
    r#"self.sites.push(ImportCallSite{start,end,phase});}}self.slash=matchword{"#,
    r#""this"|"super"|"true"|"false"|"null"=>SlashMeaning::Divide,"#,
    r#"wordifSelf::is_reserved_word(word)=>SlashMeaning::Regexp,"#,
    r#"_=>SlashMeaning::Divide,};self.previous_was_dot=false;}"#,
);

const SCAN_TEMPLATE_BODY_FINGERPRINT: &str = concat!(
    r#"whileletSome(byte)=self.bytes.get(self.index).copied(){matchbyte{"#,
    r#"b'\\'=>{self.index+=1;self.index+=self.char_len_at(self.index.min(self.bytes.len()));}"#,
    r#"b'`'=>{self.index+=1;self.slash=SlashMeaning::Divide;"#,
    r#"self.previous_was_dot=false;returnOk(());}"#,
    r#"b'$'ifself.bytes.get(self.index+1)==Some(&b'{')=>{self.index+=2;self.depth+=1;"#,
    r#"self.template_stack.push(self.depth);self.slash=SlashMeaning::Regexp;"#,
    r#"self.previous_was_dot=false;returnOk(());}"#,
    r#"_=>self.index+=self.char_len_at(self.index),}}"#,
    r#"Err("unterminated template literal".to_string())}}"#,
);

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn quoted_literal_end(source: &str, quote_start: usize, quote: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
            return Some(offset + 1);
        }
        offset += 1;
    }
    None
}

fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let value_start = start + 1;
    if value_start >= bytes.len() {
        return None;
    }
    let value_end = if bytes[value_start] == b'\\' {
        let mut offset = value_start + 1;
        if offset >= bytes.len() {
            return None;
        }
        if bytes[offset] == b'u' && bytes.get(offset + 1) == Some(&b'{') {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'}') {
                offset += 1;
            }
            if bytes.get(offset) != Some(&b'}') {
                return None;
            }
            offset + 1
        } else if bytes[offset] == b'x'
            && bytes
                .get(offset + 1..offset + 3)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
        {
            offset + 3
        } else {
            offset + 1
        }
    } else {
        value_start + source[value_start..].chars().next()?.len_utf8()
    };
    (bytes.get(value_end) == Some(&b'\'')).then_some(value_end + 1)
}

fn raw_literal_end(source: &str, start: usize, prefix_len: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote_start = start + prefix_len;
    while bytes.get(quote_start) == Some(&b'#') {
        quote_start += 1;
    }
    if bytes.get(quote_start) != Some(&b'"') {
        return None;
    }
    let hashes = quote_start - start - prefix_len;
    let mut offset = quote_start + 1;
    while offset < bytes.len() {
        if bytes[offset] == b'"'
            && bytes
                .get(offset + 1..offset + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(offset + 1 + hashes);
        }
        offset += 1;
    }
    None
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    match bytes.get(start).copied()? {
        b'"' => quoted_literal_end(source, start, b'"'),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => {
            quoted_literal_end(source, start + 1, b'"')
        }
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

fn lexically_normalized_code_with_literals(source: &str, preserve_literals: bool) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            if preserve_literals {
                normalized.push_str(&source[offset..end]);
            } else {
                normalized.push('L');
            }
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            offset += 2;
            let mut depth = 1;
            while offset < bytes.len() && depth != 0 {
                if bytes.get(offset..offset + 2) == Some(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if bytes.get(offset..offset + 2) == Some(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
            assert_eq!(depth, 0, "unterminated block comment in Rust source");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#") {
            let identifier_start = source[offset + 2..].chars().next();
            if identifier_start
                .is_some_and(|character| character == '_' || character.is_alphabetic())
            {
                offset += 2;
                continue;
            }
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            normalized.push(character);
        }
        offset += character.len_utf8();
    }
    normalized
}

fn lexically_normalized_code(source: &str) -> String {
    lexically_normalized_code_with_literals(source, false)
}

fn semantic_fingerprint(source: &str) -> String {
    lexically_normalized_code_with_literals(source, true)
}

fn count_in_normalized_rust_sources(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| {
            entry
                .expect("source directory entry should be readable")
                .path()
        })
        .map(|path| {
            if path.is_dir() {
                return count_in_normalized_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            lexically_normalized_code(&source).matches(needle).count()
        })
        .sum()
}

#[test]
fn dynamic_import_slash_meaning_is_the_exact_private_no_capability_domain() {
    let before_doc = OWNER_SOURCE
        .split_once("/// What a `/` means at the current position.")
        .expect("slash-meaning documentation marker")
        .0;
    let after_previous_item = before_doc
        .rsplit_once("\n}\n")
        .expect("preceding js_string_literal item")
        .1;
    assert_eq!(lexically_normalized_code(after_previous_item), "");

    let declaration_region = bounded(
        OWNER_SOURCE,
        "/// What a `/` means at the current position.",
        "/// One `import(`, `import.defer(` or `import.source(` call site the scanner",
    );
    assert_eq!(
        lexically_normalized_code(declaration_region),
        "enumSlashMeaning{Divide,Regexp,}"
    );
    let owner = lexically_normalized_code(OWNER_SOURCE);
    for forbidden in [
        "implSlashMeaning",
        "implCloneforSlashMeaning",
        "implCopyforSlashMeaning",
        "implPartialEqforSlashMeaning",
        "implEqforSlashMeaning",
        "forSlashMeaning",
        "SlashMeaning::default",
        "DefaultforSlashMeaning",
        "==SlashMeaning::",
        "!=SlashMeaning::",
        "SlashMeaning::Divide==",
        "SlashMeaning::Regexp==",
        "SlashMeaning::Divide!=",
        "SlashMeaning::Regexp!=",
        "matches!(self.slash",
    ] {
        assert!(!owner.contains(forbidden), "forbidden route `{forbidden}`");
    }
    assert_eq!(owner.matches("SlashMeaning").count(), 20);
    assert_eq!(owner.matches("SlashMeaning::Divide").count(), 10);
    assert_eq!(owner.matches("SlashMeaning::Regexp").count(), 8);
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_normalized_rust_sources(&source_root, "SlashMeaning"),
        43
    );
}

#[test]
fn dynamic_import_slash_meaning_keeps_the_exact_nine_by_seven_producer_census() {
    let owner = lexically_normalized_code(OWNER_SOURCE);
    assert_eq!(owner.matches("self.slash=SlashMeaning::Divide;").count(), 6);
    assert_eq!(owner.matches("self.slash=SlashMeaning::Regexp;").count(), 4);
    assert_eq!(owner.matches("slash:SlashMeaning::Regexp,").count(), 1);
    assert_eq!(owner.matches("self.slash=SlashMeaning::").count(), 10);
    assert_eq!(owner.matches("match&self.slash").count(), 1);
    assert_eq!(owner.matches("self.slash=").count(), 12);
    assert_eq!(owner.matches("self.slash").count(), 13);
    assert_eq!(owner.matches(".slash").count(), 13);
    let production = lexically_normalized_code(
        OWNER_SOURCE
            .split_once("#[cfg(test)]\nmod tests")
            .expect("production must precede the owner test module")
            .0,
    );
    assert_eq!(production.matches("slash").count(), 15);

    let run = semantic_fingerprint(bounded(
        OWNER_SOURCE,
        "    fn run(mut self) -> Result<Vec<ImportCallSite>, String> {",
        "    /// Words after which a `/` starts a regular expression rather than a",
    ));
    assert_eq!(run, RUN_FINGERPRINT);

    let scan_word = semantic_fingerprint(bounded(
        OWNER_SOURCE,
        "    fn scan_word(&mut self) {",
        "    fn char_len_at(&self",
    ));
    assert_eq!(scan_word, SCAN_WORD_FINGERPRINT);

    let template = semantic_fingerprint(bounded(
        OWNER_SOURCE,
        "    fn scan_template_body(&mut self) -> Result<(), String> {",
        "fn is_identifier_start_byte",
    ));
    assert_eq!(template, SCAN_TEMPLATE_BODY_FINGERPRINT);
}

#[test]
fn dynamic_import_slash_dispatch_is_exhaustive_after_comments() {
    let run = bounded(
        OWNER_SOURCE,
        "    fn run(mut self) -> Result<Vec<ImportCallSite>, String> {",
        "    /// Words after which a `/` starts a regular expression rather than a",
    );
    let line_comment = run
        .find("b'/' if self.bytes.get(self.index + 1) == Some(&b'/')")
        .expect("line-comment dispatch");
    let block_comment = run
        .find("b'/' if self.bytes.get(self.index + 1) == Some(&b'*')")
        .expect("block-comment dispatch");
    let slash_dispatch = run
        .find("b'/' => match &self.slash")
        .expect("slash-meaning dispatch");
    assert!(line_comment < block_comment);
    assert!(block_comment < slash_dispatch);

    let dispatch = bounded(
        run,
        "                b'/' => match &self.slash {",
        "                b'\\'' | b'\"' => {",
    );
    assert_eq!(
        lexically_normalized_code(dispatch),
        "SlashMeaning::Regexp=>{self.skip_regexp()?;self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}SlashMeaning::Divide=>{self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;self.index+=self.char_len_at(self.index);}},"
    );
    assert!(!lexically_normalized_code(dispatch).contains("_=>"));

    let line_helper = bounded(
        OWNER_SOURCE,
        "    fn skip_line_comment(&mut self)",
        "    fn skip_block_comment(&mut self)",
    );
    let block_helper = bounded(
        OWNER_SOURCE,
        "    fn skip_block_comment(&mut self)",
        "    fn skip_number(&mut self)",
    );
    assert!(!lexically_normalized_code(line_helper).contains("self.slash"));
    assert!(!lexically_normalized_code(block_helper).contains("self.slash"));

    let generic_punctuator = bounded(
        run,
        "                _ => {",
        "            }\n        }\n        Ok(self.sites)",
    );
    assert_eq!(
        lexically_normalized_code(generic_punctuator)
            .matches("self.index+=self.char_len_at(self.index);")
            .count(),
        1
    );
}
