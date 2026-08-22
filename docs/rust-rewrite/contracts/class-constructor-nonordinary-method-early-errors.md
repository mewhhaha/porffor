# Contract: remaining class-constructor method and private-name early errors

**Status:** Normative T07 extension, 2026-08-20

## Spec invariant

The ClassElement early-error rules reject four constructor-shaped forms that
are distinct from an ordinary non-static constructor definition:

- a non-static async method whose literal property name is `"constructor"`;
- a non-static getter whose literal property name is `"constructor"`;
- a non-static setter whose literal property name is `"constructor"`; and
- any private ClassElement whose PrivateIdentifier is `#constructor`.

Each condition is decided before evaluation and produces a `SyntaxError` under
both Script and Module goals. They remain separate diagnostic conditions: an
async method, getter, setter and forbidden private name are different grammar
boundaries even though all reject a class element associated with the word
`constructor`.

Static public async methods and accessors named `constructor` are not
constructor definitions. Public methods and accessors with a computed name
whose value is `"constructor"` likewise have no syntactic PropName equal to
`"constructor"`. A computed string name `"#constructor"` is not a
PrivateIdentifier. All of those boundaries remain valid.

## Measured Boa boundary

Pinned `boa_parser-0.21.1` emits four exact, case-sensitive literals from
`vendor/boa_parser-0.21.1/src/parser/statement/declaration/hoistable/class_decl/mod.rs`:

| Condition | Complete literal | Producer sites |
| --- | --- | ---: |
| private `#constructor` ClassElement | `class constructor may not be a private method` | 7 |
| non-static async constructor method | `class constructor may not be an async method` | 1 |
| non-static constructor getter | `class constructor may not be a getter method` | 1 |
| non-static constructor setter | `class constructor may not be a setter method` | 1 |

The private-name producers cover fields, ordinary methods, generator methods,
async methods and async-generator methods across static and non-static parser
branches. They all enforce the same forbidden PrivateIdentifier condition and
therefore share one diagnostic code. The other three complete literals each
have one producer and one code. None is matched by the existing generator-
method row.

Before this extension, the one classifier had no matching row for any of these
messages. Entry parsing therefore returned `P_PARSE_MALFORMED`; the same
retained failure from a dependency Module became an `Unsupported` IR
diagnostic with no native error type. Neither result represented the specified
early `SyntaxError`.

## Encoding

- Add four `EarlyErrorCode` variants with one wire spelling each:
  `ClassConstructorAsyncMethod`, `ClassConstructorGetter`,
  `ClassConstructorSetter` and `ClassPrivateConstructorName`.
- Add exactly four parse-failure rows. Each row uses its complete Boa literal
  as both fragment and witness, so the four adjacent conditions cannot shadow
  one another or the existing generator condition.
- Map all four variants through `lila-ir`'s exhaustive `rejection_kind` match
  to `IrDiagnosticKind::EarlyError`. Phase and error type remain derived as
  `Early` and `SyntaxError` at both front-end and retained-module boundaries.
- Preserve `ParseClassified` as the only parse-stage gate and retain the
  parse-once ownership path. No parser or classifier copy is introduced.

The const-sized closed domain grows from 24 to **28** variants and the one
parse-failure table grows from 22 to **26** rows. Existing const assertions
continue to encode table population, exact witness ownership, wire-name
closure, classifier reachability, interpolation-guard separation and
parse-to-IR phase consistency.

## Durable regressions

Front-end source tests require Script and Module goals to reject declaration
and expression forms for all four conditions. Every rejection carries its
specific code, early phase, `SyntaxError` and a source span. Positive witnesses
preserve static and computed public async methods/accessors named
`constructor`, plus a computed public field named `"#constructor"`.

The IR regression parses one real Module source for each condition and sends
the retained `ParseError` through `module_parse_failure_diagnostic`. The
message-boundary table test independently fixes every complete literal-to-code
mapping.

## Pinned conformance evidence

Pinned Test262 revision `aa55200d1310384c5cf69ea95b2a2ecba457007b`
contains 18 direct `phase: parse`, `type: SyntaxError` witnesses: declaration
and expression pairs for the async, getter and setter constructor cases, plus
six private-`#constructor` shapes in each class form. They live under:

- `language/{expressions,statements}/class/elements/syntax/early-errors/grammar-special-meth-ctor-{async-meth,get,set}.js`;
- `language/{expressions,statements}/class/elements/syntax/early-errors/grammar-{,static-}privatename-constructor.js`; and
- `language/{expressions,statements}/class/elements/syntax/early-errors/grammar-static-private-{meth,async-meth,gen-meth,async-gen-meth}-constructor.js`.

The adjacent `syntax/valid/grammar-static-ctor-{async-meth,accessor-meth}-valid.js`
files preserve the static public boundaries for both class forms. These
fixtures establish the classification boundary, not a current Wasm-AOT pass
claim.

## Deliberate separations and nonclaims

This extension classifies parser rejections Boa already produces. It does not
implement async execution, class construction, method/accessor installation,
private-element storage or lowering. It does not combine these conditions with
duplicate or generator constructors, close the class parser bucket, complete
T07, refresh a snapshot or change a published count.

No Test262 file or CLI fixture is added. The focused front and retained-module
tests pass, and the complete adjacent expression and statement early-error
subtrees each report `444/444` under Wasm-AOT at the harness-declared
`aa55200d1310384c5cf69ea95b2a2ecba457007b` pin. These results prove the
bounded classification and adjacent parser surface, not the full language
tree, T07 closure or a current aggregate publication.
