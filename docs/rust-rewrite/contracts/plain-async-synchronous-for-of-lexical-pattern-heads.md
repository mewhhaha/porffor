# Plain-async synchronous `for-of` lexical pattern heads

## Boundary

A synchronous `for-of` with one direct body `await` in a plain async function
admits `let` and `const` array or object binding patterns. The admitted forms
include nested patterns, non-suspending defaults, rest elements and
properties, and empty array or object patterns.

`AsyncFunctionForOfIteratorPlanIr` receives a closed head witness and derives
where IteratorValue may be stored. A direct binding uses activation or
iteration-environment storage, a prepared assignment uses a compiler-only
entry local, and a lexical pattern pairs that entry local with an exact set of
iteration storage names, TDZ placeholder names, and BindingInitialization
statements. The public storage enum has exactly those three cases. Its
`EntryLocal` case has no source `BindingMode`; the plan retains the lexical
mode separately, so a `const` pattern cannot be mistaken for a mutable source
binding.

The plan constructor rejects incomplete or ambiguous witnesses. It checks
duplicate names and slots, exact iteration and TDZ name sets, slot bounds,
pattern mode, the shape and names of every initialization target, entry-local
name collisions, and whether empty patterns incorrectly claim an iteration
Environment Record. Invalid combinations cannot reach Wasm lowering.

## Environment and execution order

Capture analysis materializes the complete BoundNames set for this exact
plain-async loop shape, even when only one name is captured by a closure. Slot
assignment happens before capture hops are computed. The backend can therefore
create one complete fresh Environment Record for every successful iterator
step of a nonempty pattern and make it the async activation's current
environment before any head default or body statement runs. Both direct reads
after the await and closures retained after loop exhaustion observe that
iteration's cells.

Lowering predeclares every pattern name in its final iteration storage with an
uninitialized lifecycle before lowering any computed key or default. A forward
default such as `[first = later, later]` therefore reaches `later` in the same
iteration's TDZ instead of an outer or global binding. The separate head TDZ
Environment Record remains live for closures created while the iterable is
evaluated.

For each entry-state invocation, the backend performs these phases in order:

1. step the outer Iterator Record and stop if it is done;
2. for a nonempty pattern, create and publish the fresh iteration Environment
   Record and initialize all lexical cells to the uninitialized state;
3. open the outer IteratorClose frame;
4. write IteratorValue to the compiler-only entry local;
5. execute lexical BindingInitialization and the body prefix;
6. suspend and resume the body;
7. save the completion, restore the parent environment when one was created,
   and close the outer iterator when required.

An empty array pattern still runs the array destructuring protocol, including
its inner IteratorClose. An empty object pattern still performs the object
coercion and throws on `null` or `undefined`; neither is optimized into an
empty prefix. Both skip the iteration Environment Record phase because they
have no BoundNames.

## Abrupt completion

Computed keys, property reads, defaults, nested destructuring, rest copying,
and lexical initialization run inside the outer loop's close frame and before
the body. Array destructuring owns its inner IteratorClose independently. If
initialization throws, the inner iterator closes first when applicable and the
outer iterator closes next. Both preserving-Throw paths retain the original
initialization error if either `return` method also throws. A post-await write
to a `const` BoundName throws and closes the outer iterator through the same
completion path.

## Evidence

`wasm_plain_async_sync_for_of_lexical_pattern_heads.js` covers a complete
nested fresh environment, closures created before and after suspension,
uncaptured direct reads, mutable `let`, a computed object key, successful and
abrupt object-rest copying, a forward-default TDZ, a captured head TDZ,
post-await `const` assignment, nested and outer IteratorClose Throw precedence,
and semantic empty array and object patterns.

The focused IR tests pin the closed storage selection, complete environment
layout, lexical initialization ownership, forward-default TDZ lowering, and
empty-pattern semantics. The structure guard pins the exhaustive three-case
backend match, entry-local isolation, fresh-environment lookup, and fixture and
documentation ownership.

`cargo fmt --all -- --check` and
`cargo check -p lila-ir -p lila-aot-wasm -p lila-cli --all-targets` pass. The
IR `for_of` filter passes `27/27`, and its exact unsupported-shape rejection
witness passes `1/1`. The main structure target passes `6/6`; five affected
companions pass `22/22`, for `28/28` across six targets. The exact lexical
oracle and four retained iterator-record, member-head, nonlexical-pattern, and
captured-environment oracles pass `5/5`. The fixture passes `node --check` and
prints its success marker under the Node semantic baseline.

The pinned Test262 checkout contains no executable leaf combining a lexical
binding-pattern head with a directly awaiting plain-async synchronous
`for-of` body, so this fixture is source-free evidence and contributes no
Test262 numerator or denominator.

## Nonclaims

Resource patterns, suspension in the iterable or pattern, direct `break` or
`continue`, multiple or nested body suspensions, async-generator owners, and
`for await` remain outside this boundary. Body-local declaration shapes and
the inner array-destructuring protocol-error Realm policy are separate work.
