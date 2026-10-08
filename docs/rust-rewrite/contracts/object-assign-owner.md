# Object.assign compiler owner

The private `builtins/object/assign.rs` owner contains the complete native
Object.assign entry. The GC draft completes ToObject(target), then processes
sources in argument order, skipping null and undefined. Each source is boxed
once and supplies one own-key snapshot. Each key's current own descriptor is
acquired before Get; enumerable keys perform Get followed by Set with Throw=true
on the retained original target. String/Symbol identities and original abrupt
completions remain whole. Target setters and source getters remain observable.

The historical extraction checkpoint covered the earlier source. Current
source-spelling guards are retired. Existing semantic assign controls remain
required; this draft has no compile, emitted-Wasm or runtime acceptance. Finish
the complete dry source pass before verification under the confirmed 4096 MiB
aggregate cgroup cap.
