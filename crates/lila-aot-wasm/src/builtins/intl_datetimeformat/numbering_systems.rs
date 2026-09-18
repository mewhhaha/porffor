mod generated;

pub(super) use generated::NUMBERING_SYSTEMS;

pub(super) const DEFAULT_NUMBERING_SYSTEM: DtfNumberingSystem =
    NUMBERING_SYSTEMS[generated::DEFAULT_INDEX];

/// The renderer indexes ten equal-width UTF-8 scalars by decimal value. The
/// constructor checks this layout in every generated constant, including the
/// supplementary-plane tables and non-contiguous `hanidec` digits.
#[derive(Clone, Copy)]
pub(super) struct DtfNumberingSystem {
    identifier: &'static str,
    digits: &'static str,
    digit_utf8_width: u64,
    decimal_separator: &'static str,
}

impl DtfNumberingSystem {
    const fn new(
        identifier: &'static str,
        digits: &'static str,
        decimal_separator: &'static str,
    ) -> Self {
        let bytes = digits.as_bytes();
        assert!(bytes.len() % 10 == 0);
        let width = bytes.len() / 10;
        assert!(width >= 1 && width <= 4);
        let mut digit = 0;
        while digit < 10 {
            let start = digit * width;
            let first = bytes[start];
            assert!(match width {
                1 => first < 0x80,
                2 => first >= 0xc2 && first < 0xe0,
                3 => first >= 0xe0 && first < 0xf0,
                4 => first >= 0xf0 && first < 0xf5,
                _ => false,
            });
            let mut byte = 1;
            while byte < width {
                assert!(bytes[start + byte] & 0xc0 == 0x80);
                byte += 1;
            }
            digit += 1;
        }
        assert!(!decimal_separator.is_empty());
        Self {
            identifier,
            digits,
            digit_utf8_width: width as u64,
            decimal_separator,
        }
    }

    pub(super) const fn identifier(self) -> &'static str {
        self.identifier
    }

    pub(super) const fn digits(self) -> &'static str {
        self.digits
    }

    pub(super) const fn digit_utf8_width(self) -> u64 {
        self.digit_utf8_width
    }

    pub(super) const fn decimal_separator(self) -> &'static str {
        self.decimal_separator
    }
}
