//! The %Array.prototype% identity arm of the append helpers is emitted only
//! while the main export installs the Array constructor and its prototype.
//!
//! An append writes straight into an ordinary object's property table.
//! %Array.prototype% is an Array exotic object, so builtins installed into it
//! must take the array named-descriptor path, and the append helpers guard
//! that with a run-time identity check. Only the main export's installation
//! of the Array constructor appends into %Array.prototype%; every other append
//! targets an object its caller just allocated or another intrinsic. Before `AppendTargetScope`, the check was keyed on
//! `is_main()` and so was inlined, with a whole array descriptor definition,
//! at every script-level append in the main export.

const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn code_only(source: &str) -> String {
    source
        .lines()
        .map(|line| match line.find("//") {
            Some(comment) => &line[..comment],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn only_the_main_array_installation_widens_the_append_target_scope() {
    let emit = code_only(EMIT_SOURCE);

    let domain = bounded(
        &emit,
        "pub(crate) enum AppendTargetScope {",
        "pub(crate) enum OrdinarySetDataOnReceiverEmission {",
    );
    assert!(domain.contains("Self::RealmBootstrap => true,"));
    assert!(domain.contains("Self::OrdinaryObjects => false,"));
    assert!(!domain.contains("_ =>"));

    // Private field, one initial value for every builder.
    assert!(emit.contains("    append_target_scope: AppendTargetScope,"));
    assert!(!emit.contains("pub(crate) append_target_scope:"));
    assert_eq!(
        emit.matches("append_target_scope: AppendTargetScope::OrdinaryObjects,")
            .count(),
        1
    );

    // The one widening is a scoped call that restores the narrow scope on
    // every exit, and its only caller installs the Array constructor and
    // %Array.prototype% in the main export.
    assert_eq!(
        emit.matches("self.append_target_scope = AppendTargetScope::RealmBootstrap;")
            .count(),
        1
    );
    assert_eq!(
        emit.matches("self.append_target_scope = AppendTargetScope::OrdinaryObjects;")
            .count(),
        1
    );
    let scoped = bounded(
        &emit,
        "pub(crate) fn with_realm_bootstrap_appends<T>(",
        "\n    }\n",
    );
    let widen = scoped
        .find("self.append_target_scope = AppendTargetScope::RealmBootstrap;")
        .expect("the scoped widening");
    let run = scoped
        .find("let installed = install(self);")
        .expect("the install runs inside the widened scope");
    let narrow = scoped
        .find("self.append_target_scope = AppendTargetScope::OrdinaryObjects;")
        .expect("the scope is narrowed before any result is returned");
    let result = scoped.rfind("installed").expect("the install result");
    assert!(widen < run && run < narrow && narrow < result);
    assert!(
        !scoped.contains('?'),
        "no early exit may skip the narrowing"
    );

    let bootstrap = code_only(BOOTSTRAP_SOURCE);
    assert_eq!(bootstrap.matches("with_realm_bootstrap_appends").count(), 1);
    let roots = bounded(
        &bootstrap,
        "pub(crate) fn init_runtime_roots(&mut self, function: &mut Function)",
        "pub(crate) fn init_script_global_object(",
    );
    let main_only = roots
        .find("if !self.is_main() {")
        .expect("only the main export installs the entry realm");
    let array_prototype_created = roots
        .find("function.instruction(&Instruction::GlobalSet(ARRAY_PROTOTYPE_GLOBAL_INDEX));")
        .expect("%Array.prototype% is allocated before it is installed into");
    let scoped = bounded(
        roots,
        "self.with_realm_bootstrap_appends(|builder| {",
        "})?;",
    );
    let scoped_at = roots
        .find("self.with_realm_bootstrap_appends(|builder| {")
        .expect("the Array installation runs in the bootstrap scope");
    assert!(main_only < array_prototype_created && array_prototype_created < scoped_at);
    let normalized: String = scoped.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(
        normalized,
        "builder.init_builtin_constructor_object(StandardBuiltinId::ArrayConstructor,\
         ARRAY_PROTOTYPE_GLOBAL_INDEX,function,)"
    );

    // Crate-wide census: no other module names the widening or the scope.
    let source_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut pending = vec![source_root.clone()];
    let mut widening_mentions = Vec::new();
    let mut bootstrap_scope_mentions = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("crate `src` is readable") {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                pending.push(path);
                continue;
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let code = code_only(&std::fs::read_to_string(&path).expect("readable source"));
            let relative = path
                .strip_prefix(&source_root)
                .expect("scanned path is under `src`")
                .display()
                .to_string();
            for _ in 0..code.matches("with_realm_bootstrap_appends").count() {
                widening_mentions.push(relative.clone());
            }
            for _ in 0..code.matches("AppendTargetScope::RealmBootstrap").count() {
                bootstrap_scope_mentions.push(relative.clone());
            }
        }
    }
    widening_mentions.sort();
    bootstrap_scope_mentions.sort();
    // The definition in emit.rs and the one call in bootstrap.rs.
    assert_eq!(widening_mentions, ["builtins/bootstrap.rs", "emit.rs"]);
    // Only the scoped widening itself names the wide scope.
    assert_eq!(bootstrap_scope_mentions, ["emit.rs"]);
}

#[test]
fn append_helpers_gate_the_array_prototype_arm_on_the_scope_alone() {
    let objects = code_only(OBJECTS_SOURCE);
    for (helper, next) in [
        (
            "pub(crate) fn emit_object_append_data_property_with_flags(",
            "\n    pub(crate) fn ",
        ),
        (
            "pub(crate) fn emit_object_append_accessor_property_with_flags(",
            "\n    pub(crate) fn ",
        ),
    ] {
        let body = bounded(&objects, helper, next);
        assert!(
            !body.contains("is_main()"),
            "{helper} must not key on is_main"
        );
        assert_eq!(
            body.matches("self.append_target_scope().may_target_array_prototype()")
                .count(),
            1,
            "{helper}"
        );
        assert_eq!(
            body.matches("if may_target_array_prototype {").count(),
            2,
            "{helper} opens and closes the arm under the same scope"
        );
        let arm = bounded(body, "if may_target_array_prototype {", "Instruction::Else");
        assert!(arm.contains("GlobalGet(ARRAY_PROTOTYPE_GLOBAL_INDEX)"));
        assert_eq!(body.matches("ARRAY_PROTOTYPE_GLOBAL_INDEX").count(), 1);
    }
}
