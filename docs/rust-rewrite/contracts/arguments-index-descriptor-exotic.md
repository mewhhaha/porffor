# Arguments indexed descriptors: ordinary application with one retained mapping

## Scope

This contract covers an Arguments object's indexed `[[GetOwnProperty]]`,
`[[DefineOwnProperty]]`, `[[Set]]`, and `[[Delete]]` seams. It is the Arguments
counterpart to the Array-index integration in
`property-descriptor-lattice.md`; named `length`, `callee`, iterator and
ordinary named-property behavior remain separate representation branches.

The normative basis is ECMA-262 10.4.4.1-5. An Arguments index owns an
ordinary stored descriptor and may additionally own a ParameterMap entry. The
mapping is not a descriptor kind or attribute. In this backend it is encoded
by two inseparable run-time facts in the descriptor word:

- bit 5 says that the index is mapped;
- bits 32-63 identify the environment slot to which it is mapped.

Treating the first fact as a Boolean and reconstructing the second after a
descriptor store is unsound: the store is permitted to replace the descriptor
word, so a later read can silently turn every nonzero parameter slot into slot
zero.

## Private lifecycle owner

`functions/arguments_index_mapping.rs` is the sole owner of the non-`Copy`
carrier and its capture, read, write, restore and consuming-release methods.
The parent declares the child privately and neither imports nor re-exports the
carrier. Callers use the existing inherent methods and infer the returned type;
only the child can construct the paired `mapped`/`slot` locals or inspect their
fields. This makes a mapping assembled from unrelated scratch locals a privacy
error instead of a convention violation.

## DefineOwnProperty protocol

The backend protocol is the specification order, with the invisible
ParameterMap represented by a typed local carrier.

1. Read the existing descriptor word and capture one
   `ArgumentsIndexMappingLocals` value containing both mapping presence and
   slot before any mutation.
2. Construct and validate one `WasmDescriptor`. Classification comes only from
   the canonical 6.2.6 descriptor lattice.
3. Project the current indexed property into `StoredDescriptorLocals`. A
   mapped data property observes its environment value; an accessor projection
   observes its stored getter and setter without calling either.
4. If the indexed own property is absent, reject the define when the Arguments
   object is non-extensible. This check also precedes every indexed or
   ParameterMap mutation.
5. For an existing descriptor, run the shared stored-descriptor compatibility
   validator. A failed define changes neither the indexed entry nor the
   ParameterMap.
6. Apply the ordinary descriptor to indexed storage.
7. Only after successful application, consume the original mapping fact:
   - an accessor descriptor detaches the mapping;
   - a data descriptor with `[[Value]]` updates the captured environment slot;
   - a data descriptor with `[[Writable]]: false` detaches the mapping;
   - otherwise the complete original mapping, including its slot, is restored;
   - a generic descriptor preserves the complete original mapping.

The special step-4 copy from 10.4.4.2 is observable when a mapped data
descriptor supplies `[[Writable]]: false` but omits `[[Value]]`: ordinary
storage must receive the current ParameterMap value before the mapping is
detached. Reading the effective current data value before application supplies
that value without manufacturing a second descriptor classification.

## GetOwnProperty and Set boundaries

Indexed `[[GetOwnProperty]]` first chooses the stored data/accessor kind. The
Arguments tag changes only where a mapped data value comes from; it must not
force an accessor entry through data-descriptor materialization. Getter and
setter identity are exposed without invocation.

Indexed `[[Set]]` retains the direct mapped/existing-own path. When no indexed
own descriptor exists, it must first walk the prototype chain with the
original Arguments receiver. An inherited setter handles the write, an
inherited non-writable descriptor blocks it, and only an otherwise unhandled
write reaches receiver-side indexed creation. That final creation returns
false for a non-extensible Arguments receiver rather than writing the entry.
The prototype scan branches on the prototype object's representation before
reading indexed storage: Array and Arguments entries share the descriptor
word, but their presence bounds are different. Treating an Arguments prototype
as an Array can therefore hide a real inherited descriptor.

