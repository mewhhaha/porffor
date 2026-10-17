/// An immutable operand for one checked P image, supplied anew per instance.
#[derive(Clone, Debug)]
pub struct NativeProgramLiteralOperand {
    pub(crate) name: String,
    pub(crate) value: i32,
}
impl NativeProgramLiteralOperand {
    pub fn import_name(&self) -> &str {
        &self.name
    }
    pub const fn value(&self) -> i32 {
        self.value
    }
}
