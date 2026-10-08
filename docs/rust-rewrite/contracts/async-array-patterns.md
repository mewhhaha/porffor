# Plain async Array patterns

Plain Async functions can give an Array binding or assignment pattern its own
Await continuation. The checked source owns acquisition, the complete pattern
body, its exact Await and iterator-operation tapes, and the IteratorClose exit.
It refuses Yield and does not claim an AsyncGenerator continuation. Existing
whole awaited initializers with eager patterns keep their original owner.

The complete IR carrier retains the original raw RHS and the actual
`ArrayIteratorStorageIr`, validates unique allocation rows and complete body
ranges, and binds every dedicated iterator-operation statement to its source
kind and state. Bare iterator operations remain inadmissible in generic async
sequences. Nested Array patterns own distinct real IteratorRecords and independent
operation tapes; all nested storage and result cells share the parent allocation
alias census.

Source lowering keeps the original lexical storage and TDZ obligations. Actual
target preparation and Put semantics have one private shared physical owner,
consumed by Generator and Async adapters. Identifier targets retain their original
Record or cell before IteratorStep and a default; member targets retain their raw
base and key, leaving the original Put owner to apply nullish checks, key
conversion, private brands and writes. Object recursion consumes the admitted
raw/boxed source, normalized key, GetV, Put and rest owners already used by
ordinary destructuring. Assignment returns the original whole RHS.

Defaults branch on the acquired value being Undefined. An Await-bearing default
uses the existing plain async If dispatcher with disjoint complete arm ranges;
defined values skip both Promise observation and the entire default prefix.
The scoped Pattern branch context is bound to the current loop depth and actual
checked pattern consumer. It does not grant general loop or mixed-generator
continuation admission. Computed names, target operands, defaults and nested
patterns use the existing admitted async expression prefix semantics in their
original order. Stale heap-shape facts are cleared across user code.

Native Generator and Async consumers share acquisition, the original Iterator
step/rest bodies and synchronous IteratorClose. Acquisition lies outside that
iterator's own close obligation. Fresh and resumed body entry reconstructs the
same close frame before injected completion; normal Await preserves the original
native iterator and captured Identifier edges. Own committed Return/Throw retires
abandoned Identifier references without clearing a calling activation. Step,
done and value protocol failures mark the original record Done; target, default
and Put failures close it. Incoming whole Throw wins an abrupt close, and nested
obligations unwind in their original order. No new GC layout or execution model
is introduced.

Meaningful source and real JS-to-Wasm controls cover complete ranges, distinct
nested iterator cells, lazy defaults and complete expression branches, preserved
eager awaited initializers, mixed-generator refusal, cached next/GetValue,
elision/rest, interleaved activations and GC, TDZ and inferred names, raw target
ordering, primitive errors, nested close precedence, private fields and caught
rejection followed by a fresh reference. They are authored source. Compilation,
runtime execution and the final joined verification checkpoint remain pending.
