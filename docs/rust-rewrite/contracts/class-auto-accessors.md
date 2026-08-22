# Class auto-accessors: one descriptor pair, one hidden backing field

## Decision

An auto-accessor is one class element with two distinct effects:

1. class definition installs a generated getter/setter pair; and
2. instance or static initialization adds one fresh hidden private field which
   stores the value.

Lila must preserve both effects explicitly. It must not lower
`accessor x = value` to a public data field, infer it from source text, or
represent it as two unrelated ordinary methods plus a field whose relationship
is only conventional. Public/private and instance/static placement change the
installation targets, but not this descriptor-plus-backing invariant.

This is the bounded prerequisite for the auto-accessor part of T09. It does not
implement decorators or broaden the dynamic-source boundary.

## Local source and evidence boundary

The checked-in Test262 sources supply the locally pinned grammar text for
`FieldDefinition`:

```text
accessor [no LineTerminator here] ClassElementName Initializeropt
```

The exact generated sources are:

- `language/expressions/class/elements/syntax/valid/grammar-field-accessor.js`;
- `language/statements/class/elements/syntax/valid/grammar-field-accessor.js`;
- `language/expressions/class/elements/field-definition-accessor-no-line-terminator.js`; and
- `language/statements/class/elements/field-definition-accessor-no-line-terminator.js`.

The local semantic sources are
`staging/decorators/public-auto-accessor.js` and
`staging/decorators/private-auto-accessor.js`. They are proposal-feature tests:
their metadata names `decorators`, and neither file contains an actual `@`
decorator. `staging/decorators/accessor-as-identifier.js` is the adjacent
contextual-keyword control.

A focused Wasm-AOT probe reports the declaration grammar file as `0/2`, both
exact failures being Runtime/NotImplemented with detail `auto-accessor class
field`. The probe declares Test262 revision
`aa55200d1310384c5cf69ea95b2a2ecba457007b`, which is also the current vendored
Test262 suite-tree identity. The probe's compiler commit and executable predate
the current code head, so it selects this work but is not current-SHA completion
evidence.

The implementation sources corroborate the present boundary:

- `vendor/boa_ast-0.21.1/src/function/class.rs` has public instance/static
  `AccessorFieldDefinition` variants and retains decorator expressions;
- `vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`
  recognizes the no-LineTerminator contextual keyword, but currently maps
  private auto-accessors to ordinary private-field variants and loses their
  semantic kind;
- `crates/lila-ir/src/lowering.rs` rejects the two public variants as
  `auto-accessor class field` and cannot distinguish the private form; and
- `vendor/boa_engine-0.21.1/src/bytecompiler/class.rs` demonstrates the useful
  decomposition into a fresh private backing name plus generated functions.
  It is seam evidence, not a product-path oracle.

The relevant specification operations are `ClassDefinitionEvaluation`, class
field-definition evaluation, `InitializeInstanceElements`, `DefineField`,
`PrivateFieldAdd`, `PrivateGet`, and `PrivateSet`. The repository labels its
ECMA pin `ecma262-current-draft`; there is no separately versioned local
ECMA-262 checkout. Therefore the checked-in generated `info` blocks and exact
Test262 behavior are the revision-bearing authority for this contract. Any
decorator implementation must additionally pin the matching decorators
proposal algorithms instead of silently following a newer web draft.

## Semantic matrix

| Source element | Definition-time exposure | Initialization receiver | Hidden storage |
| --- | --- | --- | --- |
| `accessor x` | complete accessor descriptor on `C.prototype` | each constructed instance | fresh private field on that instance |
| `static accessor x` | complete accessor descriptor on `C` | `C` | fresh private field on `C` |
| `accessor #x` | private accessor entry for source name `#x` | each constructed instance | a second, unspellable private field on that instance |
| `static accessor #x` | private accessor entry for source name `#x` on `C` | `C` | a second, unspellable private field on `C` |

The generated getter performs `PrivateGet(this, backing)` and returns that
value. The generated setter performs `PrivateSet(this, backing, value)` and
returns `undefined`. The functions are strict, non-constructable class
accessors. They capture the class private environment; they do not close over
the constructor as a substitute for `this`.

