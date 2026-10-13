# Promise.try callback order and identity

The authority is ECMA-262 commit
`5345883164f463e87f8b40aca4956157ecba8783`,
[`sec-promise.try`](https://github.com/tc39/ecma262/blob/5345883164f463e87f8b40aca4956157ecba8783/spec.html#L51306)
and `sec-promise-resolve`. The downloaded pinned spec has SHA-256
`5194892f5a5f595962929b7099e028b1db44c8b68c44f4ce20f77b266507d57d`.

Promise.try first requires an Object receiver, then calls the live callback with
undefined this and the remaining argument List. A non-callable callback produces
an abrupt TypeError completion from the borrowed builtin's defining Realm.
On abrupt completion it constructs a capability, calls its reject function with
the callback's thrown value, and returns the capability's promise. On normal
completion it performs PromiseResolve with the receiver and returned value.

PromiseResolve reads an actual Promise's live constructor property and returns
that Promise when the constructor matches. No capability is constructed on that
path; even a non-constructor Object receiver can select it. Otherwise capability
construction and resolve follow the callback. Constructor getters, constructor
validation, user construction and resolve/reject throws retain their own ordering
and synchronous abrupt completion. The abstract operation does not read the
receiver's observable resolve method.

The frozen original compiler's complete 30-ID Promise.try family has 26 Success
and four Bugs: both modes of avoids-wrap.js and avoids-wrap-for-subclass.js.
Its raw snapshot is
`target/full-pin-optimized-4e01400fc-20261013/families/closure-2631633-1791860241099423743-0/first/closure-first-built-ins_Promise_try-12108905760811851745.json`,
SHA-256 `5e191ae23e8f7a90066187d45eb796c97752861ab70a59349e48a3c47f45691a`.
The Promise emitter is unchanged through accepted compiler `4a3968e5e`.

The repair shares static Promise settlement emission using already evaluated
constructor/value operands. Promise.try selects reject or PromiseResolve only
after the callback completion. Two existing native controls keep every input and
their original IDs, but correct stale expectations of eager constructor effects
and pre-callback validation of non-constructor Objects. Four new strict/sloppy
controls cover identity and observable resolve exclusion, both settlement paths,
live constructor Get and borrowed Realm errors, differing constructors, and
construction/settlement throws.

This source batch needs one optimized build, focused controls, the complete
original Promise.try family, retained original family pass IDs, affected native
inventories and prior constructor/Iterator/RegExp/ArrayBuffer fixes. Authorship
does not establish native or pinned acceptance. Full suites and task closure
remain separate gates.
