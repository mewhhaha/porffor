//! Shared Intl property order for entry and created Realm installation.

use super::super::*;
use super::IntrinsicInstall;
use crate::functions::NonArrayRealmIntrinsicSlot;

#[derive(Clone, Copy)]
pub(crate) enum IntlIntrinsicPropertyKind {
    Getter,
    Method,
}

pub(crate) struct IntlIntrinsicProperty {
    pub(crate) name: &'static str,
    pub(crate) builtin: StandardBuiltinId,
    pub(crate) kind: IntlIntrinsicPropertyKind,
}

pub(crate) struct IntlConstructorProperties {
    pub(crate) prototype_name: &'static str,
    pub(crate) prototype_slot: NonArrayRealmIntrinsicSlot,
    pub(crate) constructor: &'static [IntlIntrinsicProperty],
    pub(crate) prototype: &'static [IntlIntrinsicProperty],
}

const LOCALE_PROTOTYPE_PROPERTIES: &[IntlIntrinsicProperty] = &[
    IntlIntrinsicProperty {
        name: "language",
        builtin: StandardBuiltinId::IntlLocalePrototypeLanguageGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "script",
        builtin: StandardBuiltinId::IntlLocalePrototypeScriptGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "region",
        builtin: StandardBuiltinId::IntlLocalePrototypeRegionGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "baseName",
        builtin: StandardBuiltinId::IntlLocalePrototypeBaseNameGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "calendar",
        builtin: StandardBuiltinId::IntlLocalePrototypeCalendarGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "collation",
        builtin: StandardBuiltinId::IntlLocalePrototypeCollationGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "firstDayOfWeek",
        builtin: StandardBuiltinId::IntlLocalePrototypeFirstDayOfWeekGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "hourCycle",
        builtin: StandardBuiltinId::IntlLocalePrototypeHourCycleGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "caseFirst",
        builtin: StandardBuiltinId::IntlLocalePrototypeCaseFirstGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "numeric",
        builtin: StandardBuiltinId::IntlLocalePrototypeNumericGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "numberingSystem",
        builtin: StandardBuiltinId::IntlLocalePrototypeNumberingSystemGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "variants",
        builtin: StandardBuiltinId::IntlLocalePrototypeVariantsGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "toString",
        builtin: StandardBuiltinId::IntlLocalePrototypeToString,
        kind: IntlIntrinsicPropertyKind::Method,
    },
];

const DATE_TIME_FORMAT_CONSTRUCTOR_PROPERTIES: &[IntlIntrinsicProperty] =
    &[IntlIntrinsicProperty {
        name: "supportedLocalesOf",
        builtin: StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf,
        kind: IntlIntrinsicPropertyKind::Method,
    }];

const DATE_TIME_FORMAT_PROTOTYPE_PROPERTIES: &[IntlIntrinsicProperty] = &[
    IntlIntrinsicProperty {
        name: "format",
        builtin: StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter,
        kind: IntlIntrinsicPropertyKind::Getter,
    },
    IntlIntrinsicProperty {
        name: "formatToParts",
        builtin: StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts,
        kind: IntlIntrinsicPropertyKind::Method,
    },
    IntlIntrinsicProperty {
        name: "formatRange",
        builtin: StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange,
        kind: IntlIntrinsicPropertyKind::Method,
    },
    IntlIntrinsicProperty {
        name: "formatRangeToParts",
        builtin: StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts,
        kind: IntlIntrinsicPropertyKind::Method,
    },
    IntlIntrinsicProperty {
        name: "resolvedOptions",
        builtin: StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions,
        kind: IntlIntrinsicPropertyKind::Method,
    },
];

/// Namespace membership is separately proven by IntlNamespaceMembers. A newly
/// catalogued constructor must also supply its complete intrinsic properties.
pub(crate) fn intl_constructor_properties(
    builtin: StandardBuiltinId,
) -> Option<IntlConstructorProperties> {
    match builtin {
        StandardBuiltinId::IntlLocaleConstructor => Some(IntlConstructorProperties {
            prototype_name: "Intl.Locale",
            prototype_slot: NonArrayRealmIntrinsicSlot::IntlLocalePrototype,
            constructor: &[],
            prototype: LOCALE_PROTOTYPE_PROPERTIES,
        }),
        StandardBuiltinId::IntlDateTimeFormatConstructor => Some(IntlConstructorProperties {
            prototype_name: "Intl.DateTimeFormat",
            prototype_slot: NonArrayRealmIntrinsicSlot::IntlDateTimeFormatPrototype,
            constructor: DATE_TIME_FORMAT_CONSTRUCTOR_PROPERTIES,
            prototype: DATE_TIME_FORMAT_PROTOTYPE_PROPERTIES,
        }),
        _ => None,
    }
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn install_intl_locale_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.install_intl_constructor_intrinsics(context, function)
    }

    pub(crate) fn install_intl_date_time_format_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.install_intl_constructor_intrinsics(context, function)
    }

    fn install_intl_constructor_intrinsics(
        &mut self,
        context: &IntrinsicInstall<'_>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let properties = intl_constructor_properties(context.builtin).ok_or_else(|| {
            EmitError::unsupported("missing represented Intl constructor properties")
        })?;
        function.instruction(&Instruction::GlobalGet(context.prototype_global_index));
        function.instruction(&Instruction::LocalSet(context.prototype_object_local));
        for (receiver, entries) in [
            (context.object_local, properties.constructor),
            (context.prototype_object_local, properties.prototype),
        ] {
            for property in entries {
                let meta = self
                    .functions
                    .get(&property.builtin.function_id())
                    .ok_or_else(|| {
                        EmitError::unsupported(format!(
                            "missing {} metadata",
                            property.builtin.debug_name()
                        ))
                    })?;
                self.emit_function_value_payload(meta, function)?;
                function.instruction(&Instruction::LocalSet(context.payload_local));
                self.emit_define_intl_intrinsic_function_property(
                    receiver,
                    property,
                    context.payload_local,
                    function,
                )?;
            }
        }
        self.emit_define_intl_intrinsic_to_string_tag(
            context.prototype_object_local,
            properties.prototype_name,
            function,
        )
    }

    pub(crate) fn emit_define_intl_intrinsic_function_property(
        &mut self,
        receiver: u32,
        property: &IntlIntrinsicProperty,
        callable: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(property.name)));
        function.instruction(&Instruction::LocalSet(key));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        let result = match property.kind {
            IntlIntrinsicPropertyKind::Getter => self
                .emit_object_append_accessor_property_with_flags(
                    receiver,
                    key,
                    Some((callable, tag)),
                    None,
                    false,
                    true,
                    function,
                ),
            IntlIntrinsicPropertyKind::Method => self.emit_object_append_data_property_with_flags(
                receiver, key, callable, tag, true, false, true, function,
            ),
        };
        self.release_temp_local(tag);
        self.release_temp_local(key);
        result
    }

    pub(crate) fn emit_define_intl_intrinsic_to_string_tag(
        &mut self,
        receiver: u32,
        name: &str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key = self.reserve_temp_local();
        let value = self.reserve_temp_local();
        let tag = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.toStringTag"),
        ));
        function.instruction(&Instruction::LocalSet(key));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag));
        let result = self.emit_object_append_data_property_with_flags(
            receiver, key, value, tag, false, false, true, function,
        );
        self.release_temp_local(tag);
        self.release_temp_local(value);
        self.release_temp_local(key);
        result
    }
}
