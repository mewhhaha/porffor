//! Bounded, hook-free observations of rooted completion values. Backends expose
//! their actual heap through `SnapshotBackend`; this module alone assigns IDs
//! and traversal order. A graph is an observation, not function equivalence.

use serde::{Deserialize, Deserializer, Serialize};

mod capture;
mod validation;
mod wire_bounds;
pub use capture::capture_snapshot;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SnapshotLimits {
    pub(crate) nodes: u32,
    pub(crate) symbols: u32,
    pub(crate) realms: u32,
    pub(crate) properties: u32,
    pub(crate) utf16_units: u32,
    pub(crate) bigint_digits: u32,
    pub(crate) work: u32,
    pub(crate) depth: u32,
}

impl SnapshotLimits {
    pub const HARD_MAX: Self = Self {
        nodes: 4096,
        symbols: 2048,
        realms: 64,
        properties: 32768,
        utf16_units: 262144,
        bigint_digits: 65536,
        work: 2_000_000,
        depth: 512,
    };

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        nodes: u32,
        symbols: u32,
        realms: u32,
        properties: u32,
        utf16_units: u32,
        bigint_digits: u32,
        work: u32,
        depth: u32,
    ) -> Result<Self, SnapshotRejection> {
        let limits = Self {
            nodes,
            symbols,
            realms,
            properties,
            utf16_units,
            bigint_digits,
            work,
            depth,
        };
        for dimension in SnapshotBudgetDimension::ALL {
            if limits.get(dimension) == 0 || limits.get(dimension) > Self::HARD_MAX.get(dimension) {
                return Err(SnapshotRejection::InvalidGraph {
                    detail: format!("invalid {dimension:?} limit"),
                });
            }
        }
        Ok(limits)
    }

    pub const fn get(self, dimension: SnapshotBudgetDimension) -> u32 {
        match dimension {
            SnapshotBudgetDimension::Nodes => self.nodes,
            SnapshotBudgetDimension::Symbols => self.symbols,
            SnapshotBudgetDimension::Realms => self.realms,
            SnapshotBudgetDimension::Properties => self.properties,
            SnapshotBudgetDimension::Utf16Units => self.utf16_units,
            SnapshotBudgetDimension::BigIntDigits => self.bigint_digits,
            SnapshotBudgetDimension::Work => self.work,
            SnapshotBudgetDimension::Depth => self.depth,
        }
    }
}

impl Default for SnapshotLimits {
    fn default() -> Self {
        Self {
            nodes: 512,
            symbols: 256,
            realms: 16,
            properties: 4096,
            utf16_units: 32768,
            bigint_digits: 4096,
            work: 262144,
            depth: 64,
        }
    }
}

