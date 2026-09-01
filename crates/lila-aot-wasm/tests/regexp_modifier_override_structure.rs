const IR: &str = include_str!("../../lila-ir/src/regexp.rs");
const IR_LIB: &str = include_str!("../../lila-ir/src/lib.rs");
const WASM: &str = include_str!("../src/builtins/regexp.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn regexp_modifier_override_is_a_non_copyable_three_variant_abi_domain() {
    let declaration = bounded(
        IR,
        "pub enum RegExpModifierOverride {",
        "\n}\n\nimpl RegExpModifierOverride {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    assert_eq!(variants, ["Inherit,", "ForceOn,", "ForceOff,"]);
    let documented_declaration =
        "/// The local `m` or `s` behavior selected by a RegExp modifier group.\npub enum RegExpModifierOverride {";
    assert!(IR.contains(documented_declaration));
    let declaration_prefix = bounded(
        IR,
        "struct Modifiers {",
        "pub enum RegExpModifierOverride {",
    );
    assert!(!declaration_prefix.contains("#["));
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!declaration.contains(capability));
        assert!(!IR.contains(&format!("impl {capability} for RegExpModifierOverride")));
    }
    assert!(!IR.contains("#[derive(Clone, Copy)]\npub enum RegExpModifierOverride"));
    assert!(IR_LIB.contains("RegExpModifierOverride, RegExpProgram"));

    let implementation = bounded(
        IR,
        "impl RegExpModifierOverride {",
        "\n}\n\nstruct PatternParser<'a>",
    );
    let normalized = without_whitespace(implementation);
    assert!(normalized.contains(
        "pubconstfnoperand_code(&self)->u64{matchself{Self::Inherit=>0,Self::ForceOn=>1,Self::ForceOff=>2,}}"
    ));
    for forbidden in ["_=>", "unreachable!", "Default::default"] {
        assert!(!normalized.contains(forbidden));
    }
}

#[test]
fn dot_and_assertion_constructors_name_the_inherited_override_code() {
    for (constructor, next_constructor) in [
        (
            "    pub const fn dot() -> Self {",
            "    pub const fn named_backreference(",
        ),
        (
            "    pub const fn assert_start() -> Self {",
            "    pub const fn assert_end() -> Self {",
        ),
        (
            "    pub const fn assert_end() -> Self {",
            "    pub const fn lookbehind_start() -> Self {",
        ),
    ] {
        let body = bounded(IR, constructor, next_constructor);
        assert_eq!(
            body.matches("operand0: RegExpModifierOverride::Inherit.operand_code(),")
                .count(),
            1,
            "constructor `{constructor}`"
        );
        assert!(
            !body.contains("operand0: 0,"),
            "constructor `{constructor}`"
        );
    }
    assert_eq!(
        IR.matches("operand0: RegExpModifierOverride::Inherit.operand_code(),")
            .count(),
        3,
        "dot and the two assertion constructors are the exact inherited-code producers"
    );
}

#[test]
fn parser_names_every_override_and_restores_outer_state_before_propagation() {
    let modifiers = bounded(IR, "struct Modifiers {", "\n}\n\n/// The local `m`");
    assert!(modifiers.contains("multiline: RegExpModifierOverride,"));
    assert!(modifiers.contains("dot_all: RegExpModifierOverride,"));
    assert!(!modifiers.contains("Option<bool>"));

    let initialization = bounded(IR, "let mut parser = PatternParser {", "ranges:");
    assert_eq!(
        initialization
            .matches("RegExpModifierOverride::Inherit")
            .count(),
        2
    );

    let prefix = bounded(
        IR,
        "fn parse_modifier_group_prefix(",
        "    fn parse_group_name(",
    );
    let normalized_prefix = without_whitespace(prefix);
    assert!(normalized_prefix.contains(
        "multiline:match&self.modifiers.multiline{RegExpModifierOverride::Inherit=>RegExpModifierOverride::Inherit,RegExpModifierOverride::ForceOn=>RegExpModifierOverride::ForceOn,RegExpModifierOverride::ForceOff=>RegExpModifierOverride::ForceOff,}"
    ));
    assert!(normalized_prefix.contains(
        "dot_all:match&self.modifiers.dot_all{RegExpModifierOverride::Inherit=>RegExpModifierOverride::Inherit,RegExpModifierOverride::ForceOn=>RegExpModifierOverride::ForceOn,RegExpModifierOverride::ForceOff=>RegExpModifierOverride::ForceOff,}"
    ));
    assert_eq!(
        prefix.matches("= RegExpModifierOverride::ForceOn;").count(),
        2
    );
    assert_eq!(
        prefix
            .matches("= RegExpModifierOverride::ForceOff;")
            .count(),
        2
    );
    for forbidden in ["Some(true)", "Some(false)", "self.modifiers;"] {
        assert!(!prefix.contains(forbidden));
    }

    let modifier_group = bounded(
        IR,
        "Some(b'i' | b'm' | b's' | b'-') => {",
        "                    _ => {",
    );
    let replace = modifier_group
        .find("std::mem::replace(&mut self.modifiers, modifiers)")
        .expect("nested modifier state must be installed by moving the outer state out");
    let parse = modifier_group
        .find("let body = self.alternatives(Some(atom_offset));")
        .expect("nested modifier body must retain its Result before propagation");
    let restore = modifier_group
        .find("self.modifiers = outer;")
        .expect("outer modifier state must be restored");
    let propagate = modifier_group
        .find("body: body?")
        .expect("nested parse failure must propagate after restoration");
    assert!(replace < parse && parse < restore && restore < propagate);
    assert!(!modifier_group.contains("let outer = self.modifiers;"));
}

