//! Intrinsic methods preserve their canonical property installation order.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_array_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("from", StandardBuiltinId::ArrayFrom),
            ("fromAsync", StandardBuiltinId::ArrayFromAsync),
            ("of", StandardBuiltinId::ArrayOf),
            ("isArray", StandardBuiltinId::ArrayIsArray),
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
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Species),
            AccessorDescriptor::Getter(AccessorGetter::new(StandardBuiltinId::ArraySpeciesGetter)),
            context.realm,
            true,
            function,
        )?;
        for (name, builtin) in [
            ("toString", StandardBuiltinId::TypedArrayPrototypeToString),
            ("concat", StandardBuiltinId::ArrayPrototypeConcat),
            ("join", StandardBuiltinId::ArrayPrototypeJoin),
            ("slice", StandardBuiltinId::ArrayPrototypeSlice),
            ("splice", StandardBuiltinId::ArrayPrototypeSplice),
            ("fill", StandardBuiltinId::ArrayPrototypeFill),
            ("sort", StandardBuiltinId::ArrayPrototypeSort),
            (
                "toLocaleString",
                StandardBuiltinId::ArrayPrototypeToLocaleString,
            ),
            ("flat", StandardBuiltinId::ArrayPrototypeFlat),
            ("flatMap", StandardBuiltinId::ArrayPrototypeFlatMap),
            ("at", StandardBuiltinId::ArrayPrototypeAt),
            ("toReversed", StandardBuiltinId::ArrayPrototypeToReversed),
            ("toSpliced", StandardBuiltinId::ArrayPrototypeToSpliced),
            ("toSorted", StandardBuiltinId::ArrayPrototypeToSorted),
            ("with", StandardBuiltinId::ArrayPrototypeWith),
            ("reverse", StandardBuiltinId::ArrayPrototypeReverse),
            ("copyWithin", StandardBuiltinId::ArrayPrototypeCopyWithin),
            ("includes", StandardBuiltinId::ArrayPrototypeIncludes),
            ("indexOf", StandardBuiltinId::ArrayPrototypeIndexOf),
            ("lastIndexOf", StandardBuiltinId::ArrayPrototypeLastIndexOf),
            ("find", StandardBuiltinId::ArrayPrototypeFind),
            ("findIndex", StandardBuiltinId::ArrayPrototypeFindIndex),
            ("findLast", StandardBuiltinId::ArrayPrototypeFindLast),
            (
                "findLastIndex",
                StandardBuiltinId::ArrayPrototypeFindLastIndex,
            ),
            ("every", StandardBuiltinId::ArrayPrototypeEvery),
            ("some", StandardBuiltinId::ArrayPrototypeSome),
            ("forEach", StandardBuiltinId::ArrayPrototypeForEach),
            ("filter", StandardBuiltinId::ArrayPrototypeFilter),
            ("map", StandardBuiltinId::ArrayPrototypeMap),
            ("reduce", StandardBuiltinId::ArrayPrototypeReduce),
            ("reduceRight", StandardBuiltinId::ArrayPrototypeReduceRight),
            ("pop", StandardBuiltinId::ArrayPrototypePop),
            ("push", StandardBuiltinId::ArrayPrototypePush),
            ("shift", StandardBuiltinId::ArrayPrototypeShift),
            ("unshift", StandardBuiltinId::ArrayPrototypeUnshift),
            ("keys", StandardBuiltinId::ArrayPrototypeKeys),
            ("entries", StandardBuiltinId::ArrayPrototypeEntries),
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
                IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Iterator),
            ],
            StandardBuiltinId::ArrayPrototypeValues,
            context.realm,
            true,
            true,
            function,
        )?;
        // Arguments objects capture the exact installed canonical values function
        // through its strong Realm intrinsic slot, without a public property Get.
        let schema = self.runtime_schema();
        let unscopables = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(None, function)?,
            function,
        );
        let object = schema.reserve_value_local(function);
        object.set_reference(&unscopables, schema, function);
        let value = schema.reserve_value_local(function);
        value.set_scalar(crate::gc_types::ScalarValue::Boolean(true), function);
        for name in [
            "at",
            "copyWithin",
            "entries",
            "fill",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "flat",
            "flatMap",
            "includes",
            "keys",
            "toReversed",
            "toSorted",
            "toSpliced",
            "values",
        ] {
            self.emit_install_intrinsic_data(
                &object,
                IntrinsicKey::Name(name),
                &value,
                true,
                true,
                true,
                function,
            )?;
        }
        self.emit_install_intrinsic_data(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Unscopables),
            &object,
            false,
            false,
            true,
            function,
        )?;
        value.clear(function);
        object.clear(function);
        unscopables.clear(function);
        Ok(())
    }
}
