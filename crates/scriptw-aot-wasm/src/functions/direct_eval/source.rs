//! Candidate selection proven only by an actual first-argument string literal.
use scriptw_ir::{decode_js_string_utf16, ExprIr, TypedExpr};

/// Its private constructor cannot infer a constant from a variable's type or
/// from a later argument. Every other argument form retains the full dispatch.
pub(crate) struct DirectEvalSource(Option<Vec<u16>>);

impl DirectEvalSource {
    pub(super) fn of(arguments: &[TypedExpr]) -> Self {
        Self(match arguments.first().map(|argument| &argument.expr) {
            Some(ExprIr::String(source)) => Some(decode_js_string_utf16(source)),
            _ => None,
        })
    }

    pub(super) fn admits(&self, source: &str) -> bool {
        match &self.0 {
            // Host transport spellings may differ for identical JS code units.
            Some(units) => *units == decode_js_string_utf16(source),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests;
