use super::*;

pub(super) struct StaticClosure<'a> {
    sources: &'a ModuleGraphSources,
    records: &'a [Result<ModuleRecordIr, Vec<IrDiagnostic>>],
    admission: GraphAdmission,
}

impl<'a> StaticClosure<'a> {
    pub(super) fn new(
        sources: &'a ModuleGraphSources,
        records: &'a [Result<ModuleRecordIr, Vec<IrDiagnostic>>],
        admission: GraphAdmission,
    ) -> Self {
        Self {
            sources,
            records,
            admission,
        }
    }

    fn may_resolve(&self, referrer: ModuleUnitId, request: &ModuleRequestKeyIr) -> bool {
        match self.admission {
            GraphAdmission::LoadedClosure => true,
            GraphAdmission::CompleteCatalog => self.records[referrer as usize]
                .as_ref()
                .is_ok_and(|record| record.catalog_key_is_eligible(request)),
        }
    }

    pub(super) fn target(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestKeyIr,
    ) -> Option<ModuleUnitId> {
        if !self.may_resolve(referrer, request) {
            return None;
        }
        self.sources
            .resolutions
            .iter()
            .find_map(|(owner, resolved_request, target)| {
                (*owner == referrer
                    && resolved_request == request
                    && self
                        .sources
                        .modules
                        .get(*target as usize)
                        .is_some_and(|source| source.kind().matches_request(request)))
                .then_some(*target)
            })
    }

    pub(super) fn members(&self, root: ModuleUnitId) -> BTreeSet<ModuleUnitId> {
        self.members_and_execution(root).0
    }

    /// Source phase retains a parsed record but does not open its dependencies
    /// or its dynamic call sites. The second set contains only opened records.
    pub(super) fn members_and_execution(
        &self,
        root: ModuleUnitId,
    ) -> (BTreeSet<ModuleUnitId>, BTreeSet<ModuleUnitId>) {
        let mut members = BTreeSet::new();
        let mut expanded = BTreeSet::new();
        let mut pending = vec![(root, true)];
        while let Some((module, recursive)) = pending.pop() {
            let source = &self.sources.modules[module as usize];
            // Duplicate agreeing Module rows retain one identity and all their
            // request rows. A Script root is outside the Module map even when
            // its URL is also loaded as Module source.
            let included = members.insert(module);
            let expands = recursive && expanded.insert(module);
            if included || expands {
                pending.extend(self.sources.modules.iter().enumerate().filter_map(
                    |(index, other)| {
                        (source_identity(other) == source_identity(source))
                            .then_some((index as u32, recursive))
                    },
                ));
            }
            // A later evaluation request must also expand agreeing rows first
            // reached by source phase: their distinct host resolution rows may
            // supply dependencies that this row alone cannot resolve.
            if !expands {
                continue;
            }
            let Ok(record) = &self.records[module as usize] else {
                continue;
            };
            pending.extend(record.requested_modules.iter().filter_map(|request| {
                self.target(module, request.key())
                    .map(|target| (target, request.phase().loads_dependencies()))
            }));
        }
        (members, expanded)
    }

    /// Keep each source's retained parse product, including a Script root.
    /// Target closures are still rooted at a Module; projection changes source
    /// indices and resolution rows without changing any source's parse goal.
    pub(super) fn project(
        &self,
        members: &BTreeSet<ModuleUnitId>,
        root: ModuleUnitId,
    ) -> ModuleGraphSources {
        let indices = members
            .iter()
            .enumerate()
            .map(|(index, original)| (*original, index as u32))
            .collect::<BTreeMap<_, _>>();
        ModuleGraphSources {
            realm_requests: self
                .sources
                .realm_requests
                .iter()
                .filter_map(|(request, resolution)| {
                    let resolution = match resolution {
                        super::super::RealmModuleResolutionIr::Loaded(module) => {
                            super::super::RealmModuleResolutionIr::Loaded(*indices.get(module)?)
                        }
                        super::super::RealmModuleResolutionIr::Rejected(message) => {
                            super::super::RealmModuleResolutionIr::Rejected(message.clone())
                        }
                    };
                    Some((request.clone(), resolution))
                })
                .collect(),
            modules: members
                .iter()
                .map(|index| self.sources.modules[*index as usize].clone())
                .collect(),
            entry: indices[&root],
            resolutions: self
                .sources
                .resolutions
                .iter()
                .filter_map(|(referrer, request, target)| {
                    if !self.may_resolve(*referrer, request) {
                        return None;
                    }
                    Some((
                        *indices.get(referrer)?,
                        request.clone(),
                        *indices.get(target)?,
                    ))
                })
                .collect(),
        }
    }
}
