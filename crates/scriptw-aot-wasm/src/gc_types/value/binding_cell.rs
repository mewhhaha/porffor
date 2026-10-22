//! A mutable binding owns one inline value. Reads copy a snapshot into rooted
//! locals; writes never mutate a StoredValue shared by another semantic owner.
use super::*;

impl GcStructType<BindingCell> {
    pub(crate) fn read_into<N: GcFieldNullability>(
        &self,
        cell: &GcLocal<BindingCell, N>,
        destination: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        self.field(Self::TAG)
            .read(cell, schema, function)
            .store(destination.tag(), function);
        self.field(Self::SCALAR)
            .read(cell, schema, function)
            .store_i64(destination.scalar(), function);
        self.field(Self::REFERENCE)
            .read(cell, schema, function)
            .store_eq(destination.reference(), function);
    }

    pub(crate) fn write_from<N: GcFieldNullability>(
        &self,
        cell: &GcLocal<BindingCell, N>,
        value: &ValueLocals,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        // Straight-line field stores cannot reenter JS or allocate a Wasm GC
        // object. The input EqRef local roots the new reference throughout.
        self.field(Self::TAG)
            .write(cell, GcOperand::i32_local(value.tag()), schema, function);
        self.field(Self::SCALAR).write(
            cell,
            GcOperand::i64_local(value.scalar()),
            schema,
            function,
        );
        // Always update this edge, including null on scalar/undefined writes.
        self.field(Self::REFERENCE).write(
            cell,
            GcOperand::eq_reference(&value.reference()),
            schema,
            function,
        );
    }
}
