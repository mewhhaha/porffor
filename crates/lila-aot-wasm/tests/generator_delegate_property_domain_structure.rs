const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

#[test]
fn delegate_property_domains_are_private_capability_free_and_single_owned() {
    let attributes = DELEGATION_SOURCE
        .split_once("enum GeneratorDelegateProperty")
        .expect("closed enum declaration")
        .0
        .rsplit("\n\n")
        .next()
        .expect("enum declaration prefix");
    assert!(!attributes.contains("#[derive"));
    let declaration = bounded(DELEGATION_SOURCE, "enum GeneratorDelegateProperty {", "}\n");
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "AsyncIterator,",
            "Iterator,",
            "Next,",
            "Return,",
            "Throw,",
            "Done,",
            "Value,"
        ]
    );
    assert!(!DELEGATION_SOURCE.contains("pub enum GeneratorDelegateProperty"));
    assert!(!DELEGATION_SOURCE.contains("pub(crate) enum GeneratorDelegateProperty"));
    assert!(!DELEGATION_SOURCE.contains("pub(super) enum GeneratorDelegateProperty"));
}
