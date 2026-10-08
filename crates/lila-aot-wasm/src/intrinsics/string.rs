//! Fresh intrinsic properties retain completed values and defining Realm.

use super::super::*;
use super::{IntrinsicInstall, IntrinsicKey};

impl FunctionBuilder<'_> {
    pub(crate) fn install_string_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        for builtin in [
            StandardBuiltinId::StringPrototypeToString,
            StandardBuiltinId::StringPrototypeValueOf,
            StandardBuiltinId::StringPrototypeCharAt,
            StandardBuiltinId::StringPrototypeConcat,
            StandardBuiltinId::StringPrototypeCharCodeAt,
            StandardBuiltinId::StringPrototypeCodePointAt,
            StandardBuiltinId::StringPrototypeAt,
            StandardBuiltinId::StringPrototypeAnchor,
            StandardBuiltinId::StringPrototypeBig,
            StandardBuiltinId::StringPrototypeBlink,
            StandardBuiltinId::StringPrototypeBold,
            StandardBuiltinId::StringPrototypeFixed,
            StandardBuiltinId::StringPrototypeFontcolor,
            StandardBuiltinId::StringPrototypeFontsize,
            StandardBuiltinId::StringPrototypeItalics,
            StandardBuiltinId::StringPrototypeLink,
            StandardBuiltinId::StringPrototypeSmall,
            StandardBuiltinId::StringPrototypeStrike,
            StandardBuiltinId::StringPrototypeSub,
            StandardBuiltinId::StringPrototypeSubstr,
            StandardBuiltinId::StringPrototypeSubstring,
            StandardBuiltinId::StringPrototypeSup,
            StandardBuiltinId::StringPrototypeMatch,
            StandardBuiltinId::StringPrototypeMatchAll,
            StandardBuiltinId::StringPrototypeReplace,
            StandardBuiltinId::StringPrototypeReplaceAll,
            StandardBuiltinId::StringPrototypeSearch,
            StandardBuiltinId::StringPrototypeIndexOf,
            StandardBuiltinId::StringPrototypeLastIndexOf,
            StandardBuiltinId::StringPrototypeSlice,
            StandardBuiltinId::StringPrototypeSplit,
            StandardBuiltinId::StringPrototypePadStart,
            StandardBuiltinId::StringPrototypePadEnd,
            StandardBuiltinId::StringPrototypeRepeat,
            StandardBuiltinId::StringPrototypeEndsWith,
            StandardBuiltinId::StringPrototypeIncludes,
            StandardBuiltinId::StringPrototypeStartsWith,
            StandardBuiltinId::StringPrototypeNormalize,
            StandardBuiltinId::StringPrototypeLocaleCompare,
            StandardBuiltinId::StringPrototypeToLocaleLowerCase,
            StandardBuiltinId::StringPrototypeToLocaleUpperCase,
            StandardBuiltinId::StringPrototypeToLowerCase,
            StandardBuiltinId::StringPrototypeToUpperCase,
            StandardBuiltinId::StringPrototypeTrim,
            StandardBuiltinId::StringPrototypeTrimStart,
            StandardBuiltinId::StringPrototypeTrimEnd,
            StandardBuiltinId::StringPrototypeIsWellFormed,
            StandardBuiltinId::StringPrototypeToWellFormed,
        ] {
            match builtin {
                StandardBuiltinId::StringPrototypeTrimStart => self
                    .emit_install_intrinsic_method_aliases(
                        context.prototype,
                        &[
                            IntrinsicKey::Name("trimStart"),
                            IntrinsicKey::Name("trimLeft"),
                        ],
                        builtin,
                        context.realm,
                        true,
                        true,
                        function,
                    )?,
                StandardBuiltinId::StringPrototypeTrimEnd => self
                    .emit_install_intrinsic_method_aliases(
                        context.prototype,
                        &[
                            IntrinsicKey::Name("trimEnd"),
                            IntrinsicKey::Name("trimRight"),
                        ],
                        builtin,
                        context.realm,
                        true,
                        true,
                        function,
                    )?,
                _ => self.emit_install_intrinsic_method(
                    context.prototype,
                    IntrinsicKey::Name(builtin.string_prototype_method_name().ok_or_else(
                        || EmitError::unsupported("planned String method has no prototype name"),
                    )?),
                    builtin,
                    context.realm,
                    true,
                    true,
                    function,
                )?,
            }
        }
        self.emit_install_intrinsic_method(
            context.prototype,
            IntrinsicKey::Symbol(lila_ir::WellKnownSymbol::Iterator),
            StandardBuiltinId::StringPrototypeIterator,
            context.realm,
            true,
            true,
            function,
        )?;
        Ok(())
    }
}
