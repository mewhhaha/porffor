use std::collections::{BTreeMap, BTreeSet};
use std::panic::{self, AssertUnwindSafe};

use boa_ast::property::{MethodDefinitionKind, PropertyName};
use boa_ast::{
    declaration::{Binding, ExportDeclaration, LexicalDeclaration, VarDeclaration, Variable},
    expression::access::{
        PrivatePropertyAccess, PropertyAccess, PropertyAccessField, SuperPropertyAccess,
    },
    expression::literal::{
        ArrayLiteral, LiteralKind, ObjectLiteral, ObjectMethodDefinition, PropertyDefinition,
        TemplateElement, TemplateLiteral,
    },
    expression::operator::{
        assign::{AssignOp, AssignTarget},
        binary::{ArithmeticOp, BinaryInPrivate, BinaryOp, BitwiseOp, LogicalOp, RelationalOp},
        unary::UnaryOp,
        update::{UpdateOp, UpdateTarget},
    },
    expression::New,
    expression::{Call, Expression, RegExpLiteral, SuperCall},
    function::{
        ArrowFunction, ClassDeclaration, ClassElement, ClassElementName, ClassExpression,
        ClassMethodDefinition, FormalParameter, FormalParameterList, FunctionBody,
        FunctionDeclaration, FunctionExpression, PrivateName, StaticBlockBody,
    },
    function::{GeneratorDeclaration, GeneratorExpression},
    pattern::{ArrayPatternElement, ObjectPatternElement, Pattern},
    scope::Scope,
    statement::{
        iteration::{
            Break as AstBreak, Continue as AstContinue, DoWhileLoop, ForLoop, ForLoopInitializer,
            ForOfLoop, IterableLoopInitializer, WhileLoop,
        },
        Block, If, Labelled as AstLabelled, LabelledItem, Return as AstReturn, Statement,
        Switch as AstSwitch, Throw as AstThrow, Try as AstTry,
    },
    Declaration, ModuleItem, Script, Spanned, StatementListItem,
};
use boa_interner::Interner;
use boa_parser::{Parser, Source};
use num_bigint::{BigInt, BigUint, Sign};
use num_traits::{One, ToPrimitive, Zero};
use porffor_front::{ParseGoal, SourceUnit};
use regress::Regex;

mod analysis;
mod builtins;
mod diagnostics;
mod early_errors;
mod ir;
mod lowering;
mod lowering_helpers;
mod names;
mod operations;
pub(crate) use analysis::*;
pub use builtins::{CallableToStringRepresentation, HostBuiltinId, StandardBuiltinId};
pub use diagnostics::{IrDiagnostic, IrDiagnosticKind, LoweringStage};
pub(crate) use early_errors::validate_derived_constructor_body;
pub use ir::*;
pub(crate) use ir::{read_heap_shape_property, summarize_block};
pub use lowering::lower;
pub(crate) use lowering_helpers::*;
pub use operations::{
    find_spec_operation, spec_operation_catalog, ArithmeticBinaryOp, BindingMode, BitwiseBinaryOp,
    CompletionAbruptKind, EqualityBinaryOp, LogicalBinaryOp, NumericUpdateOp,
    OperationLoweringStatus, RelationalBinaryOp, SpecOperationCatalogEntry, SpecOperationFamily,
    SpecOperationIr, ToPrimitiveHint, UnaryNumericOp, UpdateReturnMode, SPEC_OPERATION_CATALOG,
};

