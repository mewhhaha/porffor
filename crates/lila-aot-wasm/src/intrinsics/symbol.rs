//! Intrinsic members retain property order and exact shared function identity.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_symbol_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "Symbol",
            false,
            false,
            true,
            function,
        )?;
        let schema = self.runtime_schema();
        for symbol in [
            lila_ir::WellKnownSymbol::Iterator,
            lila_ir::WellKnownSymbol::AsyncIterator,
            lila_ir::WellKnownSymbol::HasInstance,
            lila_ir::WellKnownSymbol::IsConcatSpreadable,
            lila_ir::WellKnownSymbol::Match,
            lila_ir::WellKnownSymbol::MatchAll,
            lila_ir::WellKnownSymbol::Replace,
            lila_ir::WellKnownSymbol::Search,
            lila_ir::WellKnownSymbol::Species,
            lila_ir::WellKnownSymbol::Split,
            lila_ir::WellKnownSymbol::ToPrimitive,
            lila_ir::WellKnownSymbol::ToStringTag,
            lila_ir::WellKnownSymbol::Unscopables,
            lila_ir::WellKnownSymbol::Dispose,
            lila_ir::WellKnownSymbol::AsyncDispose,
        ] {
            let reference = schema.reserve_gc_local(function).initialize(
                self.emit_well_known_symbol_reference(symbol, function)?,
                function,
            );
            let value = schema.reserve_value_local(function);
            value.set_reference(&reference, schema, function);
            self.emit_install_intrinsic_data(
                context.constructor,
                IntrinsicKey::Name(symbol.member_name()),
                &value,
                false,
                false,
                false,
                function,
            )?;
            value.clear(function);
            reference.clear(function);
        }
        // Registered symbols have one Agent-wide GC root initialized with the
        // literal roots. Constructing another Realm never resets that table.
        for (name, builtin) in [
            ("for", StandardBuiltinId::SymbolFor),
            ("keyFor", StandardBuiltinId::SymbolKeyFor),
        ] {
            if self.functions.get(&builtin.function_id()).is_some() {
                self.emit_install_intrinsic_method(
                    context.constructor,
                    IntrinsicKey::Name(name),
                    builtin,
                    context.realm,
                    true,
                    true,
                    function,
                )?;
            }
        }
        for (name, builtin) in [
            ("toString", StandardBuiltinId::SymbolPrototypeToString),
            ("valueOf", StandardBuiltinId::SymbolPrototypeValueOf),
        ] {
            if self.functions.get(&builtin.function_id()).is_some() {
                self.emit_install_intrinsic_method(
                    context.prototype,
                    IntrinsicKey::Name(name),
                    builtin,
                    context.realm,
                    true,
                    true,
                    function,
                )?;
            }
        }
        if self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeDescriptionGetter.function_id())
            .is_some()
        {
            self.emit_install_intrinsic_accessor(
                context.prototype,
                IntrinsicKey::Name("description"),
                AccessorDescriptor::Getter(AccessorGetter::new(
                    StandardBuiltinId::SymbolPrototypeDescriptionGetter,
                )),
                context.realm,
                true,
                function,
            )?;
        }
        if self
            .functions
            .get(&StandardBuiltinId::SymbolPrototypeToPrimitive.function_id())
            .is_some()
        {
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToPrimitive),
                StandardBuiltinId::SymbolPrototypeToPrimitive,
                context.realm,
                false,
                true,
                function,
            )?;
        }
        Ok(())
    }
}
