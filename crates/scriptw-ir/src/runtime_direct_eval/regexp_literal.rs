//! A static literal whose fresh result retains source data and Realm, never eval P.
use super::RuntimeDirectEvalProgram;
use crate::{ExprIr, StatementIr, StaticRegExpCompilation, ValidatedRegExpProgram};

#[derive(Debug)]
pub struct StaticRegExpRuntimeDirectEval<'a> {
    _program: &'a RuntimeDirectEvalProgram,
    source: &'a str,
    flags: &'a str,
    compiled: ValidatedRegExpProgram,
}

impl RuntimeDirectEvalProgram {
    /// Exact caller parsing, early errors and full lowering precede admission.
    /// Instantiation must be inert, and the only executed expression must be
    /// a successfully compiled static RegExp literal, with no other work.
    pub fn static_regexp_literal(&self) -> Option<StaticRegExpRuntimeDirectEval<'_>> {
        let unit = self.inert_preparation()?;
        let [StatementIr::Expression(expression)] = unit.body.statements.as_slice() else {
            return None;
        };
        let ExprIr::RegExpLiteral {
            source,
            flags,
            static_compilation: Some(StaticRegExpCompilation::Program(program)),
        } = &expression.expr
        else {
            return None;
        };
        Some(StaticRegExpRuntimeDirectEval {
            _program: self,
            source,
            flags,
            compiled: ValidatedRegExpProgram::from_program(program).ok()?,
        })
    }
}

impl StaticRegExpRuntimeDirectEval<'_> {
    pub fn source_units(&self) -> Vec<u16> {
        scriptw_front::decode_js_string_utf16(self.source)
    }

    pub fn flags_units(&self) -> Vec<u16> {
        scriptw_front::decode_js_string_utf16(self.flags)
    }

    pub fn compiled_bytes(&self) -> &[u8] {
        self.compiled.bytes()
    }
}
