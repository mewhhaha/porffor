use std::fs;
use std::path::Path;

const OWNER_SOURCE: &str = include_str!("../src/binding_lifecycle.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
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
fn instantiated_frame_is_the_exact_private_no_capability_domain() {
    assert_eq!(OWNER_SOURCE.matches("enum InstantiatedFrame {").count(), 1);
    for visibility in [
        "pub enum InstantiatedFrame",
        "pub(crate) enum InstantiatedFrame",
        "pub(super) enum InstantiatedFrame",
    ] {
        assert!(!OWNER_SOURCE.contains(visibility), "found `{visibility}`");
    }

    let declaration = bounded(
        OWNER_SOURCE,
        "/// Which Environment Record the sweep created its bindings in.",
        "impl LexicalScopeInstantiation",
    );
    assert!(!declaration.contains("#["));
    assert_eq!(
        normalized(bounded(declaration, "enum InstantiatedFrame {", "}")),
        "Pushed,Current,"
    );
    for forbidden in ["impl InstantiatedFrame", "for InstantiatedFrame"] {
        assert!(!OWNER_SOURCE.contains(forbidden), "found `{forbidden}`");
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "InstantiatedFrame"),
        9,
        "one declaration, one field, two documentation links, three producers and two consumer arms must own every mention"
    );
}

#[test]
fn instantiated_frame_constructors_preserve_the_exact_frame_and_sweep_order() {
    assert_eq!(
        OWNER_SOURCE
            .matches("frame: InstantiatedFrame::Pushed,")
            .count(),
        2
    );
    assert_eq!(
        OWNER_SOURCE
            .matches("frame: InstantiatedFrame::Current,")
            .count(),
        1
    );

    let pushed = normalized(bounded(
        OWNER_SOURCE,
        "    pub(crate) fn instantiate(\n",
        "    /// 16.1.7 GlobalDeclarationInstantiation",
    ));
    assert_eq!(
        pushed,
        "lowerer:&mutScriptLowerer<'_>,items:&[StatementListItem],)->Self{lowerer.push_instantiation_scope();letmutscope=Self{pending:BTreeMap::new(),frame:InstantiatedFrame::Pushed,};foriteminitems{scope.instantiate_item(lowerer,item);}scope}"
    );

    let current = normalized(bounded(
        OWNER_SOURCE,
        "    pub(crate) fn instantiate_in_current_scope(\n",
        "    /// 14.12.4 CaseBlockEvaluation",
    ));
    assert_eq!(
        current,
        "lowerer:&mutScriptLowerer<'_>,items:&[StatementListItem],)->Self{letmutscope=Self{pending:BTreeMap::new(),frame:InstantiatedFrame::Current,};foriteminitems{scope.instantiate_item(lowerer,item);}scope}"
    );

    let switch = normalized(bounded(
        OWNER_SOURCE,
        "    pub(crate) fn instantiate_switch(",
        "    /// Ends the statement list",
    ));
    assert_eq!(
        switch,
        "lowerer:&mutScriptLowerer<'_>,switch:&AstSwitch)->Self{lowerer.push_instantiation_scope();letmutscope=Self{pending:BTreeMap::new(),frame:InstantiatedFrame::Pushed,};forcaseinswitch.cases(){foritemincase.body().statements(){scope.instantiate_item(lowerer,item);}}scope}"
    );
}

#[test]
fn finish_consumes_the_frame_and_exhaustively_owns_pop_policy() {
    let finish = bounded(
        OWNER_SOURCE,
        "    pub(crate) fn finish(self, lowerer: &mut ScriptLowerer<'_>) {",
        "    /// LexicallyScopedDeclarations",
    );
    assert_eq!(
        normalized(finish),
        "matchself.frame{InstantiatedFrame::Pushed=>lowerer.pop_instantiation_scope(),InstantiatedFrame::Current=>{}}}"
    );
    for forbidden in [
        "self.frame ==",
        "self.frame !=",
        "matches!(self.frame",
        "_ =>",
        "Default",
    ] {
        assert!(!finish.contains(forbidden), "found `{forbidden}`");
    }
}
