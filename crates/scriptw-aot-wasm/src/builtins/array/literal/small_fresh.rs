//! Existing-layout publication proved by private, bounded literal construction.
use super::*;
use crate::gc_types::{
    ArrayIndexBucketTable, ArrayIndexEntry, ArrayIndexStorage, ArrayIndexStorageSchema,
    ArrayObjectSchema, RuntimeSchema,
};

#[must_use = "fresh indexed construction must be retired with its literal owner"]
pub(super) struct SmallFreshArrayStorage {
    storage: GcLocal<ArrayIndexStorage>,
    buckets: GcLocal<ArrayIndexBucketTable>,
    published: u32,
    last_ordinal: Option<u32>,
}

impl SmallFreshArrayStorage {
    pub(super) fn new(
        present_count: u32,
        array: &GcLocal<ArrayObject>,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) -> Option<Self> {
        let capacity = ArrayIndexStorageSchema::INITIAL_BUCKET_CAPACITY;
        if present_count == 0 || u64::from(present_count) * 4 > u64::from(capacity) * 3 {
            return None;
        }
        let storage = schema.reserve_gc_local(f).initialize(
            schema
                .field(ArrayObjectSchema::ELEMENTS)
                .read(array, schema, f)
                .reference(),
            f,
        );
        let buckets = schema.reserve_gc_local(f).initialize(
            schema
                .field(ArrayIndexStorageSchema::BUCKETS)
                .read(&storage, schema, f)
                .reference(),
            f,
        );
        Some(Self {
            storage,
            buckets,
            published: 0,
            last_ordinal: None,
        })
    }

    pub(super) fn publish(
        &mut self,
        ordinal: u32,
        descriptor: &GcLocal<PropertyDescriptor>,
        builder: &FunctionBuilder<'_>,
        f: &mut Function,
    ) {
        let capacity = ArrayIndexStorageSchema::INITIAL_BUCKET_CAPACITY;
        // Compiler-owned proof: every ordinal is unique and no insertion can
        // reach the existing table's resize threshold. No JS sees this Array
        // before FreshArrayLiteral::finish consumes its construction owner.
        assert!(self.last_ordinal.is_none_or(|previous| previous < ordinal));
        assert!(u64::from(self.published + 1) * 4 <= u64::from(capacity) * 3);
        assert!(ordinal < u32::MAX);
        self.published += 1;
        self.last_ordinal = Some(ordinal);
        let schema = builder.runtime_schema();
        let bucket = schema.reserve_i32_local(f);
        bucket.set_constant(
            FunctionBuilder::known_array_index_bucket(ordinal, capacity) as i32,
            f,
        );
        let head = schema.reserve_gc_local(f).initialize(
            schema
                .array_type::<ArrayIndexBucketTable>()
                .read(&self.buckets, bucket, schema, f)
                .reference(),
            f,
        );
        let node = schema.reserve_gc_local(f).initialize(
            schema.struct_type::<ArrayIndexEntry>().construct(
                (
                    GcOperand::i64(i64::from(ordinal)),
                    GcOperand::reference(descriptor, schema),
                    GcOperand::reference(&head, schema),
                ),
                f,
            ),
            f,
        );
        schema.array_type::<ArrayIndexBucketTable>().write(
            &self.buckets,
            bucket,
            GcOperand::nullable_reference(&node, schema),
            schema,
            f,
        );
        schema.field(ArrayIndexStorageSchema::COUNT).write(
            &self.storage,
            GcOperand::i64(i64::from(self.published)),
            schema,
            f,
        );
        node.clear(f);
        head.clear(f);
        schema.release_i32_local(bucket, f);
    }

    pub(super) fn clear(self, f: &mut Function) {
        self.buckets.clear(f);
        self.storage.clear(f);
    }
}
