//! Fixed Array literals initialize one fresh, unpublished actual GC Array.
//! Spread and suspension-owned accumulation retain their observable owner.

use super::*;
use crate::gc_types::{GcLocal, PropertyDescriptor, StoredValue};
use crate::heap::StoredPropertyAttributes;

/// There is no constructor from an existing Array and no borrowed-array view.
/// Only consuming finish can publish this fresh object's whole Value.
#[must_use = "a fresh literal stays private until all elements are initialized"]
struct FreshArrayLiteral {
    array: GcLocal<ArrayObject>,
    absent_accessor: GcLocal<StoredValue>,
    index: I64Local,
}

impl FreshArrayLiteral {
    fn new(
        length: u32,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Result<Self, EmitError> {
        let schema = builder.runtime_schema();
        let length_local = schema.reserve_i64_local(f);
        length_local.set_constant(i64::from(length), f);
        let array = builder.emit_array_literal_object(length_local, f)?;
        schema.release_i64_local(length_local, f);
        let undefined = schema.reserve_value_local(f);
        undefined.set_undefined(f);
        let absent_accessor = schema.reserve_gc_local(f).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&undefined, f),
            f,
        );
        undefined.clear(f);
        let index = schema.reserve_i64_local(f);
        Ok(Self {
            array,
            absent_accessor,
            index,
        })
    }

    /// The sole caller derives ordinals from a length-checked fixed literal.
    /// Every non-hole ordinal is below 2^32-1 and is published exactly once.
    fn publish_element(
        &mut self,
        ordinal: u32,
        value: &ValueLocals,
        builder: &mut FunctionBuilder<'_>,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = builder.runtime_schema();
        let stored = schema
            .reserve_gc_local(f)
            .initialize(schema.struct_type::<StoredValue>().from_value(value, f), f);
        let descriptor = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<PropertyDescriptor>().construct(
                (
                    GcOperand::descriptor_word(
                        StoredPropertyAttributes::Data {
                            writable: true,
                            enumerable: true,
                            configurable: true,
                        }
                        .descriptor_word(),
                    ),
                    GcOperand::reference(&stored, schema),
                    GcOperand::reference(&self.absent_accessor, schema),
                    GcOperand::reference(&self.absent_accessor, schema),
                ),
                f,
            ),
            f,
        );
        self.index.set_constant(i64::from(ordinal), f);
        builder.emit_array_indexed_publish_descriptor(&self.array, self.index, &descriptor, f)?;
        descriptor.clear(f);
        stored.clear(f);
        Ok(())
    }

    fn finish(self, output: &ValueLocals, builder: &FunctionBuilder<'_>, f: &mut Function) {
        let schema = builder.runtime_schema();
        output.set_reference(&self.array, schema, f);
        schema.release_i64_local(self.index, f);
        self.absent_accessor.clear(f);
        self.array.clear(f);
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn compile_array_literal_payload(
        &mut self,
        elements: &[TypedExpr],
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        // A fixed Array literal can contain at most 2^32-1 elements, and its
        // largest occupied ordinal is consequently at most 2^32-2. Validate
        // once during compilation; never narrow a usize silently to an index.
        let length = u32::try_from(elements.len()).map_err(|_| {
            EmitError::unsupported("Array literal exceeds the maximum ECMAScript Array length")
        })?;
        let schema = self.runtime_schema();
        let value = schema.reserve_value_local(f);
        let mut literal = FreshArrayLiteral::new(length, self, f)?;
        for (ordinal, element) in elements.iter().enumerate() {
            if matches!(element.expr, ExprIr::ArrayHole) {
                continue;
            }
            self.compile_expr_to_value(element, &value, f)?;
            self.emit_propagate_current_throw_if_needed(f);
            let ordinal = u32::try_from(ordinal).expect("literal length validated before emission");
            literal.publish_element(ordinal, &value, self, f)?;
        }
        literal.finish(output, self, f);
        value.clear(f);
        Ok(())
    }
}
