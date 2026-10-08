# Explicit budgets over admitted measurements

`lila performance check --mode compiler|runtime --baseline PATH --candidate PATH
--policy PATH --output-dir PATH` checks existing reports. It does not run a
benchmark. Each report first passes its original raw-sample, retained workload,
Wasm validation/footprint and (for runtime) completion/output admission.

The baseline and candidate must have equal workload, hardware, platform, build,
resource configuration, sample count and idle-machine acknowledgment. Compiler
source/executable identities may differ and both remain in the resulting report.
Debug and optimized profiles cannot silently compare as the same measurement
configuration.

The bounded JSON policy has `version: 1`, `minimum_samples: 3..100` and 1–1024
unique `rules`. Every rule names an exact `fixture`, a closed `metric`, a
`statistic` and one or both limits:

- `absolute_maximum`: an unsigned integer in that metric's unit.
- `maximum_increase_basis_points`: 0–1,000,000, where 100 basis points is 1%.

Both limits must pass when both are present. Relative admission compares exact
integers in a wider type: `candidate * 10000 <= baseline * (10000 + allowance)`.
A zero baseline requires a zero candidate for a relative rule. No floating-point
rounding or undefined ratio becomes a pass.

Timing metrics use nanoseconds and support `median` or `p95`: `compiler-prepare`,
`compiler-lower`, `compiler-emit`, `compiler-validate`, `runtime-engine`,
`runtime-module`, `runtime-store-and-linker`, `runtime-instantiate`,
`runtime-export-lookup`, `runtime-execution-and-completion` and `runtime-total`.
`gc-capacity-before` and `gc-capacity-after` support the same distribution
statistics in bytes; they are reserved capacity, not allocation or live-size
measurements.

Deterministic footprint metrics require `exact`: `module-bytes`,
`code-body-bytes`, `largest-code-body-bytes`, `identical-extra-body-bytes`,
`data-payload-bytes`, `defined-functions`, `imported-functions` and
`identical-extra-bodies`. The last three use counts; the others use bytes.
Unavailable metrics, wrong metric/statistic combinations, absent fixtures,
duplicate rules, single measurements and incompatible reports are errors.

For example, this illustrative policy permits a 5% increase in one compiler
stage's median. It is not a calibrated project budget:

```json
{
  "version": 1,
  "minimum_samples": 5,
  "rules": [{
    "fixture": "wasm_functions.js",
    "metric": "compiler-prepare",
    "statistic": "median",
    "absolute_maximum": null,
    "maximum_increase_basis_points": 500
  }]
}
```

The new output directory retains exact policy and report bytes plus their hashes,
both complete measurement identities and every rule decision. An exceeded budget
still writes its report and returns failure. The original reports' retained
workload/artifact/output directories remain the source evidence and must be kept.
Passing these explicit rules is not a conformance result or a claim that all
subsystems are budgeted.

Source-only controls exercise exact boundaries/zero/overflow, raw outliers,
minimum sample count, incompatible identities, unavailable metrics and policy
refusals. Compilation, execution, calibrated tracked thresholds and sustained
idle-machine acceptance remain deferred to the whole source checkpoint.
