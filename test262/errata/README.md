# Explicit Test262 corrections

These are locally reviewed amendments to three upstream fixtures, affecting
four execution IDs. They are **not unmodified upstream Test262**. The vendor
tree, embedded source pins, original failures and canonical chart stay intact.
There are no skipped tests or expected-failure rules.

`manifest.json` binds the exact base specification/upstream/tree revisions,
original and replacement file hashes, execution modes, reasons and specification
sources. Each replacement retains its upstream test path and license.

- The Annex B test's 2017 explanation adds implicit `arguments` to formal
  parameter names. The pinned specification instead adds it to a separate
  parameter-binding list. The correction expects the permitted outer function
  copy in all three existing cases and adds an explicit-formal control that
  must prevent the copy. Pre-block and in-block assertions remain.
- `links.js` assumes Johnston (UM) and Honolulu (US) share a primary identifier.
  ECMA-402 requires country-preserving primaries. Johnston moves to the test's
  existing constructor/name-preservation group alongside other cross-country
  default tzdb links. All other alias equality and offset assertions remain.
  This does not settle the abstract algorithm's backzone edge: it expects a
  Link where the pinned data has a Zone. The provider's self-primary policy
  remains documented, rather than presented as an explicit normative branch.
- The pending labelled-function staging test expects recursive block-labelled
  declarations and outer copies under labels. The pinned rules distinguish a
  direct labelled function's lexical scope from a label chain and from Annex B's
  direct-declaration outer-copy candidates. The amendment preserves all thirteen
  call groups and strict-mode syntax checks. Positive calls before and after
  a direct labelled declaration must succeed inside its scope; out-of-scope
  accesses must throw `ReferenceError`. A block label chain contributes no binding.
  V8's different behavior remains explicitly recorded in the comparison.

Check the base and replacement hashes without writing:

```sh
python3 scripts/prepare_test262_errata.py --check
```

Prepare a new, independent suite copy explicitly:

```sh
python3 scripts/prepare_test262_errata.py --output target/test262-reviewed-errata
# Pass --suite-root target/test262-reviewed-errata/suite to a separate test run.
```

The output path must be new and separate from the inputs. A complete receipt
records the amended tree hash and labels it ineligible for stock Test262
publication. An interrupted copy has an incomplete receipt. Source pins are
never rewritten; the stock source-pin check must reject the amended tree.
Use a distinct snapshot family and retain the preparation receipt with results.
Do not use amended snapshots to replace the canonical stock baseline or chart.

The [2026-10-21 failed-case replay](../../docs/rust-rewrite/failed-case-replay-20261021.md)
retests all sixteen failures from the completed baseline: twelve original executions
pass and four fail; all sixteen pass with the current explicit corrections.
The [earlier 27-case comparison](../../docs/rust-rewrite/test262-errata-20261020.md)
covered the first two corrected fixtures, before adding the labelled-function
amendment. These are focused results, not complete amended or unmodified-suite
verification. The original four failures remain in unamended runs.
A separate provider-specific [offset probe](controls/johnston-offsets.js) also
passes all eleven original epoch/dated samples against the pinned shared
Johnston/Honolulu data, while preserving both requested names. This probe is
not an upstream test or a claim about all valid tzdb build configurations.
No upstream report has been sent.
