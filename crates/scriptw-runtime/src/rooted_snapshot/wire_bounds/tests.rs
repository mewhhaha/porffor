use super::*;
use std::cell::Cell;
use std::rc::Rc;

fn limits(units: u32) -> SnapshotLimits {
    SnapshotLimits::new(4, 4, 2, 4, units, 8, 1_000, 8).unwrap()
}

fn decode(json: &str, limits: SnapshotLimits) -> Result<GraphWire, serde_json::Error> {
    let mut decoder = serde_json::Deserializer::from_str(json);
    let mut budget = DecodeBudget::new(limits);
    GraphWire::deserialize(Bounded {
        inner: &mut decoder,
        budget: &mut budget,
        context: Context::Value(Role::Other),
        depth: 0,
    })
}

#[test]
fn decoder_aggregates_units_across_keys_and_values_before_graph_admission() {
    let limits = limits(2);
    let wire = serde_json::json!({
        "version":1,"limits":limits,"root":{"type":"object","id":0},"realm_count":1,"symbols":[],
        "nodes":[{"id":0,"kind":{"kind":"ordinary"},"extensible":true,"prototype":{"type":"null"},"anchors":[],
            "properties":[{"key":{"type":"string","units":[65]},"descriptor":{
                "kind":"data","value":{"type":"string","units":[66,67]},"writable":true,"enumerable":true,"configurable":true
            }}]}]
    });
    let error = decode(&wire.to_string(), limits).unwrap_err();
    assert!(error.to_string().contains("Utf16Units"));
    let mut fits = wire;
    fits["nodes"][0]["properties"][0]["descriptor"]["value"]["units"] = serde_json::json!([66]);
    let graph = validation::admit(decode(&fits.to_string(), limits).unwrap()).unwrap();
    assert_eq!(graph.nodes().len(), 1);
}

#[test]
fn decoder_aggregates_decimal_digits_across_distinct_properties() {
    let limits = limits(16);
    let mut properties = Vec::new();
    for key in [65, 66] {
        properties.push(serde_json::json!({"key":{"type":"string","units":[key]},"descriptor":{
            "kind":"data","value":{"type":"big_int","decimal":"12345"},"writable":true,"enumerable":true,"configurable":true
        }}));
    }
    let wire = serde_json::json!({
        "version":1,"limits":limits,"root":{"type":"object","id":0},"realm_count":1,"symbols":[],
        "nodes":[{"id":0,"kind":{"kind":"ordinary"},"extensible":true,"prototype":{"type":"null"},"anchors":[],"properties":properties}]
    });
    assert!(decode(&wire.to_string(), limits)
        .unwrap_err()
        .to_string()
        .contains("BigIntDigits"));
}

struct Sequence {
    remaining: usize,
    decoded: Rc<Cell<usize>>,
}
struct Scalar(Rc<Cell<usize>>);
impl<'de> Deserializer<'de> for Scalar {
    type Error = de::value::Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.set(self.0.get() + 1);
        visitor.visit_u16(65)
    }
    serde::forward_to_deserialize_any! { bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any }
}
impl<'de> Deserializer<'de> for Sequence {
    type Error = de::value::Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(self)
    }
    serde::forward_to_deserialize_any! { bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf option unit unit_struct newtype_struct seq tuple tuple_struct map struct enum identifier ignored_any }
}
impl<'de> SeqAccess<'de> for Sequence {
    type Error = de::value::Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        seed.deserialize(Scalar(Rc::clone(&self.decoded))).map(Some)
    }
    fn size_hint(&self) -> Option<usize> {
        Some(usize::MAX)
    }
}
struct Probe;
impl<'de> Visitor<'de> for Probe {
    type Value = Vec<u16>;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded sequence")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        assert_eq!(
            sequence.size_hint(),
            None,
            "an untrusted hint cannot reserve graph storage"
        );
        let mut output = Vec::new();
        while let Some(value) = sequence.next_element()? {
            output.push(value);
        }
        Ok(output)
    }
}

#[test]
fn decoder_rejects_before_decoding_an_over_budget_element_and_hides_allocation_hints() {
    let decoded = Rc::new(Cell::new(0));
    let mut budget = DecodeBudget::new(limits(2));
    let error = Bounded {
        inner: Sequence {
            remaining: 3,
            decoded: Rc::clone(&decoded),
        },
        budget: &mut budget,
        context: Context::Value(Role::Units),
        depth: 0,
    }
    .deserialize_seq(Probe)
    .unwrap_err();
    assert!(error.to_string().contains("Utf16Units"));
    assert_eq!(
        decoded.get(),
        2,
        "third value must not reach its allocating decoder"
    );
}

#[test]
fn streaming_outcome_accepts_late_tags_and_limits_but_rejects_foreign_payloads() {
    let limits = serde_json::to_string(&limits(2)).unwrap();
    let graph = format!(
        r#"{{"root":{{"units":[65],"type":"string"}},"nodes":[],"symbols":[],"realm_count":1,"version":1,"limits":{limits}}}"#
    );
    let wire = format!(r#"{{"graph":{graph},"kind":"captured"}}"#);
    let observed: SnapshotOutcome = serde_json::from_str(&wire).unwrap();
    let SnapshotOutcome::Captured { graph } = observed else {
        panic!("missing captured graph")
    };
    assert!(matches!(graph.root(), SnapshotValue::String { units } if units.units() == [65]));
    let original = serde_json::to_value(SnapshotOutcome::Captured { graph }).unwrap();
    let mut foreign = original.clone();
    foreign["reason"] = serde_json::Value::Null;
    assert!(serde_json::from_value::<SnapshotOutcome>(foreign).is_err());
    let mut rejected = original;
    rejected["kind"] = "rejected".into();
    rejected["reason"] = serde_json::json!({"kind":"budget_exceeded","dimension":"nodes"});
    assert!(serde_json::from_value::<SnapshotOutcome>(rejected).is_err());
}
