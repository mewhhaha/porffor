use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

static NEXT_REALM_ID: AtomicU64 = AtomicU64::new(1);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IntrinsicKind {
    ObjectConstructor,
    ObjectPrototype,
    FunctionConstructor,
    FunctionPrototype,
    ArrayConstructor,
    ArrayPrototype,
    BigIntConstructor,
    BigIntPrototype,
    DateConstructor,
    DatePrototype,
    ProxyConstructor,
    ArrayBufferConstructor,
    ArrayBufferPrototype,
    DataViewConstructor,
    DataViewPrototype,
    TypedArrayConstructor,
    TypedArrayPrototype,
    Uint8ArrayConstructor,
    Uint8ArrayPrototype,
    TypeErrorConstructor,
    TypeErrorPrototype,
    IteratorPrototype,
    ThrowTypeError,
}

impl IntrinsicKind {
    pub const ALL: &'static [Self] = &[
        Self::ObjectConstructor,
        Self::ObjectPrototype,
        Self::FunctionConstructor,
        Self::FunctionPrototype,
        Self::ArrayConstructor,
        Self::ArrayPrototype,
        Self::BigIntConstructor,
        Self::BigIntPrototype,
        Self::DateConstructor,
        Self::DatePrototype,
        Self::ProxyConstructor,
        Self::ArrayBufferConstructor,
        Self::ArrayBufferPrototype,
        Self::DataViewConstructor,
        Self::DataViewPrototype,
        Self::TypedArrayConstructor,
        Self::TypedArrayPrototype,
        Self::Uint8ArrayConstructor,
        Self::Uint8ArrayPrototype,
        Self::TypeErrorConstructor,
        Self::TypeErrorPrototype,
        Self::IteratorPrototype,
        Self::ThrowTypeError,
    ];

    pub const fn descriptor(self) -> &'static IntrinsicDescriptor {
        match self {
            Self::ObjectConstructor => &INTRINSIC_DESCRIPTORS[0],
            Self::ObjectPrototype => &INTRINSIC_DESCRIPTORS[1],
            Self::FunctionConstructor => &INTRINSIC_DESCRIPTORS[2],
            Self::FunctionPrototype => &INTRINSIC_DESCRIPTORS[3],
            Self::ArrayConstructor => &INTRINSIC_DESCRIPTORS[4],
            Self::ArrayPrototype => &INTRINSIC_DESCRIPTORS[5],
            Self::BigIntConstructor => &INTRINSIC_DESCRIPTORS[6],
            Self::BigIntPrototype => &INTRINSIC_DESCRIPTORS[7],
            Self::DateConstructor => &INTRINSIC_DESCRIPTORS[8],
            Self::DatePrototype => &INTRINSIC_DESCRIPTORS[9],
            Self::ProxyConstructor => &INTRINSIC_DESCRIPTORS[10],
            Self::ArrayBufferConstructor => &INTRINSIC_DESCRIPTORS[11],
            Self::ArrayBufferPrototype => &INTRINSIC_DESCRIPTORS[12],
            Self::DataViewConstructor => &INTRINSIC_DESCRIPTORS[13],
            Self::DataViewPrototype => &INTRINSIC_DESCRIPTORS[14],
            Self::TypedArrayConstructor => &INTRINSIC_DESCRIPTORS[15],
            Self::TypedArrayPrototype => &INTRINSIC_DESCRIPTORS[16],
            Self::Uint8ArrayConstructor => &INTRINSIC_DESCRIPTORS[17],
            Self::Uint8ArrayPrototype => &INTRINSIC_DESCRIPTORS[18],
            Self::TypeErrorConstructor => &INTRINSIC_DESCRIPTORS[19],
            Self::TypeErrorPrototype => &INTRINSIC_DESCRIPTORS[20],
            Self::IteratorPrototype => &INTRINSIC_DESCRIPTORS[21],
            Self::ThrowTypeError => &INTRINSIC_DESCRIPTORS[22],
        }
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
}

