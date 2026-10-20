use super::*;

/// Runs only raw backend inspections while the original completion and every
/// discovered identity remain rooted. A rejection never becomes a partial graph.
pub fn capture_snapshot<B: SnapshotBackend>(
    backend: &mut B,
    value: &B::Value,
    limits: SnapshotLimits,
) -> SnapshotOutcome {
    match capture(backend, value, limits) {
        Ok(graph) => SnapshotOutcome::Captured { graph },
        Err(reason) => SnapshotOutcome::Rejected { reason },
    }
}

struct Capture<'a, B: SnapshotBackend> {
    backend: &'a mut B,
    budget: SnapshotBudget,
    objects: Vec<(B::Object, u32)>,
    symbols: Vec<B::Symbol>,
    realms: Vec<B::Realm>,
    nodes: Vec<SnapshotNode>,
    symbol_data: Vec<SnapshotSymbol>,
}

fn capture<B: SnapshotBackend>(
    backend: &mut B,
    value: &B::Value,
    limits: SnapshotLimits,
) -> Result<RootedSnapshotGraph, SnapshotRejection> {
    let entry = backend.entry_realm();
    let mut capture = Capture {
        backend,
        budget: SnapshotBudget::new(limits),
        objects: Vec::new(),
        symbols: Vec::new(),
        realms: vec![entry],
        nodes: Vec::new(),
        symbol_data: Vec::new(),
    };
    capture.budget.charge(SnapshotBudgetDimension::Realms, 1)?;
    let root = capture.value(value, 0)?;
    while capture.nodes.len() < capture.objects.len() {
        let id = capture.nodes.len();
        let (object, depth) = capture.objects[id].clone();
        capture.budget.work(1)?;
        let data = capture.backend.object(&object, &mut capture.budget)?;
        let kind = match data.kind {
            SnapshotRawObjectKind::Ordinary => SnapshotObjectKind::Ordinary {},
            SnapshotRawObjectKind::Array => SnapshotObjectKind::Array {},
            SnapshotRawObjectKind::Function {
                constructable,
                realm,
            } => SnapshotObjectKind::Function {
                constructable,
                realm: capture.realm(realm)?,
            },
        };
        let prototype = capture.value(&data.prototype, depth + 1)?;
        let mut raw_anchors = data.anchors;
        capture.budget.work(raw_anchors.len())?;
        // Backend inventories need not enumerate labels in the same order.
        raw_anchors.sort_by_key(|anchor| anchor.intrinsic);
        let mut anchors = Vec::with_capacity(raw_anchors.len());
        for anchor in raw_anchors {
            anchors.push(SnapshotAnchor {
                realm: capture.realm(anchor.realm)?,
                intrinsic: anchor.intrinsic,
            });
        }
        let mut properties = Vec::with_capacity(data.properties.0.len());
        for property in data.properties.0 {
            capture.budget.work(1)?;
            let key = match property.key {
                SnapshotRawKey::String(units) => SnapshotKey::String { units },
                SnapshotRawKey::Symbol(symbol) => SnapshotKey::Symbol {
                    id: capture.symbol(symbol)?,
                },
            };
            let descriptor = match property.descriptor {
                SnapshotRawDescriptor::Data {
                    value,
                    writable,
                    enumerable,
                    configurable,
                } => SnapshotDescriptor::Data {
                    value: capture.value(&value, depth + 1)?,
                    writable,
                    enumerable,
                    configurable,
                },
                SnapshotRawDescriptor::Accessor {
                    get,
                    set,
                    enumerable,
                    configurable,
                } => SnapshotDescriptor::Accessor {
                    get: capture.value(&get, depth + 1)?,
                    set: capture.value(&set, depth + 1)?,
                    enumerable,
                    configurable,
                },
            };
            properties.push(SnapshotProperty { key, descriptor });
        }
        capture.nodes.push(SnapshotNode {
            id: id as u32,
            kind,
            extensible: data.extensible,
            prototype,
            anchors,
            properties,
        });
    }
    validation::admit(GraphWire {
        version: 1,
        limits,
        root,
        nodes: capture.nodes,
        symbols: capture.symbol_data,
        realm_count: capture.realms.len() as u32,
    })
}

impl<B: SnapshotBackend> Capture<'_, B> {
    fn value(&mut self, value: &B::Value, depth: u32) -> Result<SnapshotValue, SnapshotRejection> {
        self.budget.work(1)?;
        Ok(match self.backend.value(value, &mut self.budget)? {
            SnapshotRawValue::Undefined => SnapshotValue::Undefined {},
            SnapshotRawValue::Null => SnapshotValue::Null {},
            SnapshotRawValue::Boolean(value) => SnapshotValue::Boolean { value },
            SnapshotRawValue::Number(bits) => SnapshotValue::Number {
                bits: crate::ObservedNumber::from_bits(bits).bits(),
            },
            SnapshotRawValue::String(units) => SnapshotValue::String { units },
            SnapshotRawValue::BigInt(decimal) => SnapshotValue::BigInt { decimal },
            SnapshotRawValue::Object(object) => SnapshotValue::Object {
                id: self.object(object, depth)?,
            },
            SnapshotRawValue::Symbol(symbol) => SnapshotValue::Symbol {
                id: self.symbol(symbol)?,
            },
        })
    }

    fn object(&mut self, object: B::Object, depth: u32) -> Result<u32, SnapshotRejection> {
        for (index, (known, _)) in self.objects.iter().enumerate() {
            self.budget.work(1)?;
            if self.backend.same_object(known, &object)? {
                return Ok(index as u32);
            }
        }
        self.budget.check_depth(depth)?;
        self.budget.charge(SnapshotBudgetDimension::Nodes, 1)?;
        let id = self.objects.len() as u32;
        self.objects.push((object, depth));
        Ok(id)
    }

    fn symbol(&mut self, symbol: B::Symbol) -> Result<u32, SnapshotRejection> {
        for (index, known) in self.symbols.iter().enumerate() {
            self.budget.work(1)?;
            if self.backend.same_symbol(known, &symbol)? {
                return Ok(index as u32);
            }
        }
        self.budget.charge(SnapshotBudgetDimension::Symbols, 1)?;
        let data = self.backend.symbol(&symbol, &mut self.budget)?;
        let id = self.symbols.len() as u32;
        self.symbols.push(symbol);
        self.symbol_data.push(SnapshotSymbol { id, data });
        Ok(id)
    }

    fn realm(&mut self, realm: B::Realm) -> Result<u32, SnapshotRejection> {
        for (index, known) in self.realms.iter().enumerate() {
            self.budget.work(1)?;
            if self.backend.same_realm(known, &realm)? {
                return Ok(index as u32);
            }
        }
        self.budget.charge(SnapshotBudgetDimension::Realms, 1)?;
        let id = self.realms.len() as u32;
        self.realms.push(realm);
        Ok(id)
    }
}
