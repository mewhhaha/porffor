use std::fmt;

use lila_ir::ValueKind;
use num_bigint::{BigInt, Sign};

use crate::heap::{
    HEAP_BIGINT_LIMBS_CAP_OFFSET, HEAP_BIGINT_LIMBS_LEN_OFFSET, HEAP_BIGINT_LIMBS_PTR_OFFSET,
    HEAP_BIGINT_RECORD_SIZE, HEAP_BIGINT_SIGN_OFFSET, HEAP_BIGINT_VALUE_TAG,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmRuntimeValueTag {
    ValueKind(ValueKind),
    HeapBigInt,
}

impl WasmRuntimeValueTag {
    pub const fn from_tag(tag: i32) -> Option<Self> {
        if tag == HEAP_BIGINT_VALUE_TAG as i32 {
            return Some(Self::HeapBigInt);
        }
        match ValueKind::from_tag(tag) {
            Some(kind) => Some(Self::ValueKind(kind)),
            None => None,
        }
    }

    pub const fn value_kind(self) -> ValueKind {
        match self {
            Self::ValueKind(kind) => kind,
            Self::HeapBigInt => ValueKind::BigInt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmRuntimeDecodeError {
    message: String,
}

impl WasmRuntimeDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for WasmRuntimeDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for WasmRuntimeDecodeError {}

pub fn decode_heap_bigint_decimal<E>(
    record_address: u64,
    memory_byte_len: usize,
    mut read_memory: impl FnMut(usize, &mut [u8]) -> Result<(), E>,
) -> Result<String, WasmRuntimeDecodeError>
where
    E: fmt::Display,
{
    if record_address == 0 {
        return Err(WasmRuntimeDecodeError::new(
            "heap BigInt completion has a null record address",
        ));
    }
    let record_start = usize::try_from(record_address).map_err(|_| {
        WasmRuntimeDecodeError::new(format!(
            "heap BigInt record address {record_address} does not fit the host address space"
        ))
    })?;
    let record_byte_len = HEAP_BIGINT_RECORD_SIZE as usize;
    let record_end = record_start
        .checked_add(record_byte_len)
        .filter(|end| *end <= memory_byte_len)
        .ok_or_else(|| {
            WasmRuntimeDecodeError::new(format!(
                "heap BigInt record span {record_start}..{} exceeds Wasm memory length \
                 {memory_byte_len}",
                record_start.saturating_add(record_byte_len)
            ))
        })?;
    let mut record = vec![0; record_end - record_start];
    read_memory(record_start, &mut record).map_err(|error| {
        WasmRuntimeDecodeError::new(format!(
            "failed to read heap BigInt record at address {record_address}: {error}"
        ))
    })?;

    let sign_value = read_record_u64(&record, HEAP_BIGINT_SIGN_OFFSET) as i64;
    let sign = match sign_value {
        -1 => Sign::Minus,
        0 => Sign::NoSign,
        1 => Sign::Plus,
        _ => {
            return Err(WasmRuntimeDecodeError::new(format!(
                "heap BigInt record at address {record_address} has invalid sign {sign_value}"
            )));
        }
    };
    let limbs_address = read_record_u64(&record, HEAP_BIGINT_LIMBS_PTR_OFFSET);
    let limb_count = read_record_u64(&record, HEAP_BIGINT_LIMBS_LEN_OFFSET);
    let limb_capacity = read_record_u64(&record, HEAP_BIGINT_LIMBS_CAP_OFFSET);
    if limb_count == 0 {
        return Err(WasmRuntimeDecodeError::new(format!(
            "heap BigInt record at address {record_address} has zero limbs"
        )));
    }
    if limb_count > limb_capacity {
        return Err(WasmRuntimeDecodeError::new(format!(
            "heap BigInt record at address {record_address} has limb count {limb_count} \
             greater than capacity {limb_capacity}"
        )));
    }
    if limbs_address == 0 {
        return Err(WasmRuntimeDecodeError::new(format!(
            "heap BigInt record at address {record_address} has a null limbs address"
        )));
    }

    let limb_byte_len = limb_count.checked_mul(8).ok_or_else(|| {
        WasmRuntimeDecodeError::new(format!(
            "heap BigInt record at address {record_address} has overflowing limb count \
             {limb_count}"
        ))
    })?;
    let limbs_start = usize::try_from(limbs_address).map_err(|_| {
        WasmRuntimeDecodeError::new(format!(
            "heap BigInt limbs address {limbs_address} does not fit the host address space"
        ))
    })?;
    let limb_byte_len = usize::try_from(limb_byte_len).map_err(|_| {
        WasmRuntimeDecodeError::new(format!(
            "heap BigInt limb storage length for {limb_count} limbs does not fit the host \
             address space"
        ))
    })?;
    let limbs_end = limbs_start
        .checked_add(limb_byte_len)
        .filter(|end| *end <= memory_byte_len)
        .ok_or_else(|| {
            WasmRuntimeDecodeError::new(format!(
                "heap BigInt limb span {limbs_start}..{} for {limb_count} limbs exceeds Wasm \
                 memory length {memory_byte_len}",
                limbs_start.saturating_add(limb_byte_len)
            ))
        })?;
    let mut limb_bytes = vec![0; limbs_end - limbs_start];
    read_memory(limbs_start, &mut limb_bytes).map_err(|error| {
        WasmRuntimeDecodeError::new(format!(
            "failed to read {limb_count} heap BigInt limbs at address {limbs_address}: {error}"
        ))
    })?;

    let magnitude_is_zero = limb_bytes.iter().all(|byte| *byte == 0);
    if (sign_value == 0) != magnitude_is_zero {
        return Err(WasmRuntimeDecodeError::new(format!(
            "heap BigInt record at address {record_address} has sign {sign_value} inconsistent \
             with its magnitude"
        )));
    }

    Ok(BigInt::from_bytes_le(sign, &limb_bytes).to_string())
}

fn read_record_u64(record: &[u8], offset: u64) -> u64 {
    let start = offset as usize;
    u64::from_le_bytes(
        record[start..start + 8]
            .try_into()
            .expect("heap BigInt record field should span eight bytes"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECORD_ADDRESS: usize = 64;
    const LIMBS_ADDRESS: usize = 128;

    fn encode_heap_bigint(sign: i64, limbs: &[u64]) -> Vec<u8> {
        let mut memory = vec![0; LIMBS_ADDRESS + limbs.len() * 8];
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_SIGN_OFFSET as usize,
            sign as u64,
        );
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_LIMBS_PTR_OFFSET as usize,
            LIMBS_ADDRESS as u64,
        );
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_LIMBS_LEN_OFFSET as usize,
            limbs.len() as u64,
        );
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_LIMBS_CAP_OFFSET as usize,
            limbs.len() as u64,
        );
        for (index, limb) in limbs.iter().copied().enumerate() {
            write_u64(&mut memory, LIMBS_ADDRESS + index * 8, limb);
        }
        memory
    }

    fn write_u64(memory: &mut [u8], offset: usize, value: u64) {
        memory[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn heap_bigint_tag_preserves_its_semantic_value_kind() {
        assert_eq!(
            WasmRuntimeValueTag::from_tag(HEAP_BIGINT_VALUE_TAG as i32),
            Some(WasmRuntimeValueTag::HeapBigInt)
        );
        assert_eq!(
            WasmRuntimeValueTag::HeapBigInt.value_kind(),
            ValueKind::BigInt
        );
        assert_eq!(
            WasmRuntimeValueTag::from_tag(ValueKind::BigInt.tag()),
            Some(WasmRuntimeValueTag::ValueKind(ValueKind::BigInt))
        );
    }

    #[test]
    fn heap_bigint_decoder_formats_every_little_endian_limb() {
        let memory = encode_heap_bigint(1, &[1, 1, 1]);

        let decimal = decode_heap_bigint_decimal(
            RECORD_ADDRESS as u64,
            memory.len(),
            |offset, destination| {
                destination.copy_from_slice(&memory[offset..offset + destination.len()]);
                Ok::<(), String>(())
            },
        )
        .expect("valid heap BigInt should decode");

        assert_eq!(decimal, "340282366920938463481821351505477763073");
    }

    #[test]
    fn heap_bigint_decoder_preserves_a_negative_sign() {
        let memory = encode_heap_bigint(-1, &[u64::MAX]);

        let decimal = decode_heap_bigint_decimal(
            RECORD_ADDRESS as u64,
            memory.len(),
            |offset, destination| {
                destination.copy_from_slice(&memory[offset..offset + destination.len()]);
                Ok::<(), String>(())
            },
        )
        .expect("valid negative heap BigInt should decode");

        assert_eq!(decimal, "-18446744073709551615");
    }

    #[test]
    fn heap_bigint_decoder_rejects_an_out_of_bounds_limb_span_before_reading_it() {
        let mut memory = encode_heap_bigint(1, &[1]);
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_LIMBS_LEN_OFFSET as usize,
            1_000,
        );
        write_u64(
            &mut memory,
            RECORD_ADDRESS + HEAP_BIGINT_LIMBS_CAP_OFFSET as usize,
            1_000,
        );
        let mut read_count = 0;

        let error = decode_heap_bigint_decimal(
            RECORD_ADDRESS as u64,
            memory.len(),
            |offset, destination| {
                read_count += 1;
                destination.copy_from_slice(&memory[offset..offset + destination.len()]);
                Ok::<(), String>(())
            },
        )
        .expect_err("out-of-bounds heap BigInt limbs should fail");

        assert_eq!(read_count, 1);
        assert!(error
            .message()
            .contains("for 1000 limbs exceeds Wasm memory length"));
    }
}
