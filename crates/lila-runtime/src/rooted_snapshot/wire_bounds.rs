//! Bound graph retention while serde is reading, including internally tagged
//! enum content buffers. The budget is local to this decode and field order is
//! irrelevant. Buffers already owned by an external deserializer are not ours.

use super::*;
use serde::de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use std::fmt;

const MAX_WIRE_DEPTH: u16 = 64;
const MAX_LABEL_BYTES: usize = 256;

pub(super) fn graph<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<RootedSnapshotGraph, D::Error> {
    let mut budget = DecodeBudget::new(SnapshotLimits::HARD_MAX);
    let wire = GraphWire::deserialize(Bounded {
        inner: deserializer,
        budget: &mut budget,
        context: Context::Value(Role::Other),
        depth: 0,
    })?;
    validation::admit(wire).map_err(de::Error::custom)
}

/// A streaming outer envelope is required: deriving an internally tagged enum
/// here would first buffer the entire graph before its bounded decoder ran.
pub(super) fn outcome<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<SnapshotOutcome, D::Error> {
    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    enum Kind {
        Captured,
        Rejected,
    }
    enum Field<T> {
        Absent,
        Present(T),
    }
    impl<T> Default for Field<T> {
        fn default() -> Self {
            Self::Absent
        }
    }
    impl<'de, T: Deserialize<'de>> Deserialize<'de> for Field<T> {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            T::deserialize(deserializer).map(Self::Present)
        }
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Wire {
        kind: Kind,
        #[serde(default)]
        graph: Field<RootedSnapshotGraph>,
        #[serde(default)]
        reason: Field<SnapshotRejection>,
    }
    match Wire::deserialize(deserializer)? {
        Wire {
            kind: Kind::Captured,
            graph: Field::Present(graph),
            reason: Field::Absent,
        } => Ok(SnapshotOutcome::Captured { graph }),
        Wire {
            kind: Kind::Rejected,
            reason: Field::Present(reason),
            graph: Field::Absent,
        } => Ok(SnapshotOutcome::Rejected { reason }),
        _ => Err(de::Error::custom(
            "snapshot outcome has missing or foreign payload fields",
        )),
    }
}

#[derive(Clone, Copy)]
enum Role {
    Other,
    Nodes,
    Symbols,
    Properties,
    Units,
    Decimal,
    Anchors,
}
impl Role {
    fn field(name: &str) -> Self {
        match name {
            "nodes" => Self::Nodes,
            "symbols" => Self::Symbols,
            "properties" => Self::Properties,
            "anchors" => Self::Anchors,
            "units" | "description" | "key" => Self::Units,
            "decimal" => Self::Decimal,
            _ => Self::Other,
        }
    }
}
enum Context<'a> {
    Value(Role),
    Key(&'a mut Role),
}
impl Context<'_> {
    fn role(&self) -> Role {
        match self {
            Self::Value(role) => *role,
            Self::Key(_) => Role::Other,
        }
    }
    fn key(&mut self, name: &str) {
        if let Self::Key(role) = self {
            **role = Role::field(name);
        }
    }
}

