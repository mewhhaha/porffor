use super::*;
mod map;
mod set;

#[derive(Clone, Copy)]
pub(super) enum MapOperation {
    Get,
    Has,
    Delete,
    Set,
    GetOrInsert,
    GetOrInsertComputed,
}
#[derive(Clone, Copy)]
pub(super) enum SetOperation {
    Add,
    Has,
    Delete,
}
