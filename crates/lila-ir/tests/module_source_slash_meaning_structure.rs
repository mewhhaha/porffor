use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/modules/source.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn code_without_whitespace(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn slash_meaning_is_a_private_two_row_domain_without_incidental_capabilities() {
    assert_eq!(OWNER_SOURCE.matches("enum SlashMeaning").count(), 1);
    assert!(!OWNER_SOURCE.contains("pub enum SlashMeaning"));

    let declaration = bounded(
        OWNER_SOURCE,
        "/// What a `/` means at the current position.",
        "struct Scanner<'a>",
    );
    assert!(!declaration.contains("#[derive"));
    assert_eq!(
        code_without_whitespace(bounded(declaration, "enum SlashMeaning {", "}",)),
        "Divide,Regexp,"
    );
    assert!(!OWNER_SOURCE.contains("impl SlashMeaning"));
    assert!(!OWNER_SOURCE.contains("Default for SlashMeaning"));
    assert!(!OWNER_SOURCE.contains("== SlashMeaning::"));
    assert!(!OWNER_SOURCE.contains("!= SlashMeaning::"));
    assert!(!OWNER_SOURCE.contains("matches!(self.slash"));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "SlashMeaning"),
        43,
        "the module-source domain must retain its 23 mentions beside the dynamic-source scanner's 20 mentions"
    );
    assert_eq!(OWNER_SOURCE.matches("SlashMeaning").count(), 23);
}

#[test]
fn slash_meaning_keeps_the_exact_nine_by_ten_producer_census() {
    assert_eq!(OWNER_SOURCE.matches("SlashMeaning::Divide").count(), 10);
    assert_eq!(OWNER_SOURCE.matches("SlashMeaning::Regexp").count(), 11);
    assert_eq!(
        OWNER_SOURCE
            .matches("self.slash = SlashMeaning::Divide;")
            .count(),
        6
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("self.slash = SlashMeaning::Regexp;")
            .count(),
        7
    );
    assert_eq!(
        OWNER_SOURCE.matches("slash: SlashMeaning::Regexp,").count(),
        1
    );

    let run = code_without_whitespace(bounded(
        OWNER_SOURCE,
        "fn run(&mut self) -> Result<(), StripError> {",
        "fn char_len(&self) -> usize {",
    ));
    for producer in [
        "self.skip_string(byte)?;self.slash=SlashMeaning::Divide;self.previous_was_dot=false;",
        "b'('|b'['=>{self.depth+=1;self.index+=1;self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;}",
        "b'{'=>{self.depth+=1;self.index+=1;self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;}",
        "b')'|b']'=>{self.depth=self.depth.saturating_sub(1);self.index+=1;self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}",
        "self.depth=self.depth.saturating_sub(1);self.index+=1;self.slash=SlashMeaning::Divide;self.previous_was_dot=false;",
        "b'.'=>{self.index+=1;self.slash=SlashMeaning::Regexp;self.previous_was_dot=true;}",
        "byteifbyte.is_ascii_digit()=>{self.skip_number();self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}",
    ] {
        assert!(run.contains(producer), "missing run producer `{producer}`");
    }

    let punctuator_projection = bounded(
        OWNER_SOURCE,
        "self.slash = if two == Some(\"++\") || two == Some(\"--\") {",
        "self.previous_was_dot = false;",
    );
    assert_eq!(
        code_without_whitespace(punctuator_projection),
        "SlashMeaning::Divide}else{SlashMeaning::Regexp};"
    );

    let word_projection = bounded(OWNER_SOURCE, "self.slash = match word {", "};");
    assert_eq!(
        code_without_whitespace(word_projection),
        "\"this\"|\"super\"|\"true\"|\"false\"|\"null\"=>SlashMeaning::Divide,\
         wordifis_reserved_word(word)=>SlashMeaning::Regexp,_=>SlashMeaning::Divide,"
    );

    let scan_word = code_without_whitespace(bounded(
        OWNER_SOURCE,
        "fn scan_word(&mut self) -> Result<(), StripError> {",
        "fn scan_import_declaration(&mut self)",
    ));
    for producer in [
        "self.index=end;self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;returnOk(());",
        "self.index=edit.end;self.edits.push(edit);self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;returnOk(());",
    ] {
        assert!(
            scan_word.contains(producer),
            "missing word producer `{producer}`"
        );
    }

    let template = code_without_whitespace(bounded(
        OWNER_SOURCE,
        "fn scan_template_body(&mut self) -> Result<(), StripError> {",
        "fn is_identifier_start_byte",
    ));
    for producer in [
        "b'`'=>{self.index+=1;self.slash=SlashMeaning::Divide;self.previous_was_dot=false;returnOk(());}",
        "self.template_stack.push(self.depth);self.slash=SlashMeaning::Regexp;self.previous_was_dot=false;returnOk(());",
    ] {
        assert!(
            template.contains(producer),
            "missing template producer `{producer}`"
        );
    }
}

#[test]
fn slash_dispatch_is_exhaustive_after_comments_and_preserves_both_state_transitions() {
    let run = bounded(
        OWNER_SOURCE,
        "fn run(&mut self) -> Result<(), StripError> {",
        "fn char_len(&self) -> usize {",
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

    let dispatch = bounded(run, "b'/' => match &self.slash {", "b'\\'' | b'\"' => {");
    assert_eq!(
        code_without_whitespace(dispatch),
        "SlashMeaning::Regexp=>{self.skip_regexp()?;\
         self.slash=SlashMeaning::Divide;self.previous_was_dot=false;}\
         SlashMeaning::Divide=>{self.slash=SlashMeaning::Regexp;\
         self.previous_was_dot=false;self.index+=self.char_len();}},"
    );
    assert!(!dispatch.contains("_ =>"));

    for helper in [
        "fn skip_line_comment(&mut self)",
        "fn skip_block_comment(&mut self)",
    ] {
        let helper_body = if helper.contains("line") {
            bounded(OWNER_SOURCE, helper, "fn skip_block_comment(&mut self)")
        } else {
            bounded(OWNER_SOURCE, helper, "fn skip_string(&mut self")
        };
        assert!(
            !helper_body.contains("self.slash"),
            "{helper} must preserve slash context"
        );
    }

    let generic_punctuator = bounded(run, "_ => {", "            }\n        }\n        Ok(())");
    assert_eq!(
        generic_punctuator
            .matches("self.index += self.char_len();")
            .count(),
        1
    );
}
