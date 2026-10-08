# Actual conformance timings and selected data footprints

This source slice adds `lila performance conformance` and `lila performance data`.
It does not run a suite, benchmark, data generator or exporter. The code and its
controls are authored; compilation and execution belong to the combined batch
checkpoint. Existing generated README status counts are unchanged.

## Conformance evidence

`performance conformance --snapshot NAME --output-dir PATH` consumes the original
matrix snapshot admission. `--suite-root`, `--snapshot-dir` and
`--execution-backend wasm-aot|spec-exec` select existing evidence; Wasm-AOT is the
default. The output directory must be new. The SDK counterpart is
`ConformanceRunner::load_performance_evidence`.

The report preserves the recorded compiler, source pins, backend, aggregate and
node byte digests, completed/expected node counts, timeout execution IDs and the
retained slowest executions. Sloppy, strict and Module execution identities stay
distinct. Partial matrices remain explicitly partial. Timeout classifications
come from the original validated snapshots; this reporter does not reinterpret
failure strings or turn failures into passes.

Every newly completed matrix node records an actual monotonic interval around
`execute_cases` and result summarization. The JSON-encoded companion `.timing` carries
the exact terminal snapshot SHA-256, compiler/pin/node/backend binding, configured
workers, timeout, platform and whether resume was requested. Only a consumed
`NodeInvocationTimer` can mint a completed interval in production. Publication
uses a synced temporary file followed by rename.
The separate extension keeps timing records outside the original `.json`
snapshot discovery domain.

The interval includes resume and checkpoint work, and excludes suite discovery
and final snapshot publication. It measures the last completed invocation, not
the cumulative lifetime of a resumed node. A failed or interrupted invocation
does not mint a terminal timing. The report sums only retained invocation
intervals and reports how many nodes have them. A historical missing sidecar is
explicitly unavailable; a present sidecar with a wrong schema, identity or byte
binding is rejected. The report never sums selected slow-case durations and
calls that node wall time, and it is not an idle-machine throughput benchmark.

## Selected data bytes

`performance data --bundle PATH [--bundle PATH ...] --output-dir PATH` admits
1–16 existing exports sequentially through
`SelectedIntlDataBundle::from_export_bytes`. It retains the original bytes and
reports each actual typed component image's section, byte size and SHA-256,
alongside the exact provider identity and ICU/CLDR/Unicode/tzdb versions. The
same `component_images` owner feeds existing Wasm sections and bundle export.
Sparse bundles retain exactly their checked dependency closure.

Component bytes plus framing/identity bytes equal the admitted input size.
Named time zones and time-zone names have their own physical component rows.
Unicode data shared by several components is not assigned an invented disjoint
total. Native compiler tables, linked code size, heap use and process peak remain
explicitly outside this serialized-image measurement. There is no data export
or Collator generation in this reporting path.

An incomplete marker is written before reading inputs. `report.json` appears
only after every requested bundle is admitted and retained. Bad/truncated data,
excessive inputs, duplicate options and output reuse are errors. This is a
footprint comparison artifact, not new conformance or speed evidence.
