use std::collections::BTreeSet;

use crate::{ExportName, ImportNameIr, ModuleRequestIr, ModuleRequestKeyIr};

use super::graph::ModuleGraphIr;
use super::record::{push_unique_name, ModuleUnitId};
use super::resolved_binding::{ModuleBindingNameIr, ResolvedBindingIr};

impl ModuleGraphIr {
    /// Target of `request` made by `referrer`, if the host resolved it.
    ///
    /// There is deliberately no fallback lookup of `request.specifier` in
    /// [`Self::keys`]. A request spelling is relative to its referrer and is
    /// not a host-normalized [`crate::ModuleKey`]; comparing the two domains used to
    /// make an omitted host resolution silently look resolved when their raw
    /// strings happened to match.
    #[must_use]
    pub fn resolve_request(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestIr,
    ) -> Option<ModuleUnitId> {
        self.resolve_request_key(referrer, request.key())
    }

    /// Target of a phase-free host request key made by `referrer`.
    #[must_use]
    pub fn resolve_request_key(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestKeyIr,
    ) -> Option<ModuleUnitId> {
        self.resolutions.get(&(referrer, request.clone())).copied()
    }

    /// `GetExportedNames` (16.2.1.6.2). Source order, as the spec defines it;
    /// namespace `[[OwnPropertyKeys]]` sorting happens in `namespace.rs`.
    #[must_use]
    pub fn exported_names(&self, module: ModuleUnitId) -> Vec<ExportName> {
        let mut export_star_set = BTreeSet::new();
        let mut names = Vec::new();
        self.collect_exported_names(module, &mut export_star_set, &mut names);
        names
    }

    fn collect_exported_names(
        &self,
        module: ModuleUnitId,
        export_star_set: &mut BTreeSet<ModuleUnitId>,
        names: &mut Vec<ExportName>,
    ) {
        // 1-2. A module already on the star path contributes nothing more.
        if !export_star_set.insert(module) {
            return;
        }
        let Some(unit) = self.units.get(module as usize) else {
            return;
        };
        for entry in &unit.record.local_export_entries {
            push_unique_name(names, &entry.export_name);
        }
        for entry in &unit.record.indirect_export_entries {
            push_unique_name(names, &entry.export_name);
        }
        for entry in &unit.record.star_export_entries {
            let Some(target) = self.resolve_request(module, &entry.request) else {
                continue;
            };
            let mut star_names = Vec::new();
            self.collect_exported_names(target, export_star_set, &mut star_names);
            for name in star_names {
                // `export *` never re-exports `default`.
                if !name.is_default() {
                    push_unique_name(names, &name);
                }
            }
        }
    }

    /// `ResolveExport` (16.2.1.6.3).
    ///
    /// The lookup key is a D2 name, always: the specification matches it against
    /// `e.[[ExportName]]`, and the `[[ImportName]]` an importer passes in is the
    /// requested module's D2 read from the other side. Passing a `[[LocalName]]`
    /// is `E0308`.
    #[must_use]
    pub fn resolve_export(
        &self,
        module: ModuleUnitId,
        export_name: &ExportName,
    ) -> ResolvedBindingIr {
        let mut resolve_set = Vec::new();
        self.resolve_export_inner(module, export_name, &mut resolve_set)
    }

    fn resolve_export_inner(
        &self,
        module: ModuleUnitId,
        export_name: &ExportName,
        resolve_set: &mut Vec<(ModuleUnitId, ExportName)>,
    ) -> ResolvedBindingIr {
        // 1. A repeated (module, exportName) pair is a circular request.
        if resolve_set
            .iter()
            .any(|(seen, name)| *seen == module && name == export_name)
        {
            return ResolvedBindingIr::NotFound;
        }
        resolve_set.push((module, export_name.clone()));

        let Some(unit) = self.units.get(module as usize) else {
            return ResolvedBindingIr::NotFound;
        };

        // 4. Local exports resolve to this module.
        // The match is D2 = D2 and the result is D1: the whole shape of
        // 16.2.1.6.2 step 4.a, and the one place the two domains meet.
        for entry in &unit.record.local_export_entries {
            if &entry.export_name == export_name {
                return ResolvedBindingIr::Resolved {
                    module,
                    binding: ModuleBindingNameIr::Name(entry.local_name.clone()),
                };
            }
        }

        // 5. Indirect exports delegate to the requested module.
        for entry in &unit.record.indirect_export_entries {
            if &entry.export_name != export_name {
                continue;
            }
            let Some(target) = self.resolve_request(module, &entry.request) else {
                return ResolvedBindingIr::NotFound;
            };
            return match &entry.import_name {
                ImportNameIr::Namespace => ResolvedBindingIr::Resolved {
                    module: target,
                    binding: ModuleBindingNameIr::Namespace,
                },
                ImportNameIr::Name(name) => self.resolve_export_inner(target, name, resolve_set),
            };
        }

        // 6. `default` is never provided by `export *`.
        if export_name.is_default() {
            return ResolvedBindingIr::NotFound;
        }

        // 7-8. Every star path must agree, otherwise the name is ambiguous.
        let mut star_resolution = ResolvedBindingIr::NotFound;
        for entry in &unit.record.star_export_entries {
            let Some(target) = self.resolve_request(module, &entry.request) else {
                continue;
            };
            let resolution = self.resolve_export_inner(target, export_name, resolve_set);
            if matches!(resolution, ResolvedBindingIr::Ambiguous) {
                return ResolvedBindingIr::Ambiguous;
            }
            if matches!(resolution, ResolvedBindingIr::Resolved { .. }) {
                if matches!(star_resolution, ResolvedBindingIr::NotFound) {
                    star_resolution = resolution;
                } else if star_resolution != resolution {
                    return ResolvedBindingIr::Ambiguous;
                }
            }
        }
        star_resolution
    }

    // `cell_name` used to live here. It was the sole producer of a *fourth*
    // name domain — `$m{unit}$` prefixed onto a `[[LocalName]]` — which a
    // per-unit-environment backend would need and which the source-text linker
    // in use today must never emit, because the merged scope names an
    // exporter's binding exactly as the exporter spells it. Its one caller
    // filled `ModuleNamespaceExportIr::cell`, whose only reader was a test.
    //
    // Nothing produces such a name now, so nothing can leak one into generated
    // Script text. `modules::namespace::namespace_target_reference` is the
    // single authority for what a resolved binding reads as, and it returns a
    // `MergedName`. If a per-unit-environment backend is ever built, the name
    // it needs is a *different* type from `MergedName` and must be spelled as
    // one — see `modules::namespace`'s module docs.
}
