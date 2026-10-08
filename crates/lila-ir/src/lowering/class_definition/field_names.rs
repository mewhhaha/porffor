use crate::ir::{ClassFieldKeyIr, ClassFieldNameIr, PrivateNameId};
use std::collections::BTreeMap;

pub(super) fn class_field_initializer_name(
    key: &ClassFieldKeyIr,
    private_names: &BTreeMap<String, PrivateNameId>,
) -> ClassFieldNameIr {
    match key {
        ClassFieldKeyIr::Public(name) => ClassFieldNameIr::Static(name.clone()),
        ClassFieldKeyIr::ComputedPublic(slot) => ClassFieldNameIr::Computed(*slot),
        ClassFieldKeyIr::Private(private_name_id) => ClassFieldNameIr::Static(
            private_names
                .iter()
                .find_map(|(name, id)| (*id == *private_name_id).then(|| format!("#{name}")))
                .expect("private field source name must remain visible"),
        ),
    }
}
