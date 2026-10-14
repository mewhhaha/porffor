# Strict callable binding early errors

ECMA-262 10.2.1 makes the name and parameters of a function strict mode code
when its FunctionBody contains a Use Strict Directive. Identifier early errors
in 12.1.1 therefore reject the strict reserved spellings in that header, even
though the parser reads it before the directive.

Pinned Boa 0.21.1 performs the late eval/arguments and duplicate-parameter
checks but omits these strict reserved identifier checks. Front validates the
successful AST before publishing ParsedScript or ParsedModule. The visitor
checks declaration names, explicit expression binding names and BoundNames of
parameters for every ordinary, generator, async, async-generator, arrow and
object-method form. It uses Boa's RESERVED_IDENTIFIERS_STRICT decoded symbols;
Unicode escape spelling cannot change the decision. Existing parser rejection
order, contextual Yield/Await grammar and non-simple-parameter rules remain
owned by the parser.

An inferred function display name, method property key or parameter initializer
is not a header binding. The binding visitor excludes property names and
expressions, while the callable visitor independently visits nested functions
there. Sloppy generator declaration names and ordinary nested function scopes
remain admitted unless that callable introduces its own strict directive.

The dedicated StrictModeReservedBindingIdentifier code is appended to the
closed EarlyErrorCode domain to preserve all prior discriminants. Its typed AST
witness reports Early/SyntaxError independently of message text. The IR
exhaustive phase map and P7-P9 assertions include it. Host UTF-8 diagnostics and
ECMAScript UTF-16 diagnostics use the same actual parser Position conversion
as existing parse failures, including CRLF, astral code points, unpaired
surrogates in comments and literal transport-marker characters.

All Script/Module, eval UTF-8/UTF-16 and combined CreateDynamicFunction parse
routes run this check. No runtime parser, source-name switch, source-shape
exception or dynamic-source admission budget is involved.

The unchanged pinned generator syntax fixture has thirteen invalid strings.
The frozen-source pure Front diagnostic rejects twelve but accepts only its
line-91 strict generator declaration named yield; all five original sloppy
controls and its entire outer fixture parse. The private regression target
covers those exact inputs as well as the general callable/binding matrix and
source-position controls. Native and broader conformance verification remains
pending until the repaired source is compiled; no aggregate status is claimed.