impl<'de> Deserialize<'de> for SnapshotLimits {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            nodes: u32,
            symbols: u32,
            realms: u32,
            properties: u32,
            utf16_units: u32,
            bigint_digits: u32,
            work: u32,
            depth: u32,
        }
        let w = Wire::deserialize(deserializer)?;
        Self::new(
            w.nodes,
            w.symbols,
            w.realms,
            w.properties,
            w.utf16_units,
            w.bigint_digits,
            w.work,
            w.depth,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotBudgetDimension {
    Nodes,
    Symbols,
    Realms,
    Properties,
    Utf16Units,
    BigIntDigits,
    Work,
    Depth,
}

impl SnapshotBudgetDimension {
    const ALL: [Self; 8] = [
        Self::Nodes,
        Self::Symbols,
        Self::Realms,
        Self::Properties,
        Self::Utf16Units,
        Self::BigIntDigits,
        Self::Work,
        Self::Depth,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotExotic {
    Proxy,
    Arguments,
    BoundFunction,
    GeneratorFunction,
    AsyncFunction,
    Buffer,
    TypedArray,
    Promise,
    Map,
    Set,
    WeakCollection,
    Temporal,
    ModuleNamespace,
    BoxedPrimitive,
    Date,
    RegExp,
    Iterator,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotRejection {
    BudgetExceeded { dimension: SnapshotBudgetDimension },
    UnsupportedExotic { exotic: SnapshotExotic },
    InvalidGraph { detail: String },
    BackendInvariant { detail: String },
}

impl core::fmt::Display for SnapshotRejection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for SnapshotRejection {}

/// All dynamic copies must be charged before allocation. The property-list
/// constructor similarly makes an unchecked property vector unrepresentable.
pub struct SnapshotBudget {
    limits: SnapshotLimits,
    used: [u32; 8],
}

impl SnapshotBudget {
    pub fn new(limits: SnapshotLimits) -> Self {
        Self {
            limits,
            used: [0; 8],
        }
    }
    pub const fn limits(&self) -> SnapshotLimits {
        self.limits
    }
    pub fn remaining(&self, dimension: SnapshotBudgetDimension) -> u32 {
        self.limits
            .get(dimension)
            .saturating_sub(self.used[dimension as usize])
    }
    pub fn charge(
        &mut self,
        dimension: SnapshotBudgetDimension,
        count: usize,
    ) -> Result<(), SnapshotRejection> {
        let count =
            u32::try_from(count).map_err(|_| SnapshotRejection::BudgetExceeded { dimension })?;
        let used = self.used[dimension as usize]
            .checked_add(count)
            .filter(|used| *used <= self.limits.get(dimension))
            .ok_or(SnapshotRejection::BudgetExceeded { dimension })?;
        self.used[dimension as usize] = used;
        Ok(())
    }
    pub fn work(&mut self, count: usize) -> Result<(), SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::Work, count)
    }
    pub fn check_depth(&self, depth: u32) -> Result<(), SnapshotRejection> {
        if depth > self.limits.depth {
            Err(SnapshotRejection::BudgetExceeded {
                dimension: SnapshotBudgetDimension::Depth,
            })
        } else {
            Ok(())
        }
    }
    pub fn copy_utf16(&mut self, units: &[u16]) -> Result<SnapshotString, SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::Utf16Units, units.len())?;
        Ok(SnapshotString(units.to_vec()))
    }
    pub fn collect_utf16<I: ExactSizeIterator<Item = u16>>(
        &mut self,
        units: I,
    ) -> Result<SnapshotString, SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::Utf16Units, units.len())?;
        Ok(SnapshotString(units.collect()))
    }
    pub fn copy_bigint(&mut self, decimal: &str) -> Result<SnapshotBigInt, SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::BigIntDigits, decimal.len())?;
        SnapshotBigInt::checked(decimal.to_owned())
    }
    /// The producer must bound its conversion before constructing this String.
    pub fn take_bigint(&mut self, decimal: String) -> Result<SnapshotBigInt, SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::BigIntDigits, decimal.len())?;
        SnapshotBigInt::checked(decimal)
    }
    pub fn read_utf16(
        &mut self,
        length: usize,
        mut read: impl FnMut(usize) -> Result<u16, SnapshotRejection>,
    ) -> Result<SnapshotString, SnapshotRejection> {
        self.charge(SnapshotBudgetDimension::Utf16Units, length)?;
        self.work(length)?;
        let mut units = Vec::with_capacity(length);
        for index in 0..length {
            units.push(read(index)?);
        }
        Ok(SnapshotString(units))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SnapshotString(Vec<u16>);
impl SnapshotString {
    pub fn units(&self) -> &[u16] {
        &self.0
    }
}
impl<'de> Deserialize<'de> for SnapshotString {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        wire_bounds::units(deserializer).map(Self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct SnapshotBigInt(String);
impl SnapshotBigInt {
    fn checked(decimal: String) -> Result<Self, SnapshotRejection> {
        crate::ObservedBigInt::parse_canonical_decimal(decimal.clone().into_boxed_str()).map_err(
            |_| SnapshotRejection::InvalidGraph {
                detail: "noncanonical BigInt".into(),
            },
        )?;
        Ok(Self(decimal))
    }
    pub fn decimal(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for SnapshotBigInt {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::checked(wire_bounds::decimal(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotCompletionKind {
    Normal,
    Throw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotOutcome {
    Captured { graph: RootedSnapshotGraph },
    Rejected { reason: SnapshotRejection },
}

impl<'de> Deserialize<'de> for SnapshotOutcome {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        wire_bounds::outcome(deserializer)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotCompletion {
    pub kind: SnapshotCompletionKind,
    pub outcome: SnapshotOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotValue {
    Undefined {},
    Null {},
    Boolean { value: bool },
    Number { bits: u64 },
    String { units: SnapshotString },
    BigInt { decimal: SnapshotBigInt },
    Object { id: u32 },
    Symbol { id: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotKey {
    String { units: SnapshotString },
    Symbol { id: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotDescriptor {
    Data {
        value: SnapshotValue,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: SnapshotValue,
        set: SnapshotValue,
        enumerable: bool,
        configurable: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotProperty {
    pub key: SnapshotKey,
    pub descriptor: SnapshotDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotObjectKind {
    Ordinary {},
    Array {},
    Function { constructable: bool, realm: u32 },
}

// Each adapter consumes ALL and an exhaustive label mapping. Adding a variant
// cannot leave a second manually maintained inventory list behind.
macro_rules! snapshot_labels {
    ($(#[$attribute:meta])* pub enum $name:ident { $($variant:ident),+ $(,)? }) => {
        $(#[$attribute])*
        pub enum $name { $($variant),+ }
        impl $name { pub const ALL: &'static [Self] = &[$(Self::$variant),+]; }
    };
}

snapshot_labels! {
    /// Deliberately closed anchor coverage. Labels are derived from retained Realm
    /// tables, never from mutable `constructor`, names or prototype guessing.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum SnapshotIntrinsic {
    ObjectConstructor, ObjectPrototype, FunctionConstructor, FunctionPrototype,
    ArrayConstructor, ArrayPrototype, ErrorConstructor, ErrorPrototype,
    EvalErrorConstructor, EvalErrorPrototype, RangeErrorConstructor, RangeErrorPrototype,
    ReferenceErrorConstructor, ReferenceErrorPrototype, SyntaxErrorConstructor, SyntaxErrorPrototype,
    TypeErrorConstructor, TypeErrorPrototype, UriErrorConstructor, UriErrorPrototype,
    AggregateErrorConstructor, AggregateErrorPrototype,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotAnchor {
    pub realm: u32,
    pub intrinsic: SnapshotIntrinsic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotNode {
    pub id: u32,
    pub kind: SnapshotObjectKind,
    pub extensible: bool,
    pub prototype: SnapshotValue,
    pub anchors: Vec<SnapshotAnchor>,
    pub properties: Vec<SnapshotProperty>,
}

snapshot_labels! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum SnapshotWellKnownSymbol { AsyncIterator, HasInstance, IsConcatSpreadable, Iterator,
    Match, MatchAll, Replace, Search, Species, Split, ToPrimitive, ToStringTag, Unscopables,
    Dispose, AsyncDispose }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapshotSymbolOrigin {
    Local {},
    Registry { key: SnapshotString },
    WellKnown { name: SnapshotWellKnownSymbol },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSymbolData {
    pub description: Option<SnapshotString>,
    pub origin: SnapshotSymbolOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotSymbol {
    pub id: u32,
    pub data: SnapshotSymbolData,
}

/// Realm zero is always the retained entry Realm. Other IDs are assigned on
/// their first function-realm or intrinsic-anchor encounter, independently of
/// backend allocation/creation order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct RootedSnapshotGraph(GraphWire);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GraphWire {
    version: u32,
    limits: SnapshotLimits,
    root: SnapshotValue,
    nodes: Vec<SnapshotNode>,
    symbols: Vec<SnapshotSymbol>,
    realm_count: u32,
}

impl RootedSnapshotGraph {
    pub fn limits(&self) -> SnapshotLimits {
        self.0.limits
    }
    pub fn root(&self) -> &SnapshotValue {
        &self.0.root
    }
    pub fn nodes(&self) -> &[SnapshotNode] {
        &self.0.nodes
    }
    pub fn symbols(&self) -> &[SnapshotSymbol] {
        &self.0.symbols
    }
    pub fn realm_count(&self) -> u32 {
        self.0.realm_count
    }
}

impl<'de> Deserialize<'de> for RootedSnapshotGraph {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        wire_bounds::graph(deserializer)
    }
}

pub enum SnapshotRawValue<O, S> {
    Undefined,
    Null,
    Boolean(bool),
    Number(u64),
    String(SnapshotString),
    BigInt(SnapshotBigInt),
    Object(O),
    Symbol(S),
}
pub enum SnapshotRawKey<S> {
    String(SnapshotString),
    Symbol(S),
}
pub enum SnapshotRawDescriptor<V> {
    Data {
        value: V,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: V,
        set: V,
        enumerable: bool,
        configurable: bool,
    },
}
pub struct SnapshotRawProperty<V, S> {
    pub key: SnapshotRawKey<S>,
    pub descriptor: SnapshotRawDescriptor<V>,
}

pub struct SnapshotProperties<V, S>(Vec<SnapshotRawProperty<V, S>>);
impl<V, S> Default for SnapshotProperties<V, S> {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl<V, S> SnapshotProperties<V, S> {
    pub fn push(
        &mut self,
        budget: &mut SnapshotBudget,
        property: SnapshotRawProperty<V, S>,
    ) -> Result<(), SnapshotRejection> {
        budget.charge(SnapshotBudgetDimension::Properties, 1)?;
        self.0.push(property);
        Ok(())
    }
}

pub enum SnapshotRawObjectKind<R> {
    Ordinary,
    Array,
    Function { constructable: bool, realm: R },
}
pub struct SnapshotRawAnchor<R> {
    pub realm: R,
    pub intrinsic: SnapshotIntrinsic,
}
pub struct SnapshotObjectData<V, S, R> {
    pub kind: SnapshotRawObjectKind<R>,
    pub extensible: bool,
    pub prototype: V,
    pub anchors: Vec<SnapshotRawAnchor<R>>,
    pub properties: SnapshotProperties<V, S>,
}

/// Adapters must never invoke JS operations or hooks. Values and identities stay
/// rooted throughout capture. Charge `work` before scans, UTF16/BigInt before
/// copies/conversion, and use `SnapshotProperties::push` for every own property.
/// Return actual integer-index order, then string insertion order, then Symbol
/// insertion order. Include virtual Array length and raw complete descriptors.
pub trait SnapshotBackend {
    type Value: Clone;
    type Object: Clone;
    type Symbol: Clone;
    type Realm: Clone;
    fn entry_realm(&self) -> Self::Realm;
    fn same_object(
        &mut self,
        left: &Self::Object,
        right: &Self::Object,
    ) -> Result<bool, SnapshotRejection>;
    fn same_symbol(
        &mut self,
        left: &Self::Symbol,
        right: &Self::Symbol,
    ) -> Result<bool, SnapshotRejection>;
    fn same_realm(
        &mut self,
        left: &Self::Realm,
        right: &Self::Realm,
    ) -> Result<bool, SnapshotRejection>;
    fn value(
        &mut self,
        value: &Self::Value,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotRawValue<Self::Object, Self::Symbol>, SnapshotRejection>;
    fn symbol(
        &mut self,
        symbol: &Self::Symbol,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotSymbolData, SnapshotRejection>;
    fn object(
        &mut self,
        object: &Self::Object,
        budget: &mut SnapshotBudget,
    ) -> Result<SnapshotObjectData<Self::Value, Self::Symbol, Self::Realm>, SnapshotRejection>;
}