That last rule is observable for static inheritance. Reading or writing an
inherited static public auto-accessor with a subclass receiver throws
`TypeError`, because the subclass lacks the base class's hidden static field.
The pinned public semantic source asserts this exact behavior. By contrast, a
derived instance receives its base-class backing fields during base instance
initialization, so an inherited instance accessor works on it.

## Descriptor and backing invariants

For a public auto-accessor, the installed own property is an accessor
descriptor with both `[[Get]]` and `[[Set]]`, `[[Enumerable]]: false`, and
`[[Configurable]]: true`. It has no `[[Value]]` or `[[Writable]]`. Instance
descriptors live on the prototype, not each instance; static descriptors live
on the constructor. The current backend's
`emit_object_define_accessor` already constructs this descriptor shape and
preserves the ordinary descriptor-merging rules.

For a private auto-accessor, the source private name denotes one private
accessor entry with a getter and setter. It creates no public property and has
no reflectable property descriptor. Its hidden backing is a separate private
field entry. In the existing five-row private-element protocol this means:

- the exposed private accessor uses a receiver `Brand` plus shared
  `GetterDefinition` and `SetterDefinition` rows; and
- the hidden storage uses a `Field(receiver, value)` row under a distinct
  private-name token.

Every auto-accessor element receives a fresh backing token for every class
evaluation. Duplicate public keys are allowed and still receive distinct
backings. Later definitions may overwrite either or both visible descriptor
halves according to ordinary `DefineProperty` ordering, but they do not delete
an earlier element's initializer or backing-field installation. The pinned
public semantic source covers duplicates and the three-way ordering between an
auto-accessor, an ordinary getter, and an ordinary setter for literal and
computed names.

A source private name and its hidden backing token must never compare equal.
The hidden token must also not appear in source private-name lookup, debugging
spelling, object properties, or the public class shape. A magic source-name
string is not an acceptable representation of a hidden token.

## Evaluation and initialization order

Computed public names are evaluated and converted to property keys exactly
once during class definition, in source order with other computed names. The
result is used to name and install the generated pair; it is not re-evaluated
per instance. Any abrupt completion stops class definition before static
initialization.

All method and accessor definitions are available before field initialization.
The two initialization schedules remain the schedules T09 already owns:

- instance private method/accessor brands are added first, then public fields,
  private fields, and auto-accessor backings run in class-element source order;
- a base constructor initializes these elements before its body, while a
  derived constructor does so after `super()` supplies its receiver and before
  the remainder of the derived body; and
- static fields, auto-accessor backings, and static blocks run on `C` in source
  order after definitions have been installed and the inner class binding is
  initialized.

For each auto-accessor initialization, the initializer is called with the
receiver as `this`; omission produces `undefined`. Only after that call
completes is the hidden private field added. Consequently, a read of the same
auto-accessor from inside its initializer observes a missing backing and throws
`TypeError`. If evaluation or private-field addition completes abruptly, the
remaining instance/static elements do not run and the original thrown value is
preserved. The pin's `nonextensible-applies-to-private` behavior also applies
to hidden backing addition: making the receiver non-extensible before the add
causes the same `TypeError` as an ordinary private field.

### Decorator boundary

The vendored AST retains decorator lists, but current Lila lowering and the
vendored execution path do not consume them. Supporting base auto-accessors
must therefore reject any non-empty decorator list at the lowering boundary;
silently emitting the undecorated pair is wrong.

When decorators become part of the selected conformance pin, their separate
lowering must preserve these phase boundaries:

1. decorator expressions and computed names evaluate in their specified
   class-definition order;
2. decorators receive the generated `{ get, set }` pair and may replace
   `get`, `set`, and the value-transforming `init` only according to the pinned
   accessor-decorator algorithm;
3. lowering stores replacement functions and initializer functions in their
   already-specified execution order; the backend does not reverse or compose
   them again;
4. the original initializer value passes through the decorator initializer
   chain before `PrivateFieldAdd`; and
