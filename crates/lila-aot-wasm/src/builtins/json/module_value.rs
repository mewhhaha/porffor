//! Compile-time JSON trees use the original JSON data allocation operations.
use super::*;
use lila_ir::{encode_js_string_utf16, JsonModuleValueIr, JsonValue};

enum Work<'a> {
    Value(&'a JsonValue, usize),
    Object(&'a [(Vec<u16>, JsonValue)], usize, usize),
    Array(&'a [JsonValue], usize, usize),
    Property(usize, usize, &'a [u16]),
    Element(usize, usize, usize),
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_json_module_value(
        &mut self,
        plan: &JsonModuleValueIr,
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let realm = self.json_module_realm(plan, f)?;
        let s = self.runtime_schema();
        let mut roots = vec![Some(s.reserve_value_local(f))];
        let mut work = vec![Work::Value(plan.value(), 0)];
        while let Some(operation) = work.pop() {
            match operation {
                Work::Value(value, slot) => {
                    let out = roots[slot].as_ref().expect("live JSON value root");
                    match value {
                        JsonValue::Null => out.set_scalar(ScalarValue::Null, f),
                        JsonValue::Boolean(value) => {
                            out.set_scalar(ScalarValue::Boolean(*value), f)
                        }
                        JsonValue::Number(bits) => {
                            out.set_scalar(ScalarValue::NumberBits(*bits as i64), f)
                        }
                        JsonValue::String(units) => {
                            let string =
                                self.emit_json_string(&encode_js_string_utf16(units), f)?;
                            out.set_reference(&string, s, f);
                            string.clear(f);
                        }
                        JsonValue::Object(entries) => {
                            let object = self.emit_json_plain_object(&realm, f)?;
                            out.set_reference(&object, s, f);
                            object.clear(f);
                            work.push(Work::Object(entries, 0, slot));
                        }
                        JsonValue::Array(values) => {
                            let length = s.reserve_i64_local(f);
                            f.instruction(&Instruction::I64Const(values.len() as i64));
                            length.store(f);
                            let prototype = s.reserve_value_local(f);
                            let array_proto = s
                                .reserve_gc_local(f)
                                .initialize(self.emit_load_realm_array_prototype(&realm, f), f);
                            prototype.set_reference(&array_proto, s, f);
                            let array = s.reserve_gc_local(f).initialize(
                                self.emit_alloc_array_payload_with_length_and_prototype(
                                    length, &prototype, f,
                                )?,
                                f,
                            );
                            out.set_reference(&array, s, f);
                            array.clear(f);
                            array_proto.clear(f);
                            prototype.clear(f);
                            s.release_i64_local(length, f);
                            work.push(Work::Array(values, 0, slot));
                        }
                    }
                }
                Work::Object(entries, index, parent) => {
                    if let Some((key, value)) = entries.get(index) {
                        let child = roots.len();
                        roots.push(Some(s.reserve_value_local(f)));
                        work.push(Work::Object(entries, index + 1, parent));
                        work.push(Work::Property(parent, child, key));
                        work.push(Work::Value(value, child));
                    }
                }
                Work::Array(values, index, parent) => {
                    if let Some(value) = values.get(index) {
                        let child = roots.len();
                        roots.push(Some(s.reserve_value_local(f)));
                        work.push(Work::Array(values, index + 1, parent));
                        work.push(Work::Element(parent, child, index));
                        work.push(Work::Value(value, child));
                    }
                }
                Work::Property(parent, child, key) => {
                    let key = self.emit_json_string(&encode_js_string_utf16(key), f)?;
                    self.emit_json_define(
                        roots[parent].as_ref().unwrap(),
                        &key,
                        roots[child].as_ref().unwrap(),
                        false,
                        f,
                    )?;
                    key.clear(f);
                    roots[child].take().unwrap().clear(f);
                }
                Work::Element(parent, child, index) => {
                    let ordinal = s.reserve_i64_local(f);
                    f.instruction(&Instruction::I64Const(index as i64));
                    ordinal.store(f);
                    let key = self.emit_json_index_key(ordinal, f)?;
                    self.emit_json_define(
                        roots[parent].as_ref().unwrap(),
                        &key,
                        roots[child].as_ref().unwrap(),
                        false,
                        f,
                    )?;
                    key.clear(f);
                    s.release_i64_local(ordinal, f);
                    roots[child].take().unwrap().clear(f);
                }
            }
        }
        output.copy_from(roots[0].as_ref().unwrap(), f);
        roots[0].take().unwrap().clear(f);
        realm.clear(f);
        Ok(())
    }
}
