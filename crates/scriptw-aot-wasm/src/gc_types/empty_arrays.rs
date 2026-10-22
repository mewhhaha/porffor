//! Empty private storage can share identity: a zero-length array has no valid
//! mutable index. Consumers still allocate or replace storage before any write.
//! Globals are immutable, typed and owned by each instantiated runtime/Store.
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) struct EmptyArrayGlobal<T: GcArrayHeapType> {
    index: u32,
    target: PhantomData<fn() -> T>,
}

impl<T: GcArrayHeapType> Copy for EmptyArrayGlobal<T> {}
impl<T: GcArrayHeapType> Clone for EmptyArrayGlobal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: GcArrayHeapType> EmptyArrayGlobal<T> {
    fn declare(layouts: &GcLayoutRegistry, globals: &mut GlobalLedger) -> Self {
        let index = globals.len();
        let array_type_index = layouts.array_type::<T>().type_index().raw();
        globals.global(
            GlobalType {
                val_type: ValType::Ref(layouts.reference(T::LAYOUT, GcNullability::NonNullable)),
                mutable: false,
                shared: false,
            },
            &ConstExpr::extended([Instruction::ArrayNewFixed {
                array_type_index,
                array_size: 0,
            }]),
        );
        Self {
            index,
            target: PhantomData,
        }
    }

    pub(super) fn index(self) -> u32 {
        self.index
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct EmptyArrays {
    pub(super) values: EmptyArrayGlobal<ValueArray>,
    pub(super) properties: EmptyArrayGlobal<PropertyTable>,
    pub(super) property_index: EmptyArrayGlobal<OrdinaryPropertyIndex>,
    pub(super) private_elements: EmptyArrayGlobal<PrivateElementTable>,
}

impl EmptyArrays {
    pub(super) fn declare(layouts: &GcLayoutRegistry, globals: &mut GlobalLedger) -> Self {
        Self {
            values: EmptyArrayGlobal::declare(layouts, globals),
            properties: EmptyArrayGlobal::declare(layouts, globals),
            property_index: EmptyArrayGlobal::declare(layouts, globals),
            private_elements: EmptyArrayGlobal::declare(layouts, globals),
        }
    }
}