An accessor setter may perform any ordinary ECMAScript write. A dynamically
tagged Arguments receiver must therefore enter an Arguments-aware `[[Set]]`
route before the ordinary-object heap layout is considered. That route first
honours an own named accessor or non-writable data descriptor, then an
inherited accessor or non-writable data descriptor, and creates a fresh own
data property only when neither blocks or handles the write. Own creation and
updates use the Arguments named-property table. Using `HEAP_PTR_OFFSET`/
`HEAP_LEN_OFFSET` as an ordinary property table for an Arguments value aliases
its indexed-entry buffer and is a representation error, not an acceptable
shortcut.

## Encoded invariants

- `ArgumentsIndexMappingLocals` is private-field, non-`Copy`, and
  `#[must_use]`. Its constructor extracts both presence and slot from the same
  pre-mutation descriptor word. ParameterMap reads/writes and mapping restore
  accept that carrier rather than an index whose slot they rediscover.
- The carrier literal and every `mapped`/`slot` projection remain in the private
  lifecycle child. The five capture sites, three reads, four writes, one restore
  and five consuming releases are the complete recursive caller census.
- The Arguments index define boundary accepts one validated `WasmDescriptor`;
  the former data/accessor helpers with positional presence Booleans do not
  exist.
- Compatibility goes through `StoredDescriptorLocals` and
  `emit_validate_stored_descriptor` before the first indexed or mapping store.
- An absent indexed descriptor plus the non-extensible Arguments flag rejects
  before the first indexed or mapping store.
- Mapping restore emits both bit 5 and the captured slot payload in one helper.
  No define path may OR `ARGUMENTS_DESCRIPTOR_MAPPED` by itself.
- Accessor descriptor materialization is selected by the stored descriptor
  kind for both Array and Arguments indexed entries.
- Dynamic Arguments named writes dispatch through ordinary `[[Set]]` semantics
  backed by Arguments/array-named storage. Own and inherited descriptor
  handling precede fresh creation, and no named scan uses the indexed buffer as
  an ordinary object property table.
- Prototype `[[Set]]` lookup recognizes both Array and Arguments indexed
  representations and selects the matching descriptor-presence reader before
  inspecting shared descriptor flags or raw accessor storage.
- Ordinary prototype mutation and observation preserve the Array-or-Arguments
  representation tag alongside the shared prototype payload; prototype-chain
  dispatch must not reconstruct an Arguments prototype as an Object.

## Durable witnesses

The structural regression pins the typed descriptor boundary, mapping carrier,
validation-before-mutation order, complete slot restoration, accessor
materialization, Arguments-aware ordinary named `[[Set]]` dispatch and the
absent-index extensibility guard. The exact 153 moved source lines retain
SHA-256
`1866bb0a7938406f35929397e94c201335092e48a0cd9f631e36803b30511195`;
the 158-line private child has SHA-256
`7d68d462b7a5419a1306d21cb0ddcef8ebcfc886a396199db2a1fb7a9a25fa43`.
The existing CLI Arguments fixture owns
consumer-level mapped/detached behavior, nonzero mapped slots,
deleted-index/accessor redefinition, own and inherited named setters,
non-writable named data, non-extensible absent-index rejection, and Arguments
objects used as indexed setter/non-writable prototypes.

This bounded lane does not claim complete special-property closure. Arguments
`length` and `callee` retain their separate write branches and remain follow-up
audit surfaces. `Symbol.isConcatSpreadable` boolean coercion and delete
semantics are likewise explicitly deferred rather than evidence supplied by
the ordinary named `[[Set]]` route.

The focused current-pin witnesses are
`built-ins/Object/defineProperties/15.2.3.7-6-a-279.js` and
`15.2.3.7-6-a-280.js`, in both sloppy and strict variants. They prove only
their two exact cases; they are not evidence for the complete Arguments or
Object descriptor subtrees.

This source-equivalent extraction uses only the focused structure target,
module-boundary and task-plan audits, scoped formatting and `git diff --check`.
The structure target passes `4/4`, and each dry audit is green.
The CLI witness, semantic golden, workspace compilation and broad suites remain
owned by the coordinated shared verification checkpoint.
