# Pinned CLDR47 BCP47 keyword aliases

These are the complete `common/bcp47/*.xml` files from Unicode CLDR47, release
commit `2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. The adjacent manifest records
upstream release/tag identity, SHA-256 and byte length of every source and the
Unicode license. Files retain their upstream bytes. The release matches
`IntlDataVersions::PINNED.cldr` and ICU4X locale data2.0.0; no newer CLDR or ICU
release is mixed into this provider.

From the repository root:

```sh
python3 scripts/generate-intl-keyword-aliases.py
python3 scripts/generate-intl-keyword-aliases.py --check
python3 -m unittest discover -s scripts/tests -p test_generate_intl_keyword_aliases.py -v
```

The standard-library Python generator reads every manifest-pinned XML file.
It emits `src/provider/keyword_aliases/generated.rs`: 60 Unicode keyword-value
aliases and five transform keyword-value aliases, ordered by key and complete
value. `preferred` replacements take precedence over a deprecated type's name;
legacy aliases are accepted only when they match the Unicode value grammar.
Identical repeated mappings are merged, conflicting mappings and cycles fail,
and every replacement must be terminal. All key aliases in this pinned data
use legacy syntax; a future well-formed key alias requires an explicit key
replacement operation before generation can succeed. No defined attribute
alias is omitted: this release has no attribute entries.

The 590 excluded old-syntax alias rows cannot appear as complete BCP47 values
because they contain invalid characters or subtag lengths. This is grammar
validation, not a Test262 filter. The optional `--report PATH` retains this
complete exclusion census with source filenames. Deprecated values without a
preferred replacement stay unchanged. Locale identifiers need only be
well-formed; the provider does not reject unregistered keyword values.

The provider data digest is SHA-256 of these bytes in order:

1. UTF-8 `lila-intl-locale-provider-v2` followed by a NUL byte;
2. the 32-byte pinned ICU locale archive SHA-256 from the manifest;
3. the 32-byte SHA-256 of the exact manifest bytes; and
4. the 32-byte SHA-256 of sorted UTF-8 rows, each encoded as
   `extension<TAB>key<TAB>alias<TAB>canonical<LF>`.

That digest is embedded in the existing artifact identity. Changed data rejects
old cached artifacts before instantiation. It does not change the host ABI or
claim that ICU/CLDR data is embedded in emitted Wasm.

The runtime matches aliases against a full key value, so `ca-islamicc` changes
to `ca-islamic-civil` while `ca-islamicc-true` stays intact. Unicode values
preserve their original subtags using ICU's public lossless collection API.
ICU2 has no corresponding public transform-value API, so private typed
transform-key records retain complete validated values and replace only the
serialized field portion after language canonicalization. A whole transform
`true` stays present; a whole Unicode `true` is empty. Subdivision aliases for
`rg` and `sd` continue to use the existing pinned ICU tables.

This table concerns locale keyword identifiers. It does not add formatter,
calendar arithmetic, time-zone normalization or other Intl services.

Sources: [CLDR47 BCP47 directory](https://github.com/unicode-org/cldr/tree/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/common/bcp47),
[CLDR47 key/type definitions](https://github.com/unicode-org/cldr/blob/2ef784e3a4168bc2a43cd1b5b9839b6636f5899c/docs/ldml/tr35.md#u-extension-data-files),
and [UTS35 canonicalization](https://unicode.org/reports/tr35/#Canonical_Unicode_Locale_Identifiers).
