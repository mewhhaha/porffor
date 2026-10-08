# Array iterator receiver policy

Status: typed GC source migration; compilation and execution pending.

The six Array/TypedArray keys, values and entries entry points retain their
separate generic Array-like and validated TypedArray admission policies. Native
entry owns receiver conversion and initial validation. Iterator creation takes
an already retained whole Object value or typed `GcLocal<TypedArrayObject>`;
it does not reconstruct a receiver from a payload or public named properties.

`emit_array_iterator_next_from_locals` is the common next entry. It checks the
concrete Array or TypedArray iterator GC record without calling receiver traps.
A generic Array iterator holding a TypedArray also takes the same live typed
buffer validation before comparing its index with the current length. Ordinary
Array-like receivers perform Get(length) then ToLength on every active step.
A terminal iterator performs neither lookup nor backing-store validation.

The index advances before an observable element Get. A getter throw preserves
the original whole completion and leaves the index advanced; a reentrant call
therefore observes the successor. Key iteration performs no element Get.
Entries use the real List-to-Array producer, without species or JS-visible
internal properties. Fresh IteratorResult allocation uses the executing
builtin Realm's Object prototype.

The old `array_iterator_receiver_policy_structure.rs` spelling/hash mirror is
retired with the raw representation. Existing CLI producer controls remain.
The new `aot_gc_iterator_entries.rs` control includes a token-throwing Array
getter, exact successive index observation, and live/done TypedArray cases.
It is authored in both script modes and remains unrun during dry coding.

Earlier Batch AE and the 2026-08-29 raw-source resizable iterator results are
historical predecessor evidence; they do not verify this GC source. This
migration publishes no conformance counts and does not complete T15/T16/T17.
The full atomic GC batch must be compiled and exercised after source authoring.

The current [Array iterator algorithms](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-createarrayiterator)
use direct internal slots. They require no additional closure execution state
for these native next methods.
