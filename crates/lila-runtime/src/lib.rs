use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static NEXT_REALM_ID: AtomicU64 = AtomicU64::new(1);

macro_rules! agent_host_operations {
    ($($variant:ident = $wire:literal;)+) => {
        /// The closed operation domain carried by the Wasm `agent_call` host import.
        ///
        /// The explicit discriminants are a stable wire ABI shared by the AOT
        /// producer and the engine consumer. Unknown integers can enter only at
        /// the host boundary, where [`Self::from_wire`] rejects them. Once
        /// decoded, an exhaustive match makes a newly added operation a compile
        /// error until the engine supplies its behavior.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(i64)]
        pub enum AgentHostOperation {
            $($variant = $wire,)+
        }

        impl AgentHostOperation {
            /// The stable `i64` value written into emitted Wasm.
            pub const fn wire(self) -> i64 {
                self as i64
            }

            /// Decode the untrusted operation word received by the host import.
            pub const fn from_wire(wire: i64) -> Option<Self> {
                match wire {
                    $($wire => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }

        $(const _: () = assert!(AgentHostOperation::$variant.wire() == $wire);)+
    };
}

agent_host_operations! {
    Start = 1;
    Broadcast = 2;
    ReceiveBroadcast = 3;
    Report = 4;
    ReportLength = 5;
    ReportCopy = 6;
    Sleep = 7;
    MonotonicNow = 8;
    Leaving = 9;
    RegisterAsyncWaiter = 10;
    PollAsyncWaiter = 11;
    NotifyAsyncWaiters = 12;
    CancelAsyncWaiter = 13;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AgentId(u64);

impl AgentId {
    pub const MAIN: Self = Self(1);

    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RealmId(u64);

impl RealmId {
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicRole {
    Constructor,
    Prototype,
    Function,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicFunctionMetadata {
    pub name: &'static str,
    pub length: u32,
    pub length_name_configurable: bool,
    pub constructable: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicDescriptor {
    pub kind: IntrinsicKind,
    pub spec_name: &'static str,
    pub role: IntrinsicRole,
    pub prototype: Option<IntrinsicKind>,
    pub function: Option<IntrinsicFunctionMetadata>,
    pub link: IntrinsicLink,
}

impl IntrinsicDescriptor {
    pub const fn is_callable(&self) -> bool {
        self.function.is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicLink {
    None,
    ConstructorToPrototype(IntrinsicKind),
    PrototypeToConstructor(IntrinsicKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntrinsicPropertyKey {
    String(&'static str),
    WellKnownSymbol(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicPropertyValue {
    Intrinsic(IntrinsicKind),
    String(&'static str),
    U32(u32),
    Undefined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicPropertyAttributes {
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
}

impl IntrinsicPropertyAttributes {
    pub const BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE: Self = Self {
        writable: false,
        enumerable: false,
        configurable: true,
    };

    pub const BUILTIN_FUNCTION_LENGTH_NAME_FIXED: Self = Self {
        writable: false,
        enumerable: false,
        configurable: false,
    };

    pub const CONSTRUCTOR_PROTOTYPE: Self = Self {
        writable: false,
        enumerable: false,
        configurable: false,
    };

    pub const PROTOTYPE_CONSTRUCTOR: Self = Self {
        writable: true,
        enumerable: false,
        configurable: true,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntrinsicPropertyDescriptor {
    pub owner: IntrinsicKind,
    pub key: IntrinsicPropertyKey,
    pub value: IntrinsicPropertyValue,
    pub attributes: IntrinsicPropertyAttributes,
}

macro_rules! intrinsic_registry {
    (
        $(
            IntrinsicDescriptor {
                kind: IntrinsicKind::$kind:ident,
                spec_name: $spec_name:literal,
                role: IntrinsicRole::$role:ident,
                prototype: $prototype:expr,
                function: $function:expr,
                link: $link:expr,
            },
        )+
    ) => {
        #[repr(usize)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum IntrinsicKind {
            $($kind),+
        }

        impl IntrinsicKind {
            const COUNT: usize = [$(Self::$kind),+].len();

            pub const ALL: &'static [Self] = &[$(Self::$kind),+];

            pub const fn descriptor(self) -> &'static IntrinsicDescriptor {
                &INTRINSIC_DESCRIPTORS[self as usize]
            }
        }

        pub const INTRINSIC_DESCRIPTORS: [IntrinsicDescriptor; IntrinsicKind::COUNT] = [
            $(
                IntrinsicDescriptor {
                    kind: IntrinsicKind::$kind,
                    spec_name: $spec_name,
                    role: IntrinsicRole::$role,
                    prototype: $prototype,
                    function: $function,
                    link: $link,
                },
            )+
        ];
    };
}

intrinsic_registry! {
    IntrinsicDescriptor {
        kind: IntrinsicKind::ObjectConstructor,
        spec_name: "%Object%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "Object",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::ObjectPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ObjectPrototype,
        spec_name: "%Object.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: None,
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::ObjectConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::FunctionConstructor,
        spec_name: "%Function%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "Function",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::FunctionPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::FunctionPrototype,
        spec_name: "%Function.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: true,
            constructable: false,
        }),
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::FunctionConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayConstructor,
        spec_name: "%Array%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "Array",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::ArrayPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayPrototype,
        spec_name: "%Array.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::ArrayConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::BigIntConstructor,
        spec_name: "%BigInt%",
        role: IntrinsicRole::Function,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "BigInt",
            length: 1,
            length_name_configurable: true,
            constructable: false,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::BigIntPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::BigIntPrototype,
        spec_name: "%BigInt.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::BigIntConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DateConstructor,
        spec_name: "%Date%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "Date",
            length: 7,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::DatePrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DatePrototype,
        spec_name: "%Date.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::DateConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ProxyConstructor,
        spec_name: "%Proxy%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "Proxy",
            length: 2,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::None,
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayBufferConstructor,
        spec_name: "%ArrayBuffer%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "ArrayBuffer",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::ArrayBufferPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayBufferPrototype,
        spec_name: "%ArrayBuffer.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::ArrayBufferConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DataViewConstructor,
        spec_name: "%DataView%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "DataView",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::DataViewPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DataViewPrototype,
        spec_name: "%DataView.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::DataViewConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypedArrayConstructor,
        spec_name: "%TypedArray%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "TypedArray",
            length: 0,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::TypedArrayPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypedArrayPrototype,
        spec_name: "%TypedArray.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::TypedArrayConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::Uint8ArrayConstructor,
        spec_name: "%Uint8Array%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::TypedArrayConstructor),
        function: Some(IntrinsicFunctionMetadata {
            name: "Uint8Array",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::Uint8ArrayPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::Uint8ArrayPrototype,
        spec_name: "%Uint8Array.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::TypedArrayPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::Uint8ArrayConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypeErrorConstructor,
        spec_name: "%TypeError%",
        role: IntrinsicRole::Constructor,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "TypeError",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        }),
        link: IntrinsicLink::ConstructorToPrototype(IntrinsicKind::TypeErrorPrototype),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypeErrorPrototype,
        spec_name: "%TypeError.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::PrototypeToConstructor(IntrinsicKind::TypeErrorConstructor),
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::IteratorPrototype,
        spec_name: "%IteratorPrototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
        link: IntrinsicLink::None,
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ThrowTypeError,
        spec_name: "%ThrowTypeError%",
        role: IntrinsicRole::Function,
        prototype: Some(IntrinsicKind::FunctionPrototype),
        function: Some(IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: false,
            constructable: false,
        }),
        link: IntrinsicLink::None,
    },
}

const fn function_length_name_attributes(
    function: IntrinsicFunctionMetadata,
) -> IntrinsicPropertyAttributes {
    if function.length_name_configurable {
        IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE
    } else {
        IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_FIXED
    }
}

const fn function_name_descriptor(
    owner: IntrinsicKind,
    function: IntrinsicFunctionMetadata,
) -> IntrinsicPropertyDescriptor {
    IntrinsicPropertyDescriptor {
        owner,
        key: IntrinsicPropertyKey::String("name"),
        value: IntrinsicPropertyValue::String(function.name),
        attributes: function_length_name_attributes(function),
    }
}

const fn function_length_descriptor(
    owner: IntrinsicKind,
    function: IntrinsicFunctionMetadata,
) -> IntrinsicPropertyDescriptor {
    IntrinsicPropertyDescriptor {
        owner,
        key: IntrinsicPropertyKey::String("length"),
        value: IntrinsicPropertyValue::U32(function.length),
        attributes: function_length_name_attributes(function),
    }
}

const fn constructor_prototype_descriptor(
    owner: IntrinsicKind,
    prototype: IntrinsicKind,
) -> IntrinsicPropertyDescriptor {
    IntrinsicPropertyDescriptor {
        owner,
        key: IntrinsicPropertyKey::String("prototype"),
        value: IntrinsicPropertyValue::Intrinsic(prototype),
        attributes: IntrinsicPropertyAttributes::CONSTRUCTOR_PROTOTYPE,
    }
}

const fn prototype_constructor_descriptor(
    owner: IntrinsicKind,
    constructor: IntrinsicKind,
) -> IntrinsicPropertyDescriptor {
    IntrinsicPropertyDescriptor {
        owner,
        key: IntrinsicPropertyKey::String("constructor"),
        value: IntrinsicPropertyValue::Intrinsic(constructor),
        attributes: IntrinsicPropertyAttributes::PROTOTYPE_CONSTRUCTOR,
    }
}

const fn intrinsic_property_descriptor_count() -> usize {
    let mut count = 0;
    let mut index = 0;
    while index < INTRINSIC_DESCRIPTORS.len() {
        match INTRINSIC_DESCRIPTORS[index].function {
            Some(_) => count += 2,
            None => {}
        }
        match INTRINSIC_DESCRIPTORS[index].link {
            IntrinsicLink::None => {}
            IntrinsicLink::ConstructorToPrototype(_) | IntrinsicLink::PrototypeToConstructor(_) => {
                count += 1
            }
        }
        index += 1;
    }
    count
}

const INTRINSIC_PROPERTY_DESCRIPTOR_COUNT: usize = intrinsic_property_descriptor_count();
const INTRINSIC_PROPERTY_PLACEHOLDER: IntrinsicPropertyDescriptor = IntrinsicPropertyDescriptor {
    owner: IntrinsicKind::ObjectConstructor,
    key: IntrinsicPropertyKey::String("name"),
    value: IntrinsicPropertyValue::Undefined,
    attributes: IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_FIXED,
};

const fn build_intrinsic_property_descriptors(
) -> [IntrinsicPropertyDescriptor; INTRINSIC_PROPERTY_DESCRIPTOR_COUNT] {
    let mut properties = [INTRINSIC_PROPERTY_PLACEHOLDER; INTRINSIC_PROPERTY_DESCRIPTOR_COUNT];
    let mut property_index = 0;
    let mut descriptor_index = 0;

    while descriptor_index < INTRINSIC_DESCRIPTORS.len() {
        let descriptor = INTRINSIC_DESCRIPTORS[descriptor_index];
        match descriptor.function {
            Some(function) => {
                properties[property_index] = function_name_descriptor(descriptor.kind, function);
                property_index += 1;
                properties[property_index] = function_length_descriptor(descriptor.kind, function);
                property_index += 1;
            }
            None => {}
        }
        match descriptor.link {
            IntrinsicLink::None => {}
            IntrinsicLink::ConstructorToPrototype(prototype) => {
                properties[property_index] =
                    constructor_prototype_descriptor(descriptor.kind, prototype);
                property_index += 1;
            }
            IntrinsicLink::PrototypeToConstructor(constructor) => {
                properties[property_index] =
                    prototype_constructor_descriptor(descriptor.kind, constructor);
                property_index += 1;
            }
        }
        descriptor_index += 1;
    }

    if property_index != INTRINSIC_PROPERTY_DESCRIPTOR_COUNT {
        panic!("intrinsic property descriptor count drifted");
    }
    properties
}

const fn role_is_constructor_side(role: IntrinsicRole) -> bool {
    match role {
        IntrinsicRole::Constructor | IntrinsicRole::Function => true,
        IntrinsicRole::Prototype => false,
    }
}

const fn role_is_prototype_side(role: IntrinsicRole) -> bool {
    match role {
        IntrinsicRole::Prototype => true,
        IntrinsicRole::Constructor | IntrinsicRole::Function => false,
    }
}

const fn validate_intrinsic_registry() {
    let mut index = 0;
    while index < INTRINSIC_DESCRIPTORS.len() {
        let descriptor = INTRINSIC_DESCRIPTORS[index];
        if descriptor.kind as usize != index {
            panic!("intrinsic kind and descriptor order differ");
        }

        match descriptor.role {
            IntrinsicRole::Constructor => match descriptor.function {
                Some(function) => {
                    if !function.constructable {
                        panic!("constructor intrinsic is not constructable");
                    }
                }
                None => panic!("constructor intrinsic is not callable"),
            },
            IntrinsicRole::Function => match descriptor.function {
                Some(function) => {
                    if function.constructable {
                        panic!("function intrinsic is unexpectedly constructable");
                    }
                }
                None => panic!("function intrinsic is not callable"),
            },
            IntrinsicRole::Prototype => match descriptor.function {
                Some(function) => {
                    if function.constructable {
                        panic!("prototype intrinsic is unexpectedly constructable");
                    }
                }
                None => {}
            },
        }

        match descriptor.prototype {
            Some(prototype) => {
                if prototype as usize == index {
                    panic!("intrinsic inherits from itself");
                }
            }
            None => {}
        }

        match descriptor.link {
            IntrinsicLink::None => {}
            IntrinsicLink::ConstructorToPrototype(prototype) => {
                if !role_is_constructor_side(descriptor.role) {
                    panic!("prototype property is owned by a prototype row");
                }
                let target = INTRINSIC_DESCRIPTORS[prototype as usize];
                if !role_is_prototype_side(target.role) {
                    panic!("prototype property does not target a prototype row");
                }
                match target.link {
                    IntrinsicLink::None => {
                        panic!("constructor-to-prototype link has no reciprocal link")
                    }
                    IntrinsicLink::ConstructorToPrototype(_) => {
                        panic!("constructor-to-prototype link has the wrong reciprocal kind")
                    }
                    IntrinsicLink::PrototypeToConstructor(constructor) => {
                        if constructor as usize != index {
                            panic!("constructor-to-prototype reciprocal target differs");
                        }
                    }
                }
            }
            IntrinsicLink::PrototypeToConstructor(constructor) => {
                if !role_is_prototype_side(descriptor.role) {
                    panic!("constructor property is not owned by a prototype row");
                }
                let target = INTRINSIC_DESCRIPTORS[constructor as usize];
                if !role_is_constructor_side(target.role) {
                    panic!("constructor property does not target a constructor/function row");
                }
                match target.link {
                    IntrinsicLink::None => {
                        panic!("prototype-to-constructor link has no reciprocal link")
                    }
                    IntrinsicLink::ConstructorToPrototype(prototype) => {
                        if prototype as usize != index {
                            panic!("prototype-to-constructor reciprocal target differs");
                        }
                    }
                    IntrinsicLink::PrototypeToConstructor(_) => {
                        panic!("prototype-to-constructor link has the wrong reciprocal kind")
                    }
                }
            }
        }
        index += 1;
    }
}

const _: [(); 23] = [(); IntrinsicKind::COUNT];
const _: [(); 46] = [(); INTRINSIC_PROPERTY_DESCRIPTOR_COUNT];
const _: () = validate_intrinsic_registry();

pub const INTRINSIC_PROPERTY_DESCRIPTORS: [IntrinsicPropertyDescriptor;
    INTRINSIC_PROPERTY_DESCRIPTOR_COUNT] = build_intrinsic_property_descriptors();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntrinsicId {
    realm_id: RealmId,
    kind: IntrinsicKind,
}

impl IntrinsicId {
    pub const fn realm_id(self) -> RealmId {
        self.realm_id
    }

    pub const fn kind(self) -> IntrinsicKind {
        self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmIntrinsics {
    realm_id: RealmId,
}

impl RealmIntrinsics {
    const fn new(realm_id: RealmId) -> Self {
        Self { realm_id }
    }

    pub const fn get(&self, kind: IntrinsicKind) -> IntrinsicId {
        IntrinsicId {
            realm_id: self.realm_id,
            kind,
        }
    }

    pub const fn descriptor(&self, kind: IntrinsicKind) -> &'static IntrinsicDescriptor {
        kind.descriptor()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RealmObjectKind {
    GlobalObject,
    GlobalThis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RealmObjectId {
    realm_id: RealmId,
    kind: RealmObjectKind,
}

impl RealmObjectId {
    pub const fn realm_id(self) -> RealmId {
        self.realm_id
    }

    pub const fn kind(self) -> RealmObjectKind {
        self.kind
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalEnvironmentId {
    realm_id: RealmId,
}

impl GlobalEnvironmentId {
    pub const fn realm_id(self) -> RealmId {
        self.realm_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealmGlobal {
    global_object: RealmObjectId,
    global_this: RealmObjectId,
    global_environment: GlobalEnvironmentId,
}

impl RealmGlobal {
    const fn new(realm_id: RealmId) -> Self {
        Self {
            global_object: RealmObjectId {
                realm_id,
                kind: RealmObjectKind::GlobalObject,
            },
            global_this: RealmObjectId {
                realm_id,
                kind: RealmObjectKind::GlobalThis,
            },
            global_environment: GlobalEnvironmentId { realm_id },
        }
    }

    pub const fn global_object(self) -> RealmObjectId {
        self.global_object
    }

    pub const fn global_this(self) -> RealmObjectId {
        self.global_this
    }

    pub const fn global_environment(self) -> GlobalEnvironmentId {
        self.global_environment
    }
}

/// A UTC Unix-epoch timestamp in whole milliseconds.
///
/// Construction enforces the ECMAScript time-value domain once, so host clocks
/// cannot feed Date or Temporal a finite timestamp outside their shared range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UtcEpochMilliseconds(i64);

impl UtcEpochMilliseconds {
    pub const MIN: i64 = -8_640_000_000_000_000;
    pub const MAX: i64 = 8_640_000_000_000_000;

    pub const fn new(milliseconds: i64) -> Option<Self> {
        if milliseconds >= Self::MIN && milliseconds <= Self::MAX {
            Some(Self(milliseconds))
        } else {
            None
        }
    }

    pub const fn get(self) -> i64 {
        self.0
    }

    fn from_system_time(time: std::time::SystemTime) -> Self {
        let (milliseconds, before_epoch) =
            match time.duration_since(std::time::SystemTime::UNIX_EPOCH) {
                Ok(duration) => (duration.as_millis(), false),
                Err(error) => (error.duration().as_millis(), true),
            };
        let milliseconds = i64::try_from(milliseconds).unwrap_or(i64::MAX);
        let milliseconds = if before_epoch {
            -milliseconds
        } else {
            milliseconds
        };
        Self(milliseconds.clamp(Self::MIN, Self::MAX))
    }
}

/// A reading from a monotonic clock's private epoch, in nanoseconds.
///
/// It is intentionally not interchangeable with [`UtcEpochMilliseconds`]. The
/// private epoch has no calendar meaning and is useful only for measuring an
/// elapsed duration against another reading from the same [`HostClock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicClockInstant(u64);

impl MonotonicClockInstant {
    pub const fn new(nanoseconds: u64) -> Self {
        Self(nanoseconds)
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn saturating_duration_since(self, earlier: Self) -> MonotonicClockDuration {
        MonotonicClockDuration(self.0.saturating_sub(earlier.0))
    }
}

/// A non-negative elapsed duration measured by a [`HostClock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MonotonicClockDuration(u64);

impl MonotonicClockDuration {
    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn as_i64_saturating(self) -> i64 {
        if self.0 > i64::MAX as u64 {
            i64::MAX
        } else {
            self.0 as i64
        }
    }

    pub fn as_milliseconds_f64(self) -> f64 {
        self.0 as f64 / 1_000_000.0
    }
}

/// Host-owned wall and monotonic clock services.
///
/// The two domains have distinct result types so a wall-clock timestamp cannot
/// accidentally become an Atomics timeout deadline or an agent monotonic
/// reading. Implementations must return nondecreasing monotonic instants.
pub trait HostClock: Send + Sync {
    fn utc_epoch_milliseconds(&self) -> UtcEpochMilliseconds;

    fn monotonic_instant(&self) -> MonotonicClockInstant;
}

/// Production clock backed by the operating system wall and monotonic clocks.
#[derive(Debug)]
pub struct SystemHostClock {
    monotonic_origin: std::time::Instant,
}

impl SystemHostClock {
    pub fn new() -> Self {
        Self {
            monotonic_origin: std::time::Instant::now(),
        }
    }
}

impl Default for SystemHostClock {
    fn default() -> Self {
        Self::new()
    }
}

impl HostClock for SystemHostClock {
    fn utc_epoch_milliseconds(&self) -> UtcEpochMilliseconds {
        UtcEpochMilliseconds::from_system_time(std::time::SystemTime::now())
    }

    fn monotonic_instant(&self) -> MonotonicClockInstant {
        MonotonicClockInstant::new(
            u64::try_from(self.monotonic_origin.elapsed().as_nanos()).unwrap_or(u64::MAX),
        )
    }
}

/// The complete result domain of one `Math.random` host read.
///
/// The field is private so an embedder cannot return NaN, an infinity, or the
/// upper endpoint and make the engine violate ECMAScript's `[0, 1)` contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RandomUnitInterval(f64);

impl RandomUnitInterval {
    pub fn new(value: f64) -> Option<Self> {
        if value.is_finite() && (0.0..1.0).contains(&value) {
            Some(Self(if value == 0.0 { 0.0 } else { value }))
        } else {
            None
        }
    }

    /// Maps every source word onto one of the `2^53` exactly representable
    /// binary64 fractions in `[0, 1)`.
    pub fn from_entropy_word(word: u64) -> Self {
        const DENOMINATOR: f64 = (1_u64 << 53) as f64;
        Self((word >> 11) as f64 / DENOMINATOR)
    }

    pub const fn get(self) -> f64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostRandomError {
    EntropyUnavailable,
}

impl core::fmt::Display for HostRandomError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EntropyUnavailable => formatter.write_str("host entropy is unavailable"),
        }
    }
}

impl std::error::Error for HostRandomError {}

/// Host-owned randomness for ECMAScript's `Math.random`.
pub trait HostRandom: Send + Sync {
    fn random_unit_interval(&self) -> Result<RandomUnitInterval, HostRandomError>;
}

/// Production randomness backed by the operating system entropy source.
#[derive(Debug, Default)]
pub struct SystemHostRandom;

impl HostRandom for SystemHostRandom {
    fn random_unit_interval(&self) -> Result<RandomUnitInterval, HostRandomError> {
        getrandom::u64()
            .map(RandomUnitInterval::from_entropy_word)
            .map_err(|_| HostRandomError::EntropyUnavailable)
    }
}

pub trait HostHooks: Send + Sync {
    fn shell_name(&self) -> &'static str {
        "lila-shell"
    }

    fn print_line(&self, _text: &str) {}
}

#[derive(Debug, Default)]
pub struct NullHostHooks;

impl HostHooks for NullHostHooks {}

#[derive(Clone)]
pub struct Realm {
    id: RealmId,
    agent_id: AgentId,
    intrinsics: RealmIntrinsics,
    global: RealmGlobal,
    pub shell_name: String,
    host_clock: Arc<dyn HostClock>,
    host_random: Arc<dyn HostRandom>,
    host_hooks: Arc<dyn HostHooks>,
}

impl core::fmt::Debug for Realm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Realm")
            .field("id", &self.id)
            .field("agent_id", &self.agent_id)
            .field("shell_name", &self.shell_name)
            .finish()
    }
}

pub struct RealmBuilder {
    agent_id: AgentId,
    host_clock: Box<dyn HostClock>,
    host_random: Box<dyn HostRandom>,
    host_hooks: Box<dyn HostHooks>,
}

impl Default for RealmBuilder {
    fn default() -> Self {
        Self {
            agent_id: AgentId::MAIN,
            host_clock: Box::<SystemHostClock>::default(),
            host_random: Box::<SystemHostRandom>::default(),
            host_hooks: Box::<NullHostHooks>::default(),
        }
    }
}

impl RealmBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_host_hooks(mut self, host_hooks: Box<dyn HostHooks>) -> Self {
        self.host_hooks = host_hooks;
        self
    }

    pub fn with_host_clock(mut self, host_clock: Box<dyn HostClock>) -> Self {
        self.host_clock = host_clock;
        self
    }

    pub fn with_host_random(mut self, host_random: Box<dyn HostRandom>) -> Self {
        self.host_random = host_random;
        self
    }

    pub fn with_agent_id(mut self, agent_id: AgentId) -> Self {
        self.agent_id = agent_id;
        self
    }

    pub fn host_hooks(&self) -> &dyn HostHooks {
        &*self.host_hooks
    }

    pub fn build(self) -> Realm {
        let id = RealmId(NEXT_REALM_ID.fetch_add(1, Ordering::Relaxed));
        Realm {
            id,
            agent_id: self.agent_id,
            intrinsics: RealmIntrinsics::new(id),
            global: RealmGlobal::new(id),
            shell_name: self.host_hooks.shell_name().to_string(),
            host_clock: Arc::from(self.host_clock),
            host_random: Arc::from(self.host_random),
            host_hooks: Arc::from(self.host_hooks),
        }
    }
}

impl Realm {
    pub const fn id(&self) -> RealmId {
        self.id
    }

    pub const fn agent_id(&self) -> AgentId {
        self.agent_id
    }

    pub const fn intrinsics(&self) -> &RealmIntrinsics {
        &self.intrinsics
    }

    pub const fn global(&self) -> RealmGlobal {
        self.global
    }

    pub fn host_hooks(&self) -> Arc<dyn HostHooks> {
        Arc::clone(&self.host_hooks)
    }

    pub fn host_clock(&self) -> &dyn HostClock {
        &*self.host_clock
    }

    pub fn host_random(&self) -> &dyn HostRandom {
        &*self.host_random
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct NamedHooks;

    impl HostHooks for NamedHooks {
        fn shell_name(&self) -> &'static str {
            "named-shell"
        }
    }

    #[derive(Debug)]
    struct DeterministicHostClock {
        next_monotonic_nanoseconds: AtomicU64,
    }

    impl HostClock for DeterministicHostClock {
        fn utc_epoch_milliseconds(&self) -> UtcEpochMilliseconds {
            UtcEpochMilliseconds::new(-1).expect("test wall clock is in range")
        }

        fn monotonic_instant(&self) -> MonotonicClockInstant {
            MonotonicClockInstant::new(
                self.next_monotonic_nanoseconds
                    .fetch_add(5, Ordering::Relaxed),
            )
        }
    }

    #[derive(Debug)]
    struct DeterministicHostRandom {
        next_word: AtomicU64,
        step: u64,
    }

    impl HostRandom for DeterministicHostRandom {
        fn random_unit_interval(&self) -> Result<RandomUnitInterval, HostRandomError> {
            Ok(RandomUnitInterval::from_entropy_word(
                self.next_word.fetch_add(self.step, Ordering::Relaxed),
            ))
        }
    }

    #[test]
    fn agent_host_operation_wire_domain_is_stable() {
        for (operation, wire) in [
            (AgentHostOperation::Start, 1),
            (AgentHostOperation::Broadcast, 2),
            (AgentHostOperation::ReceiveBroadcast, 3),
            (AgentHostOperation::Report, 4),
            (AgentHostOperation::ReportLength, 5),
            (AgentHostOperation::ReportCopy, 6),
            (AgentHostOperation::Sleep, 7),
            (AgentHostOperation::MonotonicNow, 8),
            (AgentHostOperation::Leaving, 9),
            (AgentHostOperation::RegisterAsyncWaiter, 10),
            (AgentHostOperation::PollAsyncWaiter, 11),
            (AgentHostOperation::NotifyAsyncWaiters, 12),
            (AgentHostOperation::CancelAsyncWaiter, 13),
        ] {
            assert_eq!(operation.wire(), wire);
            assert_eq!(AgentHostOperation::from_wire(wire), Some(operation));
        }

        for wire in [i64::MIN, -1, 0, 14, i64::MAX] {
            assert_eq!(AgentHostOperation::from_wire(wire), None);
        }
    }

    #[test]
    fn realm_builder_assigns_unique_realm_ids_in_one_agent() {
        let first = RealmBuilder::new().build();
        let second = RealmBuilder::new().build();

        assert_ne!(first.id(), second.id());
        assert_eq!(first.agent_id(), AgentId::MAIN);
        assert_eq!(second.agent_id(), AgentId::MAIN);
    }

    #[test]
    fn realm_builder_preserves_explicit_agent_and_host_hooks() {
        let realm = RealmBuilder::new()
            .with_agent_id(AgentId::new(7))
            .with_host_hooks(Box::new(NamedHooks))
            .build();

        assert_eq!(realm.agent_id(), AgentId::new(7));
        assert_eq!(realm.shell_name, "named-shell");
        assert_eq!(realm.host_hooks().shell_name(), "named-shell");
    }

    #[test]
    fn time_domains_validate_and_realm_clones_share_the_injected_clock() {
        assert_eq!(
            UtcEpochMilliseconds::new(UtcEpochMilliseconds::MIN).map(UtcEpochMilliseconds::get),
            Some(UtcEpochMilliseconds::MIN)
        );
        assert_eq!(
            UtcEpochMilliseconds::new(UtcEpochMilliseconds::MAX).map(UtcEpochMilliseconds::get),
            Some(UtcEpochMilliseconds::MAX)
        );
        assert_eq!(
            UtcEpochMilliseconds::new(UtcEpochMilliseconds::MIN - 1),
            None
        );
        assert_eq!(
            UtcEpochMilliseconds::new(UtcEpochMilliseconds::MAX + 1),
            None
        );

        let realm = RealmBuilder::new()
            .with_host_clock(Box::new(DeterministicHostClock {
                next_monotonic_nanoseconds: AtomicU64::new(100),
            }))
            .build();
        let cloned_realm = realm.clone();

        assert_eq!(realm.host_clock().utc_epoch_milliseconds().get(), -1);
        let first = realm.host_clock().monotonic_instant();
        let second = cloned_realm.host_clock().monotonic_instant();
        assert_eq!(first.get(), 100);
        assert_eq!(second.get(), 105);
        assert_eq!(second.saturating_duration_since(first).get(), 5);
        assert_eq!(first.saturating_duration_since(second).get(), 0);
        assert_eq!(
            MonotonicClockInstant::new(u64::MAX)
                .saturating_duration_since(MonotonicClockInstant::new(0))
                .as_i64_saturating(),
            i64::MAX
        );
    }

    #[test]
    fn random_domain_validates_and_realm_clones_share_the_injected_provider() {
        for invalid in [f64::NEG_INFINITY, -1.0, f64::NAN, 1.0, f64::INFINITY] {
            assert_eq!(RandomUnitInterval::new(invalid), None);
        }
        assert_eq!(
            RandomUnitInterval::new(-0.0).map(|value| value.get().to_bits()),
            Some(0)
        );
        assert_eq!(
            RandomUnitInterval::new(1.0 - f64::EPSILON).map(RandomUnitInterval::get),
            Some(1.0 - f64::EPSILON)
        );

        let realm = RealmBuilder::new()
            .with_host_random(Box::new(DeterministicHostRandom {
                next_word: AtomicU64::new(0),
                step: 1_u64 << 63,
            }))
            .build();
        let cloned_realm = realm.clone();

        assert_eq!(
            realm
                .host_random()
                .random_unit_interval()
                .expect("first deterministic random read")
                .get(),
            0.0
        );
        assert_eq!(
            cloned_realm
                .host_random()
                .random_unit_interval()
                .expect("second deterministic random read")
                .get(),
            0.5
        );
    }

    #[test]
    fn realm_intrinsic_identities_are_local_to_realm() {
        let first = RealmBuilder::new().build();
        let second = RealmBuilder::new().build();

        for kind in IntrinsicKind::ALL {
            let first_intrinsic = first.intrinsics().get(*kind);
            let second_intrinsic = second.intrinsics().get(*kind);
            assert_eq!(first_intrinsic.realm_id(), first.id());
            assert_eq!(first_intrinsic.kind(), *kind);
            assert_ne!(first_intrinsic, second_intrinsic);
        }
    }

    #[test]
    fn realm_global_identities_are_local_to_realm() {
        let first = RealmBuilder::new().build();
        let second = RealmBuilder::new().build();

        assert_eq!(first.global().global_object().realm_id(), first.id());
        assert_eq!(
            first.global().global_object().kind(),
            RealmObjectKind::GlobalObject
        );
        assert_eq!(first.global().global_this().realm_id(), first.id());
        assert_eq!(
            first.global().global_this().kind(),
            RealmObjectKind::GlobalThis
        );
        assert_eq!(first.global().global_environment().realm_id(), first.id());

        assert_ne!(
            first.global().global_object(),
            second.global().global_object()
        );
        assert_ne!(first.global().global_this(), second.global().global_this());
        assert_ne!(
            first.global().global_environment(),
            second.global().global_environment()
        );
    }

    #[test]
    fn intrinsic_registry_has_one_descriptor_per_kind() {
        assert_eq!(INTRINSIC_DESCRIPTORS.len(), IntrinsicKind::ALL.len());

        for kind in IntrinsicKind::ALL {
            let matches = INTRINSIC_DESCRIPTORS
                .iter()
                .filter(|descriptor| descriptor.kind == *kind)
                .count();
            assert_eq!(matches, 1, "{kind:?} should have exactly one descriptor");
            assert_eq!(kind.descriptor().kind, *kind);
        }
    }

    #[test]
    fn intrinsic_registry_references_resolve_inside_realm() {
        let realm = RealmBuilder::new().build();

        for descriptor in INTRINSIC_DESCRIPTORS {
            let intrinsic = realm.intrinsics().get(descriptor.kind);
            assert_eq!(intrinsic.realm_id(), realm.id());
            assert!(!descriptor.spec_name.is_empty());

            if let Some(prototype) = descriptor.prototype {
                let prototype = realm.intrinsics().get(prototype);
                assert_eq!(prototype.realm_id(), realm.id());
            }
        }
    }

    fn property(owner: IntrinsicKind, key: IntrinsicPropertyKey) -> IntrinsicPropertyDescriptor {
        *INTRINSIC_PROPERTY_DESCRIPTORS
            .iter()
            .find(|descriptor| descriptor.owner == owner && descriptor.key == key)
            .expect("intrinsic property descriptor should exist")
    }

    #[test]
    fn intrinsic_property_templates_have_unique_owner_keys() {
        for (index, descriptor) in INTRINSIC_PROPERTY_DESCRIPTORS.iter().enumerate() {
            assert!(
                IntrinsicKind::ALL.contains(&descriptor.owner),
                "{:?} owner should be an intrinsic",
                descriptor.owner
            );

            for other in &INTRINSIC_PROPERTY_DESCRIPTORS[index + 1..] {
                assert_ne!(
                    (descriptor.owner, descriptor.key),
                    (other.owner, other.key),
                    "{:?}.{:?} should have one descriptor",
                    descriptor.owner,
                    descriptor.key
                );
            }
        }
    }

    #[test]
    fn intrinsic_property_templates_reference_known_intrinsics() {
        let realm = RealmBuilder::new().build();

        for descriptor in INTRINSIC_PROPERTY_DESCRIPTORS {
            assert_eq!(
                realm.intrinsics().get(descriptor.owner).realm_id(),
                realm.id()
            );

            if let IntrinsicPropertyValue::Intrinsic(target) = descriptor.value {
                assert!(
                    IntrinsicKind::ALL.contains(&target),
                    "{target:?} should be a known intrinsic"
                );
                assert_eq!(realm.intrinsics().get(target).realm_id(), realm.id());
            }
        }
    }

    #[test]
    fn callable_intrinsics_have_name_and_length_templates() {
        for descriptor in INTRINSIC_DESCRIPTORS {
            let Some(function) = descriptor.function else {
                continue;
            };
            let expected_attributes = if function.length_name_configurable {
                IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_CONFIGURABLE
            } else {
                IntrinsicPropertyAttributes::BUILTIN_FUNCTION_LENGTH_NAME_FIXED
            };

            let name = property(descriptor.kind, IntrinsicPropertyKey::String("name"));
            assert_eq!(name.value, IntrinsicPropertyValue::String(function.name));
            assert_eq!(name.attributes, expected_attributes);

            let length = property(descriptor.kind, IntrinsicPropertyKey::String("length"));
            assert_eq!(length.value, IntrinsicPropertyValue::U32(function.length));
            assert_eq!(length.attributes, expected_attributes);
        }
    }

    #[test]
    fn typed_intrinsic_links_generate_property_templates() {
        for descriptor in INTRINSIC_DESCRIPTORS {
            match descriptor.link {
                IntrinsicLink::None => {}
                IntrinsicLink::ConstructorToPrototype(prototype) => {
                    let property =
                        property(descriptor.kind, IntrinsicPropertyKey::String("prototype"));
                    assert_eq!(property.value, IntrinsicPropertyValue::Intrinsic(prototype));
                    assert_eq!(
                        property.attributes,
                        IntrinsicPropertyAttributes::CONSTRUCTOR_PROTOTYPE
                    );
                }
                IntrinsicLink::PrototypeToConstructor(constructor) => {
                    let property =
                        property(descriptor.kind, IntrinsicPropertyKey::String("constructor"));
                    assert_eq!(
                        property.value,
                        IntrinsicPropertyValue::Intrinsic(constructor)
                    );
                    assert_eq!(
                        property.attributes,
                        IntrinsicPropertyAttributes::PROTOTYPE_CONSTRUCTOR
                    );
                }
            }
        }
    }

    #[test]
    fn proxy_intrinsic_does_not_declare_nonexistent_prototype_property() {
        assert!(INTRINSIC_PROPERTY_DESCRIPTORS
            .iter()
            .all(
                |descriptor| descriptor.owner != IntrinsicKind::ProxyConstructor
                    || descriptor.key != IntrinsicPropertyKey::String("prototype")
            ));
    }
}
