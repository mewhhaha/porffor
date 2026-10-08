use super::*;
use boa_engine::builtins::promise::PromiseState;
use lila_runtime::{
    EmbeddedModuleEntryInput, EmbeddedModuleInput, EmbeddedModuleResolutionInput,
    EmbeddedModuleSourceInput,
};

#[test]
fn static_json_source_request_rejects_with_the_actual_referrer_syntax_error() {
    let graph = EmbeddedModuleGraph::try_new_typed(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Module,
            identity: "entry.js".into(),
            source: "import source data from 'data' with { type: 'json' };".into(),
            meta_url: "lila://entry".into(),
        },
        vec![EmbeddedModuleInput::Json(EmbeddedModuleSourceInput {
            identity: "data.json".into(),
            source: "42".into(),
            meta_url: "lila://data".into(),
        })],
        vec![EmbeddedModuleResolutionInput {
            referrer: EmbeddedModuleReferrer::Module("entry.js".into()),
            specifier: "data".into(),
            attributes: vec![("type".into(), "json".into())],
            target: "data.json".into(),
        }],
    )
    .expect("exact JSON source graph");
    let (mut context, loader) =
        EmbeddedGraphModuleLoader::context(graph, false).expect("embedded Context");
    let entry = context.realm().clone();
    let module = loader
        .entry_module(&mut context)
        .expect("original Source import syntax");
    let promise = module.load(&mut context);
    context
        .run_jobs()
        .expect("host load rejects its promise, not the job checkpoint");
    let PromiseState::Rejected(error) = promise.state() else {
        panic!("no fabricated JSON Source object")
    };
    let error = error.as_object().expect("actual SyntaxError object");
    assert_eq!(
        error.borrow().prototype(),
        Some(entry.intrinsics().constructors().syntax_error().prototype())
    );
    assert_eq!(
        context.realm(),
        &entry,
        "host callback restores its caller Realm"
    );
}
