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

This batch is prepared in the next source checkout. Compilation, native
retention, pinned family replay and before/after timing are pending.

The same verification batch adds producer phase timings for compiler
fingerprints, fresh runtime emission/admission, native-engine/cache setup,
manifest publication and object packaging. The prior warm producer spent about
42 seconds overall and about two seconds in native compilation; these timers
identify the remaining work without inferring its cost from source inspection.
They change build diagnostics only, preserving artifact and cache admission.