pub use names::*;
pub(crate) use names::{
    MAX_ARRAY_INDEX, MAX_STATIC_ARRAY_SHAPE_INDEX, SCRIPT_OWNER_ID, TDZ_BINDING_STORAGE_PREFIX,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lowering::ScriptLowerer;
    use porffor_front::{parse, ParseOptions};

    fn lower_script(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        lower(&source)
    }

    fn lower_module(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::module()).expect("module should parse");
        lower(&source)
    }

    #[test]
    fn lowers_simple_script_ir() {
        let program = lower_script("let x = 40; const y = 2; x + y;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.body.statements.len(), 3);
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn lowers_no_import_module_export_declaration() {
        let program = lower_module("export const value = 1; value;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn rejects_module_imports_explicitly() {
        let program = lower_module("import value from './dep.js'; value;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("module imports")));
    }

    #[test]
    fn lowers_assignment_and_if_ir() {
        let program = lower_script("let x = 0; if (!x) { x = 5; } x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        assert!(matches!(script.body.statements[1], StatementIr::If { .. }));
        assert!(program.ir_summary().contains("ifs=1"));
        assert!(program.ir_summary().contains("assigns=1"));
    }

    #[test]
    fn lowers_object_keys_join_before_control_as_direct_array_join() {
        let program =
            lower_script(r#"var o = { a: 1 }; Object.keys(o).join(""); if (false) {} 1;"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.body.statements.iter().any(|statement| {
            matches!(
                statement,
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::CallMethod {
                        key: PropertyKeyIr::StaticString(name),
                        ..
                    },
                    ..
                }) if name == "join"
            )
        }));
    }

    #[test]
    fn lowers_loop_ir() {
        let program = lower_script("let i = 0; while (i < 3) { i = i + 1; continue; } i;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements[1],
            StatementIr::While { .. }
        ));
        assert!(program.ir_summary().contains("whiles=1"));
        assert!(program.ir_summary().contains("continues=1"));
    }

    #[test]
    fn lowers_for_multi_binding_lexical_init_ir() {
        let program = lower_script("for (let i = 0, j = 1; i < 1; i = i + 1) { j; }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::For {
            init: Some(ForInitIr::LexicalBlock(bindings)),
            ..
        } = &script.body.statements[0]
        else {
            panic!("expected multi-binding lexical for initializer");
        };
        assert_eq!(bindings.len(), 2);
        assert!(program.ir_summary().contains("lets=2"));
    }

    #[test]
    fn lowers_update_and_compound_ir() {
        let program = lower_script("let i = 2; let x = i++; x += ++i; x;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
        let summary = program.ir_summary();
        assert!(summary.contains("postfix_updates=1"));
        assert!(summary.contains("prefix_updates=1"));
        assert!(summary.contains("compound_assigns=1"));
    }

    #[test]
    fn lowers_string_compound_add_with_dynamic_rhs_ir() {
        let program = lower_script(
            r#"let result = String.fromCodePoint.apply(null, [65]); result += String.fromCodePoint.apply(null, [66]); result;"#,
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::String);
        assert!(program.ir_summary().contains("compound_assigns=1"));
    }

    #[test]
    fn lowers_dynamic_primitive_ir() {
        let program = lower_script(
            "let x = 0; x || \"fallback\"; null ?? 3; typeof missingName; \"a\" + \"b\";",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("string_concats=1"));
        assert!(summary.contains("typeof_uses=1"));
        assert!(summary.contains("nullish_ops=1"));
    }

    #[test]
    fn operations_lowers_boolean_call_to_to_boolean_spec_operation() {
        let program = lower_script("Boolean(globalThis.flag);");
        assert!(program.is_wasm_supported());
        assert!(program.ir_summary().contains("spec_operations=1"));
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::SpecOperation {
            operation,
            operands,
        } = &expr.expr
        else {
            panic!("expected Boolean call to lower to a spec operation");
        };
        assert_eq!(*operation, SpecOperationIr::ToBoolean);
        assert_eq!(operands.len(), 1);
    }

    #[test]
    fn preserves_function_or_for_htmldda_truthiness() {
        let program = lower_script("function f() {} f || 2;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::Or,
                ..
            }
        ));
    }

    #[test]
    fn lowers_identifier_logical_assignment_ir() {
        let program = lower_script("let value = 0; value ||= 2; value &&= 3; value ??= 4;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        for statement in &script.body.statements[1..] {
            let StatementIr::Expression(expr) = statement else {
                panic!("expected logical assignment expression statement");
            };
            let ExprIr::AssignIdentifier { value, .. } = &expr.expr else {
                panic!("expected identifier assignment");
            };
            assert!(matches!(value.expr, ExprIr::LogicalShortCircuit { .. }));
        }
    }

    #[test]
    fn lowers_simple_destructuring_assignment_patterns() {
        let program = lower_script(
            "var x; var base = {}; ([x = 1, base.y = 3] = [2, 4]); ({x = 5} = {x: 6});",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("assigns=2"));
        assert!(summary.contains("property_writes=1"));
        assert!(summary.contains("comma_ops=1"));
    }

    #[test]
    fn lowers_simple_destructuring_lexical_bindings() {
        let program = lower_script("const [x = 1] = [2]; const { y = 3 } = { y: 4 }; x + y;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(
            script.body.statements[0],
            StatementIr::Lexical { .. }
        ));
        assert!(matches!(
            script.body.statements[1],
            StatementIr::Lexical { .. }
        ));
    }

    #[test]
    fn lowers_coercion_core_ir() {
        let program = lower_script("1 == \"1\"; \"2\" - 1; \"10\" > \"2\"; void 1; (1, 2);");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("loose_equalities=1"));
        assert!(summary.contains("coercive_numeric_ops=1"));
        assert!(summary.contains("coercive_relational_ops=1"));
        assert!(summary.contains("void_uses=1"));
        assert!(summary.contains("comma_ops=1"));
    }

    #[test]
    fn lowers_object_function_property_call_arg_coercion_ir() {
        let program = lower_script(
            r#"var h = { get: function(_, key) { return key * 10; } }; h.get({}, "1");"#,
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("coercive_numeric_ops=2"));
    }

    #[test]
    fn lowers_heap_loose_equality_ir() {
        let program = lower_script("let object = {}; object == undefined; null != object;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("loose_equalities=2"));
        assert!(summary.contains("heap_loose_equalities=2"));
    }

    #[test]
    fn lowers_typeof_unresolved_identifier() {
        let program = lower_script("typeof missingName;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::TypeOfUnresolvedIdentifier { .. }
        ));
        assert_eq!(expr.kind, ValueKind::String);
    }

    #[test]
    fn lowers_symbol_key_for_without_global_symbol_object() {
        let program = lower_script("Symbol.keyFor(Symbol.iterator);");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert_eq!(expr.kind, ValueKind::Undefined);
        assert!(matches!(expr.expr, ExprIr::Conditional { .. }));
    }

    #[test]
    fn lowers_symbol_description_property() {
        let program = lower_script("Symbol.iterator.description.startsWith(\"Symbol.\");");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        assert_eq!(expr.kind, ValueKind::Boolean);
    }

    #[test]
    fn folds_temporal_ascii_identifier_helper_test() {
        let program = lower_script(
            "const ASCII_IDENTIFIER = /^[$_a-zA-Z][$_a-zA-Z0-9]*$/u; ASCII_IDENTIFIER.test(\"next\");",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(matches!(expr.expr, ExprIr::Boolean(true)));
    }

    #[test]
    fn lowers_switch_labels_and_debugger_ir() {
        let program = lower_script(
            "outer: while (true) { switch (2) { case 1: break; case 2: debugger; break outer; default: break; } }",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("switches=1"));
        assert!(summary.contains("labels=1"));
        assert!(summary.contains("debuggers=1"));
    }

    #[test]
    fn lowers_hoisted_var_ir() {
        let program = lower_script("x; var x = 1; if (true) { var y = 2; } y;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("vars=2"));
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(matches!(script.body.statements[1], StatementIr::Var(_)));
    }

    #[test]
    fn lowers_top_level_functions_and_calls() {
        let program = lower_script("add(1, 2); function add(x, y) { return x + y; }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        assert_eq!(script.functions[0].params.len(), 2);
        let summary = program.ir_summary();
        assert!(summary.contains("functions=2"));
        assert!(summary.contains("calls=1"));
        assert!(summary.contains("returns=2"));
    }

    #[test]
    fn marks_function_body_use_strict_directive() {
        let program = lower_script(r#"function f() { "use strict"; return this; } f();"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("function should be lowered");
        assert!(function.strict);
    }

    #[test]
    fn marks_script_use_strict_directive() {
        let program = lower_script(r#""use strict"; function f() { return this; } f();"#);
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.strict);
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("function should be lowered");
        assert!(function.strict);
    }

    #[test]
    fn lowers_block_function_declarations_as_hoisted_bindings() {
        let program = lower_script(
            "let value = 0; if (true) { value = nested(); function nested() { return 7; } } value;",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("functions=2"));
        assert!(summary.contains("calls=1"));
    }

    #[test]
    fn lowers_root_function_constructor_reference_inside_function_body() {
        let program = lower_script(
            "function Box(message) { this.message = message; } function make(message) { return new Box(message); } make(\"ok\");",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("constructs=1"));
    }

    #[test]
    fn exposes_gc_as_noop_host_builtin() {
        let program = lower_script("if (typeof gc === \"function\") { gc(); }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script.host_builtins.contains(&HostBuiltinId::Gc));
    }

    #[test]
    fn prunes_unresolved_typeof_function_guard() {
        let program =
            lower_script("if (typeof __missingHostHook === \"function\") { __missingHostHook(); }");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("calls=0"));
    }

    #[test]
    fn prunes_unresolved_typeof_and_guard_rhs() {
        let program = lower_script(
            "if (typeof Symbol !== \"undefined\" && Symbol.iterator) { missingCall(); }",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("calls=0"));
    }

    #[test]
    fn marks_explicit_extending_class_constructor_as_derived() {
        let program = lower_script("class A {} class B extends A { constructor() {} } B;");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let derived = script
            .functions
            .iter()
            .find(|function| function.name == "B")
            .expect("derived constructor should be lowered");
        assert!(derived.is_derived_constructor);
        assert!(derived.super_constructor_target.is_some());
    }

    #[test]
    fn carries_dynamic_class_heritage_to_runtime() {
        let program =
            lower_script("function make(Base) { class Derived extends Base {} return Derived; }");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("class_extends=1"));
    }

    #[test]
    fn narrows_identifier_used_as_object_property_key_to_string() {
        let program =
            lower_script("function get(obj, name) { return obj[name]; } get({ x: 1 }, \"x\");");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("property_reads=2"));
    }

    #[test]
    fn keeps_dynamic_string_property_key_on_possible_array_targets() {
        let program = lower_script(
            "function read(desc, name) { return desc.get[name]; } read({ get: function () {} }, \"length\");",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("property_reads=4"));
    }

    #[test]
    fn treats_typed_array_constructor_parameter_bytes_per_element_as_number() {
        let program =
            lower_script("function f(TA) { return 4 * TA.BYTES_PER_ELEMENT; } f(Int8Array);");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.result_kind(), ValueKind::Number);
    }

    #[test]
    fn lowers_objects_arrays_and_properties() {
        let program = lower_script("let o = { x: 1 }; let a = [1]; a[2] = 4; o.x;");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("objects=1"));
        assert!(summary.contains("arrays=1"));
        assert!(summary.contains("property_reads=1"));
        assert!(summary.contains("property_writes=1"));
    }

    #[test]
    fn folds_iterator_from_nullish_symbol_iterator_fallback() {
        let program = lower_script(
            "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let array_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
                _ => None,
            })
            .expect("array lexical should be present");
        let ExprIr::ArrayLiteral(elements) = &array_init.expr else {
            panic!("expected folded array literal, got {:?}", array_init.expr);
        };
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn folds_iterator_from_nullish_symbol_iterator_fallback_after_reassignment() {
        let program = lower_script(
            "function* g() { yield 0; yield 1; yield 2; }
             let iter = (function () {
               let n = g();
               return { [Symbol.iterator]: 0, next: () => n.next() };
             })();
             iter = (function () {
               let n = g();
               return { [Symbol.iterator]: null, next: () => n.next() };
             })();
             let array = Array.from(Iterator.from(iter));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let array_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "array" => Some(init),
                _ => None,
            })
            .expect("array lexical should be present");
        let ExprIr::ArrayLiteral(elements) = &array_init.expr else {
            panic!("expected folded array literal, got {:?}", array_init.expr);
        };
        assert_eq!(elements.len(), 3);
    }

    #[test]
    fn keeps_iterator_from_wrapper_return_observable() {
        let program = lower_script(
            "const iter = {};
             const wrapper = Iterator.from(iter);
             const result = wrapper.return();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let result_init = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "result" => Some(init),
                _ => None,
            })
            .expect("result lexical should be present");
        assert!(matches!(result_init.expr, ExprIr::CallIndirect { .. }));
    }

    #[test]
    fn gives_iterator_from_wrapper_the_wrapper_prototype_chain() {
        let program = lower_script(
            "const iter = { next() { return { done: true, value: undefined }; } };
             const wrapperPrototype = Object.getPrototypeOf(Iterator.from(iter));
             const iteratorPrototype = Object.getPrototypeOf(wrapperPrototype);",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let wrapper_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "wrapperPrototype" => Some(init),
                _ => None,
            })
            .expect("wrapper prototype lexical should be present");
        let iterator_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "iteratorPrototype" => {
                    Some(init)
                }
                _ => None,
            })
            .expect("iterator prototype lexical should be present");
        assert!(wrapper_prototype.heap_shape.is_some());
        assert_eq!(
            iterator_prototype.heap_shape.as_deref(),
            Some(ScriptLowerer::iterator_prototype_shape().as_ref())
        );
    }

    #[test]
    fn iterator_from_preserves_existing_iterator_instances() {
        let program = lower_script(
            "const GeneratorPrototype = Object.getPrototypeOf((function* () {})());
             const FromPrototype = Object.getPrototypeOf(Iterator.from((function* () {})()));",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let generator_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "GeneratorPrototype" => {
                    Some(init)
                }
                _ => None,
            })
            .expect("generator prototype lexical should be present");
        let from_prototype = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Lexical { name, init, .. } if name == "FromPrototype" => Some(init),
                _ => None,
            })
            .expect("from prototype lexical should be present");
        assert_eq!(
            generator_prototype.heap_shape.as_deref(),
            from_prototype.heap_shape.as_deref()
        );
    }

    #[test]
    fn allows_subclassing_iterator_constructor() {
        let program = lower_script("class SubIterator extends Iterator {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
            panic!("expected class lexical declaration");
        };
        assert!(matches!(init.kind, ValueKind::Function));
    }

    #[test]
    fn lowers_heap_shapes_and_array_length() {
        let program = lower_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions[0].return_kind, ValueKind::Object);
        assert!(script.functions[0].return_shape.is_some());
        let summary = program.ir_summary();
        assert!(summary.contains("array_lengths=1"));
        assert!(summary.contains("heap_shapes="));
    }

    #[test]
    fn lowers_property_access_on_dynamic_after_kind_merge() {
        let program = lower_script("let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;");
        assert!(program.is_wasm_supported());
        assert_eq!(
            program
                .script
                .as_ref()
                .expect("script ir should exist")
                .result_kind(),
            ValueKind::Dynamic
        );
    }

    #[test]
    fn lowers_nested_function_declaration() {
        let program =
            lower_script("function outer() { function inner() { return 1; } return inner(); }");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 3);
        assert!(script.functions.iter().any(|function| function.is_nested));
        let summary = program.ir_summary();
        assert!(summary.contains("nested_functions=2"));
    }

    #[test]
    fn lowers_closure_capture_and_function_expression() {
        let program = lower_script(
            "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 3);
        assert!(script
            .functions
            .iter()
            .any(|function| function.is_expression));
        assert!(script
            .functions
            .iter()
            .any(|function| !function.captured_bindings.is_empty()));
        let summary = program.ir_summary();
        assert!(summary.contains("function_exprs=1"));
        assert!(summary.contains("closures=3"));
        assert!(summary.contains("captures=1"));
    }

    #[test]
    fn lowers_script_closure_capture() {
        let program = lower_script("let x = 1; function f() { return x; } f();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.owned_env_bindings.len(), 1);
        assert_eq!(script.owned_env_bindings[0].name, "x");
        assert_eq!(script.functions[0].captured_bindings[0].name, "x");
        assert_eq!(
            script.functions[0].captured_bindings[0].slot,
            script.owned_env_bindings[0].slot
        );
        assert_eq!(script.functions[0].captured_bindings[0].hops, 0);
    }

    #[test]
    fn lowers_for_in_let_closure_capture_metadata() {
        let program = lower_script(
            "function fn(x) { let callbacks = []; for (let p in x) { callbacks.push(function () { return p; }); } return callbacks[0](); } fn({ a: 1 });",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let outer = script
            .functions
            .iter()
            .find(|function| function.name == "fn")
            .expect("outer function should be lowered");
        let owned_names = outer
            .owned_env_bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<Vec<_>>();
        let loop_binding = outer
            .owned_env_bindings
            .iter()
            .find(|binding| binding.name.starts_with("$forin.lex.") && binding.name.ends_with(".p"))
            .expect("loop binding should own an aliased env slot");
        assert!(owned_names.contains(&loop_binding.name.as_str()));
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("loop closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        assert_eq!(closure.captured_bindings[0].name, loop_binding.name);
        assert_eq!(closure.captured_bindings[0].slot, loop_binding.slot);
    }

    #[test]
    fn lowers_shadowed_for_in_let_closure_to_loop_binding() {
        let program = lower_script(
            "let x = 'outside'; var f; for (let x in { a: 0 }) { f = function () { return x; }; } f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("loop closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        let captured = &closure.captured_bindings[0];
        assert!(captured.name.starts_with("$forin.lex."));
        assert!(captured.name.ends_with(".x"));
        assert_ne!(captured.name, "x");
        assert!(script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == captured.name && binding.slot == captured.slot));
    }

    #[test]
    fn lowers_for_in_head_lexical_tdz_for_target_expression() {
        fn has_reference_error_throw(expr: &TypedExpr) -> bool {
            match &expr.expr {
                ExprIr::RuntimeThrow {
                    name: REFERENCE_ERROR_NAME,
                    ..
                } => true,
                ExprIr::ObjectLiteral(properties) => {
                    properties.iter().any(|property| match property {
                        ObjectPropertyIr::Data { value, .. }
                        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                            has_reference_error_throw(value)
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            has_reference_error_throw(key) || has_reference_error_throw(value)
                        }
                        ObjectPropertyIr::ComputedMethod { key, function }
                        | ObjectPropertyIr::ComputedGetter { key, function }
                        | ObjectPropertyIr::ComputedSetter { key, function } => {
                            has_reference_error_throw(key) || has_reference_error_throw(function)
                        }
                        ObjectPropertyIr::Method { function, .. }
                        | ObjectPropertyIr::Getter { function, .. }
                        | ObjectPropertyIr::Setter { function, .. } => {
                            has_reference_error_throw(function)
                        }
                    })
                }
                ExprIr::TypeOf { expr } => has_reference_error_throw(expr),
                _ => false,
            }
        }

        let program = lower_script("let x = 1; for (let x in { x }) {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::ForInObject { target, .. } = &script.body.statements[1] else {
            panic!("expected for-in object statement");
        };
        assert!(has_reference_error_throw(target));
    }

    #[test]
    fn lowers_for_in_head_function_capture_to_tdz_binding() {
        let program = lower_script(
            "let x = 'outside'; var f; for (let x in { i: f = function () { return typeof x; } }) {} f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let closure = script
            .functions
            .iter()
            .find(|function| function.is_expression)
            .expect("head closure should be lowered");
        assert_eq!(closure.captured_bindings.len(), 1);
        assert!(closure.captured_bindings[0]
            .name
            .starts_with(TDZ_BINDING_STORAGE_PREFIX));
        assert!(closure.captured_bindings[0].name.ends_with(".x"));
    }

    #[test]
    fn infers_array_buffer_new_shape_for_closure_capture() {
        let program = lower_script(
            "const rab = new ArrayBuffer(64, { maxByteLength: 1024 }); const f = () => rab.resize; f();",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let arrow = script
            .functions
            .iter()
            .find(|function| function.flavor == FunctionFlavor::Arrow)
            .expect("arrow function should be lowered");
        let Some(StatementIr::Return(expr)) = arrow.body.statements.first() else {
            panic!("expression arrow should lower to return");
        };
        assert!(expr
            .function_targets
            .contains(&StandardBuiltinId::ArrayBufferPrototypeResize.function_id()));
    }

    #[test]
    fn lowers_script_class_closure_capture() {
        let program = lower_script("class C {} function f() { return C; } f();");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "C"));
        assert!(script.functions.iter().any(|function| function
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "C")));
    }

    #[test]
    fn lowers_class_constructor_closure_capture() {
        let program = lower_script(
            "function outer(Base, args) { let called = 0; class Derived extends Base { constructor() { ++called; super(...args); } } return Derived; }",
        );
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let constructor = script
            .functions
            .iter()
            .find(|function| function.name == "Derived")
            .expect("derived constructor should be lowered");
        assert!(constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "called"));
        assert!(constructor
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "args"));
    }

    #[test]
    fn carries_dynamic_function_apply_argument_list_to_runtime() {
        let program = lower_script(
            "function target() {} function call(args) { return target.apply(null, args); }",
        );
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn carries_dynamic_method_receiver_to_runtime() {
        let program =
            lower_script("function call(receiver, name, args) { return receiver[name](...args); }");
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn keeps_top_level_lexicals_out_of_script_global_bindings() {
        let program = lower_script("let x = 1; const y = 2; var z = 3; function f() {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        assert!(!script
            .global_bindings
            .iter()
            .any(|binding| binding.name == "x" || binding.name == "y"));
        assert!(script.global_bindings.iter().any(|binding| {
            binding.name == "z" && matches!(binding.kind, ScriptGlobalBindingKind::Var)
        }));
        assert!(script.global_bindings.iter().any(|binding| {
            binding.name == "f" && matches!(binding.kind, ScriptGlobalBindingKind::Function)
        }));
    }

    #[test]
    fn records_unsupported_var_destructuring_without_failing_lower() {
        let program = lower_script("var { x } = foo;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("destructuring var declaration")));
    }

    #[test]
    fn rejects_assignment_to_const() {
        let program = lower_script("const x = 1; x = 2;");
        assert!(!program.is_wasm_supported());
        assert!(program
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("assignment to const binding")));
    }

    #[test]
    fn lowers_string_compound_assignment() {
        let program = lower_script("let s = \"a\"; s += \"b\";");
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("compound_assigns=1"));
        assert_eq!(
            program
                .script
                .as_ref()
                .expect("script ir should exist")
                .result_kind(),
            ValueKind::String
        );
    }

    #[test]
    fn lowers_lone_surrogate_string_literal_with_internal_marker() {
        let program = lower_script("\"\\uD800\";");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::String(value) = &expr.expr else {
            panic!("expected string literal expression");
        };
        assert_eq!(value, &format!("{JS_STRING_SURROGATE_SENTINEL}D800"));
    }

    #[test]
    fn rejects_label_on_unsupported_statement_kind() {
        let program = lower_script("label: 1;");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("label on unsupported statement kind")));
    }

    #[test]
    fn rejects_unknown_kind_numeric_use_after_var_merge() {
        let program = lower_script("var x; if (true) { x = 1; } else { x = \"a\"; } x + 1;");
        assert!(!program.is_wasm_supported());
        assert!(program.diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("unsupported in porffor wasm-aot first slice")));
    }

    #[test]
    fn lowers_dynamic_plus_proven_string_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return 1; return {}; } function format(message) { return message + \" suffix\"; } format(choose(true));",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=2"));
    }

    #[test]
    fn lowers_maybe_string_plus_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return \"name\"; return 1; } choose(true) + 1;",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=1"));
    }

    #[test]
    fn lowers_nested_maybe_string_plus_as_coercive_add() {
        let program = lower_script(
            "function choose(flag) { if (flag) return \"name\"; return 1; } choose(true) + 1 + \" suffix\";",
        );
        assert!(program.is_wasm_supported());
        let summary = program.ir_summary();
        assert!(summary.contains("heap_coercions=1"));
        assert!(summary.contains("string_concats=1"));
    }

    #[test]
    fn function_var_shadows_same_named_script_global_var() {
        let program = lower_script(
            "function helper() { var index = 0; index = index + 1; return index; } var index; for (var index in []) {} helper();",
        );
        assert!(program.is_wasm_supported());
    }

    #[test]
    fn lowers_unbound_identifier_read_as_runtime_reference_error() {
        let program = lower_script("try { missingName; } catch (e) {}");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::TryCatch { try_block, .. } = &script.body.statements[0] else {
            panic!("expected try/catch statement");
        };
        let StatementIr::Expression(expr) = &try_block.statements[0] else {
            panic!("expected try expression statement");
        };
        assert!(matches!(
            expr.expr,
            ExprIr::RuntimeThrow {
                name: REFERENCE_ERROR_NAME,
                ..
            }
        ));
    }

    #[test]
    fn lowers_catch_parameter_source_alias_for_redeclared_var() {
        let program =
            lower_script("foo = \"prior\"; try { throw 1; } catch (foo) { var foo = \"init\"; }");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script ir should exist");
        let (catch_name, catch_source_name, catch_block) = script
            .body
            .statements
            .iter()
            .find_map(|statement| {
                if let StatementIr::TryCatch {
                    catch_name,
                    catch_source_name,
                    catch_block,
                    ..
                } = statement
                {
                    Some((catch_name, catch_source_name, catch_block))
                } else {
                    None
                }
            })
            .expect("expected try/catch statement");

        assert_eq!(catch_source_name, "foo");
        assert_ne!(catch_name, catch_source_name);
        let StatementIr::Var(declarators) = &catch_block.statements[0] else {
            panic!("expected var declaration in catch block");
        };
        assert_eq!(declarators[0].name, "foo");
    }

    #[test]
    fn lowers_static_class_field_as_static_with_initializer_function() {
        let program = lower_script("class C { static x = 1; } C.x;");
        let script = program.script.as_ref().expect("script ir should exist");
        assert_eq!(script.functions.len(), 2);
        let class_expr = match &script.body.statements[0] {
            StatementIr::Lexical { init, .. } => &init.expr,
            other => panic!("expected class lexical statement, got {other:?}"),
        };
        let ExprIr::ClassDefinition(class) = class_expr else {
            panic!("expected class definition");
        };
        assert_eq!(class.fields.len(), 1);
        assert_eq!(class.fields[0].placement, ClassMethodPlacementIr::Static);
        assert!(class.fields[0].init_function_id.is_some());
    }

    #[test]
    fn does_not_fold_number_wrapper_to_boolean_to_string() {
        let program = lower_script("throw (new Number()).toString();");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Throw(value) = &script.body.statements[0] else {
            panic!("expected throw statement");
        };
        let ExprIr::String(value) = &value.expr else {
            panic!("expected static string throw value, got {:?}", value.expr);
        };
        assert_eq!(value, "0");
    }

    #[test]
    fn lowers_json_parse_reviver_with_static_string_binding() {
        let program =
            lower_script("var json = \"[1, 2]\"; JSON.parse(json, function(k, v) { return v; });");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected JSON.parse expression");
        };
        assert!(matches!(expr.expr, ExprIr::JsonParseStaticReviver { .. }));
    }

    #[test]
    fn folds_static_regexp_literal_exec() {
        let program = lower_script("/]/.exec(' ]{}')[0]; /\\c0/.exec('\\x0f\\x10\\x11');");
        assert!(program.is_wasm_supported());
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(first) = &script.body.statements[0] else {
            panic!("expected expression statement");
        };
        let ExprIr::PropertyRead { target, .. } = &first.expr else {
            panic!(
                "expected static regexp match array read, got {:?}",
                first.expr
            );
        };
        assert!(matches!(target.expr, ExprIr::ArrayLiteral(_)));
        let StatementIr::Expression(second) = &script.body.statements[1] else {
            panic!("expected expression statement");
        };
        assert!(matches!(second.expr, ExprIr::Null));
    }
}
