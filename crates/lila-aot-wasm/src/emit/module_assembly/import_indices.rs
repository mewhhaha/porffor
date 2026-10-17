//! Optional host imports claim consecutive function indices in declaration order.
pub(super) fn claim_optional_host_import_function_index(next: &mut u32) -> u32 {
    let index = *next;
    *next += 1;
    index
}
