use crate::LocalName;

use super::record::ModuleUnitId;

/// The `[[BindingName]]` half of a `ResolvedBinding` Record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleBindingNameIr {
    /// `namespace`, produced by `export * as ns from "m"`.
    Namespace,
    /// A concrete binding of the resolved module's environment.
    ///
    /// A `[[LocalName]]` **of the resolving module**, not of the module that
    /// asked: 16.2.1.6.2 step 4.a.i takes it from `e.[[LocalName]]` of whichever
    /// module the recursion ended in, which is why
    /// [`ResolvedBindingIr::Resolved`] carries the module alongside it.
    Name(LocalName),
    /// The module source object of the resolved module.
    ///
    /// Produced only by `import source x from "m"`. Not a binding of `m` at
    /// all: `m` is loaded and parsed but never instantiated, so there is no
    /// environment to name.
    ModuleSource,
}

/// Result of `ResolveExport` (16.2.1.6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedBindingIr {
    /// A `ResolvedBinding` Record.
    Resolved {
        /// Module owning the binding.
        module: ModuleUnitId,
        /// `[[BindingName]]`.
        binding: ModuleBindingNameIr,
    },
    /// `ambiguous`: two `export *` paths reached different bindings.
    Ambiguous,
    /// `null`: no such export, or the request was circular.
    NotFound,
}
