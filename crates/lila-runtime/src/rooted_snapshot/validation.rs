use std::collections::HashSet;

use super::*;

fn invalid(detail: &'static str) -> SnapshotRejection {
    SnapshotRejection::InvalidGraph {
        detail: detail.into(),
    }
}

pub(super) fn admit(graph: GraphWire) -> Result<RootedSnapshotGraph, SnapshotRejection> {
    if graph.version != 1 {
        return Err(invalid("unsupported snapshot version"));
    }
    let mut budget = SnapshotBudget::new(graph.limits);
    budget.charge(SnapshotBudgetDimension::Nodes, graph.nodes.len())?;
    budget.charge(SnapshotBudgetDimension::Symbols, graph.symbols.len())?;
    budget.charge(SnapshotBudgetDimension::Realms, graph.realm_count as usize)?;
    if graph.realm_count == 0 {
        return Err(invalid("missing entry Realm"));
    }
    let mut admission = Admission {
        graph: &graph,
        budget,
        objects: vec![false; graph.nodes.len()],
        symbols: vec![false; graph.symbols.len()],
        realms: vec![false; graph.realm_count as usize],
        depths: vec![0; graph.nodes.len()],
        next_object: 0,
        next_symbol: 0,
        next_realm: 1,
        anchors: HashSet::new(),
        registries: HashSet::new(),
        well_known: HashSet::new(),
    };
    admission.realms[0] = true;
    admission.value(&graph.root, 0)?;
    for (index, node) in graph.nodes.iter().enumerate() {
        if node.id as usize != index || !admission.objects[index] {
            return Err(invalid("unreachable or out-of-order node"));
        }
        admission.budget.work(1)?;
        let depth = admission.depths[index] + 1;
        if let SnapshotObjectKind::Function { realm, .. } = node.kind {
            admission.realm(realm)?;
        }
        if !matches!(
            node.prototype,
            SnapshotValue::Null {} | SnapshotValue::Object { .. }
        ) {
            return Err(invalid("prototype is neither Object nor null"));
        }
        admission.value(&node.prototype, depth)?;
        let mut previous_anchor = None;
        for anchor in &node.anchors {
            admission.budget.work(1)?;
            let order = (anchor.intrinsic, anchor.realm);
            if previous_anchor.is_some_and(|previous| previous >= order)
                || !admission.anchors.insert(*anchor)
            {
                return Err(invalid("duplicate or unordered intrinsic anchor"));
            }
            previous_anchor = Some(order);
            admission.realm(anchor.realm)?;
        }
        admission
            .budget
            .charge(SnapshotBudgetDimension::Properties, node.properties.len())?;
        let mut keys = HashSet::new();
        let mut key_phase = 0_u8;
        let mut previous_index = None;
        let mut array_length = None;
        let mut largest_index = None;
        for property in &node.properties {
            admission.budget.work(1)?;
            if !keys.insert(&property.key) {
                return Err(invalid("duplicate own property key"));
            }
            match &property.key {
                SnapshotKey::String { units } => {
                    admission.units(units)?;
                    match array_index(units.units()) {
                        Some(index) => {
                            if key_phase != 0
                                || previous_index.is_some_and(|previous| previous >= index)
                            {
                                return Err(invalid(
                                    "integer property keys are not in numeric order",
                                ));
                            }
                            previous_index = Some(index);
                            largest_index = Some(index);
                        }
                        None => {
                            if key_phase == 2 {
                                return Err(invalid("string key follows Symbol key"));
                            }
                            key_phase = 1;
                        }
                    }
                    if units.units() == [108, 101, 110, 103, 116, 104]
                        && node.kind == (SnapshotObjectKind::Array {})
                    {
                        array_length = Some(array_length_value(&property.descriptor)?);
                    }
                }
                SnapshotKey::Symbol { id } => {
                    key_phase = 2;
                    admission.symbol(*id)?;
                }
            }
            match &property.descriptor {
                SnapshotDescriptor::Data { value, .. } => admission.value(value, depth)?,
                SnapshotDescriptor::Accessor { get, set, .. } => {
                    for value in [get, set] {
                        match value {
                            SnapshotValue::Undefined {} => {}
                            SnapshotValue::Object { id } => {
                                if !graph.nodes.get(*id as usize).is_some_and(|node| {
                                    matches!(node.kind, SnapshotObjectKind::Function { .. })
                                }) {
                                    return Err(invalid("accessor value is not callable"));
                                }
                            }
                            SnapshotValue::Null {}
                            | SnapshotValue::Boolean { .. }
                            | SnapshotValue::Number { .. }
                            | SnapshotValue::String { .. }
                            | SnapshotValue::BigInt { .. }
                            | SnapshotValue::Symbol { .. } => {
                                return Err(invalid("accessor value is not callable or undefined"));
                            }
                        }
                        admission.value(value, depth)?;
                    }
                }
            }
        }
        if node.kind == (SnapshotObjectKind::Array {}) {
            let length = array_length.ok_or_else(|| invalid("Array has no own length"))?;
            if largest_index.is_some_and(|index| index >= length) {
                return Err(invalid("Array index is outside its length"));
            }
        }
    }
    if admission.next_object != graph.nodes.len()
        || admission.next_symbol != graph.symbols.len()
        || admission.next_realm != graph.realm_count as usize
    {
        return Err(invalid("snapshot contains unreachable identities"));
    }
    Ok(RootedSnapshotGraph(graph))
}

