//! Fresh literal allocation cannot invoke user code or observe an eval environment.
use super::*;
use crate::gc_types::{NonNullable, RegExpObject, StringValue};
mod native;

impl FunctionBuilder<'_> {
    pub(crate) fn compile_regexp_literal_allocate_helper(&mut self) -> Result<Function, EmitError> {
        let mut function = self.begin_helper_body(RuntimeHelperId::RegExpLiteralAllocate);
        let parameters = self
            .helper_parameters::<crate::runtime_helpers::RegExpLiteralAllocateParameters>(
                &mut function,
            );
        let schema = self.runtime_schema();
        let header = schema.reserve_gc_local(&mut function).initialize(
            self.emit_alloc_plain_object_with_prototype(
                Some(&parameters.prototype),
                &mut function,
            )?,
            &mut function,
        );
        let object = schema.reserve_gc_local(&mut function).initialize(
            schema.struct_type::<RegExpObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&parameters.source, schema),
                    GcOperand::reference(&parameters.flags, schema),
                    GcOperand::nullable_reference(&parameters.program, schema),
                    GcOperand::reference(&parameters.realm, schema),
                    GcOperand::boolean(true),
                ),
                &mut function,
            ),
            &mut function,
        );
        let name = schema
            .reserve_gc_local::<StringValue, NonNullable>(&mut function)
            .initialize(
                self.emit_interned_string_reference("lastIndex", &mut function)?,
                &mut function,
            );
        let key = PropertyKeyLocals::from_string(schema, &name, &mut function);
        let initial = schema.reserve_value_local(&mut function);
        initial.set_scalar(ScalarValue::NumberBits(0), &mut function);
        let absent = schema.reserve_value_local(&mut function);
        absent.set_scalar(ScalarValue::Undefined, &mut function);
        let descriptor = self.emit_alloc_property_descriptor(
            StoredPropertyAttributes::Data {
                writable: true,
                enumerable: false,
                configurable: false,
            },
            &initial,
            &absent,
            &absent,
            &mut function,
        );
        // This helper owns the fresh extensible header. It has no properties
        // and has never escaped: appending this fixed data descriptor cannot
        // run a setter, Proxy trap, coercion or any other JavaScript callback.
        self.emit_ordinary_append_property_entry(&header, &key, &descriptor, &mut function)?;
        let _ = object.load(schema, &mut function);
        descriptor.clear(&mut function);
        absent.clear(&mut function);
        initial.clear(&mut function);
        key.clear(&mut function);
        name.clear(&mut function);
        object.clear(&mut function);
        header.clear(&mut function);
        parameters.release(&mut function);
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }
}
