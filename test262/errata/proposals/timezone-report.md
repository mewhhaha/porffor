# ECMA-402/Test262: a historical Zone is missing from primary-ID resolution

Affected Test262 test: `intl402/Temporal/ZonedDateTime/links.js`, both modes.
Observed ECMA-402 revision: `730ca2b88c305862db7be2f83c510dcca33ab48c`.

Pinned IANA 2026a provides these inputs:

| Input | Value |
| --- | --- |
| Default `backward` | `Link Pacific/Honolulu Pacific/Johnston` |
| `backzone` | `Zone Pacific/Johnston`; no matching Johnston Link |
| Geographic country | Johnston: UM; Honolulu: US |
| UM entries in `zone.tab` | `Pacific/Midway`, `Pacific/Wake` |

The country-preservation requirement prevents Johnston from resolving to
Honolulu's primary identity. But the multiple-country-row algorithm searches
only backzone Links, then asserts a matching Link exists. These inputs contain
a Zone instead. The original test nevertheless requires Johnston/Honolulu
equality at the same instant.

The rules were introduced by
[ECMA-402 602aaf0 / PR 825](https://github.com/tc39/ecma402/commit/602aaf0b408d3cb427f1598fe079ae3d54ed8b8c).
No accepted clarification resolving this topology was found in the inspected
sources. Geographic country metadata and the Zone/Link distinction must remain
separate from identical transition bytes.

Two coordinated drafts are provided:

1. [ECMA-402 patch](ecma402-backzone-zone.patch): when this branch finds a
   historical Zone with the identifier's own name, retain it as primary;
   otherwise perform the existing Link lookup. This is general, not a Johnston
   exception, and preserves the country's identity.
2. [Test262 patch](test262-johnston.patch): move Johnston into the existing
   construction/name-preservation group, alongside other historical
   cross-country links. Keep all other equality/offset assertions.

[Identity probe](../reproducers/johnston-identity.js) observes distinct identities
with equal offsets at the epoch in Scriptw. The separate
[provider-specific offset control](../controls/johnston-offsets.js) retains all
eleven original samples. The installed V8, SpiderMonkey and QuickJS bindings
have no Temporal; their availability observations do not validate this proposal.