struct DecodeBudget {
    budget: SnapshotBudget,
    anchors: usize,
}
impl DecodeBudget {
    fn new(limits: SnapshotLimits) -> Self {
        Self {
            budget: SnapshotBudget::new(limits),
            anchors: 0,
        }
    }
    fn work<E: de::Error>(&mut self) -> Result<(), E> {
        self.budget.work(1).map_err(E::custom)
    }
    fn element<E: de::Error>(&mut self, role: Role) -> Result<(), E> {
        let dimension = match role {
            Role::Nodes => SnapshotBudgetDimension::Nodes,
            Role::Symbols => SnapshotBudgetDimension::Symbols,
            Role::Properties => SnapshotBudgetDimension::Properties,
            Role::Units => SnapshotBudgetDimension::Utf16Units,
            Role::Anchors => {
                let maximum = self.budget.limits().get(SnapshotBudgetDimension::Realms) as usize
                    * SnapshotIntrinsic::ALL.len();
                if self.anchors >= maximum {
                    return Err(E::custom("snapshot intrinsic anchor ceiling exceeded"));
                }
                self.anchors += 1;
                return Ok(());
            }
            Role::Other | Role::Decimal => {
                return Err(E::custom("sequence outside the snapshot wire domain"))
            }
        };
        self.budget.charge(dimension, 1).map_err(E::custom)
    }
    fn text<E: de::Error>(&mut self, role: Role, bytes: usize) -> Result<(), E> {
        if matches!(role, Role::Decimal) {
            self.budget
                .charge(SnapshotBudgetDimension::BigIntDigits, bytes)
                .map_err(E::custom)
        } else if bytes <= MAX_LABEL_BYTES {
            Ok(())
        } else {
            Err(E::custom("snapshot wire label ceiling exceeded"))
        }
    }
}

struct Bounded<'a, D> {
    inner: D,
    budget: &'a mut DecodeBudget,
    context: Context<'a>,
    depth: u16,
}
struct BoundVisitor<'a, V> {
    inner: V,
    budget: &'a mut DecodeBudget,
    context: Context<'a>,
    depth: u16,
    fields: Option<&'static [&'static str]>,
}

macro_rules! decode_methods {
    ($($method:ident $(($($argument:ident: $type:ty),+))?),+ $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, $($($argument: $type,)+)? visitor: V) -> Result<V::Value, Self::Error> {
            self.budget.work::<D::Error>()?;
            if self.depth > MAX_WIRE_DEPTH { return Err(de::Error::custom("snapshot wire nesting ceiling exceeded")); }
            self.inner.$method($($($argument,)+)? BoundVisitor {
                inner: visitor, budget: self.budget, context: self.context, depth: self.depth, fields: None,
            })
        }
    )+};
}

