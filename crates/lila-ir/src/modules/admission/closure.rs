use super::*;

pub(super) struct StaticClosure<'a> {
    sources: &'a ModuleGraphSources,
    records: &'a [Option<SourceTextModuleRecordIr>],
}

impl<'a> StaticClosure<'a> {
    pub(super) fn new(
        sources: &'a ModuleGraphSources,
        records: &'a [Option<SourceTextModuleRecordIr>],
    ) -> Self {
        Self { sources, records }
    }

    pub(super) fn target(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestKeyIr,
    ) -> Option<ModuleUnitId> {
        self.sources
            .resolutions
            .iter()
            .find_map(|(owner, resolved_request, target)| {
                (*owner == referrer
                    && resolved_request == request
                    && self.sources.modules.get(*target as usize).is_some())
                .then_some(*target)
            })
    }

    /// Every request key the host resolved for `referrer`, in row order.
    pub(super) fn request_keys(
        &self,
        referrer: ModuleUnitId,
    ) -> impl Iterator<Item = &'a ModuleRequestKeyIr> + '_ {
        self.sources
            .resolutions
            .iter()
            .filter(move |(owner, _, _)| *owner == referrer)
            .map(|(_, request, _)| request)
    }

    pub(super) fn members(&self, root: ModuleUnitId) -> BTreeSet<ModuleUnitId> {
        let mut members = BTreeSet::new();
        let mut pending = vec![root];
        while let Some(module) = pending.pop() {
            if !members.insert(module) {
                continue;
            }
            let source = &self.sources.modules[module as usize];
            // Duplicate agreeing host rows retain one identity and all their
            // request rows. Contradictory rows were rejected before projection.
            pending.extend(
                self.sources
                    .modules
                    .iter()
                    .enumerate()
                    .filter_map(|(index, other)| {
                        (other.goal() == source.goal() && other.key() == source.key())
                            .then_some(index as u32)
                    }),
            );
            let Some(record) = &self.records[module as usize] else {
                continue;
            };
            pending.extend(
                record
                    .requested_modules
                    .iter()
                    .filter_map(|request| self.target(module, request.key())),
            );
        }
        members
    }

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
