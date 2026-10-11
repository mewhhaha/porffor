# Raw Bytes Module data

An exact sole `type: "bytes"` import attribute selects a Bytes Module. The
host loader receives both the resolved locator and requested record kind.
It reads raw file bytes, including invalid UTF-8, NULs and empty payloads.
No JavaScript, JSON or UTF-8 parse processes Bytes data. The host's metadata
URL remains unchanged. Source Text, JSON and Bytes records at one locator
remain distinct; repeated imports and aliases share the selected record.

`ModuleRecordKey` carries locator and kind throughout discovery, admission and
linking. A closed synthetic-value enum owns the native JSON or Bytes default;
a record cannot carry both. Bytes records export only `default`, have no source
requests or top-level await, and retain exactly one default cell.

The shared embedded graph has a closed payload enum and fingerprints exact raw
bytes and kind. The differential wire retains existing Source Text/JSON rows;
a Bytes row owns a byte vector and must omit text. Missing, dual and wrong-kind
payloads reject before execution. The Test262 catalog reads binary fixtures and
includes exact Bytes resolution rows for computed fixture requests.

Native evaluation allocates an immutable, fixed ArrayBuffer and a Uint8Array in
the defining Module Realm using canonical intrinsics. Public constructors are
not consulted. The unpublished backing is initialized internally from checked
pooled static data in a constant-size loop, then published as the native default.
Both static and dynamic imports use the same initialized record. The optional
oracle owns context-local instances keyed by locator and kind in each real Realm.

Regression coverage includes raw binary and empty payloads, mixed locator kinds,
canonical aliases, fingerprints and wire round-trips, rejected named imports,
filesystem confinement, computed binary fixtures, canonical prototypes and
immutable-buffer mutation rejection. This implementation still requires actual
native and pinned-fixture acceptance; source/type checks alone do not establish
Test262 conformance.

The 2026-10-11 batch passes complete all-feature/all-target workspace types,
470 tooling controls and 27 prechecks. Eight idempotent Intl producers pass
24 actual generation/check invocations with all 482 inspected Intl files stable
on repeat. The optional native verifier can omit normal incremental scratch
while preserving compiled library reuse. Actual native, current pinned and full
acceptance remain pending. The [machine record](bytes-module-repairs-20261011.json)
retains the original timeouts, structural failure and unchanged size ceiling.