5. element `addInitializer` callbacks run after that element's backing field
   has been added, at the corresponding instance or static point.

Decorator expression/call errors, invalid return shapes, and non-callable
replacement members stop class definition or element initialization at the
operation that produced them. Exact reversal/composition rules must be copied
from the revision selected at that time; this base contract deliberately does
not encode an unpinned decorator API into IR.

## Realm and error behavior

The generated getter, setter, and initializer functions are created in the
class evaluation realm and retain its lexical and private environments. Their
initial `[[Prototype]]` is that realm's `%Function.prototype%`. A getter or
setter borrowed through a descriptor remains callable from another realm, but
its private-name identity does not change.

Calling a generated accessor with a primitive, an unrelated object, or a
subclass constructor that lacks the hidden static backing throws `TypeError`
from the generated function's active realm. This is the existing
`PrivateGet`/`PrivateSet` wrong-receiver path, not an accessor-specific error
object. Cross-realm callers must not cause entry-realm TypeError allocation.
Initializer and decorator calls propagate arbitrary thrown values unchanged.

Duplicate public auto-accessor names are legal. A private auto-accessor counts
as the complete getter/setter use of its source private name, so another field,
method, getter, setter, auto-accessor, or static/instance declaration of that
same private name is an early `SyntaxError`. The parser must report that from
the class private-name declaration table, not from backend duplicate rows.

## Minimal typed front-end and IR seam

The first prerequisite is AST fidelity. Add private instance/static
auto-accessor variants to the vendored AST/parser, or an equally typed
`lila-front` normalization while the parser still knows the token kind. Do not
recover private auto-accessors by inspecting source text after parsing.

The class private environment must then distinguish two domains:

- source-visible private-name spelling to `PrivateNameId`; and
- all allocated private slots, including unspellable auto-accessor backings.

Today `ClassDefinitionIr::private_name_ids` serves both lookup and allocation
size. The minimal replacement is a class-private-environment plan carrying the
visible map and the total slot domain. A private `AutoAccessorBackingNameIr`
constructor mints a token in that class scope without inserting a fake key into
the visible map.

The semantic IR needs one canonical auto-accessor record, referenced from both
phases, with exactly these facts:

- public property key or exposed source private name;
- instance/static placement;
- fresh `AutoAccessorBackingNameIr`;
- generated getter and setter function identities as a required pair; and
- optional field-initializer function identity.

`ClassElementDefinitionIr` gains a closed auto-accessor definition variant.
The instance initialization plan and `ClassStaticElementIr` gain a closed
auto-accessor-backing variant. A private ID/index into one plan-owned table is
preferable to copying these facts into three independently constructible
records: lowering creates one semantic element, and exhaustive consumers see
its definition and exactly one placement-appropriate initialization event.

The generated pair should reuse `FunctionProtocolIr::{ClassGetter,
ClassSetter}` and ordinary private-read/write IR bodies. A small typed factory
must accept only an `AutoAccessorBackingNameIr` and return the required pair;
there is no need for an auto-accessor-only call ABI or backend opcode. This
keeps realm, strict-`this`, function allocation, throw propagation, and private
environment capture on the existing function path.

## Backend consumption

`compile_class_definition_payload` consumes the new definition variant in
element order:

- evaluate/cache the public key once when needed;
- materialize both generated functions with the current class private
  environment and placement target;
- call `emit_object_define_accessor` once with both functions for public
  exposure; or
- add the private getter and setter definition rows for private exposure.

Static private accessor brands are installed on `C` with other static private
method/accessor brands. Instance private accessor brands remain in
`emit_initialize_instance_elements`' first phase.

The initialization variant calls the existing class-field initializer path and
then `emit_private_field_add` with only the backing token. It must never call
`emit_object_define_enumerable_data`. Public key caches are definition data;
the backing initialization event does not need or receive the public key.

All matches over `ClassElementDefinitionIr`, instance initialization elements,
`ClassStaticElementIr`, class statistics, and computed-key/context allocation
must be exhaustive. An unsupported decorator is rejected before any of these
records exist.