impl IntrinsicDescriptor {
    pub const fn is_callable(&self) -> bool {
        self.function.is_some()
    }
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

pub const INTRINSIC_DESCRIPTORS: [IntrinsicDescriptor; 23] = [
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ObjectPrototype,
        spec_name: "%Object.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: None,
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayPrototype,
        spec_name: "%Array.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::BigIntPrototype,
        spec_name: "%BigInt.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DatePrototype,
        spec_name: "%Date.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::ArrayBufferPrototype,
        spec_name: "%ArrayBuffer.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::DataViewPrototype,
        spec_name: "%DataView.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypedArrayPrototype,
        spec_name: "%TypedArray.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::Uint8ArrayPrototype,
        spec_name: "%Uint8Array.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::TypedArrayPrototype),
        function: None,
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
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::TypeErrorPrototype,
        spec_name: "%TypeError.prototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
    },
    IntrinsicDescriptor {
        kind: IntrinsicKind::IteratorPrototype,
        spec_name: "%IteratorPrototype%",
        role: IntrinsicRole::Prototype,
        prototype: Some(IntrinsicKind::ObjectPrototype),
        function: None,
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
    },
];

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

pub const INTRINSIC_PROPERTY_DESCRIPTORS: [IntrinsicPropertyDescriptor; 46] = [
    function_name_descriptor(
        IntrinsicKind::ObjectConstructor,
        IntrinsicFunctionMetadata {
            name: "Object",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::ObjectConstructor,
        IntrinsicFunctionMetadata {
            name: "Object",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::ObjectConstructor,
        IntrinsicKind::ObjectPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::ObjectPrototype,
        IntrinsicKind::ObjectConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::FunctionConstructor,
        IntrinsicFunctionMetadata {
            name: "Function",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::FunctionConstructor,
        IntrinsicFunctionMetadata {
            name: "Function",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::FunctionConstructor,
        IntrinsicKind::FunctionPrototype,
    ),
    function_name_descriptor(
        IntrinsicKind::FunctionPrototype,
        IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: true,
            constructable: false,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::FunctionPrototype,
        IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: true,
            constructable: false,
        },
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::FunctionPrototype,
        IntrinsicKind::FunctionConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::ArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "Array",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::ArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "Array",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::ArrayConstructor,
        IntrinsicKind::ArrayPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::ArrayPrototype,
        IntrinsicKind::ArrayConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::BigIntConstructor,
        IntrinsicFunctionMetadata {
            name: "BigInt",
            length: 1,
            length_name_configurable: true,
            constructable: false,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::BigIntConstructor,
        IntrinsicFunctionMetadata {
            name: "BigInt",
            length: 1,
            length_name_configurable: true,
            constructable: false,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::BigIntConstructor,
        IntrinsicKind::BigIntPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::BigIntPrototype,
        IntrinsicKind::BigIntConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::DateConstructor,
        IntrinsicFunctionMetadata {
            name: "Date",
            length: 7,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::DateConstructor,
        IntrinsicFunctionMetadata {
            name: "Date",
            length: 7,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(IntrinsicKind::DateConstructor, IntrinsicKind::DatePrototype),
    prototype_constructor_descriptor(IntrinsicKind::DatePrototype, IntrinsicKind::DateConstructor),
    function_name_descriptor(
        IntrinsicKind::ProxyConstructor,
        IntrinsicFunctionMetadata {
            name: "Proxy",
            length: 2,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::ProxyConstructor,
        IntrinsicFunctionMetadata {
            name: "Proxy",
            length: 2,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_name_descriptor(
        IntrinsicKind::ArrayBufferConstructor,
        IntrinsicFunctionMetadata {
            name: "ArrayBuffer",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::ArrayBufferConstructor,
        IntrinsicFunctionMetadata {
            name: "ArrayBuffer",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::ArrayBufferConstructor,
        IntrinsicKind::ArrayBufferPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::ArrayBufferPrototype,
        IntrinsicKind::ArrayBufferConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::DataViewConstructor,
        IntrinsicFunctionMetadata {
            name: "DataView",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::DataViewConstructor,
        IntrinsicFunctionMetadata {
            name: "DataView",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::DataViewConstructor,
        IntrinsicKind::DataViewPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::DataViewPrototype,
        IntrinsicKind::DataViewConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::TypedArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "TypedArray",
            length: 0,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::TypedArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "TypedArray",
            length: 0,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::TypedArrayConstructor,
        IntrinsicKind::TypedArrayPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::TypedArrayPrototype,
        IntrinsicKind::TypedArrayConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::Uint8ArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "Uint8Array",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::Uint8ArrayConstructor,
        IntrinsicFunctionMetadata {
            name: "Uint8Array",
            length: 3,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::Uint8ArrayConstructor,
        IntrinsicKind::Uint8ArrayPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::Uint8ArrayPrototype,
        IntrinsicKind::Uint8ArrayConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::TypeErrorConstructor,
        IntrinsicFunctionMetadata {
            name: "TypeError",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::TypeErrorConstructor,
        IntrinsicFunctionMetadata {
            name: "TypeError",
            length: 1,
            length_name_configurable: true,
            constructable: true,
        },
    ),
    constructor_prototype_descriptor(
        IntrinsicKind::TypeErrorConstructor,
        IntrinsicKind::TypeErrorPrototype,
    ),
    prototype_constructor_descriptor(
        IntrinsicKind::TypeErrorPrototype,
        IntrinsicKind::TypeErrorConstructor,
    ),
    function_name_descriptor(
        IntrinsicKind::ThrowTypeError,
        IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: false,
            constructable: false,
        },
    ),
    function_length_descriptor(
        IntrinsicKind::ThrowTypeError,
        IntrinsicFunctionMetadata {
            name: "",
            length: 0,
            length_name_configurable: false,
            constructable: false,
        },
    ),
];

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

pub trait HostHooks: Send + Sync {
    fn shell_name(&self) -> &'static str {
        "porffor-shell"
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
    host_hooks: Box<dyn HostHooks>,
}

impl Default for RealmBuilder {
    fn default() -> Self {
        Self {
            agent_id: AgentId::MAIN,
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
    fn constructor_and_prototype_links_are_declared_as_property_templates() {
        for constructor in INTRINSIC_DESCRIPTORS
            .iter()
            .filter(|descriptor| descriptor.role == IntrinsicRole::Constructor)
        {
            let Some(prototype_kind) =
                constructor
                    .spec_name
                    .strip_prefix('%')
                    .and_then(|_| match constructor.kind {
                        IntrinsicKind::ObjectConstructor => Some(IntrinsicKind::ObjectPrototype),
                        IntrinsicKind::FunctionConstructor => {
                            Some(IntrinsicKind::FunctionPrototype)
                        }
                        IntrinsicKind::ArrayConstructor => Some(IntrinsicKind::ArrayPrototype),
                        IntrinsicKind::DateConstructor => Some(IntrinsicKind::DatePrototype),
                        IntrinsicKind::ArrayBufferConstructor => {
                            Some(IntrinsicKind::ArrayBufferPrototype)
                        }
                        IntrinsicKind::DataViewConstructor => {
                            Some(IntrinsicKind::DataViewPrototype)
                        }
                        IntrinsicKind::TypedArrayConstructor => {
                            Some(IntrinsicKind::TypedArrayPrototype)
                        }
                        IntrinsicKind::Uint8ArrayConstructor => {
                            Some(IntrinsicKind::Uint8ArrayPrototype)
                        }
                        IntrinsicKind::TypeErrorConstructor => {
                            Some(IntrinsicKind::TypeErrorPrototype)
                        }
                        _ => None,
                    })
            else {
                continue;
            };
            let prototype = property(constructor.kind, IntrinsicPropertyKey::String("prototype"));
            assert_eq!(
                prototype.value,
                IntrinsicPropertyValue::Intrinsic(prototype_kind)
            );
            assert_eq!(
                prototype.attributes,
                IntrinsicPropertyAttributes::CONSTRUCTOR_PROTOTYPE
            );

            let back_link = property(prototype_kind, IntrinsicPropertyKey::String("constructor"));
            assert_eq!(
                back_link.value,
                IntrinsicPropertyValue::Intrinsic(constructor.kind)
            );
            assert_eq!(
                back_link.attributes,
                IntrinsicPropertyAttributes::PROTOTYPE_CONSTRUCTOR
            );
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
