//! ECMAScript module records, linking and loading (T12).
//!
//! `record` owns `ParseModule` and the static entry tables (16.2.1.6.1,
//! 16.2.2, 16.2.3). `early` owns the module early errors (16.2.3.1) and the
//! classification of boa's own module-goal static-semantics failures.
//! `graph_build` owns transitive assembly; `graph_resolution` owns
//! `GetExportedNames` / `ResolveExport`; `graph_evaluation_classification` owns
//! evaluation-mode classification and unsupported phase policy;
//! `graph_evaluation_order` owns `InnerModuleEvaluation` order and
//! strongly-connected components; `graph_async_evaluation` owns async-module
//! propagation and pending-dependency queries; and `graph_materialization`
//! owns the evaluation-to-runtime query boundary.
//! `graph` retains the linked record and linking orchestration. `link` merges
//! the per-module bodies into the single `ScriptIr` the backend emits, and
//! `source` is the lexical scanner it uses to delete module-goal-only syntax
//! from a unit's text.
//! `namespace` owns module namespace exotic objects, deferred namespaces and
//! module source objects. `dynamic` owns the `import()` component registry.
//!
//! All three module request *phases* link. The evaluation phase is the default;
//! `import defer` makes a unit's body a thunk its namespace calls on first
//! touch; `import source` loads and parses a unit without instantiating it.
//! `graph_evaluation_classification::classify_evaluation_modes` is the single
//! authority for which unit gets which treatment.
//!
//! `lila-ir` performs no IO. The host resolves and reads every source and
//! hands the closure over as a [`ModuleGraphSources`]; nothing in this
//! directory touches the filesystem.

mod dynamic;
mod early;
mod evaluation_mode;
mod graph;
mod graph_async_evaluation;
mod graph_build;
mod graph_evaluation_classification;
mod graph_evaluation_order;
mod graph_materialization;
mod graph_resolution;
mod import_phase;
mod link;
mod link_error;
mod loaded_sources;
mod module_key;
mod module_unit;
mod namespace;
mod record;
mod resolved_binding;
mod source;

pub use dynamic::*;
pub use evaluation_mode::ModuleEvaluationModeIr;
pub use graph::*;
pub use import_phase::ImportPhaseIr;
pub use link::*;
pub use link_error::ModuleLinkErrorIr;
pub use loaded_sources::{ModuleGraphSources, ModuleSourceIr};
pub use module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};
pub use module_unit::ModuleUnitIr;
pub use namespace::*;
pub use record::*;
pub use resolved_binding::{ModuleBindingNameIr, ResolvedBindingIr};

pub(crate) use dynamic::lower_import_call;
pub(crate) use graph::link;
pub(crate) use graph_build::build_graph;
pub(crate) use link::linked_script_source;
