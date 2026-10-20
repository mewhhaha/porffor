//! Intrinsic members retain property order and exact shared function identity.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};
use crate::objects::{AccessorDescriptor, AccessorGetter};

impl FunctionBuilder<'_> {
    pub(crate) fn install_regexp_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for (name, builtin) in [
            ("compile", StandardBuiltinId::RegExpPrototypeCompile),
            ("toString", StandardBuiltinId::RegExpPrototypeToString),
            ("exec", StandardBuiltinId::RegExpPrototypeExec),
            ("test", StandardBuiltinId::RegExpPrototypeTest),
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
        for (name, getter) in [
            ("source", StandardBuiltinId::RegExpPrototypeSourceGetter),
            (
                "hasIndices",
                StandardBuiltinId::RegExpPrototypeHasIndicesGetter,
            ),
            ("global", StandardBuiltinId::RegExpPrototypeGlobalGetter),
            (
                "ignoreCase",
                StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter,
            ),
            (
                "multiline",
                StandardBuiltinId::RegExpPrototypeMultilineGetter,
            ),
            ("dotAll", StandardBuiltinId::RegExpPrototypeDotAllGetter),
            ("unicode", StandardBuiltinId::RegExpPrototypeUnicodeGetter),
            (
                "unicodeSets",
                StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter,
            ),
            ("sticky", StandardBuiltinId::RegExpPrototypeStickyGetter),
            ("flags", StandardBuiltinId::RegExpPrototypeFlagsGetter),
        ] {
            self.emit_install_intrinsic_accessor(
                context.prototype,
                IntrinsicKey::Name(name),
                AccessorDescriptor::Getter(AccessorGetter::new(getter)),
                context.realm,
                true,
                function,
            )?;
        }
        for (symbol, builtin) in [
            (
                scriptw_ir::WellKnownSymbol::Match,
                StandardBuiltinId::RegExpPrototypeSymbolMatch,
            ),
            (
                scriptw_ir::WellKnownSymbol::MatchAll,
                StandardBuiltinId::RegExpPrototypeSymbolMatchAll,
            ),
            (
                scriptw_ir::WellKnownSymbol::Replace,
                StandardBuiltinId::RegExpPrototypeSymbolReplace,
            ),
            (
                scriptw_ir::WellKnownSymbol::Search,
                StandardBuiltinId::RegExpPrototypeSymbolSearch,
            ),
            (
                scriptw_ir::WellKnownSymbol::Split,
                StandardBuiltinId::RegExpPrototypeSymbolSplit,
            ),
        ] {
            self.emit_install_intrinsic_method(
                context.prototype,
                IntrinsicKey::Symbol(symbol),
                builtin,
                context.realm,
                true,
                true,
                function,
            )?;
        }
        self.emit_install_intrinsic_method(
            context.constructor,
            IntrinsicKey::Name("escape"),
            StandardBuiltinId::RegExpEscape,
            context.realm,
            true,
            true,
            function,
        )?;
        self.emit_install_intrinsic_accessor(
            context.constructor,
            IntrinsicKey::Symbol(scriptw_ir::WellKnownSymbol::Species),
            AccessorDescriptor::Getter(AccessorGetter::new(StandardBuiltinId::RegExpSpeciesGetter)),
            context.realm,
            true,
            function,
        )?;
        self.emit_install_regexp_legacy_accessors(context, function)?;
        Ok(())
    }
}
