use super::*;
use crate::gc_types::{
    ArrayObject, ArrayObjectSchema, FunctionObject, FunctionObjectSchema, GcLocal, GcLocalSlot,
    GcOperand, StoredValue, ValueLocals,
};
use crate::operations::PropertyKeyLocals;

/// Bootstrap cannot publish this storage before ArrayCreate initializes it.
#[must_use]
pub(crate) struct ReservedRealmArrayPrototypeLocal(GcLocalSlot<ArrayObject>);

/// The completed Array exotic prototype remains rooted until Realm bootstrap
/// has installed its methods and constructor link.
#[must_use]
pub(crate) struct RealmArrayPrototypeLocal(GcLocal<ArrayObject>);

impl RealmArrayPrototypeLocal {
    pub(crate) fn array(&self) -> &GcLocal<ArrayObject> {
        &self.0
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn reserve_realm_array_prototype_local(
        &mut self,
        function: &mut Function,
    ) -> ReservedRealmArrayPrototypeLocal {
        ReservedRealmArrayPrototypeLocal(self.runtime_schema().reserve_gc_local(function))
    }

    pub(crate) fn emit_initialize_realm_array_prototype(
        &mut self,
        reserved: ReservedRealmArrayPrototypeLocal,
        object_prototype: &ValueLocals,
        function: &mut Function,
    ) -> Result<RealmArrayPrototypeLocal, EmitError> {
        let prototype = reserved.0.initialize(
            self.emit_alloc_empty_array_with_prototype(object_prototype, function)?,
            function,
        );
        Ok(RealmArrayPrototypeLocal(prototype))
    }

    pub(crate) fn emit_define_realm_array_prototype_data_with_flags(
        &mut self,
        prototype: &RealmArrayPrototypeLocal,
        name: &str,
        value: &ValueLocals,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let header = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ArrayObject>()
                .field(ArrayObjectSchema::OBJECT)
                .read(prototype.array(), schema, function)
                .reference(),
            function,
        );
        let key = if name.starts_with("Symbol.") {
            let symbol = lila_ir::WellKnownSymbol::ALL
                .into_iter()
                .find(|symbol| symbol.description() == name)
                .ok_or_else(|| EmitError::unsupported("unknown bootstrap well-known symbol"))?;
            let symbol = schema.reserve_gc_local(function).initialize(
                self.emit_well_known_symbol_reference(symbol, function)?,
                function,
            );
            let key = PropertyKeyLocals::from_symbol(schema, &symbol, function);
            symbol.clear(function);
            key
        } else {
            self.emit_function_string_key(name, function)?
        };
        self.emit_object_append_data_property_with_flags(
            &header,
            &key,
            value,
            writable,
            enumerable,
            configurable,
            function,
        )?;
        key.clear(function);
        header.clear(function);
        Ok(())
    }

    /// Install the public data links and callable prototype cache with the
    /// completed Array value, preserving the intrinsic attributes.
    pub(crate) fn emit_bind_realm_array_constructor_prototype(
        &mut self,
        constructor: &GcLocal<FunctionObject>,
        prototype: &RealmArrayPrototypeLocal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let header = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::OBJECT)
                .read(constructor, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        value.set_reference(prototype.array(), schema, function);
        let key = self.emit_function_string_key("prototype", function)?;
        self.emit_object_append_data_property_with_flags(
            &header, &key, &value, false, false, false, function,
        )?;
        key.clear(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<StoredValue>()
                .from_value(&value, function),
            function,
        );
        schema
            .struct_type::<FunctionObject>()
            .field(FunctionObjectSchema::PUBLIC_PROTOTYPE_CACHE)
            .write(
                constructor,
                GcOperand::reference(&stored, schema),
                schema,
                function,
            );
        stored.clear(function);
        value.set_reference(constructor, schema, function);
        self.emit_define_realm_array_prototype_data_with_flags(
            prototype,
            "constructor",
            &value,
            true,
            false,
            true,
            function,
        )?;
        value.clear(function);
        header.clear(function);
        Ok(())
    }

    pub(crate) fn release_realm_array_prototype_local(
        &mut self,
        prototype: RealmArrayPrototypeLocal,
        function: &mut Function,
    ) {
        prototype.0.clear(function);
    }
}
