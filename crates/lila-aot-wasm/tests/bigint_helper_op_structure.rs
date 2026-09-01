use std::fs;
use std::path::Path;

const BIGINT_SOURCE: &str = include_str!("../src/bigint.rs");
const HELPER_OP_SOURCE: &str = include_str!("../src/bigint/helper_op.rs");
const LIB_SOURCE: &str = include_str!("../src/lib.rs");
const EXPRESSIONS_SOURCE: &str = include_str!("../src/expressions.rs");
const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
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

struct NormalizedRust {
    code: String,
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push_str(&source[offset..end]);
            identifiers.push(' ');
            routes.push('L');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            identifiers.push(' ');
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            identifiers.push(' ');
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
        if bytes.get(offset..offset + 2) == Some(b"r#")
            && source[offset + 2..]
                .chars()
                .next()
                .is_some_and(|character| character == '_' || character.is_alphabetic())
        {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            code.push(character);
            identifiers.push(character);
            routes.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
        code,
        identifiers,
        routes,
    }
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn exact_route_count(source: &str, route: &str) -> usize {
    source
        .match_indices(route)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + route.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&normalize_rust(&source).identifiers, identifier)
        })
        .sum()
}

fn count_route_in_rust_sources(dir: &Path, route: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_route_in_rust_sources(&path, route);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_route_count(&normalize_rust(&source).routes, route)
        })
        .sum()
}

fn normalized_routes_in_rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return normalized_routes_in_rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let mut routes = normalize_rust(&source).routes;
            routes.push('\n');
            routes
        })
        .collect()
}

fn rust_identifier_tokens(source: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (offset, character) in source.char_indices() {
        if character.is_alphanumeric() || character == '_' {
            start.get_or_insert(offset);
        } else if let Some(start) = start.take() {
            tokens.push(&source[start..offset]);
        }
    }
    if let Some(start) = start {
        tokens.push(&source[start..]);
    }
    tokens
}

fn assert_no_i64_cast_from_identifiers(source: &str, forbidden: &[&str]) {
    let normalized = normalize_rust(source);
    let tokens = rust_identifier_tokens(&normalized.identifiers);
    for window in tokens.windows(3) {
        assert!(
            !(forbidden.contains(&window[0]) && window[1] == "as" && window[2] == "i64"),
            "raw i64 cast from `{}`",
            window[0]
        );
    }
}

