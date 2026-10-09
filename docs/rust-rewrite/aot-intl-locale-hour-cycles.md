# Locale hour-cycle information

This future source batch implements `Intl.Locale.prototype.getHourCycles`
through a typed native operation and real Wasm result construction. It is based
on the independently reviewed labelled-region/week/text source batch. This corrected
source draft preserves the rejected earlier proposal and its concrete open-slot
finding. Corrected native/compiler joining and final source checks pass.
Independent native and compiler reviews pass; compilation and runtime
verification remain pending. No conformance counts change, and T23 stays open.

The provider uses the complete pinned CLDR47 `timeData` preference table. Its
25 source rows expand to 275 unique selectors. The `allowed` sequence supplies
preference order; the compatibility `preferred` symbol is not prepended.
Projection uses the existing closed `DateTimeHourCycle` domain, maps `H`, `h`,
`K`, `k` to `h23`, `h12`, `h11`, `h24`, and deduplicates after projection. The
CLDR `hb` and `hB` forms both project to `h12`.

Any present `hc` slot produces a singleton containing its exact string,
including an unknown, compound or empty extension value. Wasm extracts that
immutable slot before the default-data host call. The constructor option
`hourCycle` remains restricted to the four standard values. Otherwise shared canonical
RegionPreference chooses an explicit region before `sd`, then genuine likely
subtags, and finally `001`. An available `rg` override precedes that region.
For each preferred region, language-region data precede regional data. Sparse
recognized regions inherit the pinned `001` data. Unavailable overrides fall
through, and absence of all selected data produces only `h23`. Whole keyword
values are checked; private use and transformed extensions do not become
Unicode overrides. Week-specific `fw` selection remains with the week owner.

The native default request validates once that no `hc` keyword is present;
its constructor rejects every present value, including an empty string. Only
this request type reaches the native default resolver. The native default
result is a private validated nonempty list of unique closed cycles.
Host ABI15 adds operation41. The response has exactly eight bytes: a little
endian `u32` count and four ordered one-byte codes (`h11`=1, `h12`=2, `h23`=3,
`h24`=4); unused bytes are zero. The Wasm reader validates length, count, every
code, uniqueness and trailing zero bytes before handing a private consuming
carrier to materialization. A fresh dense array uses the called method's
defining Realm and normal writable, enumerable, configurable index properties.
The receiver brand is checked before the host call; extra arguments are ignored.

Five paired Engine controls cover open-slot singleton and getter consistency,
regional/language preference order, extension
and option precedence, malformed and unavailable regional overrides, metadata,
fresh arrays, receiver branding and defining-Realm arrays/errors. The exact
primary pinned scope is five physical files and ten Script modes. These Rust
and product controls are unexecuted until the fresh source build is admitted:

```sh
cargo test --locked -p lila-intl locale_hour_cycles
cargo test --locked -p lila-engine --test aot_intl -- aot_intl_locale_hour_cycles::
```

Primary authorities are the current
[HourCyclesOfLocale algorithm](https://tc39.es/ecma402/#sec-hourcyclesoflocale),
[RegionPreference](https://tc39.es/ecma402/#sec-regionpreference), and
[CLDR Time Data](https://www.unicode.org/reports/tr35/tr35-dates.html#Time_Data).
Pinned test descriptions from older algorithm versions do not replace the
current normative explicit-slot singleton behavior. The closed enum constrains
the generated default table; it does not constrain explicit Unicode values.