impl<'de, D: Deserializer<'de>> Deserializer<'de> for Bounded<'_, D> {
    type Error = D::Error;
    decode_methods!(deserialize_any, deserialize_bool, deserialize_i8, deserialize_i16,
        deserialize_i32, deserialize_i64, deserialize_i128, deserialize_u8, deserialize_u16,
        deserialize_u32, deserialize_u64, deserialize_u128, deserialize_f32, deserialize_f64,
        deserialize_char, deserialize_str, deserialize_string, deserialize_bytes, deserialize_byte_buf,
        deserialize_option, deserialize_unit, deserialize_unit_struct(name: &'static str),
        deserialize_newtype_struct(name: &'static str), deserialize_seq,
        deserialize_tuple(len: usize), deserialize_tuple_struct(name: &'static str, len: usize),
        deserialize_map, deserialize_enum(name: &'static str, variants: &'static [&'static str]),
        deserialize_identifier, deserialize_ignored_any);
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.budget.work::<D::Error>()?;
        if self.depth > MAX_WIRE_DEPTH {
            return Err(de::Error::custom("snapshot wire nesting ceiling exceeded"));
        }
        self.inner.deserialize_struct(
            name,
            fields,
            BoundVisitor {
                inner: visitor,
                budget: self.budget,
                context: self.context,
                depth: self.depth,
                fields: Some(fields),
            },
        )
    }
    fn is_human_readable(&self) -> bool {
        self.inner.is_human_readable()
    }
}

macro_rules! primitive_visitors {
    ($($method:ident($type:ty)),+ $(,)?) => {$(
        fn $method<E: de::Error>(self, value: $type) -> Result<Self::Value, E> { self.inner.$method(value) }
    )+};
}
impl<'de, V: Visitor<'de>> Visitor<'de> for BoundVisitor<'_, V> {
    type Value = V::Value;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.expecting(formatter)
    }
    primitive_visitors!(
        visit_bool(bool),
        visit_i8(i8),
        visit_i16(i16),
        visit_i32(i32),
        visit_i64(i64),
        visit_i128(i128),
        visit_u8(u8),
        visit_u16(u16),
        visit_u32(u32),
        visit_u64(u64),
        visit_u128(u128),
        visit_f32(f32),
        visit_f64(f64),
        visit_char(char)
    );
    fn visit_str<E: de::Error>(mut self, value: &str) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.context.key(value);
        self.inner.visit_str(value)
    }
    fn visit_borrowed_str<E: de::Error>(mut self, value: &'de str) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.context.key(value);
        self.inner.visit_borrowed_str(value)
    }
    fn visit_string<E: de::Error>(mut self, value: String) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.context.key(&value);
        self.inner.visit_string(value)
    }
    fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.inner.visit_bytes(value)
    }
    fn visit_borrowed_bytes<E: de::Error>(self, value: &'de [u8]) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.inner.visit_borrowed_bytes(value)
    }
    fn visit_byte_buf<E: de::Error>(self, value: Vec<u8>) -> Result<Self::Value, E> {
        self.budget.text::<E>(self.context.role(), value.len())?;
        self.inner.visit_byte_buf(value)
    }
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_none()
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        self.inner.visit_unit()
    }
    fn visit_some<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_some(Bounded {
            inner,
            budget: self.budget,
            context: self.context,
            depth: self.depth + 1,
        })
    }
    fn visit_newtype_struct<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        self.inner.visit_newtype_struct(Bounded {
            inner,
            budget: self.budget,
            context: self.context,
            depth: self.depth + 1,
        })
    }
    fn visit_seq<A: SeqAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_seq(BoundSeq {
            inner,
            budget: self.budget,
            role: self.context.role(),
            fields: self.fields,
            index: 0,
            depth: self.depth + 1,
        })
    }
    fn visit_map<A: MapAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_map(BoundMap {
            inner,
            budget: self.budget,
            pending: Role::Other,
            depth: self.depth + 1,
        })
    }
    fn visit_enum<A: EnumAccess<'de>>(self, inner: A) -> Result<Self::Value, A::Error> {
        self.inner.visit_enum(BoundEnum {
            inner,
            budget: self.budget,
            depth: self.depth + 1,
        })
    }
}

struct BoundSeed<'a, T> {
    inner: T,
    budget: &'a mut DecodeBudget,
    context: Context<'a>,
    element: Option<Role>,
    depth: u16,
}
impl<'de, T: DeserializeSeed<'de>> DeserializeSeed<'de> for BoundSeed<'_, T> {
    type Value = T::Value;
    fn deserialize<D: Deserializer<'de>>(self, inner: D) -> Result<Self::Value, D::Error> {
        if let Some(role) = self.element {
            self.budget.element::<D::Error>(role)?;
        }
        self.inner.deserialize(Bounded {
            inner,
            budget: self.budget,
            context: self.context,
            depth: self.depth,
        })
    }
}
struct BoundSeq<'a, A> {
    inner: A,
    budget: &'a mut DecodeBudget,
    role: Role,
    fields: Option<&'static [&'static str]>,
    index: usize,
    depth: u16,
}
impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for BoundSeq<'_, A> {
    type Error = A::Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        let (role, element) = match self.fields {
            Some(fields) => (
                fields
                    .get(self.index)
                    .map_or(Role::Other, |field| Role::field(field)),
                None,
            ),
            None => (Role::Other, Some(self.role)),
        };
        let value = self.inner.next_element_seed(BoundSeed {
            inner: seed,
            budget: self.budget,
            context: Context::Value(role),
            element,
            depth: self.depth,
        })?;
        self.index += usize::from(value.is_some());
        Ok(value)
    }
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
struct BoundMap<'a, A> {
    inner: A,
    budget: &'a mut DecodeBudget,
    pending: Role,
    depth: u16,
}
impl<'de, A: MapAccess<'de>> MapAccess<'de> for BoundMap<'_, A> {
    type Error = A::Error;
    fn next_key_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.inner.next_key_seed(BoundSeed {
            inner: seed,
            budget: self.budget,
            context: Context::Key(&mut self.pending),
            element: None,
            depth: self.depth,
        })
    }
    fn next_value_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        let role = self.pending;
        self.pending = Role::Other;
        self.inner.next_value_seed(BoundSeed {
            inner: seed,
            budget: self.budget,
            context: Context::Value(role),
            element: None,
            depth: self.depth,
        })
    }
    fn size_hint(&self) -> Option<usize> {
        None
    }
}
struct BoundEnum<'a, A> {
    inner: A,
    budget: &'a mut DecodeBudget,
    depth: u16,
}
impl<'de, 'a, A: EnumAccess<'de>> EnumAccess<'de> for BoundEnum<'a, A> {
    type Error = A::Error;
    type Variant = BoundVariant<'a, A::Variant>;
    fn variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<(T::Value, Self::Variant), Self::Error> {
        let (value, variant) = self.inner.variant_seed(BoundSeed {
            inner: seed,
            budget: &mut *self.budget,
            context: Context::Value(Role::Other),
            element: None,
            depth: self.depth,
        })?;
        Ok((
            value,
            BoundVariant {
                inner: variant,
                budget: self.budget,
                depth: self.depth,
            },
        ))
    }
}