fn assert_no_bigint_helper_op_variant_casts_in_rust_sources(dir: &Path) {
    for entry in fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
    {
        let path = entry.expect("failed to read Rust source entry").path();
        if path.is_dir() {
            assert_no_bigint_helper_op_variant_casts_in_rust_sources(&path);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
        let normalized = normalize_rust(&source);
        let tokens = rust_identifier_tokens(&normalized.identifiers);
        for window in tokens.windows(4) {
            assert!(
                !(window[0] == "BigIntHelperOp"
                    && [
                        "Add",
                        "Sub",
                        "Mul",
                        "Div",
                        "Rem",
                        "Exp",
                        "Compare",
                        "Negate",
                        "CompareWithNumber",
                        "BitAnd",
                        "BitOr",
                        "BitXor",
                        "Shl",
                        "Shr",
                    ]
                    .contains(&window[1])
                    && window[2] == "as"
                    && window[3] == "i64"),
                "raw BigIntHelperOp i64 cast in {}",
                path.display()
            );
        }
    }
}

fn assert_exact_call(region: &str, call: &str, meaning: &str) {
    assert_eq!(
        normalize_rust(region).routes.matches(call).count(),
        1,
        "{meaning}"
    );
}

#[test]
fn bigint_helper_op_has_one_private_capability_free_owner() {
    let lexical_probe = r###"
        BigIntHelperOp /* nested /* ignored */ comment */ :: r#Add;
        // BigIntHelperOp::Sub
        let normal = "BigIntHelperOp::Mul";
        let byte = b"BigIntHelperOp::Div";
        let c_string = c"BigIntHelperOp::Rem";
        let raw = r#"BigIntHelperOp::Exp"#;
        let raw_byte = br#"BigIntHelperOp::Compare"#;
        let raw_c = cr#"BigIntHelperOp::Negate"#;
        let character = ':';
        let byte_character = b':';
        let borrowed: &'a str = value;
    "###;
    let normalized_probe = normalize_rust(lexical_probe);
    assert_eq!(
        normalized_probe.routes,
        concat!(
            "BigIntHelperOp::Add;",
            "letnormal=L;letbyte=L;letc_string=L;letraw=L;letraw_byte=L;",
            "letraw_c=L;letcharacter=L;letbyte_character=L;letborrowed:&'astr=value;"
        )
    );
    assert!(normalized_probe
        .code
        .contains("letraw_c=cr#\"BigIntHelperOp::Negate\"#;"));
    assert_eq!(
        exact_identifier_count(&normalized_probe.identifiers, "BigIntHelperOp"),
        1
    );
    assert_eq!(
        exact_route_count(&normalized_probe.routes, "BigIntHelperOp::Add"),
        1
    );

    assert_eq!(BIGINT_SOURCE.matches("\nmod helper_op;\n").count(), 1);
    assert_eq!(
        BIGINT_SOURCE
            .matches("pub(crate) use helper_op::BigIntHelperOp;")
            .count(),
        1
    );
    assert!(!BIGINT_SOURCE.contains("\npub mod helper_op;\n"));
    assert_eq!(
        normalize_rust(
            HELPER_OP_SOURCE
                .split_once("pub(crate) enum BigIntHelperOp {")
                .expect("BigInt helper-operation declaration")
                .0
        )
        .routes,
        "usesuper::*;"
    );
    let declaration = bounded(
        HELPER_OP_SOURCE,
        "pub(crate) enum BigIntHelperOp {",
        "\n}\n\nimpl BigIntHelperOp {",
    );
    assert_eq!(
        normalize_rust(declaration).routes,
        "Add,Sub,Mul,Div,Rem,Exp,Compare,Negate,CompareWithNumber,BitAnd,BitOr,BitXor,Shl,Shr,"
    );

    let normalized_all_sources = [
        HELPER_OP_SOURCE,
        BIGINT_SOURCE,
        LIB_SOURCE,
        EXPRESSIONS_SOURCE,
        OPERATIONS_SOURCE,
    ]
    .map(normalize_rust)
    .map(|source| source.routes)
    .concat();
    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!normalized_all_sources.contains(&format!("impl{capability}forBigIntHelperOp")));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "BigIntHelperOp"),
        36,
        "the owner, reexports, four typed carriers, nine producer routes and wire-code sites own every code identifier"
    );
    for (route, expected) in [
        ("BigIntHelperOp::from_arithmetic", 3),
        ("BigIntHelperOp::from_bitwise", 1),
        ("BigIntHelperOp::Add", 3),
        ("BigIntHelperOp::Sub", 1),
        ("BigIntHelperOp::Mul", 1),
        ("BigIntHelperOp::Div", 1),
        ("BigIntHelperOp::Rem", 0),
        ("BigIntHelperOp::Exp", 1),
        ("BigIntHelperOp::Compare", 3),
        ("BigIntHelperOp::Negate", 2),
        ("BigIntHelperOp::CompareWithNumber", 3),
        ("BigIntHelperOp::BitAnd", 2),
        ("BigIntHelperOp::BitOr", 2),
        ("BigIntHelperOp::BitXor", 2),
        ("BigIntHelperOp::Shl", 2),
        ("BigIntHelperOp::Shr", 1),
    ] {
        assert_eq!(
            count_route_in_rust_sources(&source_root, route),
            expected,
            "{route}"
        );
    }
}

