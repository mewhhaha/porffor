# DateTimeFormat positional numbering

The [Intl date/time provider](intl-datetime-provider.md) owns digit and fractional
separator rendering for all 77 positional numbering systems in pinned CLDR 47.
Locale defaults, Unicode extensions and explicit options are negotiated against
the same generated profile used for patterns and parts. The Wasm consumer copies
localized parts or joins them into a string; it does not substitute digits in
names or literals.

The new profile validates exactly ten distinct Unicode scalars per digit table,
including mixed UTF-8 widths, and diagnoses missing decimal symbols. Chinese
`hanidays` is a finite generated day-field override from the pinned RBNF rules.
The [locale/kernel contract](intl-datetime-locale-kernel.md) documents the source
pin, generation commands, localized field rules and verification boundary.

The former AOT numbering table, generator and duplicate data copies were removed
when rendering moved to this provider. The six observable numbering regression
tests remain unchanged:

```sh
cargo test -p lila-engine --test aot_intl_datetime_numbering
```

The frozen checkpoint16r3 Intl replay completed 394 executions: 390 Success and
4 Bug, with no timeouts. Both modes of `related-year-zh.js` and
`temporal-objects-no-time-clip-non-latin-numerals.js` remained failing at that
checkpoint. The new Chinese and Arabic profiles target those failures; their
candidate replay is pending. Historical outcomes are retained in the
[completed-baseline follow-up](completed-baseline-follow-up.md).
