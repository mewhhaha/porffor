# NumberFormat Wasm consumer and primitive protocol

The NumberFormat family compiles JavaScript observation and object behavior to
Wasm. The pinned Rust provider receives only validated locale/configuration
records and observed primitive numeric inputs. It never receives a JavaScript
object, callback, source string to execute, or parser state.

## Construction and observation

`Intl.NumberFormat` allocates through `NewTarget.prototype` before it observes
locales or options. A move-only reserved-object token becomes publishable only
after the complete private record and brand are installed. Plain calls create
an ordinary NumberFormat in the called function's Realm. Legacy chaining is not
implemented: a supplied `this` object is never branded or given a fallback
symbol.

Canonical locale-list conversion precedes `CoerceOptionsToObject`. Undefined
creates a null-prototype options object, null rejects, and every other primitive
is boxed in the called function's Realm. Constructor and `supportedLocalesOf`
share this ordering. The constructor observes these 21 options once:

1. `localeMatcher`, `numberingSystem`, `style`, `currency`, `currencyDisplay`,
   `currencySign`, `unit`, `unitDisplay`, `notation`, `minimumIntegerDigits`;
2. raw `minimumFractionDigits`, `maximumFractionDigits`,
   `minimumSignificantDigits`, `maximumSignificantDigits`;
3. `roundingIncrement`, `roundingMode`, `roundingPriority`,
   `trailingZeroDisplay`;
4. `compactDisplay`, `useGrouping`, `signDisplay`.

The non-copy digit-observation state retains all four raw digit values until
the four rounding options have been read. Only selected significant/fraction
bounds are then converted, in specification order; an ignored bound cannot
invoke a conversion hook. Currency defaults come from the same pinned currency
fraction table as the provider. Inactive option words are cleared only after
all required observations and validation.

[NumberFormat construction](https://tc39.es/ecma402/#sec-intl.numberformat)
and [SetNumberFormatDigitOptions](https://tc39.es/ecma402/#sec-setnumberformatdigitoptions)
are the source of this observation and defaulting order.

## Exact inputs and Realm ownership

The scalar path performs one `ToPrimitive(number)` operation. Strings cross
as raw UTF-16 code units, including lone surrogates; they are never narrowed to
an IEEE Number. BigInts cross as exact intrinsic decimal text. Numbers cross
as intrinsic shortest decimal text, with negative zero represented separately.
Other primitive values take the canonical Number conversion path. Symbol
conversion, option errors, receiver errors and range NaN errors use the called
builtin's Realm. User throws, including `undefined`, retain their exact value.

Range methods validate their receiver and reject either undefined endpoint
before conversion. They observe the start and then the end. Both primitive
observations finish before the pure provider normalizes both endpoints and
rejects NaN. A NaN start cannot suppress a throwing end hook. The provider
handles descending ranges and its own pinned range-sharing policy.

The cached bound `format` function retains the NumberFormat object and is
created in the getter's Realm. It ignores its call receiver. Parts arrays,
part objects, resolved-options objects and supported-locale arrays belong to
the method's Realm. Every returned part property is writable, enumerable and
configurable, in `type`, `value`, optional `source` order.

`Number.prototype.toLocaleString` and `BigInt.prototype.toLocaleString` first
extract their branded numeric receiver, then use intrinsic NumberFormat
construction/getter/call operations in the called primitive method's Realm.
They ignore mutable public `Intl` and `format` properties. The BigInt result
policy retains its separate move-only locale authority; locale input can never
enter radix conversion.

## Closed record and wire

The 176-byte NumberFormat record owns five traced references: resolved locale,
data locale, numbering system, active currency/unit text, and cached bound
format. Seventeen untraced words follow. `NumberConfigurationWord` is the
closed ordering authority shared by the AOT record and provider codec. Its
projection and heap inventory pin all fields and the record extent. The
NumberFormat prototype is a distinct Realm intrinsic, populated by the shared
entry/created-Realm Intl installers.

Intl host ABI 5 reserves operations 10 through 13 for locale resolution,
supported locales, scalar parts and range parts. Number wire version 1 uses a
16-byte header (`u64` version, `u64` operation message), followed by checked
`u64` words and length-prefixed byte spans. Request/response messages use
`2 * operation` and `2 * operation + 1`. All spans fit the memory32 boundary.
Every decoder rejects trailing bytes, invalid closed codes, inactive nonzero
words, invalid precision/increment combinations, and forged locale/numbering
relationships. Range-only `approximatelySign` requires a shared source.

The host copies each complete request before entering the provider or writing
any response, so request/result overlap is valid. An insufficient result span
returns the exact required capacity without partial writes. AOT asks for that
capacity, allocates it, then requires an exact-sized response with the expected
header. Its response reader bounds each word, span and count against the
returned response extent. Only a typed NaN-range error becomes the public
`RangeError`; malformed private requests, missing capabilities and resource
failures remain explicit host errors.

The provider returns final localized parts. Wasm joins the same parts for
`format`/`formatRange` and does not translate digits or synthesize affixes a
second time. All provider locales, numbering systems and range policies remain
owned by the pinned NumberFormat provider and its generated data identity.

## Verification boundary

The staged native target `aot_intl_numberformat` contains 16 ordinary-source
controls for the complete family, all nine rounding modes, exact large
strings/BigInts, ignored digit coercions, both endpoint abrupt ordering,
foreign Realms, newTarget, parts descriptors and primitive delegates. Codec
controls round-trip every style/precision/notation shape, corrupt each option
word, retain raw UTF-16 and exercise exact range rejection. Host adapter
controls exercise all four operation markers, capacity queries, overlapping
spans, malformed requests and no partial writes. Existing created-Realm,
provider-import and BigInt ownership/CLI fixtures remain in the verification
inventory.

Source staging and formatting are not product verification. The coordinated
provider, registration and consumer batch must pass compilation, native/CLI
controls and the full pinned NumberFormat replay before any conformance count
is updated. No Test262 execution is skipped or marked passing by this contract.
