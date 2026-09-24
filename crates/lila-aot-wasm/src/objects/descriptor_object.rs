//! 6.2.6.4 FromPropertyDescriptor: the one owner of descriptor *objects*.
//!
//! Every ordinary object whose own properties spell a Property Descriptor —
//! the result of `Object.getOwnPropertyDescriptor`, the argument of a Proxy
//! `defineProperty` trap, a module namespace descriptor, and the private
//! argument the engine hands its canonical definition builtins — is created
//! here and nowhere else.
//!
//! 6.2.6.4 steps 4-9 create each field with CreateDataPropertyOrThrow, so each
//! field is a writable, **enumerable**, configurable data property. The single
//! field writer, [`FunctionBuilder::emit_descriptor_object_field`], is private
//! and takes no attribute argument: a producer states *which* descriptor it
//! materializes and *which* prototype the object gets, and cannot state how
//! the fields are defined. Builtin attributes (writable, non-enumerable,
//! configurable) were the old default here and made
//! `Object.keys(Object.getOwnPropertyDescriptor({x: 1}, "x"))` empty.

use super::*;
use lila_ir::property_descriptor::CompleteDescriptor;

/// A `[[Writable]]`, `[[Enumerable]]` or `[[Configurable]]` field value. The
/// field is always published as a JavaScript Boolean, so there is no variant
/// that carries a tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorFlag {
    /// Decided when the compiler runs.
    Known(bool),
    /// A local holding `0` or `1`; the Boolean tag is implied.
    BooleanPayload(u32),
}

impl DescriptorFlag {
    /// A flag field whose tagged locals hold a JavaScript Boolean, such as a
    /// 6.2.6.5 ToBoolean result. Only the payload is read.
    pub(crate) fn boolean_value_presence(
        presence: Presence<TaggedLocals, u32>,
    ) -> Presence<Self, u32> {
        match presence {
            Presence::Absent => Presence::Absent,
            Presence::Present(value) => Presence::Present(Self::BooleanPayload(value.payload)),
            Presence::Runtime { present, value } => Presence::Runtime {
                present,
                value: Self::BooleanPayload(value.payload),
            },
        }
    }
}

/// The [`DescriptorCarrier`] of a descriptor about to become an object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DescriptorObjectLocals;

impl DescriptorCarrier for DescriptorObjectLocals {
    type Value = TaggedLocals;
    type Flag = DescriptorFlag;
    /// A local that is non-zero iff the field is present.
    type RuntimeFlag = u32;
}

/// The fields a descriptor object receives. Any subset is legal: 6.2.6.4 steps
/// 4-9 are each conditional on presence.
pub(crate) type DescriptorObjectFields = PartialDescriptor<DescriptorObjectLocals>;

/// The `[[Prototype]]` of a descriptor object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DescriptorObjectPrototype {
    /// 6.2.6.4 step 2 with the main Realm's %Object.prototype%.
    MainRealmObjectPrototype,
    /// 6.2.6.4 step 2 with a Realm's %Object.prototype% already in this local.
    /// The caller keeps ownership of the local.
    ObjectPrototypeLocal(u32),
    /// Not a 6.2.6.4 result: a private argument handed to the canonical
    /// `Object.defineProperty`/`Reflect.defineProperty` builtin, which never
    /// escapes to user code. Its 6.2.6.5 ToPropertyDescriptor walks the
    /// prototype chain, so a null prototype keeps `Object.prototype.get` and
    /// friends from joining the descriptor.
    PrivateCarrier,
}

/// One field's value, already paired with the field it belongs to.
#[derive(Clone, Copy)]
enum DescriptorObjectFieldValue {
    Value(TaggedLocals),
    Flag(DescriptorFlag),
}

fn field_presence<T: Copy>(
    presence: &Presence<T, u32>,
    wrap: fn(T) -> DescriptorObjectFieldValue,
) -> Presence<DescriptorObjectFieldValue, u32> {
    match presence {
        Presence::Absent => Presence::Absent,
        Presence::Present(value) => Presence::Present(wrap(*value)),
        Presence::Runtime { present, value } => Presence::Runtime {
            present: *present,
            value: wrap(*value),
        },
    }
}

