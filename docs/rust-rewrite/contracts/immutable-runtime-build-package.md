# Immutable runtime build package

The Cargo runtime-image producer obtains R exclusively through
`build_runtime_artifact_with_source_compilation_profile`. That constructor
emits R, encodes its package, fully decodes every Wasm body and the complete
typed layout, then requires exact key/layout parity before returning. The
producer compiles that returned Wasm and publishes its exact package alongside
a source/target/configuration/package/native manifest. The leaf image crate
embeds those package bytes in an immutable target array.

Engine's private `embedded_runtime::inputs` may therefore reuse that full body
validation through `unsafe with_immutable_build_package`. Its safety contract
requires exact successful producer output and immutable static bytes. Static
lifetime alone supplies no validation proof. No external slice, CLI path,
cache entry, re-encoded envelope or native image enters this boundary.

At use, canonical admission still checks the current Intl selection, source
and compilation profile, package framing and checksum, exact payload lengths,
pool boundaries, Wasm sections/types/imports/exports, Intl component frames,
core data sections, source-profile marker and reconstructed typed layout.
Component Wasm modules are rejected. Only the repeated
function-body pass uses the producer's prior result. Native-image admission
independently retains its manifest/digest/configuration checks and Wasmtime
compatibility checks. Runtime keys and serialized artifact formats are unchanged.

The safe `with_build_package` setter always restores full body validation,
including when replacing an earlier trusted candidate. All disk-cache paths
also validate every body. Rejected candidates use the ordinary checked cache
or emitter. Engine only supplies the immutable candidate to NativeHost
compilation; Portable export stays on its existing path. Unsupported/custom
identities still reject the canonical package and follow normal emission.
Candidate construction remains lazy for scalar programs.

Verification must compare the producer package's fully checked and trusted
admission results by exact Wasm bytes, runtime key and layout, and retain all
existing corruption controls. A replacement control re-signs an envelope
containing an invalid function opcode and requires the safe setter to reject
it after an earlier trusted candidate. Original cold deadlines, complete
affected scopes and fresh broad suites remain mandatory; this contract grants
no acceptance by itself.