## Staged implementation and acceptance

1. **Parser fidelity and early errors.** Preserve public/private and
   instance/static auto-accessor kinds, the no-LineTerminator contextual-keyword
   rule, escapes, and decorator lists. Keep `accessor` valid as an ordinary
   identifier/field/method name where the grammar requires it. Reject every
   duplicate-source-private-name combination.
2. **Closed IR construction.** Split visible private names from total private
   slots, mint typed backing names, generate the getter/setter pair, and emit
   linked definition/initialization events. Add focused IR invariants proving
   freshness, pairing, placement, and absence from source lookup.
3. **Public instance/static backend.** Install complete descriptors and hidden
   fields. Cover literal, string, numeric, computed-string, and Symbol keys;
   undefined/explicit initializers; detached calls; descriptor attributes;
   duplicate/ordinary-accessor overwrite order; inheritance; and abrupt
   completion.
4. **Private instance/static backend.** Install the exposed accessor brand and
   paired definitions plus the distinct backing field. Cover reads/writes,
   multiple elements, wrong receivers, non-extensible receivers, inheritance,
   and static initialization.
5. **Realm fixture.** Use a precompiled created-realm class and borrowed
   descriptor functions to pin function-prototype identity and defining-realm
   `TypeError`. No auto-accessor-specific cross-realm Test262 file exists in
   the local seven-file corpus, so this fixture is load-bearing.
6. **Decorators only at their own pin gate.** Until then, retain the explicit
   unsupported result for non-empty decorator lists. When enabled, add direct
   fixtures for expression/application order, pair replacement, `init`,
   `addInitializer`, invalid returns, and abrupt completion before claiming the
   proposal feature.

The local seven-file evidence corpus has two different gates. The raw green set
is the four generated `language/{expressions,statements}/class/elements` files
listed above plus `staging/decorators/accessor-as-identifier.js`. The public and
private staging semantic files are diagnostics, not raw acceptance gates:
`staging/decorators/public-auto-accessor.js` and
`staging/decorators/private-auto-accessor.js` necessarily execute literal
`eval` for inherited-static failures or duplicate-private early errors. They
must reach the explicit dynamic-code-generation boundary rather than being
silently skipped or counted as auto-accessor green. Durable static-source
fixtures must cover all of their auto-accessor semantics, including static
rewrites of the eval-only assertions, before the backend slice is called green.

After implementation, run the focused green gates in this order:

```sh
cargo test -p lila-ir auto_accessor --quiet
cargo test -p lila-aot-wasm auto_accessor --quiet
cargo test -p lila-cli wasm_class_auto_accessor --quiet
./target/debug/lila test262 run language/expressions/class/elements/syntax/valid/grammar-field-accessor.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/class/elements/syntax/valid/grammar-field-accessor.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/expressions/class/elements/field-definition-accessor-no-line-terminator.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run language/statements/class/elements/field-definition-accessor-no-line-terminator.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run staging/decorators/accessor-as-identifier.js --execution-backend wasm --timeout-ms 180000 --threads 1
```

Run the two semantic sources separately as diagnostics. Their expected terminal
classification is the explicit Wasm-AOT dynamic-source boundary, not success:

```sh
./target/debug/lila test262 run staging/decorators/public-auto-accessor.js --execution-backend wasm --timeout-ms 180000 --threads 1
./target/debug/lila test262 run staging/decorators/private-auto-accessor.js --execution-backend wasm --timeout-ms 180000 --threads 1
```

Then run T09's existing `function_`, `wasm_function`, and `wasm_class` gates and
the complete current-pin class/private-element filters. Five raw files, two
diagnostics and durable static fixtures are not full T09 or full Test262
evidence.

## Nonclaims

This contract does not implement code, bless the older `0/2` artifact as
current-SHA evidence, define an unpinned decorator runtime API, make dynamic
`eval` available in emitted Wasm, or close the complete class/function/private
element task. It does not replace the private-element five-row protocol or the
closed function protocol; it composes the new class element from them.