struct Admission<'a> {
    graph: &'a GraphWire,
    budget: SnapshotBudget,
    objects: Vec<bool>,
    symbols: Vec<bool>,
    realms: Vec<bool>,
    depths: Vec<u32>,
    next_object: usize,
    next_symbol: usize,
    next_realm: usize,
    anchors: HashSet<SnapshotAnchor>,
    registries: HashSet<&'a SnapshotString>,
    well_known: HashSet<SnapshotWellKnownSymbol>,
}

impl Admission<'_> {
    fn units(&mut self, units: &SnapshotString) -> Result<(), SnapshotRejection> {
        self.budget
            .charge(SnapshotBudgetDimension::Utf16Units, units.units().len())
    }
    fn realm(&mut self, id: u32) -> Result<(), SnapshotRejection> {
        self.budget.work(1)?;
        let seen = self
            .realms
            .get_mut(id as usize)
            .ok_or_else(|| invalid("dangling Realm"))?;
        if !*seen {
            if id as usize != self.next_realm {
                return Err(invalid("noncanonical Realm identity"));
            }
            *seen = true;
            self.next_realm += 1;
        }
        Ok(())
    }
    fn symbol(&mut self, id: u32) -> Result<(), SnapshotRejection> {
        self.budget.work(1)?;
        let seen = self
            .symbols
            .get_mut(id as usize)
            .ok_or_else(|| invalid("dangling Symbol"))?;
        if *seen {
            return Ok(());
        }
        if id as usize != self.next_symbol {
            return Err(invalid("noncanonical Symbol identity"));
        }
        *seen = true;
        self.next_symbol += 1;
        let symbol = &self.graph.symbols[id as usize];
        if symbol.id != id {
            return Err(invalid("out-of-order Symbol"));
        }
        if let Some(description) = &symbol.data.description {
            self.units(description)?;
        }
        match &symbol.data.origin {
            SnapshotSymbolOrigin::Local {} => {}
            SnapshotSymbolOrigin::Registry { key } => {
                self.units(key)?;
                if !self.registries.insert(key) {
                    return Err(invalid("registry Symbol has multiple identities"));
                }
            }
            SnapshotSymbolOrigin::WellKnown { name } => {
                if !self.well_known.insert(*name) {
                    return Err(invalid("well-known Symbol has multiple identities"));
                }
            }
        }
        Ok(())
    }
    fn value(&mut self, value: &SnapshotValue, depth: u32) -> Result<(), SnapshotRejection> {
        self.budget.work(1)?;
        match value {
            SnapshotValue::Undefined {}
            | SnapshotValue::Null {}
            | SnapshotValue::Boolean { .. } => Ok(()),
            SnapshotValue::Number { bits } => {
                if crate::ObservedNumber::from_bits(*bits).bits() == *bits {
                    Ok(())
                } else {
                    Err(invalid("noncanonical NaN"))
                }
            }
            SnapshotValue::String { units } => self.units(units),
            SnapshotValue::BigInt { decimal } => self.budget.charge(
                SnapshotBudgetDimension::BigIntDigits,
                decimal.decimal().len(),
            ),
            SnapshotValue::Symbol { id } => self.symbol(*id),
            SnapshotValue::Object { id } => {
                let seen = self
                    .objects
                    .get_mut(*id as usize)
                    .ok_or_else(|| invalid("dangling Object"))?;
                if !*seen {
                    if *id as usize != self.next_object {
                        return Err(invalid("noncanonical Object identity"));
                    }
                    self.budget.check_depth(depth)?;
                    *seen = true;
                    self.next_object += 1;
                    self.depths[*id as usize] = depth;
                }
                Ok(())
            }
        }
    }
}

fn array_length_value(descriptor: &SnapshotDescriptor) -> Result<u32, SnapshotRejection> {
    let SnapshotDescriptor::Data {
        value: SnapshotValue::Number { bits },
        enumerable: false,
        configurable: false,
        ..
    } = descriptor
    else {
        return Err(invalid("invalid Array length descriptor"));
    };
    let length = f64::from_bits(*bits);
    if length < 0.0
        || !length.is_finite()
        || length.fract() != 0.0
        || length > f64::from(u32::MAX)
        || *bits == (-0.0_f64).to_bits()
    {
        return Err(invalid("invalid Array length value"));
    }
    Ok(length as u32)
}

fn array_index(units: &[u16]) -> Option<u32> {
    if units.is_empty() || units.len() > 10 || (units.len() > 1 && units[0] == 48) {
        return None;
    }
    let mut value = 0_u32;
    for unit in units {
        let digit = unit.checked_sub(48).filter(|digit| *digit < 10)?;
        value = value.checked_mul(10)?.checked_add(u32::from(digit))?;
    }
    (value != u32::MAX).then_some(value)
}
