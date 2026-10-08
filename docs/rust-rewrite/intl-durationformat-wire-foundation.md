# DurationFormat primitive wire foundation

This packet is source-only. Its twelve native regression declarations have not
been executed. The coherent native/host successor imports the immutable Duration44 and wire7
new files, registers the native module and global operations36–38, and installs
the checked provider and real Wasmtime host dispatch. Compiler builtin and CLI
capability admission require the complete separately joined AOT lane. All 111 pinned DurationFormat
files and their 222 modes remain mandatory and unexecuted for this feature.

All integers below are unsigned 64-bit little-endian words. Text is one byte
length word followed by exactly that many UTF-8 bytes. Lists are one count word
followed by their entries. The complete frame is bounded by the Wasm32 address
domain; strings, cumulative localized output, locale counts and part counts
also obey the supplied checked `PartitionLimits`. There are no JavaScript
objects, pointers, UTF-16 object-property names or allocation handles in a frame.

The first two words are version1 and operation times2 plus direction. Direction0
is request; direction1 is response. Service-local tags36,37,38 retain the exact
foundation `DurationHostOp` domain. Segmenter/global0 through35 are unchanged.

| Operation | Request body | Response body |
| --- | --- | --- |
| 36 ResolveDurationFormatLocale | canonical locale list; matcher; numbering presence0/1; optional numbering text | resolved canonical locale text; actual numbering-system text; checked TwoDigitHours Boolean0/1 |
| 37 SupportedDurationFormatLocales | canonical locale list; matcher | supported original canonical locale list in requested order, retaining duplicates |
| 38 PartitionDurationFormat | resolved locale text; actual numbering text; twelve checked configuration words; ten completed Number bits | count; repeated NumberPartKind word, localized text, unit presence0/1 and optional unit word |

Matchers keep the existing checked NumberFormat codes: Lookup1, BestFit2.
Unsupported well-formed numbering options in operation36 retain normal locale
negotiation behavior. Operation38 accepts only the exact resolved locale and
numbering pair admitted by DurationProfiles through its genuine NumberProfiles
owner. Both locale-response operations are bound to the complete original
request; response decoding rejects invented, omitted, reordered or duplicated
supported entries beyond what that request actually permits.

`DurationConfigurationWord::ALL` fixes twelve hidden option words, 96bytes:
Style0, FractionalDigits1, Years2, Months3, Weeks4, Days5, Hours6, Minutes7,
Seconds8, Milliseconds9, Microseconds10, Nanoseconds11. Word offsets are index
times8. Global style codes are Long0, Short1, Narrow2, Digital3. FractionalDigits0
means absent;1 through10 represent values0 through9. Each unit word is its
effective style code in bits0 through7 and display in bit8; all higher bits are
zero. Unit styles are Long0, Short1, Narrow2, Numeric3, TwoDigit4. Displays are
Auto0 and Always1. Missing options have already been resolved by the checked
native configuration; these are effective words, not option-presence bits.
Decoding reconstructs `CheckedDurationConfiguration` once and requires its
canonical effective words to equal the frame. Closed but illegal combinations
return the typed semantic rejection `InvalidOptions`; unknown codes, reserved
bits and a noncanonical effective proof are frame faults.

The hidden completed-record span is ten words, 80bytes, in native record order:
years0, months8, weeks16, days24, hours32, minutes40, seconds48, milliseconds56,
microseconds64, nanoseconds72. Each word is the original IEEE-754 binary64 bit
pattern. `DurationPartitionRequest` stores both that immutable completed bit
sequence and the sole checked `DurationRecord` minted from it. This preserves
negative-zero transport without confusing it with the duration sign: all
negative-zero fields have neutral sign; a negative nonzero duration can still
emit a signed zero in its first displayed unit. `into_native` moves the existing
checked owners into the typed native request and never revalidates a record.

Part unit codes are Year0, Month1, Week2, Day3, Hour4, Minute5, Second6,
Millisecond7, Microsecond8, Nanosecond9. NumberPartKind keeps the existing common
wire domain. Duration output accepts literal, integer, group, decimal, fraction,
minusSign and unit; all nonliteral parts require a unit. Unitless parts are
genuine list/digital literals. The checked response owner preserves native text,
unit grouping order and at most one duration sign. Normal `format` concatenates
these exact texts; `formatToParts` materializes each kind/value and includes the
unit property only when present. The decoder does not synthesize labels,
substitute digits, choose locale punctuation or use an output oracle. Zero
parts remain legal when a checked all-auto configuration omits all zero fields.

The byte boundary reports `Malformed` for framing/domain/proof faults,
`Rejected(DurationError)` for an otherwise complete primitive semantic input,
and `Resource` for its own frame allocation/count/extent limit. Native Number/
List errors remain typed inside `DurationError`; host admission must preserve
these distinctions rather than converting every fault into a user RangeError.
The complete partition frame is consumed before aggregate semantic admission,
so a truncated/trailing frame never escapes with a partial record.

The future AOT producer owns all observable coercion and abrupt completion.
For an ordinary duration bag, the 2025 ToDurationRecord order is days, hours,
microseconds, milliseconds, minutes, months, nanoseconds, seconds, weeks,
years. Each Get is followed immediately by ToNumber and the integral check when
the value is present, before the next Get. Missing fields become zero, and an
all-missing bag throws TypeError before any host operation. A fractional,
infinite or coercion-throwing field stops that sequence immediately. Only after
all fields complete are uniform sign and the normative magnitude bounds
checked. Year/month/week magnitudes must be less than2^32; exact normalized
day/time seconds, including all subseconds, must be less than2^53. The checked
native record uses exact integer nanoseconds and does not add binary64 values.
The branded TemporalDuration path must reuse its completed stored-field proof
without invoking user getters. The wire packet does not implement a JavaScript
interpreter, a bag reader or a deferred per-field error path. These ordering and
bound rules follow the [2025 ToDurationRecord and IsValidDuration algorithms](https://tc39.es/ecma402/2025/#sec-todurationrecord).

For the future borrowed Wasm request proof, the compiler must bind the resolved
locale strings, the fixed96-byte option span and fixed80-byte completed-record
span to the same live duration owner before computing any size or writing.
Copying the completed host input must finish before native processing or output
writes; capacity, address overflow and input/output overlap need meaningful host
regressions in the eventual full batch. Every output allocation and function/
prototype/result object belongs to the called Realm. This packet supplies no
Realm or heap integration and cannot establish those properties alone.

The successor corrected one static signature defect found in the immutable
foundation: `duration_protocol.rs` currently passes
`&requested` to `DurationProfiles::supported_locales`, whose actual signature
takes `DurationSupportedLocalesRequest` by value. The successor uses
`profiles.supported_locales(requested)`. The original sealed44 files remain unchanged.
The new codecs use the actual by-value signature. The generated Duration
identity now binds all five production codec leaves, the registered native
global/provider source and actual host. Root must join the real AOT callers and
execute the native/host regressions and all pinned modes before product admission.
The locale response carries its genuine checked TwoDigitHours Boolean and
rejects a mismatched Boolean proof. Native option validation applies the
unconditional hours width override only after validating the previous style,
then validates each subsequent unit in order. All fifteen captured CLDR47 rows
currently have this flagfalse; the normative true branch is tested directly as
a closed algorithm input without forging a locale profile.
