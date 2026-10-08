#![deny(dead_code)]

use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Maps one static pattern/flags pair to its compile-time compilation.
    ///
    /// The single owner of the `RegExpProgram::compile` outcome mapping, shared
    /// by direct `RegExp` calls, `new RegExp` and RegExp literals so a third
    /// `RegExpCompileErrorKind` variant fails to build at exactly one site
    /// instead of silently inheriting "unsupported" at three.
    pub(super) fn static_regexp_compilation_for_pattern(
        pattern: &str,
        flags: &str,
    ) -> Option<StaticRegExpCompilation> {
        // Requested by lane RE-RT (batch 7, `re-rt-b7-integration.md` §5):
        // match `error.kind` once, exhaustively, with no guard and no
        // `unreachable!`. `UnsupportedFeature` means *legal pattern, this
        // compiler cannot build a program for it yet*, so it must keep falling
        // through to the runtime path where the fallback matcher gets a turn;
        // promoting it to a static SyntaxError would invent a spec violation
        // for every legal-but-unimplemented pattern. `lila-aot-wasm`'s runtime
        // RegExp table draws the same line
        // (`RuntimeRegExpEntry::{Rejected,Unsupported}`); the two must not drift.
        match RegExpProgram::compile(pattern, flags) {
            Ok(program) => Some(StaticRegExpCompilation::Program(program)),
            Err(error) => match error.kind {
                RegExpCompileErrorKind::InvalidSyntax => {
                    Some(StaticRegExpCompilation::InvalidSyntax {
                        message: format!("invalid regular-expression pattern: {error}"),
                    })
                }
                RegExpCompileErrorKind::UnsupportedFeature => None,
            },
        }
    }

    pub(super) fn static_regexp_compilation_for_direct_call(
        &self,
        callee: &TypedExpr,
        function_id: &FunctionId,
        args: &[TypedExpr],
    ) -> Option<StaticRegExpCompilation> {
        let builtin = StandardBuiltinId::from_function_id(function_id)?;
        if builtin == StandardBuiltinId::RegExpConstructor
            && !matches!(callee.expr, ExprIr::GlobalPropertyRead { ref name } if name == REGEXP_NAME)
        {
            return None;
        }
        if !matches!(
            builtin,
            StandardBuiltinId::RegExpConstructor | StandardBuiltinId::RegExpPrototypeCompile
        ) {
            return None;
        }
        let (pattern, flags) = match args {
            []
            | [TypedExpr {
                expr: ExprIr::Undefined,
                ..
            }] => ("", ""),
            [TypedExpr {
                expr: ExprIr::String(pattern),
                ..
            }] => (pattern.as_str(), ""),
            [TypedExpr {
                expr: ExprIr::String(pattern),
                ..
            }, TypedExpr {
                expr: ExprIr::String(flags),
                ..
            }, ..] => (pattern.as_str(), flags.as_str()),
            _ => return None,
        };
        Self::static_regexp_compilation_for_pattern(pattern, flags)
    }
}
