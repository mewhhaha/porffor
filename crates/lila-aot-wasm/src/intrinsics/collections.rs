//! Collection method order and aliases use completed defining-Realm values.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::objects::{AccessorDescriptor, AccessorGetter};

pub(crate) enum CollectionPrototypeIntrinsic {
    Map,
    Set,
    WeakMap,
    WeakSet,
}
impl CollectionPrototypeIntrinsic {
    const fn to_string_tag(&self) -> &'static str {
        match self {
            Self::Map => "Map",
            Self::Set => "Set",
            Self::WeakMap => "WeakMap",
            Self::WeakSet => "WeakSet",
        }
    }
    pub(crate) const fn realm_slot(&self) -> NonArrayRealmIntrinsicSlot {
        match self {
            Self::Map => NonArrayRealmIntrinsicSlot::MapPrototype,
            Self::Set => NonArrayRealmIntrinsicSlot::SetPrototype,
            Self::WeakMap => NonArrayRealmIntrinsicSlot::WeakMapPrototype,
            Self::WeakSet => NonArrayRealmIntrinsicSlot::WeakSetPrototype,
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_collection_prototype_to_string_tag(
        &mut self,
        intrinsic: CollectionPrototypeIntrinsic,
        prototype: &crate::gc_types::ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_string(
            prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            intrinsic.to_string_tag(),
            false,
            false,
            true,
            function,
        )
    }

    pub(crate) fn install_map_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_method(
            context.constructor,
            IntrinsicKey::Name("groupBy"),
            StandardBuiltinId::MapGroupBy,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Species),
            AccessorDescriptor::Getter(AccessorGetter::new(StandardBuiltinId::MapSpeciesGetter)),
            context.realm,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("clear", StandardBuiltinId::MapPrototypeClear),
            ("delete", StandardBuiltinId::MapPrototypeDelete),
            ("forEach", StandardBuiltinId::MapPrototypeForEach),
            ("get", StandardBuiltinId::MapPrototypeGet),
            ("getOrInsert", StandardBuiltinId::MapPrototypeGetOrInsert),
            (
                "getOrInsertComputed",
                StandardBuiltinId::MapPrototypeGetOrInsertComputed,
            ),
            ("has", StandardBuiltinId::MapPrototypeHas),
            ("keys", StandardBuiltinId::MapPrototypeKeys),
            ("set", StandardBuiltinId::MapPrototypeSet),
            ("values", StandardBuiltinId::MapPrototypeValues),
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
        self.emit_install_intrinsic_method_aliases(
            context.prototype,
            &[
                IntrinsicKey::Name("entries"),
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Iterator),
            ],
            StandardBuiltinId::MapPrototypeEntries,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("size"),
            AccessorDescriptor::Getter(AccessorGetter::new(
                StandardBuiltinId::MapPrototypeSizeGetter,
            )),
            context.realm,
            true,
            function,
        )?;
        self.emit_collection_prototype_to_string_tag(
            CollectionPrototypeIntrinsic::Map,
            context.prototype,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_weak_map_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("delete", StandardBuiltinId::WeakMapPrototypeDelete),
            ("get", StandardBuiltinId::WeakMapPrototypeGet),
            (
                "getOrInsert",
                StandardBuiltinId::WeakMapPrototypeGetOrInsert,
            ),
            (
                "getOrInsertComputed",
                StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed,
            ),
            ("has", StandardBuiltinId::WeakMapPrototypeHas),
            ("set", StandardBuiltinId::WeakMapPrototypeSet),
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
        self.emit_collection_prototype_to_string_tag(
            CollectionPrototypeIntrinsic::WeakMap,
            context.prototype,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_weak_set_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("add", StandardBuiltinId::WeakSetPrototypeAdd),
            ("delete", StandardBuiltinId::WeakSetPrototypeDelete),
            ("has", StandardBuiltinId::WeakSetPrototypeHas),
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
        self.emit_collection_prototype_to_string_tag(
            CollectionPrototypeIntrinsic::WeakSet,
            context.prototype,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_weak_ref_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("deref"),
            StandardBuiltinId::WeakRefPrototypeDeref,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "WeakRef",
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_finalization_registry_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("register"),
            StandardBuiltinId::FinalizationRegistryPrototypeRegister,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("unregister"),
            StandardBuiltinId::FinalizationRegistryPrototypeUnregister,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "FinalizationRegistry",
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_async_disposable_stack_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            (
                "adopt",
                StandardBuiltinId::AsyncDisposableStackPrototypeAdopt,
            ),
            (
                "defer",
                StandardBuiltinId::AsyncDisposableStackPrototypeDefer,
            ),
            ("move", StandardBuiltinId::AsyncDisposableStackPrototypeMove),
            ("use", StandardBuiltinId::AsyncDisposableStackPrototypeUse),
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
        self.emit_install_intrinsic_method_aliases(
            context.prototype,
            &[
                IntrinsicKey::Name("disposeAsync"),
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::AsyncDispose),
            ],
            StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("disposed"),
            AccessorDescriptor::Getter(AccessorGetter::new(
                StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter,
            )),
            context.realm,
            true,
            function,
        )?;
        self.emit_install_intrinsic_string(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::ToStringTag),
            "AsyncDisposableStack",
            false,
            false,
            true,
            function,
        )?;
        Ok(())
    }

    pub(crate) fn install_set_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Species),
            AccessorDescriptor::Getter(AccessorGetter::new(StandardBuiltinId::SetSpeciesGetter)),
            context.realm,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("add", StandardBuiltinId::SetPrototypeAdd),
            ("clear", StandardBuiltinId::SetPrototypeClear),
            ("delete", StandardBuiltinId::SetPrototypeDelete),
            ("difference", StandardBuiltinId::SetPrototypeDifference),
            ("forEach", StandardBuiltinId::SetPrototypeForEach),
            ("has", StandardBuiltinId::SetPrototypeHas),
            ("intersection", StandardBuiltinId::SetPrototypeIntersection),
            (
                "isDisjointFrom",
                StandardBuiltinId::SetPrototypeIsDisjointFrom,
            ),
            ("isSubsetOf", StandardBuiltinId::SetPrototypeIsSubsetOf),
            ("isSupersetOf", StandardBuiltinId::SetPrototypeIsSupersetOf),
            (
                "symmetricDifference",
                StandardBuiltinId::SetPrototypeSymmetricDifference,
            ),
            ("union", StandardBuiltinId::SetPrototypeUnion),
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
        self.emit_install_intrinsic_method_aliases(
            context.prototype,
            &[
                IntrinsicKey::Name("values"),
                IntrinsicKey::Name("keys"),
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Iterator),
            ],
            StandardBuiltinId::SetPrototypeValues,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Name("entries"),
            StandardBuiltinId::SetPrototypeEntries,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.prototype,
            IntrinsicKey::Name("size"),
            AccessorDescriptor::Getter(AccessorGetter::new(
                StandardBuiltinId::SetPrototypeSizeGetter,
            )),
            context.realm,
            true,
            function,
        )?;
        self.emit_collection_prototype_to_string_tag(
            CollectionPrototypeIntrinsic::Set,
            context.prototype,
            function,
        )?;
        Ok(())
    }
}
