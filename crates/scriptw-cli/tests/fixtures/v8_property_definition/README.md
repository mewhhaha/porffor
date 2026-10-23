# Ordinary property definition controls

These five stock-JavaScript workloads isolate the proposed shared lookup/hash
path (P020) while retaining whole-invocation timing. Each performs 2,048
`Object.defineProperty` or `Reflect.defineProperty` operations. Timings include
key and descriptor setup, object construction, any seed population and checksums.
Long keys have a 2,048-code-unit prefix; the short-key control uses three units.

- `define-long-missing.js`: new keys, including the first empty-table insertion.
- `define-short-missing.js`: the same construction with short keys.
- `define-long-present.js`: existing keys; no missing-entry insertion benefit expected.
- `define-empty-rejected.js`: a nonextensible empty object; rejection must retain the no-hash shortcut.
- `define-long-reinsert.js`: delete and reinsert to retain tombstone and growth behavior.

The manifest gives exact stock Node stdout and can be passed to the existing
`runtime_baseline` example or `scripts/benchmark-v8.py --fixtures`. Compare frozen
ordinary release products with equal warmups/samples and retained raw observations.
These fixtures alone establish no performance gain. Semantic checks, including
key/descriptor callbacks and failed definitions, live in the CLI's
`wasm_prepared_property_hash.js` fixture.
