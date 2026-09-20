use core::{fmt, num::NonZeroU32};

use super::numeric::{NumberFormatResourceError, NumericLimits};
use super::profiles::InvalidNumberProfile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PartitionLimits {
    numeric: NumericLimits,
    output_bytes: NonZeroU32,
    part_count: NonZeroU32,
}

impl PartitionLimits {
    pub const HOST_ABI: Self = Self {
        numeric: NumericLimits::HOST_ABI,
        output_bytes: NonZeroU32::MAX,
        part_count: NonZeroU32::MAX,
    };

    pub const fn new(
        numeric: NumericLimits,
        output_bytes: NonZeroU32,
        part_count: NonZeroU32,
    ) -> Self {
        Self {
            numeric,
            output_bytes,
            part_count,
        }
    }

    pub const fn numeric(self) -> NumericLimits {
        self.numeric
    }

    pub const fn output_bytes(self) -> u32 {
        self.output_bytes.get()
    }

    pub const fn part_count(self) -> u32 {
        self.part_count.get()
    }

    pub(super) fn check_bytes(self, requested: u128) -> Result<usize, NumberFormatKernelError> {
        if requested > u128::from(self.output_bytes.get()) {
            return Err(NumberPartitionResourceError::OutputExtent {
                requested,
                maximum: self.output_bytes.get(),
            }
            .into());
        }
        usize::try_from(requested).map_err(|_| {
            NumberPartitionResourceError::OutputExtent {
                requested,
                maximum: self.output_bytes.get(),
            }
            .into()
        })
    }

    pub(super) fn check_parts(self, requested: u128) -> Result<usize, NumberFormatKernelError> {
        if requested > u128::from(self.part_count.get()) {
            return Err(NumberPartitionResourceError::PartExtent {
                requested,
                maximum: self.part_count.get(),
            }
            .into());
        }
        usize::try_from(requested).map_err(|_| {
            NumberPartitionResourceError::PartExtent {
                requested,
                maximum: self.part_count.get(),
            }
            .into()
        })
    }
}

impl Default for PartitionLimits {
    fn default() -> Self {
        Self::HOST_ABI
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberPartitionResourceError {
    Numeric(NumberFormatResourceError),
    OutputExtent { requested: u128, maximum: u32 },
    PartExtent { requested: u128, maximum: u32 },
    Allocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatKernelError {
    InvalidProfile(InvalidNumberProfile),
    InvalidResolvedLocale,
    Resource(NumberPartitionResourceError),
}

impl From<NumberFormatResourceError> for NumberFormatKernelError {
    fn from(value: NumberFormatResourceError) -> Self {
        Self::Resource(NumberPartitionResourceError::Numeric(value))
    }
}

impl From<NumberPartitionResourceError> for NumberFormatKernelError {
    fn from(value: NumberPartitionResourceError) -> Self {
        Self::Resource(value)
    }
}

impl From<InvalidNumberProfile> for NumberFormatKernelError {
    fn from(value: InvalidNumberProfile) -> Self {
        Self::InvalidProfile(value)
    }
}

impl fmt::Display for NumberFormatKernelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidProfile(error) => error.fmt(formatter),
            Self::InvalidResolvedLocale => {
                formatter.write_str("invalid NumberFormat resolved locale proof")
            }
            Self::Resource(NumberPartitionResourceError::Numeric(error)) => error.fmt(formatter),
            Self::Resource(NumberPartitionResourceError::OutputExtent { requested, maximum }) => {
                write!(
                    formatter,
                    "NumberFormat needs {requested} output bytes; the limit is {maximum}"
                )
            }
            Self::Resource(NumberPartitionResourceError::PartExtent { requested, maximum }) => {
                write!(
                    formatter,
                    "NumberFormat needs {requested} parts; the limit is {maximum}"
                )
            }
            Self::Resource(NumberPartitionResourceError::Allocation) => {
                formatter.write_str("NumberFormat partition allocation failed")
            }
        }
    }
}

impl std::error::Error for NumberFormatKernelError {}

pub(super) fn owned_text(
    text: &str,
    limits: &PartitionLimits,
) -> Result<Box<str>, NumberFormatKernelError> {
    limits.check_bytes(text.len() as u128)?;
    let mut result = String::new();
    result
        .try_reserve_exact(text.len())
        .map_err(|_| NumberPartitionResourceError::Allocation)?;
    result.push_str(text);
    Ok(result.into_boxed_str())
}