pub(super) fn units<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u16>, D::Error> {
    struct Units;
    impl<'de> Visitor<'de> for Units {
        type Value = Vec<u16>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("bounded UTF-16 code units")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
            let mut units = Vec::new();
            while let Some(unit) = sequence.next_element()? {
                if units.len()
                    >= SnapshotLimits::HARD_MAX.get(SnapshotBudgetDimension::Utf16Units) as usize
                {
                    return Err(de::Error::custom("snapshot UTF-16 ceiling exceeded"));
                }
                units.push(unit);
            }
            Ok(units)
        }
    }
    deserializer.deserialize_seq(Units)
}

pub(super) fn decimal<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    struct Decimal;
    impl Visitor<'_> for Decimal {
        type Value = String;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded canonical decimal BigInt")
        }
        fn visit_str<E: de::Error>(self, value: &str) -> Result<String, E> {
            if value.len()
                > SnapshotLimits::HARD_MAX.get(SnapshotBudgetDimension::BigIntDigits) as usize
            {
                return Err(E::custom("snapshot BigInt ceiling exceeded"));
            }
            Ok(value.to_owned())
        }
        fn visit_string<E: de::Error>(self, value: String) -> Result<String, E> {
            if value.len()
                > SnapshotLimits::HARD_MAX.get(SnapshotBudgetDimension::BigIntDigits) as usize
            {
                return Err(E::custom("snapshot BigInt ceiling exceeded"));
            }
            Ok(value)
        }
    }
    deserializer.deserialize_str(Decimal)
}
struct BoundVariant<'a, A> {
    inner: A,
    budget: &'a mut DecodeBudget,
    depth: u16,
}
impl<'de, A: VariantAccess<'de>> VariantAccess<'de> for BoundVariant<'_, A> {
    type Error = A::Error;
    fn unit_variant(self) -> Result<(), Self::Error> {
        self.inner.unit_variant()
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(
        self,
        seed: T,
    ) -> Result<T::Value, Self::Error> {
        self.inner.newtype_variant_seed(BoundSeed {
            inner: seed,
            budget: self.budget,
            context: Context::Value(Role::Other),
            element: None,
            depth: self.depth,
        })
    }
    fn tuple_variant<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.inner.tuple_variant(
            len,
            BoundVisitor {
                inner: visitor,
                budget: self.budget,
                context: Context::Value(Role::Other),
                depth: self.depth,
                fields: None,
            },
        )
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.inner.struct_variant(
            fields,
            BoundVisitor {
                inner: visitor,
                budget: self.budget,
                context: Context::Value(Role::Other),
                depth: self.depth,
                fields: Some(fields),
            },
        )
    }
}

#[cfg(test)]
mod tests;
