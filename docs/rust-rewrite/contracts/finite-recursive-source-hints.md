# Finite source hints through global recursive calls

Entering a function body replaces `visible_function_names` with that owner's
own declarations. Source discovery must still recognize an analyzed script-global
function declaration reached by a recursive or mutually recursive call. Otherwise
only externally supplied source arguments are observed, and a nested activation's
body string can select the genuine unsupported dynamic-source path.

Call-argument source discovery uses existing local/captured binding targets first.
Its additional global target comes from the script owner's analyzed function-binding
plan only when no local reference shadows it and neither a with chain nor a runtime
identifier environment can interpose. General finite callee-value and return-value
discovery remain unchanged. This is optional finite-source discovery, not an exact
function value or permission to call a declaration after mutation.

Actual argument observations retain their existing canonical FunctionId/position
keys, bounded union, deduplication and fixed-point propagation. Newly discovered
global string observations are supplemental, and optional structured overflow
preserves the existing admitted set instead of erasing its primary candidates. Local source hints
retain their environment/storage keys. Runtime arguments, conversions, function
calls and per-activation capture cells remain unchanged. Only actual call arguments
supply candidates; there is no function-name/source-shape exception, budget increase,
or runtime shortcut. The existing combined parameter/body/recursion control remains intact.

Acceptance requires actual Engine execution of both recursive closures in both
modes, mutual recursion with three independent captures, and retention of the
previous capture/shadow/assignment/conversion and 256-candidate controls. IR
controls require every real body and reject unused bodies from shadowed targets.


The direct identifier head of a generic for-of loop now receives optional
source candidates from its actual iterable. Its element ValueInfo remains
Dynamic: user iterator methods can yield unrelated values, and the ordinary
iterator and live constructor tuple admission remain in emitted IR. Pattern,
property, borrowed-eval and intercepted assignment heads do not acquire facts
from the private iteration temporary. Source discovery retains the original
256-value limit; a 257-value source remains outside finite preparation.

Supplemental global text priority also follows text nested in Record and Array
candidates, including later cross-owner rescans. A full primary set is admitted
before optional text-containing structures, so supplemental discovery cannot
consume the slots of those existing primary bodies. The original exact 256-body
regression remains unchanged and covers scalar and record source paths.


Record source candidates and their possible ToString text are two views of the
same observed object. The conversion view is supplemental. The actual750c
Front+IR diagnostic found no prepared property bodies for256 records even
without global supplemental discovery:256 Record views plus one primary
`[object Object]` conversion exceeded the unchanged256-value budget. A possible
conversion view must not consume a primary object's property-source slot.
Low-count object conversion preparation remains available and live ToString
still runs before exact tuple matching.

The same diagnostic found that257 `var body` iteration strings retain only the
older optional `undefined` hoist tuple; `let` and `const` retain no tuples.
A nonempty optional catalog does not admit any of those257 real source strings.
The original overflow control retains all input strings and the256 case, and
now asserts the unchanged256 catalog cap, at least one unmatched exact input
and the original dynamic constructor operand in the generic iterator body.
Paired native controls execute all257 original input values and require the
explicit unsupported dynamic-source boundary. That boundary may occur during
live tuple selection after compilation; it must not be replaced by whole-program
static rejection merely to make the catalog empty.

The retained native record control executes the first and last of the full256
property sources in both modes, including unused indirect supplemental record
input. Independent low-count custom conversion and default-object SyntaxError
checks preserve the converted-source paths. Acceptance requires the complete
existing IR/native inventories and exact diagnostics; private authoring ran no
Cargo, rustc or native execution.
