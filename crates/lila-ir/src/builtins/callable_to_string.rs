#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableToStringRepresentation {
    ExactSource(String),
    NativeNamed(String),
    NativeAnonymous,
}

impl CallableToStringRepresentation {
    pub fn materialize(&self) -> String {
        match self {
            Self::ExactSource(source) => source.clone(),
            Self::NativeNamed(name) => format!("function {name}() {{ [native code] }}"),
            Self::NativeAnonymous => "function () { [native code] }".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callable_to_string_representations_materialize_spec_shapes() {
        assert_eq!(
            CallableToStringRepresentation::ExactSource("function f() {}".to_string())
                .materialize(),
            "function f() {}"
        );
        assert_eq!(
            CallableToStringRepresentation::NativeNamed("Array".to_string()).materialize(),
            "function Array() { [native code] }"
        );
        assert_eq!(
            CallableToStringRepresentation::NativeAnonymous.materialize(),
            "function () { [native code] }"
        );
    }
}
