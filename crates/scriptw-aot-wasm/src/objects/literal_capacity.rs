//! Reserve private storage only for bounded, statically named fresh literals.
use super::*;

impl FunctionBuilder<'_> {
    /// Called immediately after this literal's fresh ordinary object allocation,
    /// before evaluating any property. The storage record is private; replace
    /// its array fields rather than mutating the shared zero-length arrays.
    pub(super) fn emit_fresh_literal_property_capacity(
        &self,
        properties: &[ObjectPropertyIr],
        object: &GcLocal<OrdinaryObject>,
        function: &mut Function,
    ) {
        if properties.len() <= 4 {
            return;
        }
        let mut names = std::collections::HashSet::new();
        for property in properties {
            match property {
                ObjectPropertyIr::Data { key, .. }
                | ObjectPropertyIr::NonEnumerableData { key, .. }
                | ObjectPropertyIr::Method { key, .. }
                | ObjectPropertyIr::Getter { key, .. }
                | ObjectPropertyIr::Setter { key, .. } => {
                    names.insert(key);
                }
                ObjectPropertyIr::PrototypeSetter { .. } => {}
                ObjectPropertyIr::ComputedData { .. }
                | ObjectPropertyIr::ComputedMethod { .. }
                | ObjectPropertyIr::ComputedGetter { .. }
                | ObjectPropertyIr::ComputedSetter { .. }
                | ObjectPropertyIr::Spread { .. } => return,
            }
        }
        // Small literals already allocate one 4-entry table on first append.
        // Reserving them early buys no avoided growth and can add probe work.
        if names.len() <= 4 {
            return;
        }
        let Some(capacity) = u32::try_from(names.len())
            .ok()
            .and_then(u32::checked_next_power_of_two)
            .filter(|capacity| *capacity <= (1 << 30))
        else {
            return;
        };
        let schema = self.runtime_schema();
        let capacity_local = schema.reserve_i32_local(function);
        capacity_local.set_constant(capacity as i32, function);
        let entries = schema.reserve_gc_local(function).initialize(
            schema.array_type::<PropertyTable>().filled(
                GcOperand::null(schema),
                capacity_local,
                function,
            ),
            function,
        );
        capacity_local.set_constant((capacity * 2) as i32, function);
        let index = schema.reserve_gc_local(function).initialize(
            schema.array_type::<OrdinaryPropertyIndex>().filled(
                GcOperand::i32(0),
                capacity_local,
                function,
            ),
            function,
        );
        let storage = schema.reserve_gc_local(function).initialize(
            schema
                .field(OrdinaryObjectSchema::PROPERTIES)
                .read(object, schema, function)
                .reference(),
            function,
        );
        schema.field(OrdinaryPropertyStorageSchema::ENTRIES).write(
            &storage,
            GcOperand::reference(&entries, schema),
            schema,
            function,
        );
        schema.field(OrdinaryPropertyStorageSchema::INDEX).write(
            &storage,
            GcOperand::reference(&index, schema),
            schema,
            function,
        );
        storage.clear(function);
        index.clear(function);
        entries.clear(function);
        schema.release_i32_local(capacity_local, function);
    }
}
