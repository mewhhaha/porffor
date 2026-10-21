/// Values are consumed with their option, even when they look like flags.
/// Each parser owns any further value restrictions and duplicate policy.
pub(crate) struct ArgumentCursor<'a>(std::slice::Iter<'a, String>);

impl<'a> ArgumentCursor<'a> {
    pub(crate) fn new(args: &'a [String]) -> Self {
        Self(args.iter())
    }

    pub(crate) fn next(&mut self) -> Option<&'a str> {
        self.0.next().map(String::as_str)
    }

    pub(crate) fn value(&mut self, missing: &str) -> Result<&'a str, String> {
        self.next().ok_or_else(|| missing.to_owned())
    }
}
