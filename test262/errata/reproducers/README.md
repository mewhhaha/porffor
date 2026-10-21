# Standalone conformance disagreement probes

These seven files print observations, not Test262 pass/fail results. Each must
run in a fresh Script/realm. They do not replace the original fixtures or their
harness, alter the product runtime, or count toward the conformance chart.

| File | Boundary |
| --- | --- |
| `implicit-arguments.js` | Existing implicit binding replaced by an eligible block function |
| `formal-arguments.js` | Explicit formal prevents that outer copy |
| `direct-label-block.js` | Immediate labelled function's lexical initialization and outer visibility |
| `chained-label-block.js` | Label-chain lexical collection and outer visibility |
| `labelled-switch.js` | Shared CaseBlock initialization and selected-declaration outer copy |
| `top-level-label-chain.js` | Separate top-level hoisting control |
| `johnston-identity.js` | Temporal availability, requested IDs, equality and shared epoch offset |

```sh
scriptw --jobs 1 run test262/errata/reproducers/chained-label-block.js
node -e 'globalThis.print=console.log; require("node:vm").runInThisContext(require("node:fs").readFileSync(process.argv[1], "utf8"))' \
  test262/errata/reproducers/chained-label-block.js
```

Scriptw prints program JSON followed by its `RunOutcome`; keep both in raw logs.
The recorded comparisons used fresh processes, PythonMonkey 1.4.1 with bundled
`libmozjs-136a1.so`, QuickJS Python binding 1.19.4, and Node 24.19.0 / V8
13.6.233.17-node.51. PythonMonkey/QuickJS are optional diagnostic oracles, not
product dependencies. Their package identities and transcripts are in the
[resolution evidence](../../../docs/rust-rewrite/final-four-resolution-20261021.md).

QuickJS differs on the implicit-arguments and top-level-hoisting controls.
Temporal is absent from all three installed comparison engines; record that as
unavailable rather than treating it as a timezone success.