#[test]
fn bigint_helper_op_owns_all_three_exhaustive_tables() {
    let implementation = HELPER_OP_SOURCE
        .split_once("impl BigIntHelperOp {")
        .expect("BigInt helper-operation implementation")
        .1;
    let normalized_implementation = normalize_rust(implementation).routes;
    assert_eq!(
        normalized_implementation,
        concat!(
            "pub(crate)constfnruntime_code(&self)->i64{matchself{",
            "Self::Add=>0,Self::Sub=>1,Self::Mul=>2,Self::Div=>3,Self::Rem=>4,",
            "Self::Exp=>5,Self::Compare=>6,Self::Negate=>7,Self::CompareWithNumber=>8,",
            "Self::BitAnd=>9,Self::BitOr=>10,Self::BitXor=>11,Self::Shl=>12,Self::Shr=>13,}}",
            "pub(crate)constfnfrom_arithmetic(op:ArithmeticBinaryOp)->Self{matchop{",
            "ArithmeticBinaryOp::Add=>Self::Add,ArithmeticBinaryOp::Sub=>Self::Sub,",
            "ArithmeticBinaryOp::Mul=>Self::Mul,ArithmeticBinaryOp::Div=>Self::Div,",
            "ArithmeticBinaryOp::Mod=>Self::Rem,ArithmeticBinaryOp::Exp=>Self::Exp,}}",
            "pub(crate)constfnfrom_bitwise(op:BigIntBitwiseOp)->Self{matchop{",
            "BigIntBitwiseOp::And=>Self::BitAnd,BigIntBitwiseOp::Or=>Self::BitOr,",
            "BigIntBitwiseOp::Xor=>Self::BitXor,BigIntBitwiseOp::Shl=>Self::Shl,",
            "BigIntBitwiseOp::Shr=>Self::Shr,}}}"
        )
    );
    assert_eq!(normalized_implementation.matches("match").count(), 3);
    assert!(!normalized_implementation.contains("_=>"));
}

#[test]
fn bigint_helper_op_has_exactly_nine_semantic_producers() {
    let complement = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn emit_bigint_complement_to_locals(",
        "    pub(crate) fn emit_bigint_relational_i32(",
    );
    assert_exact_call(
        complement,
        concat!(
            "self.emit_bigint_binary_op_to_locals(BigIntHelperOp::BitXor,",
            "operand_payload_local,operand_tag_local,minus_one_payload_local,",
            "minus_one_tag_local,out_payload_local,out_tag_local,function,)?;"
        ),
        "complement must XOR the operand with the synthetic minus-one pair",
    );

    let relational = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn emit_bigint_relational_i32(",
        "    pub(crate) fn emit_bigint_number_relational_i32(",
    );
    assert_exact_call(
        relational,
        concat!(
            "self.emit_bigint_comparison_i32(BigIntHelperOp::Compare,op,",
            "lhs_payload_local,lhs_tag_local,rhs_payload_local,rhs_tag_local,function,)"
        ),
        "BigInt relational comparison must preserve lhs then rhs",
    );

    let number_relational = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn emit_bigint_number_relational_i32(",
        "    pub(crate) fn emit_bigint_compare_i64(",
    );
    assert_exact_call(
        number_relational,
        concat!(
            "self.emit_bigint_comparison_i32(BigIntHelperOp::CompareWithNumber,op,",
            "bigint_payload_local,bigint_tag_local,number_payload_local,",
            "number_payload_local,function,)"
        ),
        "mixed relational comparison must preserve BigInt then Number",
    );

    let compare_i64 = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn emit_bigint_compare_i64(",
        "    fn emit_bigint_comparison_payload(",
    );
    assert_exact_call(
        compare_i64,
        concat!(
            "self.emit_bigint_comparison_payload(BigIntHelperOp::Compare,",
            "lhs_payload_local,lhs_tag_local,rhs_payload_local,rhs_tag_local,function,)?;"
        ),
        "three-way BigInt comparison must preserve lhs then rhs",
    );

    let binary_number = bounded(
        EXPRESSIONS_SOURCE,
        "            ExprIr::BinaryNumber { op, lhs, rhs } => {",
        "            ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {",
    );
    let coercive_binary_number = bounded(
        EXPRESSIONS_SOURCE,
        "            ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {",
        "            ExprIr::BitwiseNumeric { op, lhs, rhs } => {",
    );
    assert_exact_call(
        binary_number,
        concat!(
            "self.compile_bigint_arithmetic_to_locals(",
            "BigIntHelperOp::from_arithmetic(*op),lhs,rhs,self.scratch_local,",
            "self.result_tag_local,function,)?;"
        ),
        "static BigInt arithmetic must preserve lhs then rhs",
    );
    assert_exact_call(
        coercive_binary_number,
        concat!(
            "self.emit_bigint_binary_op_to_locals(",
            "BigIntHelperOp::from_arithmetic(*op),lhs_payload,lhs_tag,rhs_payload,rhs_tag,",
            "self.scratch_local,self.result_tag_local,function,)?;"
        ),
        "coercive BigInt arithmetic must preserve lhs then rhs",
    );

    let bitwise = bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_bitwise_numeric_to_locals(",
        "    pub(crate) fn compile_unary_minus_numeric_to_locals(",
    );
    assert_exact_call(
        bitwise,
        concat!(
            "Some(bigint_op)=>self.emit_bigint_binary_op_to_locals(",
            "BigIntHelperOp::from_bitwise(bigint_op),lhs_payload_local,lhs_tag_local,",
            "rhs_payload_local,rhs_tag_local,payload_local,tag_local,function,)?"
        ),
        "dynamic bitwise operation must preserve lhs then rhs",
    );
    let unary_minus = bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_unary_minus_numeric_to_locals(",
        "    pub(crate) fn compile_unary_bitwise_numeric_to_locals(",
    );
    assert_exact_call(
        unary_minus,
        concat!(
            "self.emit_bigint_binary_op_to_locals(BigIntHelperOp::Negate,",
            "operand_payload_local,operand_tag_local,operand_payload_local,operand_tag_local,",
            "payload_local,tag_local,function,)?;"
        ),
        "BigInt negation must duplicate the converted operand pair",
    );
    let coercive_arithmetic = bounded(
        OPERATIONS_SOURCE,
        "    pub(crate) fn compile_coercive_binary_number_to_locals(",
        "    pub(crate) fn emit_primitive_to_numeric_locals_without_throw_return(",
    );
    assert_exact_call(
        coercive_arithmetic,
        concat!(
            "self.emit_bigint_binary_op_to_locals(BigIntHelperOp::from_arithmetic(op),",
            "lhs_payload_local,lhs_tag_local,rhs_payload_local,rhs_tag_local,",
            "payload_local,tag_local,function,)?;"
        ),
        "runtime arithmetic must preserve lhs then rhs",
    );
}

