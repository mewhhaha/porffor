# Uint8Array GC text codecs

This is source authoring in the atomic semantic GC draft. MAIN is unchanged.
Rust types, Wasm validation, fixture parsing and execution remain unverified;
all task source precedes checks. Later checks require the confirmed aggregate
4096 MiB cap, swap disabled, one worker and no uncapped fallback.

The six existing entries use GC UTF16 Strings and completed ByteArray prefix
records. Concrete Uint8Array admission rejects Proxies and other element kinds.
Writable-buffer admission precedes input and options; current bounds follow
ordered option Gets. Closed option, alphabet and final-chunk types keep invalid
domains out of decoding and encoding.

Decoding retains its whole SyntaxError separately from completed bytes. Static
methods throw before constructing a fresh typed array; mutating methods commit
the prefix before throwing. Methods consume the sole typed element bits owner
for shared and unshared Uint8 access. Static results use the saved defining-Realm
Uint8Array intrinsic; count objects and errors use the actual called Realm.
Normal paths clear temporary roots; an abrupt exit releases remaining
temporaries with the builtin Wasm activation.

Hex uses UTF16 parity and validates only the consumed prefix. Base64 retains
whitespace, alphabet, padding, final-bit, partial-chunk and capacity semantics.
The byte-list and String owners admit physical GC extents before index
conversion. Semantic references and source Strings remain real GC references.

Algorithms follow the [ECMA-262 Uint8Array methods](https://tc39.es/ecma262/multipage/indexed-collections.html#sec-uint8array-objects);
writable admission retains the already admitted immutable-buffer policy.
Six unrun paired controls cover byte values, prefix writes, UTF16 parity,
capacity counts, final chunks, option ordering and buffer resize/detach,
immutable receivers, shared bytes and borrowed Realms. Binary, native host,
sparse indexed storage and other families remain open. No new acceptance or
conformance counts are claimed.
