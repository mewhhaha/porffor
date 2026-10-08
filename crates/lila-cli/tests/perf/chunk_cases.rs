// The existing twenty-case corpus, in its original warmup/measurement order.
// Each literal owns its filename and exact required fixture bytes. Opt-in
// timing gates and retained reports consume the same compiled workload.
macro_rules! fixtures {
    ($($name:literal),+ $(,)?) => {
        pub(super) const CASES: [&str; 20] = [$($name),+];
        pub(super) const SOURCES: [&str; 20] = [
            $(include_str!(concat!("../fixtures/", $name))),+
        ];
    };
}

fixtures!(
    "wasm_var.js",
    "wasm_functions.js",
    "wasm_closures.js",
    "wasm_exceptions.js",
    "wasm_array_map_core.js",
    "wasm_array_filter_core.js",
    "wasm_string_slice_core.js",
    "wasm_string_pad_start_core.js",
    "wasm_proxy_apply.js",
    "wasm_proxy_construct.js",
    "wasm_regexp_escape_core.js",
    "wasm_regexp_symbol_match_all_last_index.js",
    "wasm_typedarray_accessors.js",
    "wasm_dataview_resizable_boundaries.js",
    "wasm_arraybuffer_isview_core.js",
    "wasm_atomics_add_load_mutation.js",
    "wasm_iterator_prototype_map.js",
    "wasm_date_core_time_values.js",
    "wasm_exponentiation_bigint_core.js",
    "wasm_host_output.js",
);
