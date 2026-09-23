use super::*;

macro_rules! regexp_opcodes {
    ($($variant:ident = $word:ident),+ $(,)?) => {
        /// Opcode facts shared by the static descriptor validator and emitted compiler.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(u64)]
        pub enum RegExpOpcode { $($variant = $word),+ }
        impl RegExpOpcode {
            pub const ALL: [Self; 24] = [$(Self::$variant),+];
            pub const fn from_word(word: u64) -> Option<Self> {
                match word { $($word => Some(Self::$variant),)+ _ => None }
            }
        }
    };
}

regexp_opcodes! {
    Accept = REGEXP_OPCODE_ACCEPT,
    LiteralAscii = REGEXP_OPCODE_LITERAL_ASCII,
    PositiveAsciiClass = REGEXP_OPCODE_POSITIVE_ASCII_CLASS,
    Split = REGEXP_OPCODE_SPLIT,
    Jump = REGEXP_OPCODE_JUMP,
    CaptureStart = REGEXP_OPCODE_CAPTURE_START,
    CaptureEnd = REGEXP_OPCODE_CAPTURE_END,
    ClearCaptureRange = REGEXP_OPCODE_CLEAR_CAPTURE_RANGE,
    Whitespace = REGEXP_OPCODE_WHITESPACE,
    Dot = REGEXP_OPCODE_DOT,
    LiteralCodePoint = REGEXP_OPCODE_LITERAL_CODE_POINT,
    UnicodeProperty = REGEXP_OPCODE_UNICODE_PROPERTY,
    NamedBackreference = REGEXP_OPCODE_NAMED_BACKREFERENCE,
    NegativeAsciiClass = REGEXP_OPCODE_NEGATIVE_ASCII_CLASS,
    NumberedBackreference = REGEXP_OPCODE_NUMBERED_BACKREFERENCE,
    AssertStart = REGEXP_OPCODE_ASSERT_START,
    AssertEnd = REGEXP_OPCODE_ASSERT_END,
    NotWhitespace = REGEXP_OPCODE_NOT_WHITESPACE,
    LookaroundStart = REGEXP_OPCODE_LOOKAROUND_START,
    LookaroundEnd = REGEXP_OPCODE_LOOKAROUND_END,
    LookaroundFailure = REGEXP_OPCODE_LOOKAROUND_FAILURE,
    ProgressSplit = REGEXP_OPCODE_PROGRESS_SPLIT,
    ProgressCheck = REGEXP_OPCODE_PROGRESS_CHECK,
    WordBoundary = REGEXP_OPCODE_WORD_BOUNDARY,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegExpOperandRule {
    Zero,
    Ascii,
    CodePoint,
    Bitmap,
    TargetPair,
    Target,
    Capture,
    CaptureRange,
    Modifier,
    Range,
    NonemptyRange,
    NamedReference,
    NumberedReference,
    LookaroundStart,
    LookaroundEnd,
    LookaroundFailure,
    ProgressSplit,
    ProgressCheck,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegExpControlFlow {
    Accept,
    Next,
    Operand0,
    Operand1,
    BothOperands,
    ProgressSplit,
    LookaroundAfter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegExpInputProgress {
    Consumes,
    MayStay,
    NumberedReference,
    CheckedOptional,
}

impl RegExpOpcode {
    pub const fn operand_rule(self) -> RegExpOperandRule {
        use RegExpOperandRule as Rule;
        match self {
            Self::Accept | Self::Whitespace | Self::NotWhitespace => Rule::Zero,
            Self::LiteralAscii => Rule::Ascii,
            Self::LiteralCodePoint => Rule::CodePoint,
            Self::PositiveAsciiClass | Self::NegativeAsciiClass => Rule::Bitmap,
            Self::Split => Rule::TargetPair,
            Self::Jump => Rule::Target,
            Self::CaptureStart | Self::CaptureEnd => Rule::Capture,
            Self::ClearCaptureRange => Rule::CaptureRange,
            Self::Dot | Self::AssertStart | Self::AssertEnd => Rule::Modifier,
            Self::UnicodeProperty => Rule::Range,
            Self::WordBoundary => Rule::NonemptyRange,
            Self::NamedBackreference => Rule::NamedReference,
            Self::NumberedBackreference => Rule::NumberedReference,
            Self::LookaroundStart => Rule::LookaroundStart,
            Self::LookaroundEnd => Rule::LookaroundEnd,
            Self::LookaroundFailure => Rule::LookaroundFailure,
            Self::ProgressSplit => Rule::ProgressSplit,
            Self::ProgressCheck => Rule::ProgressCheck,
        }
    }

    pub const fn control_flow(self) -> RegExpControlFlow {
        use RegExpControlFlow as Flow;
        match self {
            Self::Accept => Flow::Accept,
            Self::Split => Flow::BothOperands,
            Self::Jump | Self::LookaroundFailure => Flow::Operand0,
            Self::ProgressCheck => Flow::Operand1,
            Self::ProgressSplit => Flow::ProgressSplit,
            Self::LookaroundEnd => Flow::LookaroundAfter,
            Self::LiteralAscii
            | Self::PositiveAsciiClass
            | Self::CaptureStart
            | Self::CaptureEnd
            | Self::ClearCaptureRange
            | Self::Whitespace
            | Self::Dot
            | Self::LiteralCodePoint
            | Self::UnicodeProperty
            | Self::NamedBackreference
            | Self::NegativeAsciiClass
            | Self::NumberedBackreference
            | Self::AssertStart
            | Self::AssertEnd
            | Self::NotWhitespace
            | Self::LookaroundStart
            | Self::WordBoundary => Flow::Next,
        }
    }

    pub const fn input_progress(self) -> RegExpInputProgress {
        use RegExpInputProgress as Progress;
        match self {
            Self::LiteralAscii
            | Self::PositiveAsciiClass
            | Self::Whitespace
            | Self::Dot
            | Self::LiteralCodePoint
            | Self::UnicodeProperty
            | Self::NegativeAsciiClass
            | Self::NotWhitespace => Progress::Consumes,
            Self::NumberedBackreference => Progress::NumberedReference,
            Self::ProgressCheck => Progress::CheckedOptional,
            Self::Accept
            | Self::Split
            | Self::Jump
            | Self::CaptureStart
            | Self::CaptureEnd
            | Self::ClearCaptureRange
            | Self::NamedBackreference
            | Self::AssertStart
            | Self::AssertEnd
            | Self::LookaroundStart
            | Self::LookaroundEnd
            | Self::LookaroundFailure
            | Self::ProgressSplit
            | Self::WordBoundary => Progress::MayStay,
        }
    }

    pub const fn is_source_atom(self) -> bool {
        match self {
            Self::LiteralAscii
            | Self::PositiveAsciiClass
            | Self::Whitespace
            | Self::Dot
            | Self::LiteralCodePoint
            | Self::UnicodeProperty
            | Self::NamedBackreference
            | Self::NegativeAsciiClass
            | Self::NumberedBackreference
            | Self::AssertStart
            | Self::AssertEnd
            | Self::NotWhitespace
            | Self::WordBoundary => true,
            Self::Accept
            | Self::Split
            | Self::Jump
            | Self::CaptureStart
            | Self::CaptureEnd
            | Self::ClearCaptureRange
            | Self::LookaroundStart
            | Self::LookaroundEnd
            | Self::LookaroundFailure
            | Self::ProgressSplit
            | Self::ProgressCheck => false,
        }
    }

    pub const fn is_choice(self) -> bool {
        matches!(
            self.control_flow(),
            RegExpControlFlow::BothOperands | RegExpControlFlow::ProgressSplit
        )
    }

    pub const fn stops_non_consuming_walk(self, operand1: u64) -> bool {
        match self.input_progress() {
            RegExpInputProgress::Consumes | RegExpInputProgress::CheckedOptional => true,
            RegExpInputProgress::MayStay => false,
            RegExpInputProgress::NumberedReference => operand1 & REGEXP_BACKREFERENCE_NONEMPTY != 0,
        }
    }

    pub fn successors(
        self,
        instruction: RegExpInstruction,
        pc: usize,
        count: usize,
    ) -> [Option<usize>; 2] {
        let valid = |word: u64| usize::try_from(word).ok().filter(|target| *target < count);
        let a = instruction.operand0;
        let b = instruction.operand1;
        match self.control_flow() {
            RegExpControlFlow::Accept => [None, None],
            RegExpControlFlow::Next => [valid(pc as u64 + 1), None],
            RegExpControlFlow::Operand0 => [valid(a), None],
            RegExpControlFlow::Operand1 => [valid(b), None],
            RegExpControlFlow::BothOperands => [valid(a), valid(b)],
            RegExpControlFlow::ProgressSplit => [valid(a), valid(b >> 1)],
            RegExpControlFlow::LookaroundAfter => [valid(b & 0x3fff_ffff_ffff_ffff), None],
        }
    }
}
