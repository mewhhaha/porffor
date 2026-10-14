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