#[test]
fn ir_encoder_and_wasm_decoder_share_the_typed_operand_codes() {
    let encoder = bounded(IR, "fn apply_modifiers(", "fn apply_ascii_ignore_case(");
    assert!(encoder.contains("modifiers: &Modifiers"));
    let normalized_encoder = without_whitespace(encoder);
    assert!(normalized_encoder.contains(
        "REGEXP_OPCODE_DOT=>{instruction.operand0=modifiers.dot_all.operand_code();return;}"
    ));
    assert!(normalized_encoder.contains(
        "REGEXP_OPCODE_ASSERT_START|REGEXP_OPCODE_ASSERT_END=>{instruction.operand0=modifiers.multiline.operand_code();return;}"
    ));
    assert_eq!(encoder.matches(".operand_code()").count(), 2);
    for forbidden in ["None => 0", "Some(true) => 1", "Some(false) => 2"] {
        assert!(!encoder.contains(forbidden));
    }

    assert!(WASM.contains("RegExpModifierOverride, REGEXP_INSTRUCTION_WIDTH"));
    let decoder = bounded(
        WASM,
        "// `.`, `^` and `$` carry a RegExp-modifier override in `operand0`:",
        "        function.instruction(&Instruction::LocalGet(opcode));",
    );
    let normalized_decoder = without_whitespace(decoder);
    assert!(normalized_decoder.contains(concat!(
        "function.instruction(&Instruction::LocalGet(operand0));",
        "function.instruction(&Instruction::I64Const(",
        "RegExpModifierOverride::ForceOn.operand_code()asi64,));",
        "function.instruction(&Instruction::I64Eq);",
        "function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));",
        "function.instruction(&Instruction::I64Const(1));",
        "function.instruction(&Instruction::Else);",
        "function.instruction(&Instruction::LocalGet(operand0));",
        "function.instruction(&Instruction::I64Const(",
        "RegExpModifierOverride::ForceOff.operand_code()asi64,));",
        "function.instruction(&Instruction::I64Eq);",
        "function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));",
        "function.instruction(&Instruction::I64Const(0));",
        "function.instruction(&Instruction::Else);",
        "function.instruction(&Instruction::LocalGet(source));",
        "function.instruction(&Instruction::End);",
        "function.instruction(&Instruction::End);",
        "function.instruction(&Instruction::LocalSet(effective));",
    )));
    assert_eq!(
        decoder
            .matches("RegExpModifierOverride::ForceOn.operand_code()")
            .count(),
        1
    );
    assert_eq!(
        decoder
            .matches("RegExpModifierOverride::ForceOff.operand_code()")
            .count(),
        1
    );
    assert!(decoder.contains("function.instruction(&Instruction::LocalGet(source));"));
}

#[test]
fn focused_ir_tests_cover_forced_modes_and_nested_restoration() {
    let tests = bounded(
        IR,
        "fn modifier_groups_encode_dot_all_and_multiline_overrides()",
        "    #[test]\n    fn direct_non_unicode_source_quantifies_only_its_utf16_trail_unit()",
    );
    for pattern in ["(?s:.)", "(?-s:.)", "(?m:^$)", "(?-m:^$)", "(?s:(?-s:.).)."] {
        assert!(
            tests.contains(pattern),
            "missing focused pattern `{pattern}`"
        );
    }
    for variant in ["Inherit", "ForceOn", "ForceOff"] {
        assert!(
            tests.contains(&format!("RegExpModifierOverride::{variant}.operand_code()")),
            "missing focused `{variant}` assertion"
        );
    }
    let normalized_tests = without_whitespace(tests);
    assert!(normalized_tests.contains(
        "assert_eq!(dot_operands,[RegExpModifierOverride::ForceOff.operand_code(),RegExpModifierOverride::ForceOn.operand_code(),RegExpModifierOverride::Inherit.operand_code(),]);"
    ));
}
