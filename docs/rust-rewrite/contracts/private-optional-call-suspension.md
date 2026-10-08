# Private optional calls across suspension

Plain async functions and ordinary generators admit a direct private Reference
as the initial Call base of a checked Property/Call optional tail. For example,
`object.#method?.(await argument)` and `object.#method?.(yield argument)` use the
same captured call Reference as an ordinary suspended private invocation.

The existing `capture_suspended_member` owner evaluates and pins the original
base, performs the private brand check and Get once, then pins the acquired
callee before arguments. Private getter effects invalidate the caller's value
and property facts at that point. The optional callee guard owns the complete
argument region: a nullish result skips every argument and suspension; a brand
or getter Throw occurs before that guard. A non-nullish non-callable result
still evaluates arguments before the Call reports TypeError.

The captured receiver and callee remain suspension-owned values. Argument
effects, resumptions and GC cannot cause a second private Get or select a new
receiver. Whole Throw completions from getters, arguments and injected generator
resumptions retain their value and existing finally handling.

Grouping consumes the actual terminal kind. A terminal Call publishes a Value,
so a following ordinary Call has no inherited receiver. A terminal public
property preserves its Reference and receiver. If the inner optional Call
short-circuits, the ordinary outer argument still runs before its Call fails.

Private Value bases followed by a Property link, such as
`object.#value?.method(await argument)`, now consume the same actual private
GetValue effect owner before retaining the returned Value. The existing async
and generator optional tail producers then preserve the returned property's
receiver independently of the initial private base. See
[the private GetValue contract](private-get-value-effects.md).

Private links within the optional tail (`object?.#method`), Super bases and
nonordinary suspension owners keep their existing explicit boundaries. The
ordinary-generator source owner still requires an eager base and each admitted
argument's checked plain-yield form. No second state graph or reference transport
is introduced for private optional calls.

The existing IR admission/refusal cohorts cover the new direct and parenthesized
private bases plus grouped Call/Property consumers while retaining the other
refusals. Three Engine cohorts exercise awaited references, yielded references
and grouped consumers in strict and sloppy scripts through Wasm AOT. They include
real GC between acquisition and Call, acquired Proxy callees, receiver identity,
getter effects, skipped arguments, brand failures and whole abrupt values. These
controls are authored source; compilation and execution remain pending for the
combined batch checkpoint.
