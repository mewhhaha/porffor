/// A compiler/transport invariant failure; never a JavaScript SyntaxError or
/// an unsupported-language-feature answer.
#[derive(Debug)]
pub struct NativeProgramTemplateError(String);
impl std::fmt::Display for NativeProgramTemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for NativeProgramTemplateError {}
pub(super) fn invalid(error: impl std::fmt::Display) -> NativeProgramTemplateError {
    NativeProgramTemplateError(format!("invalid native P code/data image: {error}"))
}