impl FunctionBuilder<'_> {
    /// 6.2.6.4 FromPropertyDescriptor into `result_payload_local`. The caller
    /// sets the Object tag.
    ///
    /// Fields are created in [`DescriptorField::ALL`] order — value, writable,
    /// get, set, enumerable, configurable — which is 6.2.6.4's and is a
    /// *different* permutation from 6.2.6.5's read order.
    pub(crate) fn emit_from_property_descriptor(
        &mut self,
        prototype: DescriptorObjectPrototype,
        fields: &DescriptorObjectFields,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        match prototype {
            DescriptorObjectPrototype::MainRealmObjectPrototype => self
                .emit_alloc_plain_object_with_prototype(
                    None,
                    Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),
                    function,
                )?,
            DescriptorObjectPrototype::ObjectPrototypeLocal(prototype_local) => {
                self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?
            }
            DescriptorObjectPrototype::PrivateCarrier => {
                self.emit_alloc_plain_object_with_prototype(None, None, function)?
            }
        }
        function.instruction(&Instruction::LocalSet(object_local));

        // The `match` is exhaustive over `DescriptorField`, and it is the only
        // place a field is paired with its value carrier.
        for field in DescriptorField::ALL {
            let presence = match field {
                DescriptorField::Value => {
                    field_presence(&fields.value, DescriptorObjectFieldValue::Value)
                }
                DescriptorField::Writable => {
                    field_presence(&fields.writable, DescriptorObjectFieldValue::Flag)
                }
                DescriptorField::Get => {
                    field_presence(&fields.get, DescriptorObjectFieldValue::Value)
                }
                DescriptorField::Set => {
                    field_presence(&fields.set, DescriptorObjectFieldValue::Value)
                }
                DescriptorField::Enumerable => {
                    field_presence(&fields.enumerable, DescriptorObjectFieldValue::Flag)
                }
                DescriptorField::Configurable => {
                    field_presence(&fields.configurable, DescriptorObjectFieldValue::Flag)
                }
            };
            match presence {
                Presence::Absent => {}
                Presence::Present(value) => {
                    self.emit_descriptor_object_field(object_local, field, value, function)?;
                }
                Presence::Runtime { present, value } => {
                    function.instruction(&Instruction::LocalGet(present));
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::I64Ne);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_descriptor_object_field(object_local, field, value, function)?;
                    function.instruction(&Instruction::End);
                }
            }
        }

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(result_payload_local));
        self.release_temp_local(object_local);
        Ok(())
    }

    /// 6.2.6.4 for a fully populated descriptor: exactly the four keys of
    /// [`CompleteDescriptor::keys`]. An accessor has no `writable` field to
    /// pass.
    pub(crate) fn emit_from_complete_property_descriptor(
        &mut self,
        prototype: DescriptorObjectPrototype,
        descriptor: CompleteDescriptor<DescriptorObjectLocals>,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fields = match descriptor {
            CompleteDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => DescriptorObjectFields {
                value: Presence::Present(value),
                writable: Presence::Present(writable),
                get: Presence::Absent,
                set: Presence::Absent,
                enumerable: Presence::Present(enumerable),
                configurable: Presence::Present(configurable),
            },
            CompleteDescriptor::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => DescriptorObjectFields {
                value: Presence::Absent,
                writable: Presence::Absent,
                get: Presence::Present(get),
                set: Presence::Present(set),
                enumerable: Presence::Present(enumerable),
                configurable: Presence::Present(configurable),
            },
        };
        self.emit_from_property_descriptor(prototype, &fields, result_payload_local, function)
    }

    /// The private carrier for CreateDataProperty(O, P, V) when it is routed
    /// through a canonical definition builtin: `{ value: V, writable: true,
    /// enumerable: true, configurable: true }` with a null prototype.
    pub(crate) fn emit_create_data_property_descriptor_carrier(
        &mut self,
        value: TaggedLocals,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_from_complete_property_descriptor(
            DescriptorObjectPrototype::PrivateCarrier,
            CompleteDescriptor::Data {
                value,
                writable: DescriptorFlag::Known(true),
                enumerable: DescriptorFlag::Known(true),
                configurable: DescriptorFlag::Known(true),
            },
            result_payload_local,
            function,
        )
    }

    /// A data descriptor in the main Realm, with attributes known when the
    /// compiler runs.
    pub(crate) fn emit_alloc_data_descriptor_from_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        writable: bool,
        enumerable: bool,
        configurable: bool,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_from_complete_property_descriptor(
            DescriptorObjectPrototype::MainRealmObjectPrototype,
            CompleteDescriptor::Data {
                value: TaggedLocals::new(value_payload_local, value_tag_local),
                writable: DescriptorFlag::Known(writable),
                enumerable: DescriptorFlag::Known(enumerable),
                configurable: DescriptorFlag::Known(configurable),
            },
            result_payload_local,
            function,
        )
    }

    /// A data descriptor in the main Realm, with each attribute a 0/1 local.
    pub(crate) fn emit_alloc_data_descriptor_from_locals_with_flag_locals(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        writable_payload_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_from_complete_property_descriptor(
            DescriptorObjectPrototype::MainRealmObjectPrototype,
            CompleteDescriptor::Data {
                value: TaggedLocals::new(value_payload_local, value_tag_local),
                writable: DescriptorFlag::BooleanPayload(writable_payload_local),
                enumerable: DescriptorFlag::BooleanPayload(enumerable_payload_local),
                configurable: DescriptorFlag::BooleanPayload(configurable_payload_local),
            },
            result_payload_local,
            function,
        )
    }

    /// An accessor descriptor in the main Realm, with each attribute a 0/1
    /// local.
    pub(crate) fn emit_alloc_accessor_descriptor_from_locals_with_flag_local(
        &mut self,
        getter_payload_local: u32,
        getter_tag_local: u32,
        setter_payload_local: u32,
        setter_tag_local: u32,
        enumerable_payload_local: u32,
        configurable_payload_local: u32,
        result_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_from_complete_property_descriptor(
            DescriptorObjectPrototype::MainRealmObjectPrototype,
            CompleteDescriptor::Accessor {
                get: TaggedLocals::new(getter_payload_local, getter_tag_local),
                set: TaggedLocals::new(setter_payload_local, setter_tag_local),
                enumerable: DescriptorFlag::BooleanPayload(enumerable_payload_local),
                configurable: DescriptorFlag::BooleanPayload(configurable_payload_local),
            },
            result_payload_local,
            function,
        )
    }

    /// The only writer of a descriptor-object field: 6.2.6.4 steps 4-9's
    /// CreateDataPropertyOrThrow on the fresh object, i.e. a data property
    /// with `[[Writable]]`, `[[Enumerable]]` and `[[Configurable]]` all true.
    fn emit_descriptor_object_field(
        &mut self,
        object_local: u32,
        field: DescriptorField,
        value: DescriptorObjectFieldValue,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload(field.key())));
        function.instruction(&Instruction::LocalSet(key_local));
        let boolean_payload_local = self.reserve_temp_local();
        let boolean_tag_local = self.reserve_temp_local();
        let value = match value {
            DescriptorObjectFieldValue::Value(value) => value,
            DescriptorObjectFieldValue::Flag(flag) => {
                match flag {
                    DescriptorFlag::Known(flag) => {
                        function.instruction(&Instruction::I64Const(i64::from(flag)));
                    }
                    DescriptorFlag::BooleanPayload(payload_local) => {
                        function.instruction(&Instruction::LocalGet(payload_local));
                    }
                }
                function.instruction(&Instruction::LocalSet(boolean_payload_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(boolean_tag_local));
                TaggedLocals::new(boolean_payload_local, boolean_tag_local)
            }
        };
        self.emit_object_define_enumerable_data(
            object_local,
            key_local,
            value.payload,
            value.tag,
            function,
        )?;
        self.release_temp_local(boolean_tag_local);
        self.release_temp_local(boolean_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
