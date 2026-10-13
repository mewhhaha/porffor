# String character constructors use private UTF-16 construction

The shared Wasm owner for `String.fromCharCode` and `String.fromCodePoint`
converts each argument once, in left-to-right order. `fromCharCode` keeps exact
ToNumber/ToUint16 behavior. `fromCodePoint` checks integer/range validity before
advancing to the next argument. A conversion throw or RangeError prevents all
later conversions and retains its original value and defining-builtin Realm.

Each call starts one rooted unpublished `StringConstruction` with one code unit
of capacity per argument. BMP and lone-surrogate code points write one unit;
supplementary points write the high and low surrogate units. The point path
grows geometrically only when needed, copying the initialized prefix. Extents
are checked in I64 before narrowing to the selected GC runtime's signed I32
array extent; exceeding that infrastructure extent remains an uncatchable Wasm
resource rejection, consistent with other counted GC-string construction.

After conversion, char construction consumes its exact array; point
construction publishes only its exact used prefix. Both normal and abrupt
branches converge before consuming the private construction, so its typed root
is always cleared. Partial contents on an abrupt branch never become a normal
result. Arbitrary reentrant conversion hooks and explicit GC retain the private
backing and prior written units. Published String backing stays immutable.

The old path built one String per argument and concatenated it with the entire
accumulated result, repeatedly allocating and copying prefixes. The new path
performs a linear number of writes and bounded prefix copies. This is a general
builtin change, with no fixture, path, source-shape or Test262 special case.

Six paired strict/sloppy native controls cover empty/boundary/residue output,
live coercion order, exact thrown values and stopping, reentrant calls/GC,
borrowed error Realms and every unit of 10,000-argument mixed/BMP calls.
All earlier native String and pinned fromCharCode/fromCodePoint IDs must remain.
Generated Unicode RegExp property tests are useful performance witnesses
because their pinned harness constructs large code-point complements in
10,000-argument chunks. Measure the actual original case with unchanged limits
before claiming a speedup.

Actual `afba1a07a` passes fifteen complete optimized native scopes: 5,455 pass,
zero failures or ignores, all 5,449 prior IDs retained and six added controls.
All 29 focused controls pass. Both old and current workers pass the same 34
fromCharCode and 22 fromCodePoint execution IDs. Both unchanged Syriac modes
pass at 14.638/14.829 seconds, compared with the prior 31.912/34.974 seconds,
under concurrent cloud load and unchanged 60-second limits. The
[source-bound checkpoint](../string-character-buffer-checkpoint-20261013.json)
retains original bytes, pins, commands, worker/DSO identities and exact ID sets.
Full pinned String/task-wide closure remains pending.

The same verification batch adds producer phase timings for compiler
fingerprints, fresh runtime emission/admission, native-engine/cache setup,
manifest publication and object packaging. The prior warm producer spent about
42 seconds overall and about two seconds in native compilation; these timers
identify the remaining work without inferring its cost from source inspection.
They change build diagnostics only, preserving artifact and cache admission.

The actual producer records 6.521s fingerprints, 24.990s fresh emission/admission,
237.061s native compilation, 4.308s manifest publication and 1.230s packaging.
Its Cargo custom-build artifact has opt-level 0. Changed runtime Wasm requires
the observed module miss; the unchanged native dependency identity retains
2,155 compatible function stencils, with two misses. The full producer transcript
is retained before later Cargo builds can overwrite it. The workspace artifact
helper binds the successful exact package/target and single filenames member;
all 38 relevant Python controls pass on the accepted source.
