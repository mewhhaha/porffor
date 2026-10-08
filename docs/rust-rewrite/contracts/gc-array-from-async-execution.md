# Array.fromAsync GC continuation execution

The actual Standard builtin entry and both internal callbacks consume one
`ArrayFromAsyncState`. Sources, targets, mapper, receiver and saved errors are
complete StoredValue references. Capability and cached IteratorRecord references
survive native Promise jobs without address offsets or split value publication.
Source mode and stage operands use the closed GC scalar domains. Only Input,
Mapper, Iterator-result and Close Await transitions are installed by the native
algorithm. The two iterator-result property names share a closed Rust domain.

The defining builtin Realm supplies the intrinsic Promise before mapper or
input protocol observations. Mapper admission precedes GetMethod. Async then
sync iterator lookup retains original abrupt identity; GetIteratorFromMethod,
including one cached next Get, precedes target Construct. Array-like input owns
one ToObject and length snapshot before its length-bearing Construct. Default
ArrayCreate range failure rejects the intrinsic Promise inside the async
boundary. Target values remain generic; all indexed definitions and the final
strict length Set use the sole object authority.

Every input in the array-like route is awaited before its mapper and every
mapper result is awaited before definition or the next observation. Async
iterators await each next result, read done before value and retain unawaited
values when no mapper is supplied. Sync iterators consume the actual
Async-from-Sync Next/Return owner, including done-value Await and the underlying
close-on-value-rejection policy. Iterator protocol failures reject directly;
mapper and definition failures close. Close return and its result are awaited,
while the saved original Throw wins over lookup, call or Await failures. Final
length failure after done rejects without another close.

Eight authored paired Engine controls select actual Wasm-AOT execution and exact
printed completion markers: sequential whole values and mapper Await; cached
length and acquire-before-Construct; async value and done-property policy; sync
done-value and single close; mapper/definition closing and original errors; and
borrowed defining-Realm rejection; true async next/done/value abrupt cutoffs;
and synchronous PromiseResolve constructor-Get failures in mapper and iterator
Await setup. The last two distinguish close obligations without relying on a
source fingerprint. Missing async completion cannot pass these controls.
Existing meaningful CLI controls remain unchanged.

The six old source guards tied to raw state offsets, installer contexts,
normalized emitter fingerprints and token counts are retired. Their historical
contracts retain their dated original proof; it does not verify this GC source.
Compilation, Wasm validation and runtime controls are pending until the whole
task implementation checkpoint. Subsequent verification must use the confirmed
4 GiB aggregate cgroup limit and serial launcher. No conformance numbers or
full-task acceptance change.

The current algorithm was checked against the normative
[Array.fromAsync definition](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-array.fromasync).