#[test]
fn bigint_helper_op_serializes_only_through_the_exact_runtime_code_authority() {
    let closure_probe = r###"
        BigIntHelperOp /* route */ :: r#runtime_code :: <Marker>(&r#op);
        r#helper_op /* cast */ as r#i64;
    "###;
    let normalized_probe = normalize_rust(closure_probe);
    assert_eq!(
        normalized_probe.routes,
        "BigIntHelperOp::runtime_code::<Marker>(&op);helper_opasi64;"
    );
    assert_eq!(
        rust_identifier_tokens(&normalized_probe.identifiers),
        [
            "BigIntHelperOp",
            "runtime_code",
            "Marker",
            "op",
            "helper_op",
            "as",
            "i64"
        ]
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    let normalized_bigint = normalize_rust(BIGINT_SOURCE);
    assert_eq!(
        normalized_bigint.routes.matches(".runtime_code()").count(),
        21
    );
    assert_no_i64_cast_from_identifiers(BIGINT_SOURCE, &["op", "helper_op"]);
    assert_no_bigint_helper_op_variant_casts_in_rust_sources(&source_root);
    assert!(!all_routes.contains("BigIntHelperOp::runtime_code"));
    assert!(!all_routes.contains("<BigIntHelperOp>::runtime_code"));
    assert!(!all_routes.contains("BigIntHelperOpas"));
    let mut direct_serializer_count = 0;
    for (variant, expected) in [
        ("Add", 3),
        ("Sub", 1),
        ("Mul", 1),
        ("Div", 1),
        ("Rem", 0),
        ("Exp", 1),
        ("Compare", 1),
        ("Negate", 1),
        ("CompareWithNumber", 2),
        ("BitAnd", 2),
        ("BitOr", 2),
        ("BitXor", 1),
        ("Shl", 2),
        ("Shr", 1),
    ] {
        let route = format!("BigIntHelperOp::{variant}");
        let serializer = format!("{route}.runtime_code()");
        assert_eq!(
            exact_route_count(&all_routes, &serializer),
            expected,
            "{variant} recursive direct serializer"
        );
        assert_eq!(exact_route_count(&all_routes, &format!("{route}asi64")), 0);
        direct_serializer_count += expected;
    }
    assert_eq!(direct_serializer_count, 19);

    let helper_body = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn compile_bigint_arithmetic_helper(",
        "    fn emit_bigint_digit_address(",
    );
    for (site, meaning) in [
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(",
                "BigIntHelperOp::CompareWithNumber.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_decode_number_operand(2,rhs_sign,rhs_ptr,rhs_len,",
                "fraction_sign,number_class,&mutfunction,)?;"
            ),
            "mixed comparison selects Number decoding",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Sub.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::LocalGet(rhs_sign));",
                "function.instruction(&Instruction::I64Sub);",
                "function.instruction(&Instruction::LocalSet(rhs_sign));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Add.runtime_code()));",
                "function.instruction(&Instruction::LocalSet(op_local));",
                "function.instruction(&Instruction::End);"
            ),
            "subtraction normalizes to signed addition",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Negate.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::LocalGet(lhs_sign));",
                "function.instruction(&Instruction::I64Sub);",
                "function.instruction(&Instruction::LocalSet(lhs_sign));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::LocalSet(rhs_sign));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::LocalSet(rhs_len));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Add.runtime_code()));",
                "function.instruction(&Instruction::LocalSet(op_local));",
                "function.instruction(&Instruction::End);"
            ),
            "negation normalizes to signed addition with a zero rhs",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Compare.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(",
                "BigIntHelperOp::CompareWithNumber.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::I32Or);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_signed_compare(lhs_sign,lhs_ptr,lhs_len,rhs_sign,rhs_ptr,",
                "rhs_len,res_sign,&mutfunction,);"
            ),
            "both comparison rows enter the signed comparison body",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Add.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_signed_add(lhs_sign,lhs_ptr,lhs_len,rhs_sign,rhs_ptr,rhs_len,",
                "res_sign,res_ptr,res_len,&mutfunction,)?;"
            ),
            "addition selects signed addition",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Mul.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_magnitude_mul(lhs_ptr,lhs_len,rhs_ptr,rhs_len,res_ptr,res_len,",
                "&mutfunction,)?;"
            ),
            "multiplication selects magnitude multiplication",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Exp.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_pow(lhs_sign,lhs_ptr,lhs_len,rhs_sign,rhs_ptr,rhs_len,",
                "res_sign,res_ptr,res_len,&mutfunction,)?;"
            ),
            "exponentiation selects the power body",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::BitAnd.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::BitOr.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::I32Or);",
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::BitXor.runtime_code(),));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::I32Or);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_bitwise(op_local,lhs_sign,lhs_ptr,lhs_len,rhs_sign,rhs_ptr,",
                "rhs_len,res_sign,res_ptr,res_len,&mutfunction,)?;"
            ),
            "three bitwise rows enter the bitwise body",
        ),
        (
            concat!(
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Shl.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::LocalGet(op_local));",
                "function.instruction(&Instruction::I64Const(BigIntHelperOp::Shr.runtime_code()));",
                "function.instruction(&Instruction::I64Eq);",
                "function.instruction(&Instruction::I32Or);",
                "function.instruction(&Instruction::If(BlockType::Empty));",
                "self.emit_bigint_shift(op_local,lhs_sign,lhs_ptr,lhs_len,rhs_sign,rhs_ptr,",
                "rhs_len,res_sign,res_ptr,res_len,&mutfunction,)?;"
            ),
            "both shift rows enter the shift body",
        ),
    ] {
        assert_exact_call(helper_body, site, meaning);
    }

    let bitwise_body = bounded(
        BIGINT_SOURCE,
        "    fn emit_bigint_bitwise(",
        "    fn emit_bigint_shift(",
    );
    assert_exact_call(
        bitwise_body,
        concat!(
            "function.instruction(&Instruction::LocalGet(op_local));",
            "function.instruction(&Instruction::I64Const(BigIntHelperOp::BitAnd.runtime_code(),));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));",
            "function.instruction(&Instruction::LocalGet(lhs_digit));",
            "function.instruction(&Instruction::LocalGet(rhs_digit));",
            "function.instruction(&Instruction::I64And);",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(op_local));",
            "function.instruction(&Instruction::I64Const(BigIntHelperOp::BitOr.runtime_code()));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));",
            "function.instruction(&Instruction::LocalGet(lhs_digit));",
            "function.instruction(&Instruction::LocalGet(rhs_digit));",
            "function.instruction(&Instruction::I64Or);",
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(lhs_digit));",
            "function.instruction(&Instruction::LocalGet(rhs_digit));",
            "function.instruction(&Instruction::I64Xor);"
        ),
        "bitwise row codes must select AND, OR and XOR in order",
    );

    let shift_body = bounded(
        BIGINT_SOURCE,
        "    fn emit_bigint_shift(",
        "    fn emit_bigint_shift_left(",
    );
    assert_exact_call(
        shift_body,
        concat!(
            "function.instruction(&Instruction::LocalGet(op_local));",
            "function.instruction(&Instruction::I64Const(BigIntHelperOp::Shl.runtime_code()));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::LocalGet(rhs_sign));",
            "function.instruction(&Instruction::I64Const(0));",
            "function.instruction(&Instruction::I64LtS);",
            "function.instruction(&Instruction::I32Xor);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.emit_bigint_shift_left(lhs_sign,lhs_ptr,lhs_len,rhs_ptr,rhs_len,res_sign,",
            "res_ptr,res_len,function,)?;"
        ),
        "left-shift code and negative-count polarity must select shift-left",
    );

    let divmod_body = bounded(
        BIGINT_SOURCE,
        "    fn emit_bigint_divmod_op(",
        "    fn emit_bigint_pow(",
    );
    assert_exact_call(
        divmod_body,
        concat!(
            "function.instruction(&Instruction::LocalGet(op_local));",
            "function.instruction(&Instruction::I64Const(BigIntHelperOp::Div.runtime_code()));",
            "function.instruction(&Instruction::I64Eq);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "function.instruction(&Instruction::LocalGet(q_ptr));",
            "function.instruction(&Instruction::LocalSet(res_ptr));",
            "function.instruction(&Instruction::LocalGet(q_len));",
            "function.instruction(&Instruction::LocalSet(res_len));"
        ),
        "division code must select the quotient before the remainder else arm",
    );

    let binary_serializer = bounded(
        BIGINT_SOURCE,
        "    pub(crate) fn emit_bigint_binary_op_to_locals(",
        "    pub(crate) fn emit_bigint_complement_to_locals(",
    );
    assert_eq!(
        normalize_rust(binary_serializer)
            .routes
            .matches(concat!(
                "function.instruction(&Instruction::LocalGet(lhs_payload_local));",
                "function.instruction(&Instruction::LocalGet(lhs_tag_local));",
                "function.instruction(&Instruction::LocalGet(rhs_payload_local));",
                "function.instruction(&Instruction::LocalGet(rhs_tag_local));",
                "function.instruction(&Instruction::I64Const(op.runtime_code()));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::Call(helper));"
            ))
            .count(),
        1
    );
    assert_eq!(
        normalize_rust(binary_serializer)
            .routes
            .matches("op.runtime_code()")
            .count(),
        1
    );

    let comparison_serializer = bounded(
        BIGINT_SOURCE,
        "    fn emit_bigint_comparison_payload(",
        "    fn emit_bigint_comparison_i32(",
    );
    assert_eq!(
        normalize_rust(comparison_serializer)
            .routes
            .matches(concat!(
                "function.instruction(&Instruction::LocalGet(lhs_payload_local));",
                "function.instruction(&Instruction::LocalGet(lhs_tag_local));",
                "function.instruction(&Instruction::LocalGet(rhs_payload_local));",
                "function.instruction(&Instruction::LocalGet(rhs_tag_local));",
                "function.instruction(&Instruction::I64Const(helper_op.runtime_code()));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::I64Const(0));",
                "function.instruction(&Instruction::Call(helper));"
            ))
            .count(),
        1
    );
    assert_eq!(
        normalize_rust(comparison_serializer)
            .routes
            .matches("helper_op.runtime_code()")
            .count(),
        1
    );
}
