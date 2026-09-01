use std::collections::BTreeMap;

use super::{equal_map_intersection, BindingInfo};

#[derive(Clone, Default)]
pub(super) struct StaticStringBindingFacts {
    by_storage_name: BTreeMap<String, String>,
}

impl StaticStringBindingFacts {
    pub(super) fn get(&self, binding: &BindingInfo) -> Option<&String> {
        self.by_storage_name.get(&binding.storage_name)
    }

    pub(super) fn insert(&mut self, binding: &BindingInfo, value: String) {
        self.by_storage_name
            .insert(binding.storage_name.clone(), value);
    }

    pub(super) fn remove(&mut self, binding: &BindingInfo) {
        self.by_storage_name.remove(&binding.storage_name);
    }

    pub(super) fn clear(&mut self) {
        self.by_storage_name.clear();
    }

    pub(super) fn equal_intersection(left: &Self, right: &Self) -> Self {
        Self {
            by_storage_name: equal_map_intersection(&left.by_storage_name, &right.by_storage_name),
        }
    }
}
