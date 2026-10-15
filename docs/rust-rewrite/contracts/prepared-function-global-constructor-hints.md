# Constructor grammar hints in independently prepared Function bodies

The finite Function family already compiles literal source tuples and checks the
actual constructor, arguments and defining Realm at runtime. A Script global
alias such as `var Factory = (function* () {}).constructor` can supply a generator
grammar hint within that Script. Previously, `Function("Factory('yield 1;')")`
started an independent analysis without that optional hint.

The compilation-local cache now attaches canonical constructor grammar kinds to
the exact retained `ParsedScriptIdentity` of an admitted Function source. The
export contains at most 256 `(global name, grammar kind)` pairs, derived only
from analyzed Script global object variable cells and already admitted child
hints. No parent FunctionId, environment ID, storage name, AST node, runtime
value or Realm identity crosses this seam. Global lexical aliases and caller
activation/eval variables are outside this initial discovery slice.

The child reads its own source-owned hint map. A separate discovery path accepts
only a bare global identifier with no local binding, local function declaration,
preceding `with` environment or runtime identifier environment. It reads existing
scalar source candidates without generic discovery that could itself parse
returned Realm callables. Candidate tuples retain the existing 256 bound; each optional tuple also has a
64 KiB source budget.

Before an added tuple is admitted, the exact Function grammar parses it through
a syntax-only cache path. Malformed JavaScript can contribute its deferred
SyntaxError. A successful parse is eligible only if its complete tree contains
no Call, SuperCall, New, ImportCall, tagged template or optional call. This conservative
domain cannot discover nested dynamic source, direct-eval private context,
Script import sites or eager Realm module requests. Rejected hints contribute
no executable unit or module request. A later ordinary admission of the same
cached parse still records its legitimate module requests.

Accepted tuples use the existing RuntimeCandidate compilation path. They do
not enter FiniteSourceValue, callable-target facts or runtime Reference lowering.
Global accessors, replacement functions, actual constructor kind, newTarget,
argument ToString and exceptions remain live. A foreign Function resolves its
own global environment and any actual dynamic constructor uses its own Realm;
the parent hint neither selects nor asserts either identity. A kind or source
outside the prepared tuples still reaches the typed capability rejection.
Function parsing never inherits the caller's private or direct-eval permissions.

The two generator runtime fixture wrappers request invalid generator parameter
and strict-with tuples, so they fit this slice. This static diagnosis did not
prove the original first runtime rejection or the absence of later fixture
gaps. All new IR, cache and Wasm Engine controls are prepared and unexecuted at
packet creation. No Test262 fixture, status count, execution deadline, cache
policy or candidate bound is changed.
