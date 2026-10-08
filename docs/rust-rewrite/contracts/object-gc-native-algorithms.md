# Native Object algorithms with semantic GC

Native Object algorithms use complete ValueLocals, CompletionLocals and typed
concrete GC references. Public builtins call the shared internal-method
and descriptor owners; recursive GetOwnProperty uses the closed private native
algorithm factory in the actual execution Realm. Public Object/Reflect
properties cannot replace an internal algorithm by mutation.

The Object constructor distinguishes its actual active callable from another
newTarget. A different newTarget supplies GetPrototypeFromConstructor and
ignores the value argument. The ordinary constructor boxes non-nullish values
and allocates a fresh default object for null/undefined in its defining Realm.
Object.create validates object/null prototypes before processing descriptors
and uses the same retained-partial-descriptor collection/apply owner as
Object.defineProperties.

Own names and symbols consume one complete key snapshot and filter its closed
key domain without coercion hooks. Fresh result arrays use a consuming private
publisher with the current function's Realm prototype and compact length.
SameValue receives whole values, preserving NaN, signed zero and GC identity.
IsExtensible returns false for primitives. PreventExtensions, Seal and Freeze
return primitives unchanged and otherwise use the same shared internal methods.
Seal sets configurable=false without acquiring each descriptor. Freeze obtains
each current descriptor and additionally sets writable=false for data records.
Whole exceptions and Normal false rejection remain distinct.

Object.prototype.toString handles null/undefined before ToObject. IsArray runs
first, including recursive Proxy/revocation behavior. Concrete private internal
slots select the default tag; callable proxies retain the callable tag without
inheriting other target slots. Get(@@toStringTag) remains observable, accepting
only String results. Immutable GC String concatenation copies exact UTF-16
units, including lone surrogates. Date.prototype has no Date value brand.

IsPrototypeOf checks a primitive argument before boxing this, then traverses
shared GetPrototypeOf with whole Throw propagation and reference SameValue.
ValueOf returns ToObject(this). Legacy prototype and accessor operations retain
their required coercion and invocation order in private owners.

Data-property definition, strict object writes, the complete [[Set]] dispatch
and receiver-side definition call the already registered ObjectDefineData,
ObjectWrite, OrdinarySet and OrdinarySetDataOnReceiver declarations. Their
arguments retain complete Value and checked String/Symbol key locals, Boolean
flags, the trusted caller Environment and the whole Completion result.
Constant definition flags are each produced and stored independently before
the call. Strict rejection is applied only to Normal false; an original Throw
passes through unchanged. The receiver remains the raw Reference receiver
while the lookup base is boxed according to the existing PutValue order.

Each helper compiler lives beside its module-private physical kernel and calls
that kernel directly. Public facades emit a runtime call. Prototype and nested
Proxy recursion reuse the registered declaration, and the receiver kernel
retains Proxy and Array/Arguments exotic dispatch. This boundary prevents an
outside object consumer from accidentally compiling the large private kernel
at every call site. It does not add a second representation or alter helper
registration, function indices, descriptor carriers or exotic algorithms.
In particular, array and object literal data definitions share one physical
ObjectDefineData body rather than duplicating its descriptor-carrier machinery
for every element.

The seven authored GC Object controls cover descriptor collection atomicity,
Proxy observation order, borrowed-Realm primitive receivers, constructor and
null-prototype creation, String/Symbol identities, brand/UTF-16 tag behavior,
integrity/prototype order, and the shared definition/Set boundaries. The new
boundary control observes literal descriptor flags and array holes, inherited
and Proxy receiver identity, partial descriptor presence, reentrant setter
ordering and original throws, strict/sloppy/Reflect rejection, integer-indexed
conversion counts and a foreign-Realm thrown identity. Existing semantic
fixtures remain required. Ten representation/spelling guards are retired
rather than rewritten as implementation mirrors.

The MAIN138 helper-sharing changes are authored source awaiting whole-batch
verification. No compilation, emitted-Wasm validation or runtime proof is
claimed for these changes. Final verification follows complete batch authoring
under a confirmed 4096 MiB aggregate cgroup-v2 cap, zero swap, whole-group OOM
termination and one worker. Published conformance counts remain unchanged.
