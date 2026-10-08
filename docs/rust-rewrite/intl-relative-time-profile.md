# Genuine RelativeTimeFormat profile proposal

The private schema-1 profile contains the thirteen captured CLDR47 locales:
en, en-US, ar, ar-EG, zh, zh-Hans, zh-Hans-CN, de, fr, it, ja, ko and hi.
Each locale has all eight canonical units (year, quarter, month, week, day,
hour, minute and second) in long, short and narrow styles. The 312 rows contain
3,744 numeric patterns and 768 genuine relative literals. Category arrays use
zero, one, two, few, many and other in that order. Relative offsets are unique,
sorted and limited to the actual optional values from −2 through 2.

The producer consumes the existing checked `CldrProfile` and its pinned
CLDR47 commit `2ef784e3a4168bc2a43cd1b5b9839b6636f5899c`. The same captured
DTD, locale parent rules, minimum draft and default field/style aliases govern
the lookup. A producer-local relative-time count resolver implements the
captured LDML lateral inheritance rule: try the requested category and then
other within each locale before trying the original category in the parent.
Root style aliases restart lookup at the requested locale. The shared date/time
resolver is unchanged. The inheritance marker is treated as an absent value;
an empty override blocks inheritance.

No text is normalized. Numeric patterns contain zero or one `{0}` placeholder;
Arabic one/two forms can have none. Auto-relative literals contain none. Any
other braces, repeated placeholder, unresolved marker, empty value or
unconsumed value attribute is rejected. The report binds all 4,512 consumed
request paths to their actual locale/path/value and both requested and selected
plural category. Every relative-time leaf remains in the primary CLDR47
`dates/fields` branches. The existing numbering and calendar-era supplements
are verified dependencies of `CldrProfile`; they do not supply these leaves.

`source-inputs.json` binds the producer, four unchanged helper files, all 28
primary manifest inputs and the existing checked supplement closure: 51 files
in total. Native code may consume `relative_time_format/generated/profile.json`
once its constructor checks the exact locale, unit/style, category and pattern
domains. There is no public locale admission or product capability claim in
this source packet.

The first actual source export and exact-byte `--check` both exit zero. Twelve
Python controls pass, including independent English/Arabic vectors, local
plural fallback before parent lookup, style-alias restart, inheritance/empty
markers, malformed pattern rejection and an actual pinned-source tamper check.
Native compilation, codec integration, JavaScript callers and pinned
RelativeTimeFormat verification remain pending.

Refresh with `python3 scripts/generate-intl-relative-time-profile.py` and verify
with the same command plus `--check`. Run the source controls with
`python3 -m unittest discover -s scripts/tests -p test_intl_relative_time_profile.py`.
