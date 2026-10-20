# Explicit Test262 corrections

These are locally reviewed amendments to two upstream fixtures, affecting
three execution IDs. They are **not unmodified upstream Test262**. The vendor
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

The [focused comparison](../../docs/rust-rewrite/test262-errata-20261020.md)
records the same 27 execution IDs on the same immutable compiler: 24 pass and
the three disputed originals fail; all 27 pass with these explicit corrections.
This is focused validation, not a complete amended or unmodified-suite claim.
A separate provider-specific [offset probe](controls/johnston-offsets.js) also
passes all eleven original epoch/dated samples against the pinned shared
Johnston/Honolulu data, while preserving both requested names. This probe is
not an upstream test or a claim about all valid tzdb build configurations.
No upstream report has been sent.
