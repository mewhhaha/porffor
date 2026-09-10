use lila_front::{
    prepare_eval_source, DirectEvalParseContext, EvalInvocationContext, EvalParseContext,
};

fn direct(
    invocation: EvalInvocationContext,
    strict_caller: bool,
    private_names: &[&str],
) -> EvalParseContext {
    EvalParseContext::Direct(DirectEvalParseContext {
        invocation,
        strict_caller,
        private_names: private_names
            .iter()
            .map(|name| (*name).to_string())
            .collect(),
    })
}

#[test]
fn direct_eval_inherits_strict_grammar_without_source_wrapping() {
    assert!(prepare_eval_source(
        "with ({}) {}".into(),
        &direct(EvalInvocationContext::Function, false, &[])
    )
    .is_ok());
    assert!(prepare_eval_source(
        "with ({}) {}".into(),
        &direct(EvalInvocationContext::Function, true, &[])
    )
    .is_err());
    assert!(prepare_eval_source(
        "return 1;".into(),
        &direct(EvalInvocationContext::Function, false, &[])
    )
    .is_err());
}

#[test]
fn eval_new_target_super_and_private_permissions_are_from_the_caller() {
    assert!(prepare_eval_source(
        "new.target".into(),
        &direct(EvalInvocationContext::Function, false, &[])
    )
    .is_ok());
    assert!(prepare_eval_source(
        "new.target".into(),
        &direct(EvalInvocationContext::Script, false, &[])
    )
    .is_err());
    assert!(prepare_eval_source("new.target".into(), &EvalParseContext::Indirect).is_err());
    assert!(prepare_eval_source(
        "super.x".into(),
        &direct(EvalInvocationContext::Method, false, &[])
    )
    .is_ok());
    assert!(prepare_eval_source(
        "super()".into(),
        &direct(EvalInvocationContext::Method, false, &[])
    )
    .is_err());
    assert!(prepare_eval_source(
        "super()".into(),
        &direct(EvalInvocationContext::DerivedConstructor, false, &[])
    )
    .is_ok());
    assert!(prepare_eval_source(
        "this.#value".into(),
        &direct(EvalInvocationContext::Method, false, &["value"])
    )
    .is_ok());
    assert!(prepare_eval_source(
        "this.#value".into(),
        &direct(EvalInvocationContext::Method, false, &[])
    )
    .is_err());
    assert!(prepare_eval_source(
        "arguments".into(),
        &direct(EvalInvocationContext::ClassFieldInitializer, false, &[])
    )
    .is_err());
}
