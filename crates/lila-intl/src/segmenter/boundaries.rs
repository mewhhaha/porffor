use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Annotation {
    NonWord,
    Word(bool),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentBoundary {
    start: u32,
    end: u32,
    annotation: Annotation,
}
impl SegmentBoundary {
    pub const fn start(self) -> u32 {
        self.start
    }
    pub const fn end(self) -> u32 {
        self.end
    }
    pub const fn is_word_like(self) -> Option<bool> {
        match self.annotation {
            Annotation::NonWord => None,
            Annotation::Word(value) => Some(value),
        }
    }
}
/// One complete checked partition of the original UTF16 input. The source is
/// retained so containing(), slicing, and iterator users cannot rebind offsets
/// to another string or to replacement-character-decoded text.
#[derive(Debug, Clone)]
pub struct SegmenterResult {
    input: Arc<[u16]>,
    granularity: SegmenterGranularity,
    boundaries: Box<[SegmentBoundary]>,
}
impl SegmenterResult {
    pub fn input(&self) -> &[u16] {
        &self.input
    }
    pub const fn granularity(&self) -> SegmenterGranularity {
        self.granularity
    }
    pub fn boundaries(&self) -> &[SegmentBoundary] {
        &self.boundaries
    }
    pub fn containing(&self, index: u32) -> Option<&SegmentBoundary> {
        if index as usize >= self.input.len() {
            return None;
        }
        self.boundaries
            .get(self.boundaries.partition_point(|row| row.end <= index))
    }
    pub fn units(&self, row: SegmentBoundary) -> Option<&[u16]> {
        // A row from another result may have equal offsets. Its value can only
        // authorize a slice if that exact partition row exists in this owner.
        self.boundaries
            .binary_search_by_key(&row.start, |r| r.start)
            .ok()
            .filter(|&i| self.boundaries[i] == row)
            .map(|_| &self.input[row.start as usize..row.end as usize])
    }
    pub(crate) fn from_wire(
        request: &SegmentUtf16Request,
        rows: Vec<(u32, Option<bool>)>,
    ) -> Result<Self, SegmenterError> {
        Self::checked(
            request.shared_input(),
            request.configuration().granularity(),
            rows,
        )
    }
    fn checked(
        input: Arc<[u16]>,
        granularity: SegmenterGranularity,
        rows: Vec<(u32, Option<bool>)>,
    ) -> Result<Self, SegmenterError> {
        let mut start = 0;
        let mut boundaries = Vec::new();
        let extent = rows
            .len()
            .checked_mul(16)
            .and_then(|n| n.checked_add(40))
            .ok_or(SegmenterError::Resource("partition extent"))?;
        u32::try_from(extent).map_err(|_| SegmenterError::Resource("partition exceeds Wasm32"))?;
        boundaries
            .try_reserve_exact(rows.len())
            .map_err(|_| SegmenterError::Resource("partition allocation"))?;
        for (end, word) in rows {
            if end <= start || end as usize > input.len() {
                return Err(SegmenterError::InvalidBoundaries(
                    "unordered or out of bounds",
                ));
            }
            let annotation = match (granularity, word) {
                (SegmenterGranularity::Word, Some(value)) => Annotation::Word(value),
                (SegmenterGranularity::Grapheme | SegmenterGranularity::Sentence, None) => {
                    Annotation::NonWord
                }
                _ => return Err(SegmenterError::InvalidBoundaries("word annotation domain")),
            };
            if (end as usize) < input.len()
                && (0xd800..=0xdbff).contains(&input[end as usize - 1])
                && (0xdc00..=0xdfff).contains(&input[end as usize])
            {
                return Err(SegmenterError::InvalidBoundaries(
                    "splits a paired surrogate",
                ));
            }
            boundaries.push(SegmentBoundary {
                start,
                end,
                annotation,
            });
            start = end;
        }
        if start as usize != input.len() {
            return Err(SegmenterError::InvalidBoundaries("incomplete partition"));
        }
        Ok(Self {
            input,
            granularity,
            boundaries: boundaries.into_boxed_slice(),
        })
    }
}
fn push(
    rows: &mut Vec<(u32, Option<bool>)>,
    end: usize,
    word: Option<bool>,
) -> Result<(), SegmenterError> {
    let count = rows
        .len()
        .checked_add(1)
        .and_then(|n| n.checked_mul(16))
        .and_then(|n| n.checked_add(40))
        .ok_or(SegmenterError::Resource("boundary extent"))?;
    u32::try_from(count).map_err(|_| SegmenterError::Resource("boundaries exceed Wasm32"))?;
    rows.try_reserve(1)
        .map_err(|_| SegmenterError::Resource("boundary allocation"))?;
    rows.push((
        u32::try_from(end).map_err(|_| SegmenterError::Resource("boundary index"))?,
        word,
    ));
    Ok(())
}
pub fn segment_utf16(request: &SegmentUtf16Request) -> Result<SegmenterResult, SegmenterError> {
    let profile = &request.configuration().locale().profile;
    let mut rows = Vec::new();
    match request.configuration().granularity() {
        SegmenterGranularity::Grapheme => {
            let segmenter = profile.grapheme().as_borrowed();
            let mut iter = segmenter.segment_utf16(request.input());
            check_start(iter.next())?;
            for end in iter {
                push(&mut rows, end, None)?;
            }
        }
        SegmenterGranularity::Word => {
            let segmenter = profile.word.as_borrowed();
            let mut iter = segmenter.segment_utf16(request.input());
            // ICU word_type/is_word_like describe the segment preceding the
            // boundary most recently yielded, not the following segment.
            check_start(iter.next())?;
            while let Some(end) = iter.next() {
                push(&mut rows, end, Some(iter.is_word_like()))?;
            }
        }
        SegmenterGranularity::Sentence => {
            let segmenter = profile.sentence.as_borrowed();
            let mut iter = segmenter.segment_utf16(request.input());
            check_start(iter.next())?;
            for end in iter {
                push(&mut rows, end, None)?;
            }
        }
    }
    SegmenterResult::checked(
        request.shared_input(),
        request.configuration().granularity(),
        rows,
    )
}

fn check_start(start: Option<usize>) -> Result<(), SegmenterError> {
    if start == Some(0) {
        Ok(())
    } else {
        Err(SegmenterError::InvalidBoundaries(
            "native partition lacks initial zero",
        ))
    }
}
