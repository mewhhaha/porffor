# Intl.Locale core language options

This change addresses the constructor's previously ignored `language`, `script`
and `region` options on the direct JavaScript-to-Wasm path. It does not close
T23, implement all Intl.Locale options, or establish a new Test262 checkpoint.

## Observable contract

The constructor retains its existing reserved/initialized result lifecycle.
It resolves the result prototype before observing the tag, converts the tag to
its string/Locale-slot value, then coerces options before validating tag syntax.
Thus a null options argument throws TypeError even when the converted tag is
malformed; an invalid tag prevents every options property read.

For a defined options argument, the shared current-function-Realm ToObject
operation preserves object identity or boxes a primitive once. Undefined has
the semantics of a fresh, empty, null-prototype options object; it does not read
Object.prototype. The core properties are read in language/script/region order
through the shared object-read operation, with the options object as receiver.
Each property's Get, ToString and validation finishes before the next Get.
Getter exceptions, conversion exceptions and invalid subtags stop immediately.

Language accepts two or three, or five through eight, ASCII letters. Script
accepts exactly four ASCII letters. Region accepts two ASCII letters or three
ASCII digits. These are syntax checks, not registry membership checks. In
particular, unknown but well-formed subtags are accepted and mixed digit/letter
regions are rejected. A private validated-component type is required at the
field-replacement point.

The original language/script/region prefix boundary is saved before replacing
any field. Its suffix retains variants, extensions and private use. Allocation
uses the exact replacement prefix plus suffix length, not a fixed 255-byte
host-provider buffer. The existing structural canonicalizer then rebuilds the
complete tag, baseName, language, script and region slots together. Omitted
options leave existing components intact; no defined core options means no
extra reconstruction.

## Regression target

`crates/lila-engine/tests/aot_intl_locale_options.rs` contains 18 direct Wasm-AOT
regressions for replacements, insertion, casing, suffix retention, absent slots,
getter/coercion order, abrupt completion, inherited/Proxy receivers, primitive
boxing, undefined/null options, malformed inputs, unknown valid subtags, Symbol
conversion, branded Locale inputs, and constructor prototype ordering.

Run the focused target and existing construction-lifecycle invariant:

```sh
cargo test -p lila-engine --test aot_intl_locale_options -- --test-threads=1
cargo test -p lila-aot-wasm intl_locale_construction_order_tests -- --test-threads=1
```

Then use the repository's batch verification ladder, including the normal
engine/CLI suites and the pinned raw-source Intl.Locale Test262 cohort. Do not
rewrite test sources, reduce assertion helpers, add skips, or alter published
counts to make this change appear green.

## Evidence boundary

At patch preparation, all 18 exact JavaScript snippets returned true on Node
v22.16.0 in isolated contexts. This checks expected reference behavior only.
The authoring environment had no local Rust toolchain or runnable repository
checkout. The GitHub connector was subsequently used to publish the branch. Neither Cargo compilation, the Lila
regressions, nor the pinned Test262 cohort was run. These remain mandatory
before treating this implementation as verified. The generated README status
block and all conformance denominators are unchanged.

## Remaining work

The constructor still uses structural canonicalization rather than the
provider-backed alias resolution used by Intl.getCanonicalLocales. In
particular, alias resolution before applying overrides is not added here.
The `variants` option and Unicode-extension options (calendar, collation,
firstDayOfWeek, hourCycle, caseFirst, numeric and numberingSystem) remain outside
this patch, as do their additional slots/getters and broader locale services.
This bounded core-options implementation must not be presented as complete
UpdateLanguageId, Intl.Locale or ECMA-402 support.

The algorithm reference is the ECMA-402 Intl.Locale constructor and its
UpdateLanguageId operation. Step numbers evolve; ordering and the subtag
grammar, rather than an old numbered step, are the contract.
