# DisplayNames and RelativeTimeFormat provider integration

The source batch connects the six primitive operations at global tags 27–32 to
the checked DisplayNames and RelativeTimeFormat profiles and the real Engine
host import. ABI 10 binds both consumed native kernels into artifact identity.
The existing operations keep their tags and wire frames. The closed operation
census uses a u64 mask, retaining compile-time rejection of duplicate or missing
tags across all 33 operations.

DisplayNames admits its source language keys using the same pinned ICU locale
canonicalizer and reserved-language rules as runtime code input. A borrowed
adapter supplies that operation while the provider is being constructed; the
complete provider is published only after profile admission. Request decoding
uses the profile retained by the identity-matched kernel. Unknown valid codes
use the configured canonical whole-code fallback or undefined. Original UTF16
units cross the primitive frame, including isolated surrogates that native code
validation rejects. The host reports invalid code as a rejected operation;
malformed frames, inconsistent data and resource failures are host faults.

RelativeTimeFormat first selects its captured service locale, then negotiates
numbering with the checked NumberFormat profiles. Its finite-number owner retains
the original binary64 bits and signed zero. The shared decimal rounding and
cardinal-plural owners supply numeric parts and category selection. Genuine
finite input passes through exact locked `ryu-js 1.0.2` to produce ECMAScript
Number-to-string spelling before the shared decimal parser. Its safe
`Buffer::format_finite` API receives only the absolute value of the checked
finite owner. The bounded text owner checks allocation and output limits. This
preserves JavaScript's exponent thresholds and explicit exponent sign without
changing the shared numeric parser or reconstructing a value in a native VM.
Original sign bits still select past/future, including negative zero. Genuine
patterns without a numeric placeholder remain literal-only parts. Auto literals
remain distinct from numeric pattern formatting. Responses contain part kind,
UTF8 text, unit presence and the exact unit when present.

The immutable supported-values catalogue validates the reachable currency union
through real DisplayNames lookups with fallback disabled. It also resolves and
formats every declared positional numbering system through RelativeTimeFormat.
These checks run at catalogue admission and retain the existing exact domains.
They do not search the larger syntactically valid currency domain.

`scripts/generate-intl-display-relative-identity.py` binds the service production
leaves, primitive codecs, actual host consumer, shared native registry and
captured source inputs. RelativeTimeFormat additionally binds the NumberFormat
and PluralRules production owners and pinned numeric payload. The
RelativeTimeFormat recipe additionally binds the exact consumed `ryu-js`
registry version and checksum; DisplayNames has no formatter package binding.
Generated kernel manifests and Rust digests exclude independent test files and their own
identity outputs. Profile-only hashes remain separate provenance. The provider
composite uses the source-bound kernel digests.

Run source reproduction only with an explicit private repository argument:

```sh
python3 scripts/generate-intl-display-relative-identity.py --repository "$candidate_root" --check
```

The source proposal retains ten DisplayNames controls, twenty-one
RelativeTimeFormat controls, two shared-provider controls and six real host
boundary controls. The compiler and Engine fixture packets add their own ordered
coercion, Realm and output controls. The first complete candidate native run
observed 314 passes and two failures out of 316 library declarations; the public
target did not execute after that library failure. Its complete receipt remains
in history. This successor corrects the dialect fixture while retaining a
separate standard-name assertion, and supplies the canonical finite-number
spelling bridge with threshold, large-value and signed subnormal vectors inside
the existing control. The library declaration count remains 316. Successor
native verification passes all 316 library controls and all three public
calendar controls; all six host controls and the artifact identity gate also
pass. After the intrinsic-capture fixture repair both IR controls and all 11
catalogue controls pass. The temporary-lifecycle successor passes fresh
all-target compilation and all 72 heap controls. The next module checkpoint
finds a stale last-global fixture, repaired in the complete Segmenter batch.
Affected backend/Engine and pinned verification remain pending. Original failed checkpoints are retained. Source generation
and review do not establish MAIN admission or Test262 results.

The joined Segmenter batch advances the provider composite and host ABI to
11, retaining operations 0–32 and adding only operations 33–35. All six
consumed native kernels participate in artifact identity. DN/RTF primitive
frames, data domains and behavior remain unchanged by this extension.
