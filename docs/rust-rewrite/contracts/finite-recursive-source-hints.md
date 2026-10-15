# Finite source hints through recursive calls and loop bindings

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


The direct identifier head of a generic for-of loop receives optional source
candidates from its actual iterable. Array binding declarations with `var`,
`let` or `const` also project optional hints into their analyzed declaration
cells. Array candidates hold unordered possible members, not positional rows:
each supported leaf receives a bounded member union, including possible default
values. Nested array and array-rest binding patterns follow the same rule.
No row correlation or actual iterator value is inferred.

Projected text is supplemental and merges with the same cell's existing hints.
The loop may execute zero times, and earlier iterations may leave values behind;
iteration hints must not replace those prior sources. Existing primary sources
keep their priority, and overflow preserves the already retained set. Cells
with the same spelling in different analyzed environments remain separate.

Object patterns, property-access targets and elisions receive no leaf projection.
Assignment patterns are excluded, as are `var` patterns borrowing a sloppy direct
eval variable environment. A preceding `with` environment that can intercept a
reference suppresses that cell's projection. No binding facts are taken from a
pattern's private iteration temporary.

The live element ValueInfo remains Dynamic: user iterator methods can yield
unrelated values. Iterator calls, destructuring, defaults, binding writes,
per-iteration captures and effects still execute normally. Prepared eval and
constructor dispatch require the exact full live source and permitted context;
unlisted strings keep the existing typed Wasm-AOT rejection. Source discovery
retains the original 256-value limit; it cannot prepare every real source in a
257-value input.

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
existing IR/native inventories and exact diagnostics, including loop binding,
zero-iteration, primary-retention, iterator/default effects and unmatched-source
controls in both modes.


The first-argument source projection distinguishes Script compilation from
Realm module request discovery. Script compilation keeps supplemental conversion
views, including a known object ToString return, without changing its actual
argument or replacing its live conversion. Prepared source dispatch still checks
the resulting string and the captured callable's Realm. Eager Realm module
requests use primary text only; a supplemental conversion view grants no host
resolution authority.

The same projection admits every existing primary text before optional conversion
views. Across possible first-argument spread prefixes, a full 256-source primary
set leaves no spare conversion slot. Supplemental discovery cannot erase those
sources by overflowing their existing budget. The previous GC conversion,
original Throw, Symbol TypeError and captured-Realm control remains unchanged;
focused IR controls retain the forwarded and optional call routes, primary-only
module requests and the full 256-source Script boundary.
