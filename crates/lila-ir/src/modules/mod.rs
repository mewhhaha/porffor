//! ECMAScript module records, linking and loading (T12).
//!
//! `record` owns `ParseModule` and the static entry tables (16.2.1.6.1,
//! 16.2.2, 16.2.3). `early` owns the module early errors (16.2.3.1) and the
//! classification of boa's own module-goal static-semantics failures.
//! `graph` owns transitive assembly, `GetExportedNames` /
//! `ResolveExport` and evaluation order. `link` merges the per-module bodies
//! into the single `ScriptIr` the backend emits, and `source` is the lexical
//! scanner it uses to delete module-goal-only syntax from a unit's text.
//! `namespace` owns module namespace exotic objects, deferred namespaces and
//! module source objects. `dynamic` owns the `import()` component registry.
//!
//! All three module request *phases* link. The evaluation phase is the default;
//! `import defer` makes a unit's body a thunk its namespace calls on first
//! touch; `import source` loads and parses a unit without instantiating it.
//! `graph::classify_evaluation_modes` is the single authority for which unit
//! gets which treatment.
//!
//! `lila-ir` performs no IO. The host resolves and reads every source and
//! hands the closure over as a [`ModuleGraphSources`]; nothing in this
//! directory touches the filesystem.

mod dynamic;
mod early;
mod graph;
mod link;
mod namespace;
mod record;
mod source;

pub use dynamic::*;
pub use graph::*;
pub use link::*;
pub use namespace::*;
pub use record::*;

pub(crate) use dynamic::lower_import_call;
pub(crate) use graph::{build_graph, link};
pub(crate) use link::linked_script_source;
