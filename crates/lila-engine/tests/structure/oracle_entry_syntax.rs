#![cfg(feature = "spec-exec-oracle")]

use lila_engine::{
    CompileOptions, EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph, Engine,
    ExecutionBackend, ModuleLoadingPolicy, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

#[test]
fn native_oracle_entry_rejection_does_not_classify_a_runtime_syntax_error_as_parse_failure() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for goal in [EmbeddedModuleGoal::Script, EmbeddedModuleGoal::Module] {
        for (source, rejects_entry) in [
            ("let missing = ;", true),
            ("throw new SyntaxError('runtime');", false),
        ] {
            let identity = match goal {
                EmbeddedModuleGoal::Script => "entry.js",
                EmbeddedModuleGoal::Module => "entry.mjs",
            };
            let graph = EmbeddedModuleGraph::try_new(
                EmbeddedModuleEntryInput {
                    goal,
                    source: source.into(),
                    identity: identity.into(),
                    meta_url: format!("lila://{identity}"),
                },
                vec![],
                vec![],
            )
            .unwrap();
            let engine = Engine::new(RealmBuilder::new().build());
            let options = CompileOptions {
                filename: Some(identity.into()),
                module_loading_policy: ModuleLoadingPolicy::Embedded(graph),
                ..CompileOptions::default()
            };
            let run = RunOptions {
                backend: ExecutionBackend::SpecExec,
                ..RunOptions::default()
            };
            let result = match goal {
                EmbeddedModuleGoal::Script => engine.observe_script(source, options, run),
                EmbeddedModuleGoal::Module => engine.observe_module(source, options, run),
            };
            if rejects_entry {
                assert!(result.unwrap_err().is_oracle_entry_syntax_rejection());
            } else {
                assert_eq!(
                    result.unwrap().completion,
                    ObservedCompletion::Throw(ObservedJsValue::Object)
                );
            }
        }
    }
}
