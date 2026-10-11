use boa_ast::{
    expression::Expression,
    pattern::{ObjectPattern, ObjectPatternElement},
    property::PropertyName,
    visitor::{VisitWith, Visitor},
};
use boa_interner::Interner;
use lila_front::{parse, ParseOptions};
use std::ops::ControlFlow;

#[test]
fn computed_assignment_names_preserve_nested_patterns_and_defaults() {
    for source in [
        "var value; ({[key()]: value = fallback()} = source);",
        "var value; ({[key()]: {value}} = source);",
        "var value; ({[key()]: [value]} = source);",
        "var value; ({[key()]: {value} = fallback()} = source);",
        "function* values(source) { var value; ({[yield 'key']: value = yield 'default'} = source); }",
        "function* values(source) { var value; ({[yield 'outer']: {[yield 'inner']: value = yield 'default'}} = source); }",
        "function* values(source) { var value; ({[yield 'outer']: [value]} = source); }",
    ] {
        for options in [ParseOptions::script(),ParseOptions::module()] {
            parse(source,options).unwrap_or_else(|error| panic!("{source}: {error}"));
        }
    }
}

struct DefaultNames<'a> {
    interner: &'a Interner,
    names: Vec<(String, String)>,
}

impl<'ast> Visitor<'ast> for DefaultNames<'_> {
    type BreakTy = ();
    fn visit_object_pattern(&mut self, pattern: &'ast ObjectPattern) -> ControlFlow<()> {
        for element in pattern.bindings() {
            if let ObjectPatternElement::SingleName {
                name: PropertyName::Computed(_),
                ident,
                default_init: Some(default),
            } = element
            {
                let actual = match default {
                    Expression::FunctionExpression(function) => function.name(),
                    Expression::ArrowFunction(function) => function.name(),
                    Expression::ClassExpression(class) => class.name(),
                    _ => None,
                }
                .expect("actual NamedEvaluation label");
                self.names.push((
                    self.interner.resolve_expect(ident.sym()).to_string(),
                    self.interner.resolve_expect(actual.sym()).to_string(),
                ));
            }
        }
        pattern.visit_with(self)
    }
}

#[test]
fn computed_single_name_defaults_keep_inferred_and_explicit_definition_names() {
    let parsed = parse("var f,a,c,e,n; ({[key()]: f = function(){}, [key()]: a = ()=>{}, [key()]: c = class {}, [key()]: e = function Explicit(){}, [key()]: n = class Named{}} = source);",ParseOptions::script()).unwrap();
    let names = parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, interner| {
            let mut visitor = DefaultNames {
                interner,
                names: Vec::new(),
            };
            let _ = script.visit_with(&mut visitor);
            visitor.names
        });
    assert_eq!(
        names,
        vec![
            ("f".into(), "f".into()),
            ("a".into(), "a".into()),
            ("c".into(), "c".into()),
            ("e".into(), "Explicit".into()),
            ("n".into(), "Named".into())
        ]
    );
}

#[test]
fn cover_conversion_preserves_invalid_targets_rest_position_and_strict_errors() {
    for source in [
        "({[key()]: 1 = fallback()} = source);",
        "({[key()]: value += 1} = source);",
        "({...rest, [key()]: value} = source);",
        "'use strict'; ({[key()]: eval = fallback()} = source);",
        "'use strict'; ({[key()]: arguments = fallback()} = source);",
    ] {
        assert!(
            parse(source, ParseOptions::script()).is_err(),
            "invalid pattern: {source}"
        );
    }
}

#[test]
fn parenthesized_simple_targets_remain_valid_in_assignment_patterns() {
    for source in [
        "var x, target = {}; [(x), ((target.value))] = source;",
        "var x, target = {}; ({value: (x), other: ((target.value))} = source);",
        "var target = {}; [...((target.rest))] = source;",
        "var target = {}; ({...((target.rest))} = source);",
        "var target = {}; for ([(target.value)] of source) {}",
        "var target = {}; for ({value: (target.value)} of source) {}",
        "var target = {method(source) { [(super.value)] = source; ({...((super.rest))} = source); }};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).unwrap_or_else(|error| panic!("{source}: {error}"));
        }
    }
}

#[test]
fn grouping_never_admits_patterns_calls_or_optional_assignment_targets() {
    for source in [
        "[({x})] = source;",
        "[([x])] = source;",
        "({...({x})} = source);",
        "[...(call())] = source;",
        "({value: (call())} = source);",
        "[(target?.value)] = source;",
        "'use strict'; [(eval)] = source;",
        "'use strict'; ({value: (arguments)} = source);",
        "'use strict'; ({...(eval)} = source);",
        "'use strict'; [...(arguments)] = source;",
    ] {
        assert!(
            parse(source, ParseOptions::script()).is_err(),
            "invalid grouped target: {source}"
        );
    }
}
