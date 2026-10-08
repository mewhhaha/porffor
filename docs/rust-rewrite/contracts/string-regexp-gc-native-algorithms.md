# Native String and RegExp GC algorithms

This source draft belongs to the single semantic GC cutover. Immutable UTF-16
strings and whole completions remain rooted across observable conversion and
calls. Closed operation enums govern character access, ranges, search, flags
and replacement scope; owned capture witnesses admit only normalized values.

RegExp construction publishes a complete header/source/flags/program record.
Builtin execution observes lastIndex conversion before acquiring those slots.
Validated program views and a consumed scratch owner govern matching, capture
materialization and cleanup. Numeric and named indices retain shared pair
identity. String fallback consumes RegExpCreate directly.

Replacement gathers results before callbacks, then retains ordered capture
and named-group observations. Split retains exact code units and raw captures;
search preserves whole lastIndex values. MatchAll owns the cloned matcher and
passes it to the actual GC iterator factory.

These decisions follow the [String and RegExp algorithms](https://tc39.es/ecma262/multipage/text-processing.html)
and [Annex B compile](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-regexp.prototype.compile).
The hook admission targets the current object-only GetMethod wording.

Fourteen authored engine controls cover observable ordering, whole exceptions,
surrogates, GC retention, species construction, captures and result identity.
They remain unrun. Compilation, Wasm validation, execution and conformance are
unverified; no earlier result verifies this combined source.
