//! Pointer-free little-endian rows shared by the build producer and consumer.

const MAGIC: &[u8; 8] = b"LILANR01";
const HEADER_BYTES: usize = MAGIC.len() + 6 * 4;
const ROW_BYTES: [usize; 6] = [12, 4, 12, 4, 8, 12];

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct NormalizationMapping {
    pub(crate) codepoint: u32,
    pub(crate) sequence_index: u32,
    pub(crate) sequence_len: u32,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct NormalizationTables {
    pub(crate) canonical_mappings: Vec<NormalizationMapping>,
    pub(crate) canonical_sequences: Vec<u32>,
    pub(crate) compatibility_mappings: Vec<NormalizationMapping>,
    pub(crate) compatibility_sequences: Vec<u32>,
    pub(crate) combining_classes: Vec<(u32, u8)>,
    pub(crate) compositions: Vec<(u32, u32, u32)>,
}

fn scalar(value: u32) -> bool {
    char::from_u32(value).is_some()
}

fn valid_mappings(mappings: &[NormalizationMapping], sequences: &[u32]) -> bool {
    let mut end: usize = 0;
    for mapping in mappings {
        if !scalar(mapping.codepoint)
            || mapping.sequence_len == 0
            || usize::try_from(mapping.sequence_index) != Ok(end)
        {
            return false;
        }
        let Some(next) = usize::try_from(mapping.sequence_len)
            .ok()
            .and_then(|length| end.checked_add(length))
        else {
            return false;
        };
        end = next;
    }
    end == sequences.len()
        && sequences.iter().copied().all(scalar)
        && mappings
            .windows(2)
            .all(|pair| pair[0].codepoint < pair[1].codepoint)
}

impl NormalizationTables {
    fn valid(&self) -> bool {
        valid_mappings(&self.canonical_mappings, &self.canonical_sequences)
            && valid_mappings(&self.compatibility_mappings, &self.compatibility_sequences)
            && self
                .combining_classes
                .iter()
                .all(|(point, class)| scalar(*point) && *class != 0)
            && self
                .combining_classes
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
            && self.compositions.iter().all(|(first, second, composed)| {
                scalar(*first) && scalar(*second) && scalar(*composed)
            })
            && self
                .compositions
                .windows(2)
                .all(|pair| (pair[0].0, pair[0].1) < (pair[1].0, pair[1].1))
    }

    /// Counts and the complete extent are admitted before allocating any row.
    /// Mapping indices must partition their sequence image in original order.
    pub(crate) fn from_image(image: &[u8]) -> Option<Self> {
        let mut reader = Reader(image.strip_prefix(MAGIC)?);
        let counts = [
            reader.word()?,
            reader.word()?,
            reader.word()?,
            reader.word()?,
            reader.word()?,
            reader.word()?,
        ];
        let mut extent = HEADER_BYTES;
        for (&count, width) in counts.iter().zip(ROW_BYTES) {
            extent = extent.checked_add(usize::try_from(count).ok()?.checked_mul(width)?)?;
        }
        if extent != image.len() || u32::try_from(extent).is_err() {
            return None;
        }
        let result = Self {
            canonical_mappings: reader.mappings(counts[0])?,
            canonical_sequences: reader.words(counts[1])?,
            compatibility_mappings: reader.mappings(counts[2])?,
            compatibility_sequences: reader.words(counts[3])?,
            combining_classes: (0..counts[4])
                .map(|_| Some((reader.word()?, u8::try_from(reader.word()?).ok()?)))
                .collect::<Option<_>>()?,
            compositions: (0..counts[5])
                .map(|_| Some((reader.word()?, reader.word()?, reader.word()?)))
                .collect::<Option<_>>()?,
        };
        (reader.0.is_empty() && result.valid()).then_some(result)
    }

    pub(crate) fn to_image(&self) -> Vec<u8> {
        assert!(
            self.valid(),
            "normalization producer must retain ordered complete rows"
        );
        let counts = [
            self.canonical_mappings.len(),
            self.canonical_sequences.len(),
            self.compatibility_mappings.len(),
            self.compatibility_sequences.len(),
            self.combining_classes.len(),
            self.compositions.len(),
        ];
        let mut bytes = MAGIC.to_vec();
        for count in counts {
            bytes.extend_from_slice(
                &u32::try_from(count)
                    .expect("normalization count fits memory32")
                    .to_le_bytes(),
            );
        }
        for (mappings, sequences) in [
            (&self.canonical_mappings, &self.canonical_sequences),
            (&self.compatibility_mappings, &self.compatibility_sequences),
        ] {
            for mapping in mappings {
                for word in [
                    mapping.codepoint,
                    mapping.sequence_index,
                    mapping.sequence_len,
                ] {
                    bytes.extend_from_slice(&word.to_le_bytes());
                }
            }
            for word in sequences {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        for &(point, class) in &self.combining_classes {
            bytes.extend_from_slice(&point.to_le_bytes());
            bytes.extend_from_slice(&u32::from(class).to_le_bytes());
        }
        for &(first, second, composed) in &self.compositions {
            for word in [first, second, composed] {
                bytes.extend_from_slice(&word.to_le_bytes());
            }
        }
        assert!(
            u32::try_from(bytes.len()).is_ok(),
            "normalization image fits memory32"
        );
        bytes
    }
}

struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn word(&mut self) -> Option<u32> {
        let (word, tail) = self.0.split_first_chunk::<4>()?;
        self.0 = tail;
        Some(u32::from_le_bytes(*word))
    }

    fn words(&mut self, count: u32) -> Option<Vec<u32>> {
        (0..count).map(|_| self.word()).collect()
    }

    fn mappings(&mut self, count: u32) -> Option<Vec<NormalizationMapping>> {
        (0..count)
            .map(|_| {
                Some(NormalizationMapping {
                    codepoint: self.word()?,
                    sequence_index: self.word()?,
                    sequence_len: self.word()?,
                })
            })
            .collect()
    }
}
