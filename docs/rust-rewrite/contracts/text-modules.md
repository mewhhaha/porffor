# Text Module records

`type: "text"` is a distinct host request and record kind. `CreateTextModule`
retains the host-decoded String and uses the default-export synthetic module
factory, with one initialized mutable default cell and no dependency requests.
The payload is never parsed as JavaScript or JSON, even when its locator has a
`.js` or `.json` extension or names the importing Source Text Module itself.

`ModuleKindIr::Text`, `ModuleParse::Text`, `LoadedModuleKind::Text` and the
embedded graph's closed Text variants carry this choice through filesystem
loading, complete-catalog admission, graph identity, worker serialization and
synthetic evaluation. Equal locators and payloads do not merge Text, Source
Text, JSON or Bytes records. Repeated exact requests and host aliases share the
one Text record and its namespace. Additional attributes cannot select the
Text factory.

The filesystem host uses its existing strict UTF-8 decoding policy, also used
by the independent native debug oracle: malformed UTF-8 is a host load error;
a decoded leading BOM, NUL, newlines and non-ASCII characters are preserved.
`CreateTextModule` does not trim or re-decode the resulting String. An embedded
host supplies that decoded String directly. This is an explicit host policy,
not a claim that ECMAScript prescribes a filesystem encoding.

The AOT lowerer emits the admitted synthetic String through the ordinary
Wasm-GC String literal path. It does not invoke a public String constructor or
parse payload text in generated code. The debug embedded loader independently
creates Boa's default-only synthetic record in the actual referrer Realm.

Controls cover invalid JavaScript/JSON as text, empty/default-only records,
BOM/NUL/Unicode preservation, malformed UTF-8 rejection, text self-import,
same-locator JSON/Text and Source Text/Text identities, aliases, computed
dynamic requests and strict rejection of mismatched or extra attributes.
These controls and the historical seven execution IDs require native replay
before this prepared batch can be claimed as accepted.
