//! Iterator prototype accessors retain their defining Realm through FunctionContext.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::objects::{AccessorDescriptor, AccessorGetter, AccessorSetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_iterator_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("from", StandardBuiltinId::IteratorFrom),
            ("concat", StandardBuiltinId::IteratorConcat),
            ("zip", StandardBuiltinId::IteratorZip),
            ("zipKeyed", StandardBuiltinId::IteratorZipKeyed),
        ] {
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
        for (name, builtin) in [
            ("toArray", StandardBuiltinId::IteratorPrototypeToArray),
            ("forEach", StandardBuiltinId::IteratorPrototypeForEach),
            ("every", StandardBuiltinId::IteratorPrototypeEvery),
            ("some", StandardBuiltinId::IteratorPrototypeSome),
            ("find", StandardBuiltinId::IteratorPrototypeFind),
            ("reduce", StandardBuiltinId::IteratorPrototypeReduce),
            ("map", StandardBuiltinId::IteratorPrototypeMap),
            ("filter", StandardBuiltinId::IteratorPrototypeFilter),
            ("flatMap", StandardBuiltinId::IteratorPrototypeFlatMap),
            ("take", StandardBuiltinId::IteratorPrototypeTake),
            ("drop", StandardBuiltinId::IteratorPrototypeDrop),
        ] {
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
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("constructor"),
            AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(StandardBuiltinId::IteratorPrototypeConstructorGetter),
                setter: AccessorSetter::new(StandardBuiltinId::IteratorPrototypeConstructorSetter),
            },
            context.realm,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Iterator),
            StandardBuiltinId::ArrayIteratorIdentity,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Dispose),
            StandardBuiltinId::IteratorPrototypeSymbolDispose,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            AccessorDescriptor::GetterAndSetter {
                getter: AccessorGetter::new(StandardBuiltinId::IteratorPrototypeToStringTagGetter),
                setter: AccessorSetter::new(StandardBuiltinId::IteratorPrototypeToStringTagSetter),
            },
            context.realm,
            true,
            function,
        )?;
        let schema = self.runtime_schema();
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            context.realm.realm(),
            NonArrayRealmIntrinsicSlot::IteratorFromWrapperPrototype,
            &prototype,
            function,
        );
        self.emit_install_intrinsic_method(
            &prototype,
            IntrinsicKey::Name("next"),
            StandardBuiltinId::IteratorFromWrapperNext,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            &prototype,
            IntrinsicKey::Name("return"),
            StandardBuiltinId::IteratorFromWrapperReturn,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_load_non_array_realm_intrinsic(
            context.realm.realm(),
            NonArrayRealmIntrinsicSlot::IteratorHelperPrototype,
            &prototype,
            function,
        );
        self.emit_install_intrinsic_method(
            &prototype,
            IntrinsicKey::Name("next"),
            StandardBuiltinId::IteratorHelperNext,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            &prototype,
            IntrinsicKey::Name("return"),
            StandardBuiltinId::IteratorHelperReturn,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_string(
            &prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "Iterator Helper",
            false,
            false,
            true,
            function,
        )?;
        prototype.clear(function);
        Ok(())
    }
}
