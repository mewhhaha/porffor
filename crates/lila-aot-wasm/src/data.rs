use super::*;
use crate::runtime_helpers::RegExpMatcherFailure;
use icu_normalizer::{
    properties::{CanonicalCombiningClassMapBorrowed, CanonicalCompositionBorrowed},
    properties::{CanonicalDecompositionBorrowed, Decomposed},
    DecomposingNormalizerBorrowed,
};
use icu_properties::{props, CodePointSetData};
use lila_ir::ArrayAccumulationElementIr;
use lila_ir::{
    ObjectDestructuringPatternIr, OptionalChainOperationIr, RegExpCompileErrorKind, RegExpProgram,
    StaticRegExpCompilation, TemplateObjectIr, BUILTIN_REGEXP_FUNCTION_ID,
    BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID, REGEXP_OPCODE_ACCEPT, REGEXP_OPCODE_DOT,
    REGEXP_OPCODE_JUMP, REGEXP_OPCODE_LITERAL_ASCII, REGEXP_OPCODE_LITERAL_CODE_POINT,
    REGEXP_OPCODE_NEGATIVE_ASCII_CLASS, REGEXP_OPCODE_NOT_WHITESPACE,
    REGEXP_OPCODE_NUMBERED_BACKREFERENCE, REGEXP_OPCODE_POSITIVE_ASCII_CLASS, REGEXP_OPCODE_SPLIT,
    REGEXP_OPCODE_UNICODE_PROPERTY, REGEXP_OPCODE_WHITESPACE,
};
use std::sync::OnceLock;

/// Packed magic/version word at the start of an immutable named-group table.
/// The high 32 bits are the format version and the low 32 bits are `NRGT`.
pub(crate) const REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION: u64 =
    (1_u64 << 32) | u32::from_le_bytes(*b"NRGT") as u64;

pub(crate) const REALM_EVAL_SCRIPT_ESCAPE_MESSAGE: &str =
    "$262.evalScript dynamic source evaluation escaped typed lowering";

/// Runtime-error message literals that no other interning path reaches.
///
/// Why this table exists at all, stated once so it is not rediscovered a fourth
/// time. `emit_runtime_error_object` (`builtins/errors.rs`) used to define the
/// error's `message` property from its *name* payload and throw the message
/// argument away, so `e.message === e.name` for every error the runtime threw.
/// The obvious one-token repair could not land alone: `StringPool::payload`
/// takes `&self`, cannot extend the pool during emission, and **panics** with
/// ``string `..` must exist in pool``. Because that function never asked the
/// pool for a message, the messages that reach only it were never required to
/// be interned -- and were not. Landing the repair without this table turns
/// `null.x` into a compiler panic.
///
/// The list is measured, not guessed. Every call site of the seven throw entry
/// points (`emit_throw_runtime_error`, `_to_active_handler`,
/// `_with_prototype_local` and the four `emit_throw_current_function_realm_*`
/// wrappers) was walked transitively through the `&str` parameters they forward
/// through, giving 908 reachable message literals; the 131 below plus the two
/// typed RegExp matcher failures are the ones absent from both this file and
/// `builtins::intl_date_time_format_pool_strings()`.
///
/// It over-approximates on purpose. A string interned twice costs nothing --
/// `intern_string` returns early on a hit -- while a string missed once is a
/// compiler panic on a program as ordinary as `null.x`. Two families here are
/// reachable only through an enum-selected `match` arm
/// (`Map`/`WeakMap`/`Set`/`WeakSet` constructor messages, `yield*` iterator
/// protocol messages), which a call-site-literal audit alone does not see.
///
/// KNOWN INCOMPLETENESS, and the reason this is a table rather than a claim:
/// a static audit cannot resolve every `&'static str` that reaches a throw
/// through a local binding or a helper's parameter. If a message is still
/// missing, the pool says so loudly at run time by name. That is the correct
/// failure mode and it is deliberately not softened -- see the standing
/// instruction on `emit_runtime_error_object` never to fall back to the name.
///
/// RegExp matcher failures are excluded from this list. Their messages are
/// derived from [`RegExpMatcherFailure::ALL`] below, so their ABI word, error
/// route and interned text cannot become parallel tables.
///
/// The right long-term shape is one `RUNTIME_ERROR_MESSAGES` domain that the
/// emitters index into, so "add a message" is one edit and "forgot to intern"
/// is a compile error. That is a refactor across ~1,120 call sites and does not
/// belong to this lane; this table is the honest intermediate.
pub(crate) const RUNTIME_ERROR_MESSAGE_LITERALS: &[&str] = &[
    REALM_EVAL_SCRIPT_ESCAPE_MESSAGE,
    "Atomics.wait cannot suspend the current agent",
    "BigInt division by zero",
    "BigInt shift result exceeds the engine resource limit",
    "BigInts do not support unsigned right shift",
    "Cannot add property to non-extensible array",
    "Cannot assign inherited typed array index on receiver",
    "Cannot assign to arguments index",
    "Cannot assign to arguments.callee",
    "Cannot assign to arguments.length",
    "Cannot assign to array index",
    "Cannot assign to array length",
    "Cannot assign to array property",
    "Cannot assign to inherited accessor without setter",
    "Cannot assign to inherited read only property",
    "Cannot change enumerable flag of non-configurable arguments accessor",
    "Cannot change enumerable flag of non-configurable arguments property",
    "Cannot change enumerable flag of non-configurable arguments.callee",
    "Cannot change kind of non-configurable arguments.callee",
    "Cannot change non-configurable arguments accessor",
    "Cannot change non-configurable arguments.callee accessor",
    "Cannot change non-writable arguments.callee",
    "Cannot change value of non-writable arguments property",
    "Cannot define array length",
    "Cannot make non-configurable arguments property writable",
    "Cannot make non-configurable arguments.callee writable",
    "Cannot read properties of null or undefined",
    "Cannot redefine non-configurable arguments accessor",
    "Cannot redefine non-configurable arguments property",
    "Cannot redefine non-configurable arguments.callee",
    "Cannot replace non-configurable accessor arguments property",
    "Cannot replace non-configurable data arguments property",
    "Function has non-object prototype in instanceof check",
    "Get target is not an object",
    "Map constructor iterator method is not callable",
    "Map constructor iterator method must return an object",
    "Map constructor iterator next method is not callable",
    "Map constructor iterator next result must be an object",
    "Map constructor iterator value must be an object",
    "Map constructor requires new",
    "Map constructor set method is not callable",
    "Map.prototype.forEach callback must be callable",
    "Math.sumPrecise input is not iterable",
    "Math.sumPrecise iterable contains too many values",
    "Math.sumPrecise iterator method must return an object",
    "Math.sumPrecise iterator next method is not callable",
    "Math.sumPrecise iterator next result must be an object",
    "Object.prototype.__proto__ setter called on null or undefined",
    "Object.prototype.__proto__ setter could not set prototype",
    "Object.prototype.hasOwnProperty called on null or undefined",
    "Promise cannot resolve to itself",
    "Promise capability constructor is not a constructor",
    "Promise capability did not initialize callable resolving functions",
    "Promise capability executor called more than once",
    "Promise constructor property is not an object",
    "Promise constructor requires new",
    "Promise executor is not callable",
    "Promise species is not a constructor",
    "Promise.all constructor resolve property is not callable",
    "Promise.all input is not iterable",
    "Promise.all iterable contains too many values",
    "Promise.all iterator method is not callable",
    "Promise.all iterator method must return an object",
    "Promise.all iterator next method is not callable",
    "Promise.all iterator next result must be an object",
    "Promise.prototype.finally called on non-object receiver",
    "Promise.prototype.then called on incompatible receiver",
    "Promise.race constructor resolve property is not callable",
    "Promise.race input is not iterable",
    "Promise.race iterator method is not callable",
    "Promise.race iterator method must return an object",
    "Promise.race iterator next method is not callable",
    "Promise.race iterator next result must be an object",
    "Proxy ownKeys trap result contained a duplicate key",
    "Proxy ownKeys trap result contained a non-property key",
    "Proxy ownKeys trap result contains an extra key for a non-extensible target",
    "Proxy ownKeys trap result does not match non-extensible target",
    "Proxy ownKeys trap result must be an object",
    "RegExp.prototype[Symbol.matchAll] receiver is not object",
    "RegExp.prototype[Symbol.replace] exec result is not an object or null",
    "RegExp.prototype[Symbol.replace] receiver is not an object",
    "RegExp.prototype[Symbol.split] constructor is not an object",
    "RegExp.prototype[Symbol.split] exec result is not an object or null",
    "RegExp.prototype[Symbol.split] species is not a constructor",
    "Set constructor add method is not callable",
    "Set constructor iterator method is not callable",
    "Set constructor iterator method must return an object",
    "Set constructor iterator next method is not callable",
    "Set constructor iterator next result must be an object",
    "Set constructor requires new",
    "Set.prototype.forEach callback must be callable",
    "String method RegExp flags must contain g",
    "TypedArray allocation size is too large",
    "TypedArray iterator method must be callable",
    "TypedArray iterator method must return an object",
    "TypedArray iterator next method must be callable",
    "TypedArray iterator next result must be an object",
    "WeakMap constructor iterator method is not callable",
    "WeakMap constructor iterator method must return an object",
    "WeakMap constructor iterator next method is not callable",
    "WeakMap constructor iterator next result must be an object",
    "WeakMap constructor iterator value must be an object",
    "WeakMap constructor requires new",
    "WeakMap constructor set method is not callable",
    "assignment to unresolvable reference",
    "cannot get function realm from a revoked Proxy",
    "for-await-of async iterator next result must be object",
    "for-await-of async iterator return result must be object",
    "for-await-of iterator method must be callable",
    "for-await-of iterator method must return object",
    "for-await-of iterator next must be callable",
    "for-await-of iterator next result must be object",
    "for-await-of iterator return must be callable",
    "for-await-of iterator return result must be object",
    "for-await-of target is not iterable",
    "lexical binding accessed before initialization",
    "private accessor has no getter",
    "private element already installed on object",
    "private element cannot be installed on non-extensible object",
    "private element has no setter",
    "private environment is missing its declared name",
    "right-hand side of private in is not an object",
    "unbound identifier",
    "yield* iterator has no throw method",
    "yield* iterator method must be callable",
    "yield* iterator method must return object",
    "yield* iterator result must be object",
    "yield* next method must be callable",
    "yield* return method must be callable",
    "yield* target is not iterable",
    "yield* throw method must be callable",
];

#[derive(Debug)]
struct StringRef {
    offset: u32,
    len: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegExpProgramRef {
    pub(crate) ptr: u32,
    pub(crate) instruction_count: u32,
    pub(crate) capture_count: u32,
    pub(crate) split_count: u32,
    pub(crate) repeatable_split_count: u32,
    pub(crate) named_group_table_ptr: u32,
}

/// Semantic identity for immutable static RegExp programs. The bytecode blob
/// contains instructions only, so capture metadata must participate here
/// instead of being appended to the static program bytes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct RegExpProgramStaticKey {
    encoded_instructions: Vec<u8>,
    capture_count: u32,
    named_groups: Vec<(String, Vec<u32>)>,
}

struct LowercaseTables {
    mappings: Vec<u8>,
    mapping_count: u32,
    cased_ranges: Vec<std::ops::RangeInclusive<u32>>,
    case_ignorable_ranges: Vec<std::ops::RangeInclusive<u32>>,
}

static LOWERCASE_TABLES: OnceLock<LowercaseTables> = OnceLock::new();

struct UppercaseTables {
    mappings: Vec<u8>,
    mapping_count: u32,
}

static UPPERCASE_TABLES: OnceLock<UppercaseTables> = OnceLock::new();

struct NormalizationMapping {
    codepoint: u32,
    sequence_index: u32,
    sequence_len: u32,
}

struct NormalizationTables {
    canonical_mappings: Vec<NormalizationMapping>,
    canonical_sequences: Vec<u32>,
    compatibility_mappings: Vec<NormalizationMapping>,
    compatibility_sequences: Vec<u32>,
    combining_classes: Vec<(u32, u8)>,
    compositions: Vec<(u32, u32, u32)>,
}

static NORMALIZATION_TABLES: OnceLock<NormalizationTables> = OnceLock::new();

impl RegExpProgramStaticKey {
    pub(crate) fn from_program(program: &RegExpProgram) -> Self {
        Self {
            encoded_instructions: program.encode(),
            capture_count: program.capture_count,
            named_groups: program
                .named_groups
                .iter()
                .map(|group| (group.name.clone(), group.capture_ids.clone()))
                .collect(),
        }
    }
}

/// Word index of `source`'s interned payload in a runtime RegExp program table
/// record.
///
/// # Why these are constants and not literals on each side
///
/// The record is written here by [`StringPool::append_runtime_regexp_program_table`]
/// and read in `expressions.rs` by `emit_runtime_regexp_program_slots`. Those
/// were the only two places that knew the layout, and each spelled it out
/// independently: the writer as the *order* of an array literal, the reader as
/// bare `16`/`24`/…/`64` offsets and a `72` stride. Nothing connected them, so
/// adding, reordering or resizing a word compiled cleanly on both sides and
/// produced garbage program slots at run time — the same silent wrong-answer
/// class this table exists to remove.
///
/// Naming the words once fixes that in three ways, and all three are compile
/// errors rather than test failures:
///
/// * the writer builds a `[u64; RUNTIME_REGEXP_RECORD_WORDS]` and assigns
///   **through these indices**, so a word with no index cannot be written and an
///   index with no word is an out-of-bounds `const` evaluation;
/// * [`RUNTIME_REGEXP_RECORD_SIZE`] is derived from the word count rather than
///   typed as `72`, so the reader's stride cannot fall behind the writer's row;
/// * the reader's offsets come from [`runtime_regexp_record_offset`] applied to
///   the same indices, so a reordering moves both sides at once.
pub(crate) const RUNTIME_REGEXP_RECORD_SOURCE_WORD: usize = 0;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`]. `flags`' interned payload.
pub(crate) const RUNTIME_REGEXP_RECORD_FLAGS_WORD: usize = 1;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`]. Static pointer to the compiled
/// program's instructions, or 0 for a non-`Program` row.
pub(crate) const RUNTIME_REGEXP_RECORD_PROGRAM_PTR_WORD: usize = 2;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_INSTRUCTION_COUNT_WORD: usize = 3;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_CAPTURE_COUNT_WORD: usize = 4;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_SPLIT_COUNT_WORD: usize = 5;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_REPEATABLE_SPLIT_COUNT_WORD: usize = 6;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_NAMED_GROUP_TABLE_PTR_WORD: usize = 7;
/// See [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`]. The `RUNTIME_REGEXP_ENTRY_KIND_*`
/// discriminant, i.e. which [`RuntimeRegExpEntry`] this row is.
pub(crate) const RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD: usize = 8;

/// Number of `u64` words in one runtime RegExp program table record.
///
/// Keep this the last word's index plus one: it is the length of the writer's
/// array, so a word added without extending it fails to compile at the
/// assignment rather than corrupting the next row.
pub(crate) const RUNTIME_REGEXP_RECORD_WORDS: usize = RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD + 1;

/// Byte stride between runtime RegExp program table records. **Derived**, never
/// typed — see [`RUNTIME_REGEXP_RECORD_SOURCE_WORD`].
pub(crate) const RUNTIME_REGEXP_RECORD_SIZE: u64 = (RUNTIME_REGEXP_RECORD_WORDS * 8) as u64;

/// Byte offset of a record word, for the emitter's `i64.load` memargs.
pub(crate) const fn runtime_regexp_record_offset(word: usize) -> u64 {
    (word * 8) as u64
}

/// The `entry_kind` word of a runtime RegExp program table record.
///
/// The record is [`RUNTIME_REGEXP_RECORD_SIZE`] bytes: the eight words the
/// emitter already read, plus this discriminant at
/// [`RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD`]. It is a word rather than a
/// sentinel (`ptr == 0`, `instruction_count == 0`) on purpose — a sentinel is
/// what the emitter used to be forced into, and it cannot tell "the compiler
/// rejected this pattern" apart from "this row was never written".
pub(crate) const RUNTIME_REGEXP_ENTRY_KIND_PROGRAM: u64 = 0;
/// See [`RUNTIME_REGEXP_ENTRY_KIND_PROGRAM`]. A row with this kind means the
/// compile-time `RegExpProgram::compile` answered
/// [`RegExpCompileErrorKind::InvalidSyntax`] — the pattern is not a legal
/// ECMAScript Pattern, so constructing a RegExp from it at run time is a spec
/// SyntaxError.
///
/// # The risk this row carries, stated in the other direction
///
/// The doc on [`RuntimeRegExpEntry`] argues one direction at length: a *missing*
/// row is a wrong answer, so seen-and-rejected must be recorded. The mirror
/// image is real and is not argued anywhere else, so it is stated here.
///
/// This row makes the compile-time compiler's `InvalidSyntax` verdict
/// **load-bearing at run time**. Before it, a pattern this compiler
/// mis-classified as `InvalidSyntax` merely fell through to the runtime fallback
/// matcher, which frequently answered it correctly. Now it throws a spurious
/// SyntaxError at all seven `emit_runtime_regexp_program_slots` call sites.
/// `lila-ir/src/regexp.rs` has ~20 `invalid_syntax(` construction sites
/// against ~7 `unsupported(` ones and none of them has been audited against the
/// grammar, so the premise "`InvalidSyntax` means the spec says invalid" is
/// assumed, not established.
///
/// Two properties widen the blast radius, and both are deliberate elsewhere:
/// the table is looked up by string **value** (`emit_string_payload_equality_i32`
/// is a real byte compare), so a runtime-concatenated string that happens to
/// equal a mis-rejected script literal also throws; and in fallback mode the
/// candidate set is every script string literal, so every mis-rejected literal
/// in a harness file becomes reachable.
///
/// Neither named gate detects this. `annexB/built-ins/RegExp/prototype/compile`
/// is 23 cases and `built-ins/RegExp/named-groups` is 36, and named groups
/// exercise the `Program` path, which is unchanged. **Measure
/// `built-ins/RegExp/prototype` (487 cases) as a delta before treating this as
/// landed, and read any new failure whose detail names SyntaxError as a
/// false-`InvalidSyntax` candidate rather than as unrelated noise.**
pub(crate) const RUNTIME_REGEXP_ENTRY_KIND_REJECTED: u64 = 1;
/// See [`RUNTIME_REGEXP_ENTRY_KIND_PROGRAM`]. A row with this kind means the
/// compile-time compiler answered [`RegExpCompileErrorKind::UnsupportedFeature`]
/// — the pattern **is** legal ECMAScript and Lila simply cannot compile it yet.
///
/// This is the distinction that makes the table worth having and the one a
/// bare "compile failed, so throw" would destroy: a `SyntaxError` here would be
/// a *new* wrong answer, thrown for a pattern the spec says is fine. Such a row
/// deliberately behaves exactly like a total miss — zeroed program slots, and
/// the runtime's own fallback matcher gets its turn. `lila-ir`'s
/// `try_lower_static_regexp_compilation` draws the same line at its
/// `Err(error) if error.kind == RegExpCompileErrorKind::InvalidSyntax` arm, and
/// the two must not drift apart.
pub(crate) const RUNTIME_REGEXP_ENTRY_KIND_UNSUPPORTED: u64 = 2;

/// The closed domain the three `RUNTIME_REGEXP_ENTRY_KIND_*` words spell.
///
/// # Why this exists on top of [`RuntimeRegExpEntry`]
///
/// [`RuntimeRegExpEntry`] closes the **writer**: a fourth outcome is
/// `error[E0004]` at `append_runtime_regexp_program_table`. That bought nothing
/// on the **reader** side, which compared a raw `u64` against two of the three
/// constants. A fourth `RUNTIME_REGEXP_ENTRY_KIND_FOO = 3` would have compiled
/// cleanly next to its siblings and fallen through both comparisons in
/// `emit_runtime_regexp_program_slots` as a miss — reinstating, one level down,
/// the exact silent-skip class this table exists to remove.
///
/// So the *decision* the emitter makes is stated here, once, as an exhaustive
/// match ([`Self::throws_syntax_error`]), and the emitter builds its comparison
/// chain by iterating [`Self::ALL`]. Adding a variant is then a compile error at
/// two exhaustive matches in this file, and the emitted comparison follows
/// automatically rather than being one more transcription.
///
/// Residual, stated rather than papered over: [`Self::ALL`] is hand-written.
/// The compiler cannot enumerate a Rust enum, so the trigger to extend it is
/// the `error[E0004]` a new variant produces at the two matches below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeRegExpEntryKind {
    Program,
    Rejected,
    Unsupported,
}

impl RuntimeRegExpEntryKind {
    /// Every kind. See the type's doc for why this is hand-written and what
    /// forces it to be kept honest.
    pub(crate) const ALL: [Self; 3] = [Self::Program, Self::Rejected, Self::Unsupported];

    /// The discriminant word written into
    /// [`RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD`].
    pub(crate) const fn word(self) -> u64 {
        match self {
            Self::Program => RUNTIME_REGEXP_ENTRY_KIND_PROGRAM,
            Self::Rejected => RUNTIME_REGEXP_ENTRY_KIND_REJECTED,
            Self::Unsupported => RUNTIME_REGEXP_ENTRY_KIND_UNSUPPORTED,
        }
    }

    /// Does a run-time hit on a row of this kind throw `SyntaxError`?
    ///
    /// This is the whole policy, and it is deliberately not `!= Program`:
    /// `Unsupported` means the pattern is legal ECMAScript that Lila cannot
    /// compile yet, so it must behave exactly like a total miss and let the
    /// runtime fallback matcher have its turn.
    pub(crate) const fn throws_syntax_error(self) -> bool {
        match self {
            Self::Program | Self::Unsupported => false,
            Self::Rejected => true,
        }
    }
}

/// What the AOT-built runtime RegExp program table says about one
/// `(source, flags)` pair.
///
/// The table is looked up **by string value** at run time, so an absent row and
/// an illegal pattern used to be the same observable state. `queue_runtime_regexp_programs`
/// wrote rows with
///
/// ```ignore
/// let Ok(program) = RegExpProgram::compile(compilation_source, flags) else {
///     continue;
/// };
/// ```
///
/// so a pattern the compiler had *seen and rejected* left no trace at all, the
/// emitted lookup fell out of its loop with no else arm, and
/// `new RegExp("(?<x>a)(?<x>b)")` returned a live RegExp carrying
/// `instruction_count == 0` instead of throwing SyntaxError. That is a
/// wrong-answer class, not a missing feature.
///
/// Making the table's value a closed type is what stops it recurring: the
/// writer below matches exhaustively, so a third outcome added later is
/// `error[E0004]` at the table writer rather than one more silently skipped
/// row. `Option<RegExpProgramRef>` would not do it — `unwrap_or`, `if let` and
/// `continue` are all one keystroke away, and `continue` is exactly what was
/// written here.
#[derive(Debug, Clone, Copy)]
enum RuntimeRegExpEntry {
    /// `RegExpProgram::compile` accepted the pair; this is its static data.
    Program(RegExpProgramRef),
    /// `RegExpProgram::compile` answered `InvalidSyntax`: the pattern is not a
    /// legal ECMAScript Pattern. The emitted lookup turns a hit on this row
    /// into a SyntaxError.
    Rejected,
    /// `RegExpProgram::compile` answered `UnsupportedFeature`: the pattern is
    /// legal and Lila cannot compile it yet. A hit on this row must **not**
    /// throw — see [`RUNTIME_REGEXP_ENTRY_KIND_UNSUPPORTED`].
    Unsupported,
}

/// [`RuntimeRegExpEntry`] before the static program data exists.
///
/// `queue_regexp_program` only queues; the `RegExpProgramRef` a `Program` row
/// needs is not known until `append_regexp_programs` has run. This carries the
/// same three answers across that gap by key instead of by ref, so the
/// intermediate never has to be an `Option` — the shape whose `None` arm is
/// what the original `continue` collapsed into.
enum CandidateOutcome {
    Program(RegExpProgramStaticKey),
    Rejected,
    Unsupported,
}

#[derive(Debug, Default)]
pub(crate) struct StringPool {
    pub(crate) bytes: Vec<u8>,
    pub(crate) template_objects: BTreeMap<u64, TemplateObjectIr>,
    refs: BTreeMap<String, StringRef>,
    script_string_literals: BTreeSet<String>,
    runtime_regexp_candidate_literals: BTreeSet<String>,
    /// Pattern strings the script names **directly at a RegExp construction
    /// site** (`new RegExp("…")`, `RegExp("…")`, `r.compile("…")`).
    ///
    /// Separate from `runtime_regexp_candidate_literals` because that set has a
    /// second job: when it is empty the candidate set falls back to *every*
    /// script string literal. Folding construction-site arguments into it would
    /// silently flip that fallback off for any script that has one, narrowing
    /// the table instead of widening it. This set is always unioned in, so it
    /// can only add rows.
    ///
    /// Measured motivation: `runtime_regexp_candidate_literals` is populated
    /// from declaration initialisers, assignments and array literals — never
    /// from call arguments. In
    /// `annexB/built-ins/RegExp/prototype/compile/duplicate-named-capturing-groups-syntax.js`
    /// the valid pattern reaches it through `let source = "(?<x>a)|(?<x>b)"`,
    /// while the invalid `"(?<x>a)(?<x>b)"` appears only as a call argument, so
    /// the set was non-empty, the fallback did not fire, and the pattern the
    /// test is *about* was never offered to the compiler at all.
    runtime_regexp_argument_literals: BTreeSet<String>,
    regexp_programs: BTreeMap<RegExpProgramStaticKey, RegExpProgramRef>,
    pending_regexp_programs: Vec<(RegExpProgramStaticKey, u32, u32, u32)>,
    runtime_regexp_programs: Vec<(String, String, RuntimeRegExpEntry)>,
    needs_runtime_regexp_programs: bool,
    pub(crate) runtime_regexp_program_table_ptr: u32,
    pub(crate) runtime_regexp_program_count: u32,
    pub(crate) uses_heap: bool,
    pub(crate) lowercase_mapping_table_ptr: u32,
    pub(crate) lowercase_mapping_count: u32,
    pub(crate) uppercase_mapping_table_ptr: u32,
    pub(crate) uppercase_mapping_count: u32,
    pub(crate) cased_range_table_ptr: u32,
    pub(crate) cased_range_count: u32,
    pub(crate) case_ignorable_range_table_ptr: u32,
    pub(crate) case_ignorable_range_count: u32,
    pub(crate) canonical_decomposition_table_ptr: u32,
    pub(crate) canonical_decomposition_count: u32,
    pub(crate) compatibility_decomposition_table_ptr: u32,
    pub(crate) compatibility_decomposition_count: u32,
    pub(crate) combining_class_table_ptr: u32,
    pub(crate) combining_class_count: u32,
    pub(crate) composition_table_ptr: u32,
    pub(crate) composition_count: u32,
}

impl StringPool {
    pub(crate) fn collect(
        script: &ScriptIr,
        function_metas: &BTreeMap<FunctionId, WasmFunctionMeta>,
        compiled_standard_builtins: &[StandardBuiltinId],
    ) -> Self {
        let mut pool = Self::default();
        // `RegExpPrototypeCompile` is what makes the `CallMethod` arm below
        // able to serve its stated purpose. That arm collects the pattern
        // argument of `r.compile("…")` but deliberately does not set this flag,
        // and for the `var r = /[ab]/; function go() { r.compile("xy"); }`
        // spelling there is no *other* setter — so before this disjunct the
        // collected literal was written into a set that was never read, the
        // table was never built, and `emit_runtime_regexp_program_slots`
        // early-returned on a zero row count.
        //
        // Keyed off the compiled-builtin set rather than off call shape because
        // that set is precise in the direction that matters: `RegExpConstructor`
        // does **not** root its prototype methods (checked in
        // `planning.rs::require_standard_builtin` — the dependency edge runs the
        // other way, `RegExpPrototypeCompile => roots RegExpConstructor`), so
        // this is true only for a module that actually compiled a `.compile`
        // call site. `RegExpConstructor` itself must NOT be added the same way
        // without measuring: it is rooted by any RegExp literal at all, and the
        // fallback candidate set is every script string literal.
        pool.needs_runtime_regexp_programs = script.functions.iter().any(|function| {
            function.super_constructor_target.as_deref() == Some(BUILTIN_REGEXP_FUNCTION_ID)
        }) || compiled_standard_builtins
            .contains(&StandardBuiltinId::RegExpPrototypeSymbolSplit)
            || compiled_standard_builtins.contains(&StandardBuiltinId::RegExpPrototypeCompile);
        for value in [
            "",
            " ",
            "          ",
            "\n",
            ": ",
            ",",
            "undefined",
            "null",
            "true",
            "false",
            "0",
            "-0",
            "\"",
            "{",
            "}",
            "{}",
            "[",
            "]",
            "[]",
            ":",
            "0.0",
            "1",
            "1.0",
            "NaN",
            "Infinity",
            "-Infinity",
            "1e-7",
            "-1e-7",
            "100000000000000000000",
            "-100000000000000000000",
            "10203040506070809000",
            "-10203040506070809000",
            "1e+22",
            "-1e+22",
            "Symbol()",
            "-1e+21",
            "\\d",
            "\\d{1}",
            "\\d{2}",
            "\\D{2}",
            "0.",
            ".",
            "\u{20BB7}",
            "\\p{Script=Han}",
            "b.",
            "c.",
            "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}",
            "[\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{200D}\u{1F466}]",
            "\u{1F468}",
            "\u{1F469}",
            "\u{1F467}",
            "\u{1F466}",
            "\u{200D}",
            "\u{F0000}D842",
            "\u{F0000}DFB7",
            "\u{F0000}D83D",
            "\u{F0000}DC68",
            "\u{F0000}DC69",
            "\u{F0000}DC67",
            "\u{F0000}DC66",
            "([\\d]{5})([-\\ ]?[\\d]{4})?$",
            "(?:(?<x>a)|(?<y>a)(?<x>b))(?:(?<z>c)|(?<z>d))",
            "(?:(?:(?<x>a)|(?<x>b)|c)\\k<x>){2}",
            "groups",
            "indices",
            "x",
            "y",
            "z",
            "a",
            "b",
            "c",
            "d",
            "abc",
            "ad",
            "aac",
            "\\ud834",
            "\\udf06",
            "[object Object]",
            "[object Arguments]",
            "[object ",
            "/",
            "prototype",
            "lastIndex",
            "index",
            "input",
            "dynamic eval unsupported",
            "$_",
            "lastMatch",
            "$&",
            "lastParen",
            "$+",
            "leftContext",
            "$`",
            "rightContext",
            "$",
            "$'",
            "$1",
            "$2",
            "$3",
            "$4",
            "$5",
            "$6",
            "$7",
            "$8",
            "$9",
            "g",
            "l",
            "77",
            "\\u0037\\u0037",
            "\\s",
            "\\w",
            "\\d+",
            "[a-z]",
            "RegExp legacy static accessor receiver must be RegExp",
            "%ThrowTypeError%",
            "constructor",
            "withResolvers",
            "try",
            "promise",
            "resolve",
            "reject",
            "callee",
            "arguments",
            "caller",
            "valueOf",
            "toString",
            "toUpperCase",
            "toLowerCase",
            "toLocaleLowerCase",
            "toLocaleUpperCase",
            "fromCodePoint",
            "String.fromCodePoint argument must be an integer from 0 through 0x10FFFF",
            "padStart",
            "padEnd",
            "repeat",
            "exec",
            "object",
            "boolean",
            "number",
            "string",
            "default",
            "function",
            "function(handle@",
            ")",
            "length",
            "name",
            "message",
            "error",
            "suppressed",
            "cause",
            "errors",
            "global",
            FUNCTION_NAME,
            OBJECT_NAME,
            ARRAY_NAME,
            ARRAY_BUFFER_NAME,
            DATA_VIEW_NAME,
            DATE_NAME,
            REGEXP_NAME,
            MATH_NAME,
            JSON_NAME,
            FLOAT64_ARRAY_NAME,
            FLOAT32_ARRAY_NAME,
            INT32_ARRAY_NAME,
            INT16_ARRAY_NAME,
            INT8_ARRAY_NAME,
            UINT32_ARRAY_NAME,
            UINT16_ARRAY_NAME,
            UINT8_ARRAY_NAME,
            UINT8_CLAMPED_ARRAY_NAME,
            "BigInt64Array",
            "BigUint64Array",
            "$Realm.Float64Array.prototype",
            "$Realm.Float32Array.prototype",
            "$Realm.Int32Array.prototype",
            "$Realm.Int16Array.prototype",
            "$Realm.Int8Array.prototype",
            "$Realm.Uint32Array.prototype",
            "$Realm.Uint16Array.prototype",
            "$Realm.Uint8Array.prototype",
            "$Realm.Uint8ClampedArray.prototype",
            "$Realm.BigInt64Array.prototype",
            "$Realm.BigUint64Array.prototype",
            "BYTES_PER_ELEMENT",
            REFLECT_NAME,
            NUMBER_NAME,
            STRING_NAME,
            BOOLEAN_NAME,
            ERROR_NAME,
            EVAL_ERROR_NAME,
            AGGREGATE_ERROR_NAME,
            SUPPRESSED_ERROR_NAME,
            RANGE_ERROR_NAME,
            SYNTAX_ERROR_NAME,
            TYPE_ERROR_NAME,
            URI_ERROR_NAME,
            REFERENCE_ERROR_NAME,
            "call",
            "apply",
            "bind",
            "anchor",
            "big",
            "blink",
            "bold",
            "fixed",
            "fontcolor",
            "fontsize",
            "italics",
            "link",
            "small",
            "strike",
            "sub",
            "substr",
            "substring",
            "sup",
            "trim",
            "trimStart",
            "trimLeft",
            "trimEnd",
            "trimRight",
            "match",
            "matchAll",
            "replace",
            "replaceAll",
            "search",
            "split",
            "concat",
            "[object Undefined]",
            "[object Null]",
            "[object Boolean]",
            "[object Number]",
            "[object String]",
            "[object Symbol]",
            "[object Object]",
            "[object Array]",
            "[object Function]",
            "[object Arguments]",
            "[object BigInt]",
            "[object Error]",
            "[object Date]",
            "from",
            "find",
            "findIndex",
            "findLast",
            "findLastIndex",
            "flat",
            "includes",
            "pop",
            "push",
            "shift",
            "unshift",
            "splice",
            "sort",
            "subarray",
            "entries",
            "values",
            "create",
            "getPrototypeOf",
            "setPrototypeOf",
            "defineProperty",
            "defineProperties",
            "getOwnPropertyDescriptor",
            "getOwnPropertyNames",
            "getOwnPropertySymbols",
            "keys",
            "hasOwn",
            "is",
            "isSealed",
            "isFrozen",
            "freeze",
            "isExtensible",
            "hasOwnProperty",
            "__proto__",
            "propertyIsEnumerable",
            "Object.prototype.propertyIsEnumerable called on null or undefined",
            "toString",
            "$IsHTMLDDA",
            "Symbol.iterator",
            "for-of target is not iterable",
            "for-of iterator method must be callable",
            "for-of iterator method must return object",
            "for-of iterator next must be callable",
            "for-of iterator next result must be object",
            "destructuring value is not iterable",
            "destructuring iterator method must return object",
            "destructuring iterator next must be callable",
            "destructuring iterator next result must be object",
            "iterator next result must be object",
            "Cannot destructure undefined or null",
            "return",
            "IteratorClose return method must be callable",
            "IteratorClose return result must be object",
            "$ArrayIterator.array",
            "$ArrayIterator.index",
            "$ArrayIterator.done",
            "$ArrayIterator.kind",
            "$StringIterator.string",
            "$StringIterator.index",
            LILA_STATIC_GENERATOR_VALUES_METHOD,
            LILA_STATIC_GENERATOR_ITERATOR_SLOT,
            "$RegExpStringIterator.regexp",
            "$RegExpStringIterator.string",
            "$RegExpStringIterator.global",
            "$RegExpStringIterator.unicode",
            "$RegExpStringIterator.done",
            "Array Iterator",
            "String Iterator",
            "Map Iterator",
            "Set Iterator",
            "Generator",
            "Generator method called on incompatible receiver",
            "Generator is already running",
            "AsyncGenerator method called on incompatible receiver",
            "Promise.resolve receiver is not an object",
            "Promise keyed constructor resolve property is not callable",
            "Promise keyed input must be an object",
            "allKeyed",
            "allSettledKeyed",
            "status",
            "fulfilled",
            "rejected",
            "value",
            "reason",
            "Array.prototype iterator method called on null or undefined",
            "Array Iterator next called on incompatible receiver",
            "Array Iterator next called on out-of-bounds TypedArray",
            "String Iterator next called on incompatible receiver",
            "Map Iterator.prototype.next receiver does not have [[Map]]",
            "Map Iterator.prototype.next receiver is not an object",
            "Map.groupBy items cannot be null or undefined",
            "Map.groupBy callback must be callable",
            "Map.groupBy iterator method must be callable",
            "Map.groupBy iterator method must return an object",
            "Map.groupBy iterator next method must be callable",
            "Map.groupBy iterator produced too many values",
            "Map.groupBy iterator next result must be an object",
            "Map.prototype.getOrInsertComputed callback must be callable",
            "Map method receiver does not have [[MapData]]",
            "Map method receiver is not an object",
            "WeakMap",
            "WeakMap.prototype.getOrInsertComputed callback must be callable",
            "WeakMap method receiver does not have [[WeakMapData]]",
            "WeakMap method receiver is not an object",
            "WeakMap key must be an object or unregistered symbol",
            "WeakSet",
            "WeakSet constructor requires new",
            "WeakSet constructor add method is not callable",
            "WeakSet constructor iterator method is not callable",
            "WeakSet constructor iterator method must return an object",
            "WeakSet constructor iterator next method is not callable",
            "WeakSet constructor iterator next result must be an object",
            "WeakSet method receiver does not have [[WeakSetData]]",
            "WeakSet method receiver is not an object",
            "WeakSet value must be an object or unregistered symbol",
            "WeakRef",
            "WeakRef constructor requires new",
            "WeakRef target cannot be held weakly",
            "WeakRef.prototype.deref receiver does not have [[WeakRefTarget]]",
            "FinalizationRegistry",
            "register",
            "unregister",
            "FinalizationRegistry constructor requires new",
            "FinalizationRegistry cleanup callback is not callable",
            "FinalizationRegistry target cannot be held weakly",
            "FinalizationRegistry target and holdings must not be the same value",
            "FinalizationRegistry unregister token cannot be held weakly",
            "FinalizationRegistry method receiver does not have [[Cells]]",
            // `AsyncDisposableStack`: the property keys its intrinsic installer
            // defines, plus every message its emitters throw. A key or message
            // spelled at an emitter and missing here is a compile-time panic in
            // every full bootstrap (`string ... must exist in pool`), not a
            // runtime miss.
            "AsyncDisposableStack",
            "use",
            "adopt",
            "defer",
            "move",
            "disposed",
            "disposeAsync",
            "AsyncDisposableStack constructor requires new",
            "AsyncDisposableStack method receiver is not an object",
            "AsyncDisposableStack method receiver does not have [[AsyncDisposableState]]",
            "AsyncDisposableStack is already disposed",
            "AsyncDisposableStack.prototype.use value is not an object",
            "AsyncDisposableStack.prototype.use value is not disposable",
            "AsyncDisposableStack.prototype.use dispose method is not callable",
            "AsyncDisposableStack.prototype.adopt onDisposeAsync is not callable",
            "AsyncDisposableStack.prototype.defer onDisposeAsync is not callable",
            "Object.groupBy items cannot be null or undefined",
            "Object.groupBy callback must be callable",
            "Object.groupBy iterator method must be callable",
            "Object.groupBy iterator method must return an object",
            "Object.groupBy iterator next method must be callable",
            "Object.groupBy iterator produced too many values",
            "Object.groupBy iterator next result must be an object",
            "Set Iterator.prototype.next receiver does not have [[Set]]",
            "Set Iterator.prototype.next receiver is not an object",
            "Set method receiver does not have [[SetData]]",
            "Set method receiver is not an object",
            "Set method argument is not a set-like object",
            "Set-like size is NaN",
            "Set-like size is negative",
            "Set-like has method is not callable",
            "Set-like keys method is not callable",
            "Set-like keys method must return an object",
            "Set-like iterator next method is not callable",
            "Set-like iterator next result must be an object",
            "RegExp String Iterator next called on incompatible receiver",
            "RegExp String Iterator exec returned non-object",
            "Iterator",
            "toArray",
            "forEach",
            "groupBy",
            "every",
            "some",
            "find",
            "reduce",
            "map",
            "filter",
            "take",
            "drop",
            "Iterator constructor cannot be called",
            "Iterator Helper",
            "Iterator helper called on incompatible receiver",
            "Iterator.from called on null or undefined",
            "Iterator.from iterator method must return object",
            "Iterator.from iterator method must be callable",
            "Iterator.from next method must be callable",
            "Iterator.prototype.forEach called on null or undefined",
            "Iterator.prototype.forEach callback must be callable",
            "Iterator.prototype.forEach next method must be callable",
            "Iterator.prototype.forEach next result must be object",
            "Iterator.prototype.every called on null or undefined",
            "Iterator.prototype.every callback must be callable",
            "Iterator.prototype.every next method must be callable",
            "Iterator.prototype.every next result must be object",
            "Iterator.prototype.some called on null or undefined",
            "Iterator.prototype.some callback must be callable",
            "Iterator.prototype.some next method must be callable",
            "Iterator.prototype.some next result must be object",
            "Iterator.prototype.find called on null or undefined",
            "Iterator.prototype.find callback must be callable",
            "Iterator.prototype.find next method must be callable",
            "Iterator.prototype.find next result must be object",
            "Iterator.prototype.reduce called on null or undefined",
            "Iterator.prototype.reduce reducer must be callable",
            "Iterator.prototype.reduce next method must be callable",
            "Iterator.prototype.reduce next result must be object",
            "Iterator.prototype.reduce of empty iterator with no initial value",
            "Iterator.prototype.map called on null or undefined",
            "Iterator.prototype.map mapper must be callable",
            "Iterator.prototype.map next method must be callable",
            "Iterator map helper next called on incompatible receiver",
            "Iterator map helper is already running",
            "Iterator map helper next result must be object",
            "Iterator map helper return called on incompatible receiver",
            "Iterator map helper return method must be callable",
            "Iterator map helper return result must be object",
            "$IteratorMapIterator",
            "$IteratorMapNext",
            "$IteratorMapMapper",
            "$IteratorMapIndex",
            "$IteratorMapDone",
            "$IteratorMapExecuting",
            "Iterator.zip called with a non-object iterables value",
            "Iterator.zip iterator method must be callable",
            "Iterator.zip iterator method must return object",
            "Iterator.zip next method must be callable",
            "Iterator.zip next result must be object",
            "Iterator.zip options must be an object or undefined",
            "Iterator.zip mode must be a string or undefined",
            "Iterator.zip mode must be shortest, longest, or strict",
            "Iterator.zip padding must be an object or undefined",
            "Iterator.zip strict mode has iterators of different lengths",
            "Iterator.zipKeyed called with a non-object iterables value",
            "Iterator.zipKeyed property value must be an object",
            "Iterator.zipKeyed iterator method must be callable",
            "Iterator.zipKeyed iterator method must return object",
            "Iterator.zipKeyed options must be an object or undefined",
            "Iterator.zipKeyed mode must be a string or undefined",
            "Iterator.zipKeyed mode must be shortest, longest, or strict",
            "Iterator.zipKeyed padding must be an object or undefined",
            "Object.fromEntries iterator method is not callable",
            "Object.fromEntries iterator method must return an object",
            "Object.fromEntries iterator next method is not callable",
            "Object.fromEntries iterator next result must be an object",
            "Object.fromEntries iterator value must be an object",
            "Iterator.concat arguments must be objects",
            "Iterator.concat iterator method must be callable",
            "Iterator.concat iterator method must return object",
            "Iterator.concat next method must be callable",
            "Iterator.concat next result must be object",
            "Iterator concat helper called on incompatible receiver",
            "Iterator concat helper is already running",
            "Iterator zip helper next called on incompatible receiver",
            "Iterator zip helper is already running",
            "Iterator zip helper next result must be object",
            "Iterator zip helper return called on incompatible receiver",
            "$IteratorZipIterators",
            "$IteratorZipNextMethods",
            "$IteratorZipOpen",
            "$IteratorZipMode",
            "$IteratorZipPadding",
            "$IteratorZipKeys",
            "$IteratorZipDone",
            "$IteratorZipExecuting",
            "$IteratorZipStarted",
            "$IteratorConcatIterables",
            "$IteratorConcatMethods",
            "$IteratorConcatCurrentIterator",
            "$IteratorConcatCurrentNext",
            "$IteratorConcatIndex",
            "$IteratorConcatActive",
            "$IteratorConcatDone",
            "$IteratorConcatExecuting",
            "mode",
            "padding",
            "shortest",
            "longest",
            "strict",
            "zipKeyed",
            "Iterator.prototype.filter called on null or undefined",
            "Iterator.prototype.filter predicate must be callable",
            "Iterator.prototype.filter next method must be callable",
            "Iterator filter helper next called on incompatible receiver",
            "Iterator filter helper is already running",
            "Iterator filter helper next result must be object",
            "Iterator filter helper return called on incompatible receiver",
            "Iterator filter helper return method must be callable",
            "Iterator filter helper return result must be object",
            "$IteratorFilterIterator",
            "$IteratorFilterNext",
            "$IteratorFilterPredicate",
            "$IteratorFilterIndex",
            "$IteratorFilterDone",
            "$IteratorFilterExecuting",
            "flatMap",
            "Iterator.prototype.flatMap called on null or undefined",
            "Iterator.prototype.flatMap mapper must be callable",
            "Iterator.prototype.flatMap next method must be callable",
            "Iterator.prototype.flatMap mapper result must be object",
            "Iterator.prototype.flatMap inner iterator method must be callable",
            "Iterator.prototype.flatMap inner iterator method must return object",
            "Iterator.prototype.flatMap inner iterator next method must be callable",
            "Iterator flatMap helper next called on incompatible receiver",
            "Iterator flatMap helper is already running",
            "Iterator flatMap helper next result must be object",
            "Iterator flatMap helper return called on incompatible receiver",
            "Iterator flatMap helper return method must be callable",
            "Iterator flatMap helper return result must be object",
            "$IteratorFlatMapIterator",
            "$IteratorFlatMapNext",
            "$IteratorFlatMapMapper",
            "$IteratorFlatMapIndex",
            "$IteratorFlatMapDone",
            "$IteratorFlatMapExecuting",
            "$IteratorFlatMapInnerIterator",
            "$IteratorFlatMapInnerNext",
            "$IteratorFlatMapInnerActive",
            "Iterator.prototype.take called on null or undefined",
            "Iterator.prototype.take next method must be callable",
            "Iterator.prototype.take limit must be a non-negative number",
            "$IteratorTakeIterator",
            "$IteratorTakeNext",
            "$IteratorTakeRemaining",
            "$IteratorTakeDone",
            "$IteratorTakeExecuting",
            "Iterator take helper next called on incompatible receiver",
            "Iterator take helper is already running",
            "Iterator take helper next result must be object",
            "Iterator take helper return called on incompatible receiver",
            "Iterator take helper return method must be callable",
            "Iterator take helper return result must be object",
            "Iterator.prototype.drop called on null or undefined",
            "Iterator.prototype.drop next method must be callable",
            "Iterator.prototype.drop limit must be a non-negative number",
            "$IteratorDropIterator",
            "$IteratorDropNext",
            "$IteratorDropRemaining",
            "$IteratorDropDone",
            "$IteratorDropExecuting",
            "Iterator drop helper next called on incompatible receiver",
            "Iterator drop helper is already running",
            "Iterator drop helper next result must be object",
            "Iterator drop helper return called on incompatible receiver",
            "Iterator drop helper return method must be callable",
            "Iterator drop helper return result must be object",
            "Iterator.prototype.toArray called on null or undefined",
            "Iterator.prototype.toArray called on incompatible receiver",
            "Iterator.prototype.toArray next method must be callable",
            "Iterator.prototype.toArray next result must be object",
            "Iterator.prototype[Symbol.dispose] return method must be callable",
            "Iterator.prototype.constructor setter called on incompatible receiver",
            "Iterator.prototype[Symbol.toStringTag] setter called on incompatible receiver",
            "Iterator.from wrapper next called on incompatible receiver",
            "Iterator.from wrapper next method must be callable",
            "Iterator.from wrapper next result must be object",
            "Iterator.from wrapper return called on incompatible receiver",
            "Iterator.from wrapper return method must be callable",
            "Iterator.from wrapper return result must be object",
            "$LilaIteratorFromWrapper",
            "$IteratorFromIterator",
            "$IteratorFromNext",
            "Array.prototype.values called on null or undefined",
            "Array.from iterator method must return object",
            "Array.from iterator method must be callable",
            "Array.from iterator next must be callable",
            "Array.from iterator next result must be object",
            "Array.from mapper is not callable",
            "Array.from called on null or undefined",
            "Array.from index property is non-configurable",
            "Array.from target is not extensible",
            "Array.fromAsync mapper is not callable",
            "Array.fromAsync input is null or undefined",
            "Array.fromAsync iterator method is not callable",
            "Array.fromAsync iterator method must return object",
            "Array.fromAsync iterator next is not callable",
            "Array.fromAsync iterator next result must be object",
            "Array.fromAsync iterator produced too many values",
            "AsyncIterator asyncDispose receiver is null or undefined",
            "AsyncIterator asyncDispose return method is not callable",
            "Array.of index property is non-configurable",
            "Array.of target is not extensible",
            "TypedArray.from receiver is not a constructor",
            "TypedArray.from mapper is not callable",
            "TypedArray.from iterator next must be callable",
            "TypedArray.from iterator next result must be object",
            "TypedArray.from constructed target is not a typed array",
            "TypedArray.from constructed target is too small",
            "TypedArray.of receiver is not a constructor",
            "TypedArray constructor requires new",
            "TypedArray byteOffset out of range",
            "TypedArray byteOffset must be aligned",
            "TypedArray byteLength out of range",
            "TypedArray byteLength must be aligned",
            "TypedArray backing buffer is detached",
            "TypedArray byteLength out of bounds",
            "TypedArray length out of range",
            "TypedArray iterator method requires a TypedArray",
            "TypedArray find method requires a TypedArray",
            "TypedArray.prototype.includes requires a TypedArray",
            "TypedArray.prototype.indexOf requires a TypedArray",
            "TypedArray.prototype.lastIndexOf requires a TypedArray",
            "TypedArray.prototype.slice requires a TypedArray",
            "TypedArray.prototype.slice has unknown element type",
            "TypedArray.prototype.slice constructor property is not an object",
            "TypedArray.prototype.slice species is not a constructor",
            "TypedArray.prototype.slice species content type differs",
            "TypedArray.prototype.find predicate is not callable",
            "TypedArray.prototype.findIndex predicate is not callable",
            "TypedArray.prototype.findLast predicate is not callable",
            "TypedArray.prototype.findLastIndex predicate is not callable",
            "TypedArray every method requires a TypedArray",
            "TypedArray some method requires a TypedArray",
            "TypedArray.prototype.every callback is not callable",
            "TypedArray.prototype.some callback is not callable",
            "TypedArray.prototype.map requires a TypedArray",
            "TypedArray.prototype.map callback is not callable",
            "TypedArray.prototype.map has unknown element type",
            "TypedArray.prototype.map constructor property is not an object",
            "TypedArray.prototype.map species is not a constructor",
            "TypedArray.prototype.map species content type differs",
            "TypedArray.prototype.filter requires a TypedArray",
            "TypedArray.prototype.filter callback is not callable",
            "TypedArray.prototype.filter has unknown element type",
            "TypedArray.prototype.filter constructor property is not an object",
            "TypedArray.prototype.filter species is not a constructor",
            "TypedArray.prototype.filter species content type differs",
            "TypedArray.prototype.forEach requires a TypedArray",
            "TypedArray.prototype.forEach callback is not callable",
            "TypedArray.prototype.reduce requires a TypedArray",
            "TypedArray.prototype.reduce callback is not callable",
            "TypedArray.prototype.reduceRight requires a TypedArray",
            "TypedArray.prototype.reduceRight callback is not callable",
            "Reduce of empty typed array with no initial value",
            "TypedArray.prototype.toString requires TypedArray",
            "TypedArray.prototype.join requires a TypedArray",
            "TypedArray.prototype.toLocaleString requires TypedArray",
            "construct",
            "ownKeys",
            "has",
            "isArray",
            "isView",
            "isInteger",
            "isSafeInteger",
            "isFinite",
            "isNaN",
            "isError",
            "escape",
            "unescape",
            "asIntN",
            "asUintN",
            "E",
            "LN10",
            "LN2",
            "LOG10E",
            "LOG2E",
            "PI",
            "SQRT1_2",
            "SQRT2",
            "abs",
            "acos",
            "acosh",
            "asin",
            "asinh",
            "atan",
            "atan2",
            "atanh",
            "cbrt",
            "ceil",
            "clz32",
            "cos",
            "cosh",
            "exp",
            "expm1",
            "f16round",
            "floor",
            "fround",
            "hypot",
            "imul",
            "log",
            "log10",
            "log1p",
            "log2",
            "max",
            "min",
            "pow",
            "random",
            "round",
            "sign",
            "sin",
            "sinh",
            "sqrt",
            "sumPrecise",
            "tan",
            "tanh",
            "trunc",
            "toExponential",
            "toFixed",
            "toPrecision",
            "MAX_VALUE",
            "MIN_VALUE",
            "EPSILON",
            "MAX_SAFE_INTEGER",
            "MIN_SAFE_INTEGER",
            "POSITIVE_INFINITY",
            "NEGATIVE_INFINITY",
            "slice",
            "detached",
            "resize",
            "transfer",
            "transferToFixedLength",
            "transferToImmutable",
            "sliceToImmutable",
            "getUint8",
            "setUint8",
            "getInt8",
            "setInt8",
            "getUint16",
            "setUint16",
            "getInt16",
            "setInt16",
            "getUint32",
            "setUint32",
            "getInt32",
            "setInt32",
            "getFloat16",
            "setFloat16",
            "getFloat32",
            "setFloat32",
            "getFloat64",
            "setFloat64",
            "now",
            "UTC",
            "getTime",
            "setTime",
            "getFullYear",
            "getUTCFullYear",
            "getMonth",
            "getUTCMonth",
            "getDate",
            "getUTCDate",
            "getDay",
            "getUTCDay",
            "getHours",
            "getUTCHours",
            "getMinutes",
            "getUTCMinutes",
            "getSeconds",
            "getUTCSeconds",
            "getMilliseconds",
            "getUTCMilliseconds",
            "getTimezoneOffset",
            "getYear",
            "setYear",
            "toUTCString",
            "toGMTString",
            "buffer",
            "byteOffset",
            "byteLength",
            "maxByteLength",
            "resizable",
            "growable",
            "grow",
            "Symbol.dispose",
            "Symbol.species",
            "Symbol.isConcatSpreadable",
            "Symbol.match",
            "Symbol.matchAll",
            "Symbol.replace",
            "Symbol.search",
            "Symbol.split",
            "Symbol.toStringTag",
            "GeneratorFunction",
            "function GeneratorFunction() { [native code] }",
            "AsyncFunction",
            "function AsyncFunction() { [native code] }",
            "AsyncGenerator",
            "AsyncGeneratorFunction",
            "function AsyncGeneratorFunction() { [native code] }",
            "[Symbol.asyncIterator]",
            "function [Symbol.asyncIterator]() { [native code] }",
            "function next() { [native code] }",
            "function return() { [native code] }",
            "function throw() { [native code] }",
            "Symbol.toPrimitive",
            "Symbol is not a constructor",
            "Symbol.keyFor argument must be a symbol",
            "Symbol.asyncIterator",
            "Symbol.hasInstance",
            "Symbol.unscopables",
            "Symbol.asyncDispose",
            "Symbol",
            "Symbol(",
            "Symbol.prototype.description requires that 'this' be a Symbol",
            "Symbol.prototype.toString requires that 'this' be a Symbol",
            "Symbol.prototype.valueOf requires that 'this' be a Symbol",
            "Symbol.prototype[Symbol.toPrimitive] requires that 'this' be a Symbol",
            "Cannot create property on symbol",
            "iterator",
            "asyncIterator",
            "hasInstance",
            "isConcatSpreadable",
            "species",
            "toStringTag",
            "toPrimitive",
            "unscopables",
            "dispose",
            "asyncDispose",
            "for",
            "keyFor",
            "description",
            "(?:)",
            "source",
            "flags",
            "gc requires a real collector in wasm-aot",
            "parse",
            "stringify",
            "rawJSON",
            "isRawJSON",
            "add",
            "and",
            "compareExchange",
            "exchange",
            "load",
            "notify",
            "or",
            "pause",
            "store",
            "sub",
            "wait",
            "waitAsync",
            "xor",
            "Atomics.add requires an integer typed array",
            "Atomics.and requires an integer typed array",
            "Atomics.compareExchange requires an integer typed array",
            "Atomics.exchange requires an integer typed array",
            "Atomics.load requires an integer typed array",
            "Atomics.notify requires an Int32Array or BigInt64Array",
            "Atomics.or requires an integer typed array",
            "Atomics.pause iterationNumber must be a finite integral Number",
            "Atomics.store requires an integer typed array",
            "Atomics.sub requires an integer typed array",
            "Atomics.wait requires a shared Int32Array or BigInt64Array",
            "Atomics.waitAsync requires a shared Int32Array or BigInt64Array",
            "Atomics.xor requires an integer typed array",
            "Atomics.add index out of range",
            "Atomics.and index out of range",
            "Atomics.compareExchange index out of range",
            "Atomics.exchange index out of range",
            "Atomics.load index out of range",
            "Atomics.notify index out of range",
            "Atomics.or index out of range",
            "Atomics.store index out of range",
            "Atomics.sub index out of range",
            "Atomics.wait blocking wait queues unsupported in wasm-aot",
            "Atomics.wait index out of range",
            "Atomics.waitAsync blocking wait queues unsupported in wasm-aot",
            "Atomics.waitAsync index out of range",
            "not-equal",
            "timed-out",
            "ok",
            "Atomics.xor index out of range",
            "toJSON",
            "hasIndices",
            "unicodeSets",
            "ignoreCase",
            "multiline",
            "dotAll",
            "unicode",
            "sticky",
            "i",
            "m",
            "s",
            "u",
            "RegExp constructor is unsupported in wasm-aot",
            "Invalid regular expression flags",
            "Invalid regular expression pattern",
            "RegExp.prototype.flags getter receiver is not an object",
            "RegExp.prototype.exec receiver is not RegExp",
            "RegExp.prototype.exec source is not string",
            "RegExp.prototype.exec unsupported pattern",
            "RegExp.prototype.test receiver is not an object",
            "RegExp.prototype.test exec result is not an object or null",
            "RegExp.prototype[Symbol.match] flags is not string",
            "RegExp.prototype[Symbol.match] exec result is not object or null",
            "RegExp.prototype[Symbol.match] is unsupported in wasm-aot",
            ".(.).",
            "^|\\udf06",
            "RegExp.prototype[Symbol.matchAll] receiver is not RegExp",
            "RegExp.prototype[Symbol.matchAll] source is not string",
            "RegExp.prototype[Symbol.matchAll] flags is not string",
            "RegExp.prototype[Symbol.matchAll] species is not a constructor",
            "RegExp.prototype[Symbol.matchAll] is unsupported in wasm-aot",
            "RegExp.prototype[Symbol.search] receiver is not RegExp",
            "RegExp.prototype[Symbol.search] source is not string",
            "RegExp.prototype[Symbol.search] flags is not string",
            "RegExp.prototype[Symbol.search] exec result is not object or null",
            "RegExp.prototype[Symbol.search] is unsupported in wasm-aot",
            "RegExp.prototype[Symbol.split] receiver must be an object",
            "\u{20BB7}",
            "\u{10FFFF}",
            "\u{20BB7}a\u{20BB7}b\u{20BB7}",
            "a\u{20BB7}b\u{10FFFF}c",
            "\\p{Script=Han}",
            "\\P{ASCII}",
            "String.prototype.matchAll RegExp flags must contain g",
            "String.prototype.matchAll RegExp @@matchAll is not callable",
            "Invalid JSON.parse text",
            "Invalid JSON.rawJSON text",
            "standard builtin body is not emitted unless referenced directly",
            "Cannot redefine JSON reviver property",
            "Cannot add JSON reviver property",
            "ArrayBuffer",
            SHARED_ARRAY_BUFFER_NAME,
            "DataView",
            "get",
            "set",
            "clear",
            "delete",
            "size",
            "async",
            "value",
            "next",
            "done",
            "proxy",
            "revoke",
            "writable",
            "enumerable",
            "configurable",
            "$Proxy.target",
            "$Proxy.handler",
            "Proxy target must be object",
            "Proxy handler must be object",
            "Proxy get trap is not callable",
            "Proxy has trap is not callable",
            "Proxy getPrototypeOf trap is not callable",
            "Proxy getPrototypeOf trap result must be object or null",
            "Proxy getPrototypeOf trap result does not match target",
            "Proxy isExtensible trap is not callable",
            "Proxy isExtensible trap result does not match target",
            "Proxy setPrototypeOf trap is not callable",
            "Proxy setPrototypeOf trap result incompatible with non-extensible target",
            "Proxy handler is null",
            "Proxy set trap returned false",
            "Proxy set trap is not callable",
            "Proxy set trap result is incompatible with target descriptor",
            "Proxy get trap returned inconsistent frozen data property",
            "Proxy get trap returned value for accessor without getter",
            "Proxy getOwnPropertyDescriptor trap is not callable",
            "Proxy getOwnPropertyDescriptor trap result must be object or undefined",
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-configurable target property",
            "Proxy getOwnPropertyDescriptor trap returned undefined for non-extensible target",
            "Proxy getOwnPropertyDescriptor trap result incompatible with non-extensible target",
            "Proxy getOwnPropertyDescriptor trap result cannot report configurable for non-configurable target property",
            "Proxy getOwnPropertyDescriptor trap result cannot report non-configurable target property",
            "Proxy getOwnPropertyDescriptor trap result cannot report non-writable target property",
            "Proxy preventExtensions trap is not callable",
            "Proxy preventExtensions trap returned false",
            "Proxy preventExtensions trap returned true for extensible target",
            "Proxy defineProperty trap is not callable",
            "Proxy defineProperty trap returned false",
            "Proxy defineProperty trap cannot add property to non-extensible target",
            "Proxy defineProperty trap cannot define non-configurable target property",
            "Proxy defineProperty trap result is incompatible with target descriptor",
            "Proxy defineProperty trap cannot define non-writable target property",
            "Proxy defineProperty trap cannot report a writable target property as non-writable",
            "Cannot define invalid TypedArray index",
            "Cannot define incompatible TypedArray index descriptor",
            "TypedArray.prototype.subarray requires TypedArray",
            "TypedArray.prototype.subarray has unknown element type",
            "TypedArray.prototype.subarray constructor property is not an object",
            "TypedArray.prototype.subarray species is not a constructor",
            "TypedArray.prototype.subarray species did not return a TypedArray",
            "TypedArray.prototype.subarray species content type differs",
            "Constructed target is not a typed array",
            "Constructed typed array is too small",
            "TypedArray.prototype.set requires TypedArray",
            "TypedArray.prototype.set backing buffer is detached",
            "TypedArray.prototype.set offset is out of range",
            "TypedArray.prototype.set source buffer is detached",
            "TypedArray.prototype.set source and target content types differ",
            "TypedArray.prototype.set source is too large",
            "TypedArray.prototype.reverse requires TypedArray",
            "TypedArray.prototype.copyWithin requires TypedArray",
            "TypedArray.prototype.sort requires TypedArray",
            "TypedArray.prototype.toReversed requires TypedArray",
            "TypedArray.prototype.toReversed has unknown element type",
            "TypedArray.prototype.toSorted requires TypedArray",
            "TypedArray.prototype.toSorted has unknown element type",
            "TypedArray.prototype.with requires TypedArray",
            "TypedArray.prototype.with index out of range",
            "TypedArray.prototype.with has unknown element type",
            "Reflect.defineProperty target must be object",
            "Reflect.defineProperty attributes must be object",
            "Property descriptor getter/setter must be callable or undefined",
            "Property descriptor cannot be both accessor and data",
            "Reflect.get target must be object",
            "Reflect.has target must be object",
            "Reflect.getPrototypeOf target must be object",
            "Reflect.getOwnPropertyDescriptor target must be object",
            "Reflect.set target must be object",
            "deleteProperty",
            "Reflect.deleteProperty target must be object",
            "Reflect.isExtensible target must be object",
            "Reflect.preventExtensions target must be object",
            "Reflect.ownKeys target must be object",
            "Object.seal could not prevent extensions",
            "Object.seal could not make an own property non-configurable",
            "Object.freeze could not prevent extensions",
            "Object.freeze could not make an own property non-configurable and non-writable",
            "Proxy deleteProperty trap is not callable",
            "Proxy deleteProperty trap returned true for non-configurable target property",
            "Proxy deleteProperty trap returned true for non-extensible target property",
            "Proxy apply trap is not callable",
            "Proxy construct trap returned non-object",
            "Proxy construct trap is not callable",
            "Proxy has trap returned false for non-configurable target property",
            "Proxy has trap returned false for non-extensible target property",
            "Proxy ownKeys trap is not callable",
            "Proxy ownKeys trap result omitted target property",
            "2",
            "3",
            "4",
            "5",
            "10000",
            "prop",
            "f",
            "v",
            LILA_GENERATOR_THROW_SLOT,
            DATE_VALUE_SLOT,
            "EvalError",
            "AggregateError",
            "RangeError",
            "SyntaxError",
            "TypeError",
            "URIError",
            "URI contains a trailing high surrogate",
            "URI contains a high surrogate without a following low surrogate",
            "URI contains an unpaired low surrogate",
            "URI percent encoding starts with an invalid UTF-8 byte",
            "URI percent encoding contains an invalid UTF-8 continuation byte",
            "URI percent encoding is not a shortest-form Unicode scalar",
            "URI contains an incomplete percent escape",
            "URI UTF-8 continuation byte is not percent-escaped",
            "URI percent escape contains a non-hex digit",
            "ReferenceError",
            "class constructor cannot be invoked without `new`",
            "dynamic Function constructor unsupported",
            "Math.sumPrecise non-number element",
            "Function.prototype.call receiver is not callable",
            "Function.prototype.apply receiver is not callable",
            "Function.prototype.bind receiver is not callable",
            "Function.prototype.toString receiver is not callable",
            "value is not callable",
            "Function.prototype.call primitive thisArg boxing unsupported",
            "Function.prototype.apply primitive thisArg boxing unsupported",
            "Function.prototype.call/apply thisArg adaptation failed",
            "Function.prototype.apply argument list must be array or arguments",
            "Object.prototype.toLocaleString called on null or undefined",
            "Object.prototype.toLocaleString target is not callable",
            "Array.prototype.concat receiver is not array",
            "Array.prototype.concat called on null or undefined",
            "Array.prototype.toLocaleString called on null or undefined",
            "Array.prototype.toLocaleString element method is not callable",
            "TypedArray.prototype.toLocaleString element method is not callable",
            "Array.prototype.flat receiver is not array",
            "Array.prototype.flat called on null or undefined",
            "Array.prototype.flat constructor is not object",
            "Array.prototype.flatMap receiver is not array",
            "Array.prototype.flatMap called on null or undefined",
            "Array.prototype.flatMap mapper is not callable",
            "Array.prototype.flatMap constructor is not object",
            "Array.prototype.flatMap cannot add property to non-extensible target",
            "Array.prototype.flatMap cannot define non-configurable target property",
            "Array.prototype.at called on null or undefined",
            "TypedArray.prototype.at called on incompatible receiver",
            "Array.prototype.includes receiver is not array",
            "Array.prototype.includes called on null or undefined",
            "Array.prototype.indexOf called on null or undefined",
            "Array.prototype.lastIndexOf called on null or undefined",
            "Array.prototype.find called on null or undefined",
            "Array.prototype.findIndex called on null or undefined",
            "Array.prototype.findLast called on null or undefined",
            "Array.prototype.findLastIndex called on null or undefined",
            "Array.prototype.find predicate is not callable",
            "Array.prototype.findIndex predicate is not callable",
            "Array.prototype.findLast predicate is not callable",
            "Array.prototype.findLastIndex predicate is not callable",
            "Array.prototype.reduce callback is not callable",
            "Array.prototype.reduceRight callback is not callable",
            "Reduce of empty array with no initial value",
            "First argument to String.prototype.endsWith must not be a RegExp",
            "First argument to String.prototype.includes must not be a RegExp",
            "String.prototype.match RegExp @@match is not callable",
            "Array.prototype.concat constructor is not object",
            "Array.prototype.concat cannot add property to non-extensible target",
            "Array.prototype.concat cannot define non-configurable target property",
            "Array species constructor is not a constructor",
            "Cannot add property to non-extensible target",
            "Cannot define non-configurable target property",
            "Array.prototype.map receiver is not array",
            "Array.prototype.map called on null or undefined",
            "Array.prototype.map mapper is not callable",
            "Array.prototype.map constructor is not object",
            "Array.prototype.every receiver is not array",
            "Array.prototype.every called on null or undefined",
            "Array.prototype.every callback is not callable",
            "Array.prototype.every constructor is not object",
            "Array.prototype.some receiver is not array",
            "Array.prototype.some called on null or undefined",
            "Array.prototype.some callback is not callable",
            "Array.prototype.some constructor is not object",
            "Array.prototype.filter receiver is not array",
            "Array.prototype.filter called on null or undefined",
            "Array.prototype.filter callback is not callable",
            "Array.prototype.filter constructor is not object",
            "Array.prototype.filter cannot add property to non-extensible target",
            "Array.prototype.filter cannot define non-configurable target property",
            "Invalid array length",
            "Array accumulation index exceeds exact backend range",
            "Array.prototype.with index out of range",
            "Array.prototype.toSpliced result exceeds maximum safe length",
            "Array.prototype.concat result exceeds maximum safe length",
            "Object.prototype.valueOf called on null or undefined",
            "Cannot convert undefined or null to object",
            "Object.setPrototypeOf target must be object",
            "Object.setPrototypeOf prototype must be object or null",
            "Object.setPrototypeOf returned false",
            "Object.hasOwn called on null or undefined",
            "Object.getOwnPropertyDescriptor called on null or undefined",
            "Object.getOwnPropertyNames called on null or undefined",
            "Object.getOwnPropertySymbols called on null or undefined",
            "Object.keys requires object",
            "Object.values requires object",
            "Object.assign called on null or undefined",
            "Object.entries called on null or undefined",
            "Object.getOwnPropertyDescriptors called on null or undefined",
            "Object.values called on null or undefined",
            "String.prototype method requires a String receiver",
            "Number.prototype method requires a Number receiver",
            "Boolean.prototype method requires a Boolean receiver",
            "Number.prototype.toString radix out of range",
            "BigInt.prototype.toString radix out of range",
            "Number.prototype.toFixed fraction digits out of range",
            "Number.prototype.toPrecision precision out of range",
            "Cannot convert a Symbol value to a number",
            "1000000000000000128",
            "4294967295",
            "Array.prototype.pop receiver is not array",
            "Array.prototype.push receiver is not array",
            "Array.prototype.push length exceeds safe integer",
            "Array.prototype.push length is not writable",
            "Cannot assign to read only property",
            "Cannot delete property",
            "Cannot add property to non-extensible object",
            "Array.prototype.push index write failed",
            "Array.prototype.unshift called on null or undefined",
            "Array.prototype.unshift cannot modify a string",
            "Array.prototype.unshift length exceeds safe integer",
            "Array.prototype.unshift cannot delete destination property",
            "Array.prototype.shift called on null or undefined",
            "Array.prototype.shift cannot modify a string",
            "Array.prototype.shift cannot delete property",
            "Array.prototype.splice receiver is not array",
            "Array.prototype.sort receiver is not array",
            "TypedArray accessor requires TypedArray",
            "Date method receiver is not Date",
            "Date value is not finite",
            "Date toISOString method is not callable",
            "Date.prototype[Symbol.toPrimitive] receiver is not an object",
            "Date.prototype[Symbol.toPrimitive] hint must be \"default\", \"number\", or \"string\"",
            "Invalid Date",
            "Sun",
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Jan",
            "Feb",
            "Mar",
            "Apr",
            "May",
            "Jun",
            "Jul",
            "Aug",
            "Sep",
            "Oct",
            "Nov",
            "Dec",
            ", ",
            " GMT",
            " GMT+0000 (Coordinated Universal Time)",
            "Thu Jan 01 1970 00:00:00 GMT+0000 (Coordinated Universal Time)",
            "Thu, 01 Jan 1970 00:00:00 GMT",
            "-",
            "+",
            ":",
            ".",
            "T",
            "Z",
            "toISOString",
            "toTemporalInstant",
            "Temporal",
            "Now",
            "Temporal.Now",
            "instant",
            "zonedDateTimeISO",
            "Instant",
            "PlainDate",
            "Temporal.PlainDate",
            "compare",
            "with",
            "withCalendar",
            "era",
            "eraYear",
            // The `era` half of `CalendarResolveFields` is one emitter shared
            // by `PlainDate`, `PlainDateTime`, `PlainYearMonth`,
            // `PlainMonthDay` and `ZonedDateTime`, so its messages are not
            // per-family and cannot sit behind any one family's gate. They
            // join the unconditional block beside the two property names the
            // same emitter reads.
            "Temporal era must be a string",
            "Temporal eraYear must be finite",
            "Temporal era and eraYear must be provided together",
            "Invalid Temporal era for this calendar",
            "Temporal era and year must agree",
            "Temporal.PlainMonthDay year is outside the supported range",
            "dayOfWeek",
            "dayOfYear",
            "weekOfYear",
            "yearOfWeek",
            "daysInWeek",
            "daysInMonth",
            "daysInYear",
            "monthsInYear",
            "inLeapYear",
            "toLocaleString",
            "Temporal.Instant",
            "from",
            "epochMilliseconds",
            "epochNanoseconds",
            "equals",
            "Intl",
            "Intl.Locale",
            "getCanonicalLocales",
            "Locale",
            "language",
            "script",
            "region",
            "baseName",
            "Intl.Locale constructor requires new",
            "Intl.Locale tag must be a string or an object",
            "Intl.Locale.prototype method called on incompatible receiver",
            "Intl.getCanonicalLocales argument must be an object",
            "Intl.getCanonicalLocales locale must be a string or an object",
            "Invalid language tag",
            "Temporal.Instant constructor requires new",
            "Temporal.Instant.from requires a string or Temporal.Instant",
            "Invalid Temporal.Instant string",
            "ZonedDateTime",
            "Temporal.ZonedDateTime",
            "timeZoneId",
            "calendarId",
            "year",
            "month",
            "monthCode",
            "day",
            "hour",
            "minute",
            "second",
            "millisecond",
            "microsecond",
            "nanosecond",
            "M01",
            "M02",
            "M03",
            "M04",
            "M05",
            "M06",
            "M07",
            "M08",
            "M09",
            "M10",
            "M11",
            "M12",
            "toInstant",
            "UTC",
            "Temporal.ZonedDateTime constructor requires new",
            "Temporal.ZonedDateTime time zone must be a string",
            "Invalid Temporal.ZonedDateTime time zone",
            "Invalid Temporal time zone identifier",
            "Temporal time zone string requires an offset or bracketed time zone",
            "Temporal time zone offset must use minute precision",
            "Temporal.ZonedDateTime calendar must be a string",
            "Invalid Temporal.ZonedDateTime calendar",
            "Temporal.ZonedDateTime receiver does not have [[InitializedTemporalZonedDateTime]]",
            "Temporal.Instant receiver does not have [[InitializedTemporalInstant]]",
            "Temporal.Instant epoch nanoseconds are outside the supported range",
            "Temporal.Instant.fromEpochMilliseconds requires an integral Number",
            "Temporal.Instant does not support implicit conversion; use compare() or equals()",
            "RegExp.escape input must be a string",
            "RegExp.prototype.compile receiver is not a direct RegExp instance",
            "RegExp.prototype.compile flags must be undefined when pattern is RegExp",
            "invalid regular-expression flag",
            "duplicate regular-expression flag",
            "Array.prototype.forEach receiver is not array",
            "Array.prototype.forEach called on null or undefined",
            "Array.prototype.forEach callback is not callable",
            "Cannot convert object to number",
            "Error.prototype.toString receiver is not object",
            "String.prototype HTML method receiver is null or undefined",
            "String.prototype method receiver is null or undefined",
            "repeat count must be non-negative and finite",
            "repeat result would exceed maximum string length",
            "First argument to String.prototype.startsWith must not be a RegExp",
            "String.prototype RegExp/string fallback is unsupported in wasm-aot",
            "String.prototype symbol hook is not callable",
            "AggregateError errors input must be array or arguments",
            "AggregateError errors input must be iterable",
            "AggregateError iterator method must return object",
            "AggregateError iterator next must be callable",
            "AggregateError iterator next result must be object",
            "assert.throws expected a throw",
            "assert.throws expected an error object",
            "assert.throws received the wrong error constructor",
            "assert.throws requires a function callback",
            "assert.throws callback is a class constructor",
            "target is not a constructor",
            "Reflect.construct target is not a constructor",
            "Reflect.construct newTarget is not a constructor",
            "Reflect.construct argumentsList must be an array",
            "Reflect.construct argumentsList must be array-like",
            "Reflect.apply argumentsList must be an array",
            "Reflect.apply argumentsList must be array-like",
            "Reflect.apply target must be callable",
            "Function.prototype.apply argument list must be array-like",
            "Object.create prototype must be object or null",
            "Object.defineProperties target must be object",
            "Object.defineProperties properties must not be null or undefined",
            "Object.defineProperties descriptor must be object",
            "Object.defineProperty attributes must be object",
            "Cannot convert object to primitive value",
            "ArrayBuffer byteLength getter requires ArrayBuffer",
            "ArrayBuffer detached getter requires ArrayBuffer",
            "ArrayBuffer slice receiver is not ArrayBuffer",
            "ArrayBuffer slice receiver is detached",
            "ArrayBuffer slice source is shorter than the resolved final bound",
            "ArrayBuffer species constructor returned invalid ArrayBuffer",
            "ArrayBuffer constructor requires new",
            "ArrayBuffer maxByteLength getter requires ArrayBuffer",
            "ArrayBuffer resizable getter requires ArrayBuffer",
            "ArrayBuffer resize receiver is not resizable ArrayBuffer",
            "ArrayBuffer resize length is out of range",
            "ArrayBuffer resize is not supported by this host",
            "ArrayBuffer transfer receiver is not ArrayBuffer",
            "ArrayBuffer transfer receiver is detached",
            "ArrayBuffer transfer length is out of range",
            "ArrayBuffer receiver is immutable",
            "ArrayBuffer receiver is SharedArrayBuffer",
            "ArrayBuffer allocation size is too large",
            "SharedArrayBuffer allocation exceeds the wasm-aot shared-memory limit",
            "TypedArray allocation exceeds the wasm-aot buffer-memory limit",
            "SharedArrayBuffer getter requires SharedArrayBuffer",
            "SharedArrayBuffer slice receiver is not SharedArrayBuffer",
            "SharedArrayBuffer species constructor returned invalid SharedArrayBuffer",
            "SharedArrayBuffer grow receiver is not growable SharedArrayBuffer",
            "SharedArrayBuffer grow length is out of range",
            "SharedArrayBuffer grow length is smaller than its current byte length",
            "detachArrayBuffer expects an ArrayBuffer",
            "detachArrayBuffer key does not match the ArrayBuffer detach key",
            "failed to start Test262 agent",
            "agent.broadcast requires SharedArrayBuffer",
            "Test262 agent stopped before receiving a broadcast",
            "DataView accessor requires DataView",
            "DataView backing buffer is detached",
            "DataView backing buffer is immutable",
            "DataView constructor requires new",
            "DataView constructor requires ArrayBuffer",
            "DataView byteOffset out of bounds",
            "DataView byteLength out of bounds",
            "DataView getUint8 index out of bounds",
            "DataView getInt8 index out of bounds",
            "DataView setUint8 index out of bounds",
            "DataView getUint16 index out of bounds",
            "DataView getInt16 index out of bounds",
            "DataView setUint16 index out of bounds",
            "DataView getUint32 index out of bounds",
            "DataView getInt32 index out of bounds",
            "DataView setUint32 index out of bounds",
            "DataView getFloat16 index out of bounds",
            "DataView setFloat16 index out of bounds",
            "DataView getFloat32 index out of bounds",
            "DataView setFloat32 index out of bounds",
            "DataView getFloat64 index out of bounds",
            "DataView setFloat64 index out of bounds",
            "DataView getBigInt64 index out of bounds",
            "DataView getBigUint64 index out of bounds",
            "DataView setBigInt64 index out of bounds",
            "DataView setBigUint64 index out of bounds",
            "cannot convert Number to BigInt",
            "cannot convert non-integer Number to BigInt",
            "cannot convert value to BigInt",
            "Cannot convert BigInt to number",
            "Cannot mix BigInt and other types",
            "BigInt exponent must be non-negative",
            "Do not know how to serialize a BigInt",
            "Converting circular structure to JSON",
            "Cannot convert Symbol to number",
            "Cannot convert a Symbol value to a string",
            "BigInt is not a constructor",
            "right-hand side of `in` is not an object",
            "Right-hand side of 'instanceof' is not callable",
            "must call super() before accessing `this`",
            "derived constructor must call super() before returning",
            "derived constructor may only return object or undefined",
            "super() called twice in derived constructor",
            "super() invalid in class extending null",
            "super property access on null base",
            "private field access on wrong object",
            GLOBAL_THIS_NAME,
            PRINT_NAME,
            IS_CONSTRUCTOR_NAME,
            "<a name=\"",
            "\">",
            "</a>",
            "<big>",
            "</big>",
            "<blink>",
            "</blink>",
            "<b>",
            "</b>",
            "<tt>",
            "</tt>",
            "<font color=\"",
            "<font size=\"",
            "</font>",
            "<i>",
            "</i>",
            "<a href=\"",
            "<small>",
            "</small>",
            "<strike>",
            "</strike>",
            "<sub>",
            "</sub>",
            "<sup>",
            "</sup>",
            "&quot;",
        ] {
            pool.intern_string(value);
        }
        // Unconditional, and it must stay unconditional: these are the messages
        // `emit_runtime_error_object` now reads out of the pool, and the paths
        // that throw them (`null.x`, an unbound identifier, a TDZ read) are
        // reachable from any program at all. Gating them behind a feature
        // predicate would reintroduce the exact `string must exist in pool`
        // panic this table exists to prevent.
        for value in RUNTIME_ERROR_MESSAGE_LITERALS {
            pool.intern_string(value);
        }
        for failure in RegExpMatcherFailure::ALL {
            pool.intern_string(failure.message());
        }
        // Every `TemporalCalendarId` spelling and canonical form, plus every
        // `Era` code, derived from the tables the emitters read rather than
        // listed again. Listing them was a standing drift risk in both
        // directions: a calendar spelling added to `TemporalCalendarId` and
        // forgotten here is the `string must exist in pool` compiler panic
        // fixed in e04bdc061, and `"gregorian"` was already interned twice
        // because it is also an `INTL_DTF_ACCEPTED_CALENDARS` row.
        //
        // Unconditional, and it cannot move behind the `Temporal.PlainDate`
        // gate below: the shared calendar helpers
        // (`compile_temporal_calendar_identifier_helper` and
        // `compile_temporal_calendar_iso_date_probe_helper`) are compiled from
        // `uses_temporal_calendar` in `emit.rs`, whose predicate also fires for
        // a program touching only `Temporal.ZonedDateTime` — and that program
        // does not satisfy the gate.
        for calendar in TemporalCalendarId::ALL {
            pool.intern_string(calendar.canonical());
            for spelling in calendar.spellings() {
                pool.intern_string(spelling);
            }
            // Every era spelling, not just `code()`: `code()` is defined as
            // `spellings()[0]`, and `CalendarResolveFields` matches an incoming
            // `era` against all of them (`ad`/`bc` are the CLDR aliases of
            // `ce`/`bce`). Interning the same table the resolver reads is what
            // makes "add an alias" a one-place change instead of three.
            //
            // The walk is `TemporalCalendarId::ALL -> eras() -> spellings()`,
            // i.e. literally the table
            // `FunctionBuilder::emit_temporal_resolve_era_to_year` reads, and
            // not a second `Era`-side list that could be short of it. That
            // matters because the resolver emits `strings.payload(spelling)`
            // for every spelling of every era of every calendar: an era
            // reachable from `eras()` but missing here is the `string must
            // exist in pool` compiler panic fixed in e04bdc061, and no
            // `const` assertion can see a list that is never consulted.
            for era in calendar.eras() {
                for spelling in era.spellings() {
                    pool.intern_string(spelling);
                }
            }
        }
        for value in crate::builtins::intl_date_time_format_pool_strings() {
            pool.intern_string(&value);
        }
        for index in 0..=31 {
            pool.intern_string(&index.to_string());
        }
        for (_, _, value) in NUMBER_TO_PRECISION_CASES {
            pool.intern_string(value);
        }
        for binding in script.global_bindings.iter() {
            pool.intern_string(&binding.name);
        }
        for meta in function_metas.values() {
            pool.intern_string(&meta.name);
            pool.intern_string(meta.runtime_name());
            pool.intern_string(&meta.to_string_value);
        }
        for builtin in StandardBuiltinId::all_functions() {
            pool.intern_string(&format!(
                "standard builtin body is not emitted unless referenced directly: {}",
                builtin.debug_name()
            ));
        }
        for function in &script.functions {
            if let Some(plan) = &function.class_instance_element_plan {
                for private_name_id in &plan.private_method_brands {
                    pool.intern_string(&private_brand_key(*private_name_id));
                }
                for field in &plan.fields {
                    match &field.key {
                        ClassFieldKeyIr::Public(key) => pool.intern_string(key),
                        ClassFieldKeyIr::ComputedPublic(_) => {}
                        ClassFieldKeyIr::Private(private_name_id) => {
                            pool.intern_string(&private_data_key(*private_name_id));
                            pool.intern_string(&private_brand_key(*private_name_id));
                        }
                    }
                }
            }
            for param in &function.params {
                pool.intern_string(&param.name);
                if let Some(default_init) = &param.default_init {
                    pool.collect_expr(default_init);
                }
            }
            for binding in &function.owned_env_bindings {
                pool.intern_string(&binding.name);
            }
            for binding in &function.captured_bindings {
                pool.intern_string(&binding.name);
            }
            pool.collect_block(&function.body);
        }
        pool.collect_block(&script.body);
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToLowerCase
                    | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            )
        }) {
            pool.intern_string("ς");
        }
        if pool.needs_runtime_regexp_programs {
            pool.queue_runtime_regexp_programs();
        }
        pool.append_regexp_programs();
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToLowerCase
                    | StandardBuiltinId::StringPrototypeToLocaleLowerCase
            )
        }) {
            pool.append_lowercase_tables();
        }
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::StringPrototypeToUpperCase
                    | StandardBuiltinId::StringPrototypeToLocaleUpperCase
            )
        }) {
            pool.append_uppercase_tables();
        }
        for property in ["offset", "offsetNanoseconds"] {
            pool.intern_string(property);
        }
        // The `Temporal.PlainDate` error messages and parser-internal literals
        // stay behind a gate: the family is 26 builtins wide and a script that
        // never touches a date would otherwise carry all of it in its data
        // segment. The gate is "any member compiled", not "the constructor
        // compiled", because the stub predicate decides per builtin.
        //
        // The property NAMES are deliberately NOT here — every builtin
        // constructor object is initialized regardless of what the script
        // references, so `install_temporal_plain_date_constructor_intrinsics`
        // needs them unconditionally.
        if compiled_standard_builtins.iter().any(|builtin| {
            matches!(
                builtin,
                StandardBuiltinId::TemporalPlainDateConstructor
                    | StandardBuiltinId::TemporalPlainDateFrom
                    | StandardBuiltinId::TemporalPlainDateCompare
            ) || builtin
                .debug_name()
                .starts_with("Temporal.PlainDate.prototype.")
                || builtin
                    .debug_name()
                    .starts_with("get Temporal.PlainDate.prototype.")
                // `Temporal.PlainDateTime`, `Temporal.PlainYearMonth` and
                // `Temporal.PlainMonthDay` reuse the `Temporal.PlainDate`
                // emitters wholesale, so they need their literals too.
                || builtin.debug_name().contains("Temporal.PlainDateTime")
                || builtin.debug_name().contains("Temporal.PlainYearMonth")
                || builtin.debug_name().contains("Temporal.PlainMonthDay")
        }) {
            for value in [
                "calendarId",
                "year",
                "month",
                "monthCode",
                "day",
                "equals",
                "toString",
                "from",
                "calendar",
                "timeZone",
                "overflow",
                "constrain",
                "reject",
                "calendarName",
                "auto",
                "always",
                "never",
                "critical",
                "iso8601",
                "",
                "-",
                "+",
                "[u-ca=",
                "[!u-ca=",
                "]",
                "M01",
                "M02",
                "M03",
                "M04",
                "M05",
                "M06",
                "M07",
                "M08",
                "M09",
                "M10",
                "M11",
                "M12",
                "Temporal.PlainDate constructor requires new",
                "Temporal.PlainDate year must be an integer",
                "Temporal.PlainDate month must be an integer",
                "Temporal.PlainDate day must be an integer",
                "Temporal.PlainDate calendar must be a string",
                "Invalid Temporal.PlainDate calendar",
                "Temporal.PlainDate is not a valid ISO date",
                "Temporal.PlainDate is outside the supported date range",
                "Temporal.PlainDate receiver does not have [[InitializedTemporalDate]]",
                "Temporal.PlainDate expects a string, a property bag, or a Temporal.PlainDate",
                "Temporal.PlainDate monthCode must be a string",
                "Temporal.PlainDate fields must be finite",
                "Temporal.PlainDate fields require year",
                "Temporal.PlainDate fields require day",
                "Temporal.PlainDate fields require month or monthCode",
                "Invalid Temporal.PlainDate monthCode",
                "Temporal.PlainDate month and monthCode must agree",
                "Temporal.PlainDate month and day must be positive",
                "Temporal.PlainDate options must be an object or undefined",
                "Invalid Temporal.PlainDate overflow option",
                // `until`/`since` reject an out-of-range smallestUnit or
                // largestUnit with this one message for both options.
                "Invalid Temporal.PlainDate unit option",
                // `CalendarEquals` in `DifferenceTemporal*`. One message per
                // family, spelled by `TemporalDifferenceGuard` and nowhere
                // else. These three entries keep their historical position in
                // this array on purpose: pool offsets are assignment-ordered,
                // so deleting them in favour of the walk below is a pure
                // reordering that still moves every later string's offset, and
                // that needs rung G. The walk is idempotent
                // (`intern_string` returns early on a hit), so it adds nothing
                // here and only fills the case this gate misses.
                TemporalDifferenceGuard::PlainDateSameCalendar.message(),
                TemporalDifferenceGuard::PlainDateTimeSameCalendar.message(),
                TemporalDifferenceGuard::PlainYearMonthSameCalendar.message(),
                "Invalid Temporal.PlainDate calendarName option",
                "Temporal.PlainDate.prototype.with requires an object",
                "Temporal.PlainDate.prototype.with does not accept calendar or timeZone",
                "Temporal.PlainDate.prototype.with does not accept a Temporal object",
                "Temporal.PlainDate.prototype.with requires at least one date field",
                "Temporal.PlainDate string must not use the UTC designator",
                "Invalid Temporal.PlainDate calendar annotation",
                "Invalid Temporal.PlainDate string",
                "Temporal.PlainDate does not support implicit conversion; use compare() or equals()",
            ] {
                pool.intern_string(value);
            }
        }
        // The `Temporal.PlainYearMonth` and `Temporal.PlainMonthDay` families
        // each share one prototype, and a realm bootstrap installs the whole
        // family without every member showing up in
        // `compiled_standard_builtins`, so these are interned unconditionally,
        // the same way the `Temporal.PlainTime` set above is.
        {
            for value in [
                "",
                "+",
                "-",
                "-01",
                "01",
                "1972",
                "1972-",
                // The shared `ToTemporalCalendarIdentifier` helper inlines the
                // ISO-date parser and is emitted for every calendar-bearing
                // Temporal family, so its PlainDate diagnostics cannot stay
                // behind the PlainDate-only gate above.
                "Invalid Temporal.PlainDate calendar annotation",
                "Invalid Temporal.PlainDate string",
                // The same shared helper's final time-string arm resolves
                // calendar annotations rather than using PlainTime's
                // ignore-calendar policy.
                "Invalid Temporal time-string calendar annotation",
                "Invalid Temporal.PlainMonthDay calendarName option",
                "Invalid Temporal.PlainMonthDay monthCode",
                "Invalid Temporal.PlainMonthDay overflow option",
                "Invalid Temporal.PlainYearMonth calendarName option",
                "Invalid Temporal.PlainYearMonth largestUnit",
                "Invalid Temporal.PlainYearMonth monthCode",
                "Invalid Temporal.PlainYearMonth overflow option",
                "Invalid Temporal.PlainYearMonth smallestUnit",
                "M01",
                "M02",
                "M03",
                "M04",
                "M05",
                "M06",
                "M07",
                "M08",
                "M09",
                "M10",
                "M11",
                "M12",
                "PlainMonthDay",
                "PlainYearMonth",
                "Temporal partial-date strings must not carry a UTC designator",
                "Temporal.PlainDate string must not use the UTC designator",
                "Temporal.PlainMonthDay",
                "Temporal.PlainMonthDay constructor requires new",
                "Temporal.PlainMonthDay day must be an integer",
                "Temporal.PlainMonthDay does not support implicit conversion; use compare() or equals()",
                "Temporal.PlainMonthDay expects a string, a property bag, or a Temporal.PlainMonthDay",
                "Temporal.PlainMonthDay fields require day",
                "Temporal.PlainMonthDay fields require month or monthCode",
                "Temporal.PlainMonthDay is not a valid ISO date",
                // `ToTemporalMonthDay` step (k): a non-ISO calendar bounds the
                // *parsed* date by `ISODateWithinLimits`, so this is thrown by
                // the shared `emit_temporal_iso_date_within_limits` rather than
                // by a `Temporal.PlainDate` emitter, and it needs its own row.
                "Temporal.PlainMonthDay is outside the supported date range",
                "Temporal.PlainMonthDay month and day must be positive",
                "Temporal.PlainMonthDay month and monthCode must agree",
                // `ToTemporalMonthDay` step (g). `emit_throw_*_range_error`
                // resolves its message through `StringPool::payload`, which
                // panics rather than interning, so an emitter made reachable
                // without a row here is a compiler panic on every program that
                // touches `Temporal.PlainMonthDay.from` or `.prototype.equals`,
                // not a test failure.
                "Temporal.PlainMonthDay month-day string with a non-ISO calendar requires a year",
                "Temporal.PlainMonthDay month must be an integer",
                "Temporal.PlainMonthDay options must be an object or undefined",
                "Temporal.PlainMonthDay receiver does not have [[InitializedTemporalMonthDay]]",
                "Temporal.PlainMonthDay reference year must be an integer",
                "Temporal.PlainMonthDay year must be finite",
                "Temporal.PlainMonthDay.prototype.toPlainDate requires a year",
                "Temporal.PlainMonthDay.prototype.toPlainDate requires an object",
                "Temporal.PlainMonthDay.prototype.with does not accept calendar or timeZone",
                "Temporal.PlainMonthDay.prototype.with does not accept a Temporal object",
                "Temporal.PlainMonthDay.prototype.with requires an object",
                "Temporal.PlainMonthDay.prototype.with requires at least one field",
                "Temporal.PlainYearMonth",
                "Temporal.PlainYearMonth arithmetic accepts only years and months",
                "Temporal.PlainYearMonth constructor requires new",
                "Temporal.PlainYearMonth day must be finite",
                "Temporal.PlainYearMonth day must be positive",
                "Temporal.PlainYearMonth does not support implicit conversion; use compare() or equals()",
                "Temporal.PlainYearMonth expects a string, a property bag, or a Temporal.PlainYearMonth",
                "Temporal.PlainYearMonth fields must be finite",
                "Temporal.PlainYearMonth fields require month or monthCode",
                "Temporal.PlainYearMonth fields require year",
                "Temporal.PlainYearMonth is not a valid ISO date",
                "Temporal.PlainYearMonth is outside the supported range",
                "Temporal.PlainYearMonth month and monthCode must agree",
                "Temporal.PlainYearMonth month must be an integer",
                "Temporal.PlainYearMonth month must be positive",
                "Temporal.PlainYearMonth monthCode must be a string",
                "Temporal.PlainYearMonth options must be an object or undefined",
                "Temporal.PlainYearMonth receiver does not have [[InitializedTemporalYearMonth]]",
                "Temporal.PlainYearMonth reference day must be an integer",
                "Temporal.PlainYearMonth year must be an integer",
                "Temporal.PlainYearMonth.prototype.toPlainDate requires a day",
                "Temporal.PlainYearMonth.prototype.toPlainDate requires an object",
                "Temporal.PlainYearMonth.prototype.with does not accept calendar or timeZone",
                "Temporal.PlainYearMonth.prototype.with does not accept a Temporal object",
                "Temporal.PlainYearMonth.prototype.with requires an object",
                "Temporal.PlainYearMonth.prototype.with requires at least one field",
                "[!u-ca=",
                "[u-ca=",
                "]",
                "add",
                "always",
                "auto",
                "calendar",
                "calendarId",
                "calendarName",
                "compare",
                "constrain",
                "critical",
                "day",
                "daysInMonth",
                "daysInYear",
                "equals",
                "era",
                "eraYear",
                "from",
                "inLeapYear",
                "iso8601",
                "largestUnit",
                "month",
                "monthCode",
                "monthsInYear",
                "never",
                "overflow",
                "reject",
                "roundingIncrement",
                "roundingMode",
                "since",
                "smallestUnit",
                "smallestUnit must be smaller than largestUnit",
                "subtract",
                "timeZone",
                "toJSON",
                "toLocaleString",
                "toPlainDate",
                "toString",
                "until",
                "valueOf",
                "with",
                "year",
            ] {
                pool.intern_string(value);
            }
        }
        // The `Temporal.PlainTime` family shares one prototype, and a realm
        // bootstrap installs the whole family without every member showing up
        // in `compiled_standard_builtins`, so these are interned
        // unconditionally, the same way the Duration set below is.
        {
            for value in [
                "PlainTime",
                "Temporal.PlainTime",
                "until",
                "since",
                "equals",
                "overflow",
                "constrain",
                "reject",
                "calendar",
                "timeZone",
                ":",
                "0000-01-01T",
                "Temporal.PlainTime constructor requires new",
                "Temporal.PlainTime field must be an integer",
                "Temporal.PlainTime field must be finite",
                "Temporal.PlainTime field is out of range",
                "Temporal.PlainTime receiver does not have [[InitializedTemporalTime]]",
                "Temporal.PlainTime requires at least one time field",
                "Temporal.PlainTime expects a string, a property bag, or a Temporal.PlainTime",
                "Invalid Temporal.PlainTime overflow option",
                "Invalid Temporal.PlainTime unit option",
                "Invalid Temporal.PlainTime rounding increment",
                "Invalid Temporal.PlainTime fractionalSecondDigits option",
                "Invalid Temporal.PlainTime string",
                "Temporal.PlainTime string must not use the UTC designator",
                "Ambiguous Temporal.PlainTime string requires the T designator",
                "Temporal.PlainTime.prototype.with requires an object",
                "Temporal.PlainTime.prototype.with does not accept calendar or timeZone",
                "Temporal.PlainTime.prototype.with does not accept a Temporal object",
                "Temporal.PlainTime.prototype.round requires a roundTo argument",
                "Temporal.PlainTime.prototype.round requires smallestUnit",
                "Temporal.PlainTime does not support implicit conversion; use compare() or equals()",
            ] {
                pool.intern_string(value);
            }
        }
        // The `Temporal.PlainDateTime` family shares one prototype and is
        // installed wholesale by a realm bootstrap, so its literals are interned
        // unconditionally the same way the PlainTime set above is.
        //
        // THIS BLOCK IS ALSO WHAT KEEPS `Temporal.ZonedDateTime.prototype`
        // INSTALLABLE, which its name does not say. `withCalendar`, `add`,
        // `subtract`, `until` and `since` below are the property keys
        // `install_temporal_zoned_date_time_constructor_intrinsics` passes to
        // `emit_object_define_function_data`, which reaches
        // `StringPool::payload` — and that PANICS rather than degrading on a
        // string that was never interned (the failure mode measured this batch
        // as 24 red `--lib` tests on ``string `...` must exist in pool``, from
        // the two ZonedDateTime guard messages). Being unconditional is what
        // makes that safe today. If this block is ever put behind a
        // PlainDateTime predicate, the ZonedDateTime prototype install goes with
        // it and every program touching `Temporal.ZonedDateTime` panics at emit;
        // give those five their own gate keyed on
        // `names::TEMPORAL_ZONED_DATE_TIME_PROTOTYPE_METHODS` at the same time.
        {
            for value in [
                "PlainDateTime",
                "iso8601",
                "calendar",
                "timeZone",
                "calendarName",
                "auto",
                "always",
                "never",
                "critical",
                "overflow",
                "constrain",
                "reject",
                "smallestUnit",
                "largestUnit",
                "roundingMode",
                "roundingIncrement",
                "fractionalSecondDigits",
                "year",
                "month",
                "monthCode",
                "day",
                "hour",
                "minute",
                "second",
                "millisecond",
                "microsecond",
                "nanosecond",
                "with",
                "add",
                "subtract",
                "until",
                "since",
                "round",
                "equals",
                "from",
                "compare",
                "toString",
                "toJSON",
                "toLocaleString",
                "valueOf",
                "withCalendar",
                "calendarId",
                "era",
                "eraYear",
                "dayOfWeek",
                "dayOfYear",
                "weekOfYear",
                "yearOfWeek",
                "daysInWeek",
                "daysInMonth",
                "daysInYear",
                "monthsInYear",
                "inLeapYear",
                "",
                "-",
                "+",
                ".",
                ":",
                "[u-ca=",
                "[!u-ca=",
                "]",
                "UTC",
                "M01",
                "M02",
                "M03",
                "M04",
                "M05",
                "M06",
                "M07",
                "M08",
                "M09",
                "M10",
                "M11",
                "M12",
                "Temporal.PlainDateTime",
                "withPlainTime",
                "toPlainDate",
                "toPlainTime",
                "toZonedDateTime",
                "T",
                "Temporal.PlainDateTime constructor requires new",
                "Temporal.PlainDateTime field must be an integer",
                "Temporal.PlainDateTime fields must be finite",
                "Temporal.PlainDateTime is not a valid ISO date",
                "Temporal.PlainDateTime is outside the supported date range",
                "Temporal.PlainDateTime calendar must be a string",
                "Invalid Temporal.PlainDateTime calendar",
                "Invalid Temporal.PlainDateTime string",
                "Invalid Temporal.PlainDateTime calendar annotation",
                "Temporal.PlainDateTime string must not use the UTC designator",
                "Temporal.PlainDateTime receiver does not have [[InitializedTemporalDateTime]]",
                "Temporal.PlainDateTime expects a string, a property bag, or a Temporal.PlainDateTime",
                "Temporal.PlainDateTime fields require year",
                "Temporal.PlainDateTime fields require day",
                "Temporal.PlainDateTime fields require month or monthCode",
                "Invalid Temporal.PlainDateTime monthCode",
                "Temporal.PlainDateTime month and monthCode must agree",
                "Temporal.PlainDateTime month and day must be positive",
                "Invalid Temporal.PlainDateTime overflow option",
                "Invalid Temporal.PlainDateTime calendarName option",
                "Invalid Temporal.PlainDateTime unit option",
                "Invalid Temporal.PlainDateTime rounding increment",
                "Temporal.PlainDateTime options must be an object or undefined",
                "Temporal.PlainDateTime.prototype.with requires an object",
                "Temporal.PlainDateTime.prototype.with does not accept calendar or timeZone",
                "Temporal.PlainDateTime.prototype.with does not accept a Temporal object",
                "Temporal.PlainDateTime.prototype.with requires at least one date or time field",
                "Temporal.PlainDateTime.prototype.round requires a roundTo argument",
                "Temporal.PlainDateTime.prototype.round requires smallestUnit",
                "Temporal.PlainDateTime.prototype.toZonedDateTime requires a time zone",
                "Temporal.PlainDateTime does not support implicit conversion; use compare() or equals()",
            ] {
                pool.intern_string(value);
            }
        }
        // The `Temporal.Duration` family shares one prototype, and a realm
        // bootstrap installs the whole family without every member showing up
        // in `compiled_standard_builtins`, so these are interned
        // unconditionally rather than gated on a member reference.
        {
            for value in [
                "Duration",
                "Temporal.Duration",
                "years",
                "months",
                "weeks",
                "days",
                "hours",
                "minutes",
                "seconds",
                "milliseconds",
                "microseconds",
                "nanoseconds",
                "year",
                "month",
                "week",
                "day",
                "hour",
                "minute",
                "second",
                "millisecond",
                "microsecond",
                "nanosecond",
                "sign",
                "blank",
                "negated",
                "abs",
                "add",
                "subtract",
                "round",
                "total",
                "with",
                "from",
                "compare",
                "toString",
                "toJSON",
                "toLocaleString",
                "valueOf",
                "largestUnit",
                "fractionalSecondDigits",
                "smallestUnit",
                "roundingIncrement",
                "roundingMode",
                "relativeTo",
                "unit",
                "auto",
                "ceil",
                "floor",
                "expand",
                "trunc",
                "halfCeil",
                "halfFloor",
                "halfExpand",
                "halfTrunc",
                "halfEven",
                "P",
                "T",
                "Y",
                "M",
                "W",
                "D",
                "H",
                "S",
                "PT0S",
                "-",
                "+",
                ".",
                "0",
                "",
                "Temporal.Duration constructor requires new",
                "Temporal.Duration receiver does not have [[InitializedTemporalDuration]]",
                "Temporal.Duration field must be an integer",
                "Invalid Temporal.Duration: fields must not exceed the supported range",
                "Temporal.Duration expects a string, a property bag, or a Temporal.Duration",
                "Temporal.Duration requires at least one duration field",
                "Invalid Temporal.Duration string",
                "Temporal.Duration options must be an object or undefined",
                "Temporal.Duration.prototype.with requires an object",
                "Temporal.Duration.prototype.with does not accept a Temporal.Duration",
                "Invalid Temporal.Duration unit option",
                "Invalid Temporal.Duration rounding mode",
                "Invalid Temporal.Duration rounding increment",
                "Temporal.Duration.prototype.round requires largestUnit, smallestUnit, or both",
                "Temporal.Duration.prototype.total requires a unit",
                "Temporal.Duration operation requires relativeTo for calendar units",
                "Temporal.Duration does not support implicit conversion; use compare()",
                "smallestUnit must be smaller than largestUnit",
            ] {
                pool.intern_string(value);
            }
        }
        if compiled_standard_builtins.contains(&StandardBuiltinId::TemporalZonedDateTimeFrom) {
            for value in [
                "Temporal.ZonedDateTime.from requires a string or Temporal.ZonedDateTime",
                "Temporal.ZonedDateTime.from options must be an object or undefined",
                "calendar",
                "day",
                "hour",
                "microsecond",
                "millisecond",
                "minute",
                "month",
                "monthCode",
                "nanosecond",
                "offset",
                "second",
                "timeZone",
                "year",
                "M01",
                "M02",
                "M03",
                "M04",
                "M05",
                "M06",
                "M07",
                "M08",
                "M09",
                "M10",
                "M11",
                "M12",
                "UTC",
                "iso8601",
                "disambiguation",
                "compatible",
                "earlier",
                "later",
                "reject",
                "use",
                "prefer",
                "ignore",
                "overflow",
                "constrain",
                "Temporal.ZonedDateTime monthCode must be a string",
                "Temporal.ZonedDateTime offset must be a string",
                "Temporal.ZonedDateTime property bag requires year",
                "Temporal.ZonedDateTime property bag requires day",
                "Temporal.ZonedDateTime property bag requires timeZone",
                "Temporal.ZonedDateTime property bag requires month or monthCode",
                "Invalid Temporal.ZonedDateTime monthCode",
                "Temporal.ZonedDateTime month and monthCode must agree",
                "Temporal.ZonedDateTime month and day must be positive",
                "Temporal.ZonedDateTime property bag field must be finite",
                "Temporal.ZonedDateTime property bag year is outside the supported instant range",
                "Temporal.ZonedDateTime property bag month is out of range",
                "Temporal.ZonedDateTime property bag date-time field is out of range",
                "Temporal.ZonedDateTime time zone must be a string",
                "Invalid Temporal.ZonedDateTime time zone",
                "Temporal.ZonedDateTime calendar must be a string",
                "Invalid Temporal.ZonedDateTime calendar",
                "Invalid Temporal.ZonedDateTime disambiguation option",
                "Invalid Temporal.ZonedDateTime offset option",
                "Invalid Temporal.ZonedDateTime overflow option",
                "Invalid Temporal.ZonedDateTime string",
                "Temporal.ZonedDateTime string requires one bracketed time zone",
                "Invalid Temporal.ZonedDateTime calendar annotation",
                "Temporal.ZonedDateTime offset does not match its fixed time zone",
            ] {
                pool.intern_string(value);
            }
        }
        // Every `DifferenceTemporal*` guard message, derived from the domain the
        // emitters read rather than listed again — the same construction as the
        // `TemporalCalendarId::ALL -> eras() -> spellings()` walk above, and for
        // the same reason.
        //
        // The message is a pool string read back with `StringPool::payload`,
        // which *panics* rather than degrading when the string was never
        // interned. Batch 6 added the two `Temporal.ZonedDateTime` guards
        // (`builtins/temporal_zoned_date_time_methods.rs`) as bare `&str`
        // literals with no matching entry here, and
        // `cargo test -p lila-aot-wasm --lib` went **24 red** on
        // ``string `...` must exist in pool`` — every test that emits a full
        // bootstrap, not only the Temporal ones, because the panic is in the
        // bootstrap and not in the feature.
        //
        // Walking the domain is what stops the fifth family repeating it: a
        // `TemporalDifferenceGuard` variant cannot compile without a `message()`
        // and an `emitting_builtins()` arm, and this loop then interns it with
        // no edit in this file. The per-guard gate is what keeps a program that
        // touches no `until`/`since` from carrying the text.
        for guard in TemporalDifferenceGuard::ALL {
            if guard
                .emitting_builtins()
                .iter()
                .any(|builtin| compiled_standard_builtins.contains(builtin))
            {
                pool.intern_string(guard.message());
            }
        }
        if compiled_standard_builtins.contains(&StandardBuiltinId::StringPrototypeNormalize)
            || compiled_standard_builtins.contains(&StandardBuiltinId::StringPrototypeLocaleCompare)
        {
            for form in ["NFC", "NFD", "NFKC", "NFKD"] {
                pool.intern_string(form);
            }
            pool.intern_string("String.prototype.normalize receiver is null or undefined");
            pool.intern_string("String.prototype.normalize form must be NFC, NFD, NFKC, or NFKD");
            pool.append_normalization_tables();
        }
        pool
    }

    fn append_normalization_tables(&mut self) {
        let tables = NORMALIZATION_TABLES.get_or_init(|| {
            let nfd = DecomposingNormalizerBorrowed::new_nfd();
            let nfkd = DecomposingNormalizerBorrowed::new_nfkd();
            let combining_classes = CanonicalCombiningClassMapBorrowed::new();
            let compositions = CanonicalCompositionBorrowed::new();
            let canonical_decomposition = CanonicalDecompositionBorrowed::new();
            let mut tables = NormalizationTables {
                canonical_mappings: Vec::new(),
                canonical_sequences: Vec::new(),
                compatibility_mappings: Vec::new(),
                compatibility_sequences: Vec::new(),
                combining_classes: Vec::new(),
                compositions: Vec::new(),
            };

            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let source = codepoint.to_string();
                let canonical: Vec<u32> = nfd.normalize(&source).chars().map(u32::from).collect();
                if canonical.as_slice() != [u32::from(codepoint)] {
                    tables.canonical_mappings.push(NormalizationMapping {
                        codepoint: u32::from(codepoint),
                        sequence_index: tables.canonical_sequences.len() as u32,
                        sequence_len: canonical.len() as u32,
                    });
                    tables.canonical_sequences.extend(canonical);
                }

                let compatibility: Vec<u32> =
                    nfkd.normalize(&source).chars().map(u32::from).collect();
                if compatibility.as_slice() != [u32::from(codepoint)] {
                    tables.compatibility_mappings.push(NormalizationMapping {
                        codepoint: u32::from(codepoint),
                        sequence_index: tables.compatibility_sequences.len() as u32,
                        sequence_len: compatibility.len() as u32,
                    });
                    tables.compatibility_sequences.extend(compatibility);
                }

                let combining_class = combining_classes.get_u8(codepoint);
                if combining_class != 0 {
                    tables
                        .combining_classes
                        .push((u32::from(codepoint), combining_class));
                }

                if let Decomposed::Expansion(first, second) =
                    canonical_decomposition.decompose(codepoint)
                {
                    if let Some(composed) = compositions.compose(first, second) {
                        tables.compositions.push((
                            u32::from(first),
                            u32::from(second),
                            u32::from(composed),
                        ));
                    }
                }
            }
            tables.compositions.sort_unstable();
            tables.compositions.dedup();
            tables
        });

        let canonical_sequences_ptr = self.append_codepoints(&tables.canonical_sequences);
        self.canonical_decomposition_table_ptr =
            self.append_normalization_mappings(&tables.canonical_mappings, canonical_sequences_ptr);
        self.canonical_decomposition_count = tables.canonical_mappings.len() as u32;
        let compatibility_sequences_ptr = self.append_codepoints(&tables.compatibility_sequences);
        self.compatibility_decomposition_table_ptr = self.append_normalization_mappings(
            &tables.compatibility_mappings,
            compatibility_sequences_ptr,
        );
        self.compatibility_decomposition_count = tables.compatibility_mappings.len() as u32;

        self.align_bytes(8);
        self.combining_class_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for (codepoint, combining_class) in &tables.combining_classes {
            self.bytes.extend_from_slice(&codepoint.to_le_bytes());
            self.bytes
                .extend_from_slice(&u32::from(*combining_class).to_le_bytes());
        }
        self.combining_class_count = tables.combining_classes.len() as u32;

        self.align_bytes(8);
        self.composition_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for (first, second, composed) in &tables.compositions {
            self.bytes.extend_from_slice(&first.to_le_bytes());
            self.bytes.extend_from_slice(&second.to_le_bytes());
            self.bytes.extend_from_slice(&composed.to_le_bytes());
            self.bytes.extend_from_slice(&0_u32.to_le_bytes());
        }
        self.composition_count = tables.compositions.len() as u32;
    }

    fn append_codepoints(&mut self, codepoints: &[u32]) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for codepoint in codepoints {
            self.bytes.extend_from_slice(&codepoint.to_le_bytes());
        }
        table_ptr
    }

    fn append_normalization_mappings(
        &mut self,
        mappings: &[NormalizationMapping],
        sequences_ptr: u32,
    ) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for mapping in mappings {
            self.bytes
                .extend_from_slice(&mapping.codepoint.to_le_bytes());
            self.bytes
                .extend_from_slice(&(sequences_ptr + mapping.sequence_index * 4).to_le_bytes());
            self.bytes
                .extend_from_slice(&mapping.sequence_len.to_le_bytes());
            self.bytes.extend_from_slice(&0_u32.to_le_bytes());
        }
        table_ptr
    }

    fn append_lowercase_tables(&mut self) {
        let tables = LOWERCASE_TABLES.get_or_init(|| {
            let mut mappings = Vec::new();
            let mut mapping_count = 0;
            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let mut lowercase = codepoint.to_lowercase();
                let first = lowercase.next().expect("lowercase mapping is never empty");
                let second = lowercase.next();
                if first == codepoint && second.is_none() {
                    continue;
                }

                let mut lowercase_bytes = Vec::with_capacity(4);
                for lowercase_codepoint in std::iter::once(first).chain(second).chain(lowercase) {
                    let mut encoded = [0; 4];
                    lowercase_bytes.extend_from_slice(
                        lowercase_codepoint.encode_utf8(&mut encoded).as_bytes(),
                    );
                }
                debug_assert!(lowercase_bytes.len() <= 4);
                mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
                mappings.extend_from_slice(&(lowercase_bytes.len() as u32).to_le_bytes());
                mappings.extend_from_slice(&lowercase_bytes);
                mappings.resize(mappings.len() + 4 - lowercase_bytes.len(), 0);
                mappings.extend_from_slice(&0_u32.to_le_bytes());
                mapping_count += 1;
            }

            LowercaseTables {
                mappings,
                mapping_count,
                cased_ranges: CodePointSetData::new::<props::Cased>()
                    .iter_ranges()
                    .collect(),
                case_ignorable_ranges: CodePointSetData::new::<props::CaseIgnorable>()
                    .iter_ranges()
                    .collect(),
            }
        });

        self.align_bytes(8);
        self.lowercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(&tables.mappings);
        self.lowercase_mapping_count = tables.mapping_count;
        self.cased_range_table_ptr = self.append_codepoint_ranges(&tables.cased_ranges);
        self.cased_range_count = tables.cased_ranges.len() as u32;
        self.case_ignorable_range_table_ptr =
            self.append_codepoint_ranges(&tables.case_ignorable_ranges);
        self.case_ignorable_range_count = tables.case_ignorable_ranges.len() as u32;
    }

    fn append_uppercase_tables(&mut self) {
        let tables = UPPERCASE_TABLES.get_or_init(|| {
            let mut mappings = Vec::new();
            let mut mapping_count = 0;
            for codepoint in (0..=char::MAX as u32).filter_map(char::from_u32) {
                let mut uppercase = codepoint.to_uppercase();
                let first = uppercase.next().expect("uppercase mapping is never empty");
                let second = uppercase.next();
                if first == codepoint && second.is_none() {
                    continue;
                }

                let mut uppercase_bytes = Vec::with_capacity(8);
                for uppercase_codepoint in std::iter::once(first).chain(second).chain(uppercase) {
                    let mut encoded = [0; 4];
                    uppercase_bytes.extend_from_slice(
                        uppercase_codepoint.encode_utf8(&mut encoded).as_bytes(),
                    );
                }
                debug_assert!(uppercase_bytes.len() <= 8);
                mappings.extend_from_slice(&(codepoint as u32).to_le_bytes());
                mappings.extend_from_slice(&(uppercase_bytes.len() as u32).to_le_bytes());
                mappings.extend_from_slice(&uppercase_bytes);
                mappings.resize(mappings.len() + 8 - uppercase_bytes.len(), 0);
                mapping_count += 1;
            }

            UppercaseTables {
                mappings,
                mapping_count,
            }
        });

        self.align_bytes(8);
        self.uppercase_mapping_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.bytes.extend_from_slice(&tables.mappings);
        self.uppercase_mapping_count = tables.mapping_count;
    }

    fn append_codepoint_ranges(&mut self, ranges: &[std::ops::RangeInclusive<u32>]) -> u32 {
        self.align_bytes(8);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        for range in ranges {
            self.bytes.extend_from_slice(&range.start().to_le_bytes());
            self.bytes.extend_from_slice(&range.end().to_le_bytes());
        }
        table_ptr
    }

    fn align_bytes(&mut self, alignment: usize) {
        let padding = (alignment - self.bytes.len() % alignment) % alignment;
        self.bytes.resize(self.bytes.len() + padding, 0);
    }

    fn collect_block(&mut self, block: &BlockIr) {
        for statement in &block.statements {
            self.collect_statement(statement);
        }
    }

    fn collect_statement(&mut self, statement: &StatementIr) {
        match statement {
            StatementIr::ModuleUnitOnce { block, .. } => self.collect_block(block),
            StatementIr::Empty
            | StatementIr::Debugger
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. } => {}
            StatementIr::Lexical { name, init, .. } => {
                self.intern_string(name);
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(init);
            }
            StatementIr::AnnexBFunctionCopy {
                source_name,
                block_storage_name,
                target,
            } => {
                self.intern_string(source_name);
                self.intern_string(block_storage_name);
                match target {
                    AnnexBFunctionCopyTargetIr::OwnerBinding { storage_name } => {
                        self.intern_string(storage_name);
                    }
                    AnnexBFunctionCopyTargetIr::ScriptGlobal { name } => {
                        self.intern_string(name);
                    }
                }
            }
            StatementIr::Expression(init) => self.collect_expr(init),
            StatementIr::GeneratorYield {
                value,
                form,
                resume_mode,
                ..
            } => {
                match form {
                    YieldForm::Plain => {}
                    YieldForm::Delegate(_) => {
                        for key in [
                            "Symbol.iterator",
                            "next",
                            "done",
                            "value",
                            "return",
                            "throw",
                        ] {
                            self.intern_string(key);
                        }
                    }
                }
                if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                    match reference.use_view() {
                        SuspendedPropertyReferenceUse::Ordinary {
                            base_and_receiver,
                            key,
                            strictness: _,
                        } => {
                            self.collect_expr(base_and_receiver);
                            self.collect_property_key(key);
                        }
                    }
                }
                self.collect_expr(value);
            }
            StatementIr::AsyncAwait { value, .. } => self.collect_expr(value),
            StatementIr::Return(value) => self.collect_expr(value),
            StatementIr::Throw(value) => self.collect_expr(value),
            StatementIr::Var(declarators) => self.collect_var_declarators(declarators),
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                for statement in statements {
                    self.collect_statement(statement);
                }
            }
            StatementIr::Block(block) => self.collect_block(block),
            StatementIr::TryCatch {
                try_block,
                catch_block,
                catch_name,
                catch_source_name,
                ..
            } => {
                self.intern_string(catch_name);
                self.intern_string(catch_source_name);
                self.collect_block(try_block);
                self.collect_block(catch_block);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                self.collect_block(try_block);
                self.collect_block(finally_block);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                catch_name,
                catch_source_name,
                ..
            } => {
                self.intern_string(catch_name);
                self.intern_string(catch_source_name);
                self.collect_block(try_block);
                self.collect_block(catch_block);
                self.collect_block(finally_block);
            }
            StatementIr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.collect_expr(condition);
                self.collect_statement(then_branch);
                if let Some(else_branch) = else_branch {
                    self.collect_statement(else_branch);
                }
            }
            StatementIr::While { condition, body } => {
                self.collect_expr(condition);
                self.collect_statement(body);
            }
            StatementIr::DoWhile { body, condition } => {
                self.collect_statement(body);
                self.collect_expr(condition);
            }
            StatementIr::For {
                init,
                test,
                update,
                body,
                ..
            } => {
                if let Some(init) = init {
                    self.collect_for_init(init);
                }
                if let Some(test) = test {
                    self.collect_expr(test);
                }
                if let Some(update) = update {
                    self.collect_expr(update);
                }
                self.collect_statement(body);
            }
            StatementIr::GeneratorLoop {
                init,
                test,
                update,
                before_suspension,
                suspension_statement,
                after_suspension,
                ..
            } => {
                if let Some(init) = init {
                    self.collect_for_init(init);
                }
                if let Some(test) = test {
                    self.collect_expr(test);
                }
                if let Some(update) = update {
                    self.collect_expr(update);
                }
                for statement in before_suspension {
                    self.collect_statement(statement);
                }
                self.collect_statement(suspension_statement);
                for statement in after_suspension {
                    self.collect_statement(statement);
                }
            }
            StatementIr::GeneratorIf {
                condition,
                then_before_yield,
                then_yield_statement,
                then_after_yield,
                else_before_yield,
                else_yield_statement,
                else_after_yield,
                ..
            } => {
                self.collect_expr(condition);
                for statement in then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                {
                    self.collect_statement(statement);
                }
            }
            StatementIr::ForOfArray { iterable, body, .. }
            | StatementIr::ForOfString { iterable, body, .. }
            | StatementIr::ForOfIterator { iterable, body, .. }
            | StatementIr::ForInArray {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInString {
                target: iterable,
                body,
                ..
            }
            | StatementIr::ForInObject {
                target: iterable,
                body,
                ..
            } => {
                self.collect_expr(iterable);
                self.collect_statement(body);
            }
            StatementIr::Switch {
                discriminant,
                lexical_declarations,
                cases,
                ..
            } => {
                self.collect_expr(discriminant);
                for declaration in lexical_declarations {
                    self.collect_statement(declaration);
                }
                for case in cases {
                    if let Some(condition) = &case.condition {
                        self.collect_expr(condition);
                    }
                    self.collect_block(&case.body);
                }
            }
            StatementIr::Labelled { statement, .. } => self.collect_statement(statement),
        }
    }

    fn collect_for_init(&mut self, init: &ForInitIr) {
        match init {
            ForInitIr::Lexical { init, .. } => {
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(init);
            }
            ForInitIr::LexicalBlock(bindings) => {
                for binding in bindings {
                    collect_finite_string_choices(
                        &binding.init,
                        &mut self.runtime_regexp_candidate_literals,
                    );
                    self.collect_expr(&binding.init);
                }
            }
            ForInitIr::Var(declarators) => self.collect_var_declarators(declarators),
            ForInitIr::Expression(expr) => self.collect_expr(expr),
            ForInitIr::Statements(statements) => {
                for statement in statements {
                    self.collect_statement(statement);
                }
            }
        }
    }

    fn collect_var_declarators(&mut self, declarators: &[VarDeclaratorIr]) {
        for declarator in declarators {
            self.intern_string(&declarator.name);
            if let Some(init) = &declarator.init {
                collect_finite_string_choices(init, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(init);
            }
        }
    }

    fn collect_expr(&mut self, expr: &TypedExpr) {
        match &expr.expr {
            // A namespace object and `import.meta` are allocated from static
            // tables the module graph owns, not from expression operands.
            ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => {
                self.uses_heap = true;
            }
            ExprIr::DynamicImport {
                specifier, options, ..
            } => {
                self.uses_heap = true;
                self.collect_expr(specifier);
                if let Some(options) = options {
                    self.collect_expr(options);
                }
            }
            ExprIr::Symbol { description } => {
                self.uses_heap = true;
                if let Some(description) = description {
                    self.collect_expr(description);
                }
            }
            ExprIr::String(value) => {
                self.intern_string(value);
                self.script_string_literals.insert(value.clone());
            }
            ExprIr::TemplateObject(template) => {
                self.uses_heap = true;
                for cooked in template.cooked.iter().flatten() {
                    self.intern_string(cooked);
                }
                for raw in &template.raw {
                    self.intern_string(raw);
                }
                self.intern_string("raw");
                if let Some(previous) = self
                    .template_objects
                    .insert(template.site_id, template.clone())
                {
                    debug_assert_eq!(previous, *template);
                }
            }
            ExprIr::RegExpLiteral {
                source,
                flags,
                program,
            } => {
                self.uses_heap = true;
                self.intern_string(source);
                self.intern_string(flags);
                if let Some(program) = program {
                    self.queue_regexp_program(program);
                }
            }
            ExprIr::BigInt(_) => {}
            ExprIr::ObjectLiteral(properties) => {
                self.uses_heap = true;
                for property in properties {
                    match property {
                        ObjectPropertyIr::PrototypeSetter { value } => {
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::Spread { source } => {
                            self.collect_expr(source);
                        }
                        ObjectPropertyIr::Data { key, value, .. }
                        | ObjectPropertyIr::NonEnumerableData { key, value } => {
                            self.intern_string(key);
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::ComputedData { key, value } => {
                            self.collect_expr(key);
                            self.collect_expr(value);
                        }
                        ObjectPropertyIr::ComputedMethod { key, function }
                        | ObjectPropertyIr::ComputedGetter { key, function }
                        | ObjectPropertyIr::ComputedSetter { key, function } => {
                            self.collect_expr(key);
                            self.collect_expr(function);
                        }
                        ObjectPropertyIr::Method { key, function }
                        | ObjectPropertyIr::Getter { key, function }
                        | ObjectPropertyIr::Setter { key, function } => {
                            self.intern_string(key);
                            self.collect_expr(function);
                        }
                    }
                }
            }
            ExprIr::ArrayLiteral(elements) => {
                self.uses_heap = true;
                for element in elements {
                    collect_finite_string_choices(
                        element,
                        &mut self.runtime_regexp_candidate_literals,
                    );
                    self.collect_expr(element);
                }
            }
            ExprIr::ArrayAccumulation(accumulation) => {
                self.uses_heap = true;
                for element in accumulation.elements() {
                    let value = match element {
                        ArrayAccumulationElementIr::Elision => continue,
                        ArrayAccumulationElementIr::Value(value) => value,
                        ArrayAccumulationElementIr::Spread(spread) => &spread.value,
                    };
                    collect_finite_string_choices(
                        value,
                        &mut self.runtime_regexp_candidate_literals,
                    );
                    self.collect_expr(value);
                }
            }
            ExprIr::PropertyRead { target, key } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::OptionalPropertyChain { target, chain } => {
                self.uses_heap = true;
                self.collect_expr(target);
                for operation in chain {
                    match operation {
                        OptionalChainOperationIr::Property { key, .. } => {
                            self.collect_property_key(key);
                        }
                        OptionalChainOperationIr::PrivateProperty { .. } => {}
                        OptionalChainOperationIr::Call { args, .. } => {
                            for arg in args {
                                self.collect_expr(arg);
                            }
                        }
                    }
                }
            }
            ExprIr::PropertyWrite {
                target, key, value, ..
            } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::PropertyUpdate { target, key, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::PropertyCompoundAssign {
                target, key, value, ..
            } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::AssignIdentifier { name, value } => {
                self.intern_string(name);
                collect_finite_string_choices(value, &mut self.runtime_regexp_candidate_literals);
                self.collect_expr(value);
            }
            ExprIr::CompoundAssignIdentifier { name, value, .. } => {
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::UnaryNumber { expr: value, .. }
            | ExprIr::UnaryBitwiseNumeric { expr: value, .. }
            | ExprIr::LogicalNot { expr: value }
            | ExprIr::Void { expr: value }
            | ExprIr::DeleteValue { expr: value } => self.collect_expr(value),
            ExprIr::SpecOperation {
                operation,
                operands,
            } => {
                if matches!(operation, SpecOperationIr::ToIndex) {
                    self.intern_string("ToIndex out of range");
                }
                if matches!(operation, SpecOperationIr::CreateDataPropertyOrThrow) {
                    self.uses_heap = true;
                    self.intern_string("value");
                    self.intern_string("writable");
                    self.intern_string("enumerable");
                    self.intern_string("configurable");
                    self.intern_string(
                        "CreateDataPropertyOrThrow symbol property keys are not supported",
                    );
                    self.intern_string("CreateDataPropertyOrThrow target is not an object");
                    self.intern_string("Cannot redefine non-configurable property");
                    self.intern_string("Cannot define property on non-extensible object");
                }
                if matches!(operation, SpecOperationIr::CopyDataProperties) {
                    self.uses_heap = true;
                    self.intern_string("enumerable");
                }
                if matches!(operation, SpecOperationIr::Set) {
                    self.uses_heap = true;
                    self.intern_string("Set symbol property keys are not supported");
                    self.intern_string("Set target is not an object");
                }
                if matches!(operation, SpecOperationIr::HasOwnProperty) {
                    self.uses_heap = true;
                    self.intern_string("HasOwnProperty target is not an object");
                }
                if matches!(operation, SpecOperationIr::GetMethod) {
                    self.uses_heap = true;
                    self.intern_string("GetMethod target is not callable");
                }
                if matches!(operation, SpecOperationIr::Construct) {
                    self.uses_heap = true;
                    self.intern_string("target is not a constructor");
                    self.intern_string("Spread argument is not an array");
                }
                if matches!(operation, SpecOperationIr::DeletePropertyOrThrow) {
                    self.uses_heap = true;
                    self.intern_string(
                        "DeletePropertyOrThrow symbol property keys are not supported",
                    );
                    self.intern_string("DeletePropertyOrThrow target is not an object");
                    self.intern_string("Cannot delete property");
                }
                for operand in operands {
                    self.collect_expr(operand);
                }
            }
            ExprIr::DeleteIdentifier { name, .. } | ExprIr::DeleteGlobalProperty { name, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::DeleteProperty { target, key, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_property_key(key);
            }
            ExprIr::TypeOf { expr } => {
                self.intern_string("undefined");
                self.intern_string("object");
                self.intern_string("boolean");
                self.intern_string("number");
                self.intern_string("bigint");
                self.intern_string("symbol");
                self.intern_string("string");
                self.intern_string("function");
                self.collect_expr(expr);
            }
            ExprIr::TypeOfUnresolvedIdentifier { .. } => {
                self.intern_string("undefined");
            }
            ExprIr::StringFromCharCode { code } => {
                self.uses_heap = true;
                self.collect_expr(code);
            }
            ExprIr::StringCharCodeAt { target, index } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_expr(index);
            }
            ExprIr::NewTarget => {}
            ExprIr::UpdateIdentifier { name, .. } => self.intern_string(name),
            ExprIr::BinaryNumber { lhs, rhs, .. }
            | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
            | ExprIr::BitwiseNumeric { lhs, rhs, .. }
            | ExprIr::CompareNumber { lhs, rhs, .. }
            | ExprIr::CompareValue { lhs, rhs, .. }
            | ExprIr::StrictEquality { lhs, rhs, .. }
            | ExprIr::LooseEquality { lhs, rhs, .. }
            | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
            | ExprIr::Comma { lhs, rhs } => {
                for value in [
                    "",
                    ",",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::MaterializeBinding { name, value, body } => {
                self.intern_string(name);
                self.collect_expr(value);
                self.collect_expr(body);
            }
            ExprIr::ArrayDestructure { value, pattern, .. } => {
                self.uses_heap = true;
                // `enumerable` is needed by a nested object-rest target
                // (`[{ a, ...rest }] = …`) reached through this pattern.
                for key in [
                    "Symbol.iterator",
                    "next",
                    "done",
                    "value",
                    "return",
                    "enumerable",
                ] {
                    self.intern_string(key);
                }
                self.collect_expr(value);
                pattern.visit_expressions(&mut |expr| self.collect_expr(expr));
                self.collect_array_destructuring_pattern_strings(pattern);
            }
            ExprIr::ObjectDestructure { value, pattern } => {
                self.uses_heap = true;
                // The iterator-protocol keys are needed by a nested array target
                // (`{ a: [b] } = …`) reached through this pattern.
                for key in [
                    "enumerable",
                    "Symbol.iterator",
                    "next",
                    "done",
                    "value",
                    "return",
                ] {
                    self.intern_string(key);
                }
                self.collect_expr(value);
                pattern.visit_expressions(&mut |expr| self.collect_expr(expr));
                self.collect_object_destructuring_pattern_strings(pattern);
            }
            ExprIr::Conditional {
                condition,
                then_expr,
                else_expr,
            } => {
                self.collect_expr(condition);
                self.collect_expr(then_expr);
                self.collect_expr(else_expr);
            }
            ExprIr::StringConcat { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("undefined");
                self.intern_string("null");
                self.intern_string("true");
                self.intern_string("false");
                self.intern_string("NaN");
                self.intern_string("Infinity");
                self.intern_string("-Infinity");
                self.intern_string("[object Object]");
                self.intern_string("[object Arguments]");
                self.intern_string("");
                self.intern_string(",");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CoerciveAdd { lhs, rhs } => {
                self.uses_heap = true;
                for value in [
                    "",
                    ",",
                    "undefined",
                    "null",
                    "true",
                    "false",
                    "NaN",
                    "Infinity",
                    "-Infinity",
                    "[object Object]",
                    "[object Arguments]",
                    "valueOf",
                    "toString",
                ] {
                    self.intern_string(value);
                }
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::CallNamed { name, args } => {
                self.uses_heap = true;
                self.intern_string(name);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::SpreadArgument(spread) => {
                self.uses_heap = true;
                for message in [
                    "Spread argument is not iterable",
                    "Spread iterator method must return object",
                    "Spread iterator next must be callable",
                    "Spread iterator next result must be object",
                ] {
                    self.intern_string(message);
                }
                self.collect_expr(&spread.value);
            }
            ExprIr::AssertSameValue {
                actual,
                expected,
                message,
            } => {
                self.uses_heap = true;
                self.intern_string(ERROR_NAME);
                self.intern_string(message);
                self.collect_expr(actual);
                self.collect_expr(expected);
            }
            ExprIr::RuntimeThrow { name, message } => {
                self.uses_heap = true;
                self.intern_string(name.as_str());
                self.intern_string(message);
            }
            ExprIr::GlobalPropertyRead { name } | ExprIr::GlobalIdentifierRead { name } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyWrite { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::GlobalPropertyUpdate { name, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
            }
            ExprIr::GlobalPropertyCompoundAssign { name, value, .. } => {
                self.uses_heap = true;
                self.intern_string(name);
                self.collect_expr(value);
            }
            ExprIr::CallIndirect {
                callee,
                this_arg,
                args,
                static_regexp_compilation,
            } => {
                self.uses_heap = true;
                // Two independent recognisers, and they are deliberately not
                // the same test.
                //
                // `resolved_callee` depends on type inference having resolved
                // the callee to `RegExp` or `RegExp.prototype.compile`. That is
                // the only one allowed to force a table into existence, because
                // forcing one is expensive: with no string-valued declaration
                // initialiser anywhere, `queue_runtime_regexp_programs` falls
                // back to *every* script string literal.
                //
                // `compile_shaped_callee` is structural — the callee is a
                // property read whose key is literally `compile`. It cannot
                // narrow anything: collected literals are only ever *unioned*
                // into the candidate set (`queue_runtime_regexp_programs`), and
                // the set they are unioned into is not the one whose emptiness
                // picks the fallback. So widening collection is free, while
                // widening the flag is not — hence the split.
                //
                // The split matters because the measured gate case
                // (`annexB/built-ins/RegExp/prototype/compile/duplicate-named-capturing-groups-syntax.js`)
                // spells its illegal pattern as `() => r.compile("(?<x>a)(?<x>b)")`
                // over a `let r = /[ab]/`, which lowers to `CallIndirect` with a
                // `PropertyRead` callee, and whether `function_targets` resolves
                // through the arrow is exactly the thing this lane could not
                // measure. `lower_indirect_method_call` (`lila-ir`) keeps the
                // method name on the callee in *both* of its shapes, so the
                // structural test answers without needing inference at all.
                let resolved_regexp_callee = matches!(callee.expr, ExprIr::GlobalPropertyRead { ref name } if name == "RegExp")
                    || callee.function_targets.iter().any(|target| {
                        matches!(
                            target.as_str(),
                            BUILTIN_REGEXP_FUNCTION_ID
                                | BUILTIN_REGEXP_PROTOTYPE_COMPILE_FUNCTION_ID
                        )
                    });
                if static_regexp_compilation.is_none() && resolved_regexp_callee {
                    self.needs_runtime_regexp_programs = true;
                }
                if static_regexp_compilation.is_none()
                    && (resolved_regexp_callee || callee_names_regexp_compile(callee))
                {
                    // The pattern argument is the one string the script is
                    // demonstrably asking the RegExp compiler about. Offer it
                    // to the compile-time compiler even when it never appears
                    // as a declaration initialiser — otherwise an *illegal*
                    // pattern spelled inline is never compiled, never rejected,
                    // and therefore has no row to throw from.
                    if let Some(pattern) = args.first() {
                        collect_finite_string_choices(
                            pattern,
                            &mut self.runtime_regexp_argument_literals,
                        );
                    }
                }
                if let Some(compilation) = static_regexp_compilation {
                    match compilation {
                        StaticRegExpCompilation::Program(program) => {
                            self.queue_regexp_program(program)
                        }
                        StaticRegExpCompilation::InvalidSyntax { message } => {
                            self.intern_string(message);
                        }
                    }
                }
                self.collect_expr(callee);
                if let Some(this_arg) = this_arg {
                    self.collect_expr(this_arg);
                }
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::JsonParseStaticReviver { value, reviver } => {
                self.uses_heap = true;
                self.collect_json_static_value(value);
                self.collect_expr(reviver);
                self.intern_string("");
                self.intern_string("source");
            }
            ExprIr::Construct {
                callee,
                args,
                static_regexp_compilation,
            } => {
                self.uses_heap = true;
                if static_regexp_compilation.is_none()
                    && (matches!(callee.expr, ExprIr::GlobalPropertyRead { ref name } if name == "RegExp")
                        || callee.function_targets.contains(BUILTIN_REGEXP_FUNCTION_ID))
                {
                    self.needs_runtime_regexp_programs = true;
                    // Same reasoning as the `CallIndirect` arm above: `new
                    // RegExp("(?<x>a)(?<x>b)")` must reach the compile-time
                    // compiler so the rejection has somewhere to live.
                    if let Some(pattern) = args.first() {
                        collect_finite_string_choices(
                            pattern,
                            &mut self.runtime_regexp_argument_literals,
                        );
                    }
                }
                self.intern_string("prototype");
                if let Some(compilation) = static_regexp_compilation {
                    match compilation {
                        StaticRegExpCompilation::Program(program) => {
                            self.queue_regexp_program(program)
                        }
                        StaticRegExpCompilation::InvalidSyntax { message } => {
                            self.intern_string(message);
                        }
                    }
                }
                self.collect_expr(callee);
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::CallMethod {
                receiver,
                key,
                args,
            } => {
                self.uses_heap = true;
                self.collect_expr(receiver);
                self.collect_property_key(key);
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "join")
                    || matches!(key, PropertyKeyIr::StaticString(name) if name == "toString")
                        && receiver
                            .possible_kinds
                            .is_subset_of(KindSet::from_kind(ValueKind::Array))
                {
                    self.intern_string("");
                    self.intern_string(",");
                    self.intern_string("Array.prototype.join receiver is not array");
                }
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "reverse") {
                    self.intern_string("Array.prototype.reverse receiver is not array");
                }
                // A literal pattern argument to `.compile(…)`, collected for the
                // same reason as the `CallIndirect` / `Construct` arms: the
                // compile-time RegExp compiler's verdict on a pattern needs a
                // row in the runtime table, and a pattern that only ever appears
                // as a call argument was never offered to it.
                //
                // Both arms are needed because the *same source text* lowers
                // differently depending on where it sits. Measured with
                // `lila inspect`: `function go() { r.compile("xy"); }` over a
                // `var r = /[ab]/` reports `method_calls=1`, while the identical
                // call over a `let r = /[ab]/` reports `method_calls=0,
                // indirect_calls=5`. Collecting at one node only would close the
                // hole for one spelling of the same program.
                //
                // Deliberately does NOT set `needs_runtime_regexp_programs`: a
                // `.compile` call on some unrelated object would then force a
                // runtime table — and, for a script with no string-valued
                // declaration initialisers, one built from EVERY script string
                // literal — for nothing. Collecting costs nothing when no table
                // is built, because `queue_runtime_regexp_programs` is then
                // never called.
                //
                // What *does* set the flag for this shape is the
                // `RegExpPrototypeCompile` disjunct in `collect`'s initialiser.
                // Without it this arm was strictly inert for the very program
                // its comment above cites, because no other setter fires on a
                // `CallMethod` node: read the two together.
                if matches!(key, PropertyKeyIr::StaticString(name) if name == "compile") {
                    if let Some(pattern) = args.first() {
                        collect_finite_string_choices(
                            pattern,
                            &mut self.runtime_regexp_argument_literals,
                        );
                    }
                }
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::InstanceOf { lhs, rhs } => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::In { lhs, rhs } => {
                self.uses_heap = true;
                self.collect_expr(lhs);
                self.collect_expr(rhs);
            }
            ExprIr::SuperConstruct { args } => {
                self.uses_heap = true;
                for arg in args {
                    self.collect_expr(arg);
                }
            }
            ExprIr::SuperPropertyRead { key } => {
                self.uses_heap = true;
                self.collect_property_key(key);
            }
            ExprIr::SuperPropertyWrite { key, value, .. } => {
                self.uses_heap = true;
                self.collect_property_key(key);
                self.collect_expr(value);
            }
            ExprIr::ClassDefinition(class) => {
                self.uses_heap = true;
                self.intern_string("prototype");
                self.intern_string("constructor");
                self.intern_string("$IsHTMLDDA");
                self.intern_string("class extends value is not a constructor or null");
                for definition in &class.element_plan.definitions {
                    match definition {
                        ClassElementDefinitionIr::PublicMethod(method) => {
                            self.collect_property_key(&method.key);
                        }
                        ClassElementDefinitionIr::PrivateMethod(method) => {
                            self.intern_string(&private_data_key(method.private_name_id));
                            self.intern_string(&private_brand_key(method.private_name_id));
                        }
                        ClassElementDefinitionIr::ComputedFieldKey { key, .. } => {
                            self.collect_property_key(key);
                        }
                    }
                }
                for static_element in &class.element_plan.static_elements {
                    let ClassStaticElementIr::Field(field) = static_element else {
                        continue;
                    };
                    match &field.key {
                        ClassFieldKeyIr::Public(key) => self.intern_string(key),
                        ClassFieldKeyIr::ComputedPublic(_) => {}
                        ClassFieldKeyIr::Private(private_name_id) => {
                            self.intern_string(&private_data_key(*private_name_id));
                            self.intern_string(&private_brand_key(*private_name_id));
                        }
                    }
                }
                if let Some(heritage) = &class.heritage {
                    self.collect_expr(heritage);
                }
            }
            ExprIr::PrivateRead { target, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
            }
            ExprIr::PrivateWrite { target, value, .. } => {
                self.uses_heap = true;
                self.collect_expr(target);
                self.collect_expr(value);
            }
            ExprIr::PrivateIn { rhs, .. } => {
                self.uses_heap = true;
                self.collect_expr(rhs);
            }
            ExprIr::Undefined
            | ExprIr::ArrayHole
            | ExprIr::Null
            | ExprIr::Boolean(_)
            | ExprIr::Number(_)
            | ExprIr::FunctionValue(_)
            | ExprIr::This
            | ExprIr::Arguments
            | ExprIr::Identifier(_) => {}
        }
    }

    fn collect_property_key(&mut self, key: &PropertyKeyIr) {
        match key {
            PropertyKeyIr::StaticString(value) => self.intern_string(value),
            PropertyKeyIr::ArrayLength => {}
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                self.collect_expr(expr)
            }
        }
    }

    fn collect_json_static_value(&mut self, value: &JsonStaticValueIr) {
        match value {
            JsonStaticValueIr::Null { source }
            | JsonStaticValueIr::Boolean { source, .. }
            | JsonStaticValueIr::Number { source, .. } => {
                self.intern_string(source);
            }
            JsonStaticValueIr::String {
                value: string_value,
                source,
            } => {
                self.intern_string(string_value);
                self.intern_string(source);
            }
            JsonStaticValueIr::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    self.intern_string(&index.to_string());
                    self.collect_json_static_value(value);
                }
            }
            JsonStaticValueIr::Object(properties) => {
                for (key, value) in properties {
                    self.intern_string(key);
                    self.collect_json_static_value(value);
                }
            }
        }
    }

    /// Interns every string a destructuring target can turn into a runtime
    /// property key.
    ///
    /// Two of those strings are easy to forget because they are not written as
    /// keys in the source: a `Binding` name is mirrored onto the script global
    /// object by `mirror_binding_to_global_object`, and an
    /// `AssignmentIdentifier` name is written through the checked global
    /// Reference writer when the target resolves globally. Both call
    /// `StringPool::payload`, which panics when the name is not pooled. Abrupt
    /// identifier References additionally contribute their typed error
    /// message; the emitter no longer owns a parallel message literal.
    ///
    /// The match is deliberately exhaustive with no wildcard arm: a new
    /// `DestructuringTargetIr` variant must fail to compile here rather than
    /// reach codegen with an un-interned name.
    fn collect_destructuring_target_strings(&mut self, target: &DestructuringTargetIr) {
        match target {
            DestructuringTargetIr::Binding { mode: _, name } => {
                self.intern_string(name);
            }
            DestructuringTargetIr::AssignmentIdentifier(reference) => {
                self.intern_string(reference.name());
                if let IdentifierWriteDisposition::Throw { error } = reference.write_disposition() {
                    self.intern_string(error.message());
                }
            }
            DestructuringTargetIr::AssignmentProperty { key, .. } => {
                self.collect_destructuring_property_key_strings(key);
            }
            // Private elements are addressed by brand token, not by a pooled
            // string; the class definition interns their keys.
            DestructuringTargetIr::AssignmentPrivate { .. } => {}
            DestructuringTargetIr::NestedArray(pattern) => {
                self.collect_array_destructuring_pattern_strings(pattern);
            }
            DestructuringTargetIr::NestedObject(pattern) => {
                self.collect_object_destructuring_pattern_strings(pattern);
            }
        }
    }

    fn collect_destructuring_property_key_strings(&mut self, key: &DestructuringPropertyKeyIr) {
        match key {
            DestructuringPropertyKeyIr::Static(key) => self.intern_string(key),
            // Computed keys are stringified at runtime; `visit_expressions`
            // already walked the key expression.
            DestructuringPropertyKeyIr::Computed(_) => {}
        }
    }

    fn collect_array_destructuring_pattern_strings(
        &mut self,
        pattern: &ArrayDestructuringPatternIr,
    ) {
        for element in &pattern.elements {
            match element {
                ArrayDestructuringElementIr::Elision => {}
                ArrayDestructuringElementIr::Target { target, default: _ }
                | ArrayDestructuringElementIr::Rest { target } => {
                    self.collect_destructuring_target_strings(target);
                }
            }
        }
    }

    fn collect_object_destructuring_pattern_strings(
        &mut self,
        pattern: &ObjectDestructuringPatternIr,
    ) {
        for property in &pattern.properties {
            self.collect_destructuring_property_key_strings(&property.key);
            self.collect_destructuring_target_strings(&property.target);
        }
        if let Some(rest) = &pattern.rest {
            self.collect_destructuring_target_strings(rest);
        }
    }

    fn intern_string(&mut self, value: &str) {
        if self.refs.contains_key(value) {
            return;
        }
        let offset = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        let bytes = Self::runtime_bytes_for_string(value);
        self.bytes.extend_from_slice(&bytes);
        self.refs.insert(
            value.to_string(),
            StringRef {
                offset,
                len: bytes.len() as u32,
            },
        );
    }

    fn queue_regexp_program(&mut self, program: &RegExpProgram) {
        assert!(
            !has_non_consuming_cycle(program),
            "compiler-created RegExp program contains a non-consuming control-flow cycle"
        );
        // Names are part of the immutable metadata table and must be present
        // in the string pool before any program blobs are appended.
        for group in &program.named_groups {
            self.intern_string(&group.name);
        }
        let key = RegExpProgramStaticKey::from_program(program);
        if self.regexp_programs.contains_key(&key)
            || self
                .pending_regexp_programs
                .iter()
                .any(|(pending, _, _, _)| pending == &key)
        {
            return;
        }
        let split_count = program
            .instructions
            .iter()
            .filter(|instruction| instruction.opcode == REGEXP_OPCODE_SPLIT)
            .count() as u32;
        self.pending_regexp_programs.push((
            key,
            program.instructions.len() as u32,
            split_count,
            repeatable_split_count(program),
        ));
    }

    fn queue_runtime_regexp_programs(&mut self) {
        let candidate_literals = if self.runtime_regexp_candidate_literals.is_empty() {
            &self.script_string_literals
        } else {
            &self.runtime_regexp_candidate_literals
        };
        let mut literals = candidate_literals.iter().cloned().collect::<Vec<_>>();
        // Unioned in, never substituted for `candidate_literals`: the
        // empty/non-empty test above is the fallback switch, and this set must
        // not be able to flip it. See the field's doc comment.
        for literal in &self.runtime_regexp_argument_literals {
            if !literals.iter().any(|source| source == literal) {
                literals.push(literal.clone());
            }
        }
        if !literals.iter().any(|source| source == "(?:)") {
            literals.push("(?:)".to_string());
        }
        if !literals.iter().any(|source| source == "[object Object]") {
            literals.push("[object Object]".to_string());
        }
        // The table is `|literals| x |flags|` rows and every row is now written,
        // including the rejected and unsupported ones, so the flags axis is a
        // multiplier on static data size and on the *linear* scan the emitted
        // lookup does at every runtime construction site. Deduplicating it is
        // therefore not tidiness: the sticky expansion below used to be able to
        // produce `"iy"` twice for a script containing both `"i"` and `"iy"`,
        // which doubled a whole column of rows.
        let mut flags = self
            .script_string_literals
            .iter()
            .filter(|value| is_regexp_flags_literal(value))
            .cloned()
            .collect::<BTreeSet<_>>();
        flags.insert(String::new());
        let sticky_flags = flags
            .iter()
            .filter(|flags| !flags.contains('y'))
            .map(|flags| format!("{flags}y"))
            .collect::<Vec<_>>();
        flags.extend(sticky_flags);
        let flags = flags.into_iter().collect::<Vec<_>>();
        for literal in &literals {
            self.intern_string(literal);
        }
        for flags in &flags {
            self.intern_string(flags);
        }
        let mut candidates = Vec::new();

        for source in &literals {
            let normalized_source = if source.is_empty() { "(?:)" } else { source };
            let compilation_source = if normalized_source == "(?:)" {
                ""
            } else {
                normalized_source
            };
            for flags in &flags {
                // Every candidate pair gets a row, including the rejected ones.
                // The `else { continue }` that used to sit here is the whole
                // defect: it made "seen and illegal" indistinguishable from
                // "never seen", and the emitted lookup could then only fall out
                // of its loop leaving a null program behind.
                match RegExpProgram::compile(compilation_source, flags) {
                    Ok(program) => {
                        let key = RegExpProgramStaticKey::from_program(&program);
                        self.queue_regexp_program(&program);
                        candidates.push((
                            normalized_source.to_string(),
                            flags.clone(),
                            CandidateOutcome::Program(key),
                        ));
                    }
                    // Exhaustive on `RegExpCompileErrorKind`. "Illegal pattern"
                    // and "legal pattern Lila cannot compile" are different
                    // answers to the program, and only the first is a
                    // SyntaxError.
                    Err(error) => candidates.push((
                        normalized_source.to_string(),
                        flags.clone(),
                        match error.kind {
                            RegExpCompileErrorKind::InvalidSyntax => CandidateOutcome::Rejected,
                            RegExpCompileErrorKind::UnsupportedFeature => {
                                CandidateOutcome::Unsupported
                            }
                        },
                    )),
                }
            }
        }

        self.append_regexp_programs();
        self.runtime_regexp_programs = candidates
            .into_iter()
            .map(|(source, flags, outcome)| {
                let entry = match outcome {
                    CandidateOutcome::Program(key) => RuntimeRegExpEntry::Program(
                        *self
                            .regexp_programs
                            .get(&key)
                            .expect("queued runtime RegExp program must have static data"),
                    ),
                    CandidateOutcome::Rejected => RuntimeRegExpEntry::Rejected,
                    CandidateOutcome::Unsupported => RuntimeRegExpEntry::Unsupported,
                };
                (source, flags, entry)
            })
            .collect();
        self.append_runtime_regexp_program_table();
    }

    fn append_runtime_regexp_program_table(&mut self) {
        if self.runtime_regexp_programs.is_empty() {
            return;
        }
        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        self.runtime_regexp_program_table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        self.runtime_regexp_program_count = self.runtime_regexp_programs.len() as u32;
        for (source, flags, entry) in &self.runtime_regexp_programs {
            // Assigned through the shared word indices rather than as a
            // positional array literal, so the writer's layout and the
            // emitter's `i64.load` offsets are the same facts rather than two
            // agreeing transcriptions. A non-`Program` row leaves the six
            // program words at their zeroed initial value, which is exactly
            // what a total miss leaves in the object's slots.
            let mut record = [0u64; RUNTIME_REGEXP_RECORD_WORDS];
            record[RUNTIME_REGEXP_RECORD_SOURCE_WORD] = self.payload(source) as u64;
            record[RUNTIME_REGEXP_RECORD_FLAGS_WORD] = self.payload(flags) as u64;
            // Exhaustive on purpose. This is the site the `continue` used to
            // hide behind: a new entry kind must be given a record encoding
            // here, or this stops compiling.
            record[RUNTIME_REGEXP_RECORD_ENTRY_KIND_WORD] = match entry {
                RuntimeRegExpEntry::Program(program) => {
                    record[RUNTIME_REGEXP_RECORD_PROGRAM_PTR_WORD] = program.ptr as u64;
                    record[RUNTIME_REGEXP_RECORD_INSTRUCTION_COUNT_WORD] =
                        program.instruction_count as u64;
                    record[RUNTIME_REGEXP_RECORD_CAPTURE_COUNT_WORD] = program.capture_count as u64;
                    record[RUNTIME_REGEXP_RECORD_SPLIT_COUNT_WORD] = program.split_count as u64;
                    record[RUNTIME_REGEXP_RECORD_REPEATABLE_SPLIT_COUNT_WORD] =
                        program.repeatable_split_count as u64;
                    record[RUNTIME_REGEXP_RECORD_NAMED_GROUP_TABLE_PTR_WORD] =
                        program.named_group_table_ptr as u64;
                    RuntimeRegExpEntryKind::Program.word()
                }
                RuntimeRegExpEntry::Rejected => RuntimeRegExpEntryKind::Rejected.word(),
                RuntimeRegExpEntry::Unsupported => RuntimeRegExpEntryKind::Unsupported.word(),
            };
            for value in record {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }

    fn append_regexp_programs(&mut self) {
        if self.pending_regexp_programs.is_empty() {
            return;
        }
        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        let pending = std::mem::take(&mut self.pending_regexp_programs);
        for (key, instruction_count, split_count, repeatable_split_count) in pending {
            let ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
            let capture_count = key.capture_count;
            self.bytes.extend_from_slice(&key.encoded_instructions);
            let named_group_table_ptr = self.append_named_group_table(&key.named_groups);
            self.regexp_programs.insert(
                key,
                RegExpProgramRef {
                    ptr,
                    instruction_count,
                    capture_count,
                    split_count,
                    repeatable_split_count,
                    named_group_table_ptr,
                },
            );
        }
    }

    fn append_named_group_table(&mut self, named_groups: &[(String, Vec<u32>)]) -> u32 {
        if named_groups.is_empty() {
            return 0;
        }

        let padding = (8 - self.bytes.len() % 8) % 8;
        self.bytes.resize(self.bytes.len() + padding, 0);
        let table_ptr = STATIC_DATA_OFFSET + self.bytes.len() as u32;
        let records_ptr = table_ptr + 32;
        let total_candidate_count: usize = named_groups
            .iter()
            .map(|(_, candidates)| candidates.len())
            .sum();
        let candidates_base = records_ptr + (named_groups.len() as u32 * 24);

        for value in [
            REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION,
            named_groups.len() as u64,
            total_candidate_count as u64,
            records_ptr as u64,
        ] {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }

        let mut candidate_offset = 0_u32;
        for (name, candidates) in named_groups {
            let name_payload = self.payload(name) as u64;
            let candidates_ptr = candidates_base + candidate_offset;
            for value in [name_payload, candidates_ptr as u64, candidates.len() as u64] {
                self.bytes.extend_from_slice(&value.to_le_bytes());
            }
            candidate_offset += (candidates.len() * 8) as u32;
        }
        for (_, candidates) in named_groups {
            for &capture_id in candidates {
                self.bytes
                    .extend_from_slice(&(capture_id as u64).to_le_bytes());
            }
        }
        table_ptr
    }

    pub(crate) fn runtime_bytes_for_string(value: &str) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(value.len());
        let mut chars = value.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != JS_STRING_SURROGATE_SENTINEL {
                Self::push_utf8_char(&mut bytes, ch);
                continue;
            }

            if chars.peek().copied() == Some(JS_STRING_SURROGATE_SENTINEL) {
                chars.next();
                Self::push_utf8_char(&mut bytes, JS_STRING_SURROGATE_SENTINEL);
                continue;
            }

            let mut consumed = String::new();
            let mut code_unit = 0_u16;
            let mut is_marker = true;
            for _ in 0..4 {
                let Some(hex) = chars.next() else {
                    is_marker = false;
                    break;
                };
                consumed.push(hex);
                let Some(value) = hex.to_digit(16) else {
                    is_marker = false;
                    break;
                };
                code_unit = (code_unit << 4) | value as u16;
            }

            if is_marker && (0xD800..=0xDFFF).contains(&code_unit) {
                Self::push_wtf8_code_unit(&mut bytes, code_unit);
            } else {
                Self::push_utf8_char(&mut bytes, JS_STRING_SURROGATE_SENTINEL);
                bytes.extend_from_slice(consumed.as_bytes());
            }
        }
        bytes
    }

    fn push_utf8_char(bytes: &mut Vec<u8>, ch: char) {
        let mut buffer = [0; 4];
        bytes.extend_from_slice(ch.encode_utf8(&mut buffer).as_bytes());
    }

    fn push_wtf8_code_unit(bytes: &mut Vec<u8>, code_unit: u16) {
        let code = code_unit as u32;
        if code < 0x80 {
            bytes.push(code as u8);
        } else if code < 0x800 {
            bytes.push((0xC0 | (code >> 6)) as u8);
            bytes.push((0x80 | (code & 0x3F)) as u8);
        } else {
            bytes.push((0xE0 | (code >> 12)) as u8);
            bytes.push((0x80 | ((code >> 6) & 0x3F)) as u8);
            bytes.push((0x80 | (code & 0x3F)) as u8);
        }
    }

    pub(crate) fn payload(&self, value: &str) -> i64 {
        let string = self
            .refs
            .get(value)
            .unwrap_or_else(|| panic!("string `{value}` must exist in pool"));
        assert!(
            string.offset < (1 << 31),
            "string pool offset {} exceeds the PropertyKey marker boundary",
            string.offset
        );
        (((string.offset as u64) << 32) | string.len as u64) as i64
    }

    pub(crate) fn property_key_symbol_payload(&self, value: &str) -> i64 {
        self.payload(value) | PROPERTY_KEY_SYMBOL_MARKER as i64
    }

    pub(crate) fn static_builtin_property_key_payload(&self, value: &str) -> i64 {
        if value.starts_with("Symbol.") {
            return self.property_key_symbol_payload(value);
        }
        self.payload(value)
    }

    pub(crate) fn regexp_program(&self, program: &RegExpProgram) -> RegExpProgramRef {
        *self
            .regexp_programs
            .get(&RegExpProgramStaticKey::from_program(program))
            .expect("collected RegExp literal program must have static data")
    }

    #[cfg(test)]
    pub(crate) fn collect_regexp_program_for_test(
        &mut self,
        program: &RegExpProgram,
    ) -> RegExpProgramRef {
        self.queue_regexp_program(program);
        self.append_regexp_programs();
        self.regexp_program(program)
    }
}

/// Does this `CallIndirect` callee *structurally* name `RegExp.prototype.compile`?
///
/// Purely a shape test on the two callee spellings `lower_indirect_method_call`
/// can produce (`lila-ir/src/lowering.rs`): an `ExprIr::PropertyRead` with a
/// static key, and a `GetV` spec operation whose second operand is the key
/// string. No type inference is consulted, which is the point — the arm that
/// uses this needs an answer for a call the inference may not have resolved
/// through an enclosing arrow.
///
/// Only ever used to *widen* candidate collection, never to force a table into
/// existence. A `.compile` on some unrelated object therefore costs at most one
/// extra literal in a set that is unioned in, and costs nothing at all when no
/// runtime table is built.
fn callee_names_regexp_compile(callee: &TypedExpr) -> bool {
    match &callee.expr {
        ExprIr::PropertyRead { key, .. } => {
            matches!(key, PropertyKeyIr::StaticString(name) if name == "compile")
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => matches!(
            operands.get(1).map(|key| &key.expr),
            Some(ExprIr::String(name)) if name == "compile"
        ),
        _ => false,
    }
}

/// Every string this expression can *statically* evaluate to, when that set is
/// finite and readable off the IR shape alone.
///
/// # The catch-all is deliberate, and it is the safe direction
///
/// The trailing `_ => {}` is not an oversight and must not be replaced by an
/// exhaustive match over [`ExprIr`]. This is a heuristic over an open,
/// hundreds-of-variants expression domain, and the two directions are not
/// symmetric:
///
/// * a shape this function **does not** recognise contributes no candidate, the
///   `(source, flags)` pair gets no row, and the runtime lookup falls through to
///   exactly the behaviour it had before the table existed — the fallback
///   matcher, and `TypeError: RegExp.prototype.exec unsupported pattern` if it
///   declines. Missing a shape costs coverage, never correctness;
/// * a shape it **does** recognise reaches `RegExpProgram::compile`, and an
///   `InvalidSyntax` verdict there becomes a
///   [`RUNTIME_REGEXP_ENTRY_KIND_REJECTED`] row, which **throws SyntaxError** at
///   every one of the seven `emit_runtime_regexp_program_slots` call sites for
///   any runtime pattern whose bytes equal that literal.
///
/// So adding an arm here is not "collect a few more literals": it widens the set
/// of patterns this compiler will refuse at run time, keyed by string value
/// rather than by syntactic origin. Read
/// [`RUNTIME_REGEXP_ENTRY_KIND_REJECTED`]'s doc — which records that
/// `lila-ir/src/regexp.rs`'s ~20 `invalid_syntax(` sites have never been
/// audited against the grammar — before widening, and measure
/// `built-ins/RegExp/prototype` as a delta afterwards.
///
/// Concatenation is the arm most obviously "missing": `new RegExp(a + b)` where
/// both halves are literals is a known open case (RE-RT probe 6, no throw
/// today). It is left out on purpose rather than by oversight, because closing
/// it is the widening this doc is warning about and it needs its own measured
/// gate.
fn collect_finite_string_choices(expr: &TypedExpr, choices: &mut BTreeSet<String>) {
    match &expr.expr {
        ExprIr::String(value) => {
            choices.insert(value.clone());
        }
        ExprIr::ArrayLiteral(elements) => {
            for element in elements {
                collect_finite_string_choices(element, choices);
            }
        }
        ExprIr::ArrayAccumulation(accumulation) => {
            for element in accumulation.elements() {
                match element {
                    ArrayAccumulationElementIr::Elision => {}
                    ArrayAccumulationElementIr::Value(value) => {
                        collect_finite_string_choices(value, choices)
                    }
                    ArrayAccumulationElementIr::Spread(spread) => {
                        collect_finite_string_choices(&spread.value, choices)
                    }
                }
            }
        }
        ExprIr::ObjectLiteral(properties) => {
            for property in properties {
                match property {
                    ObjectPropertyIr::PrototypeSetter { value }
                    | ObjectPropertyIr::Spread { source: value }
                    | ObjectPropertyIr::Data { value, .. }
                    | ObjectPropertyIr::NonEnumerableData { value, .. }
                    | ObjectPropertyIr::ComputedData { value, .. } => {
                        collect_finite_string_choices(value, choices);
                    }
                    ObjectPropertyIr::ComputedMethod { .. }
                    | ObjectPropertyIr::ComputedGetter { .. }
                    | ObjectPropertyIr::ComputedSetter { .. }
                    | ObjectPropertyIr::Method { .. }
                    | ObjectPropertyIr::Getter { .. }
                    | ObjectPropertyIr::Setter { .. } => {}
                }
            }
        }
        ExprIr::Conditional {
            then_expr,
            else_expr,
            ..
        } => {
            collect_finite_string_choices(then_expr, choices);
            collect_finite_string_choices(else_expr, choices);
        }
        _ => {}
    }
}

fn is_regexp_flags_literal(value: &str) -> bool {
    let mut seen = BTreeSet::new();
    value.chars().all(|flag| {
        matches!(flag, 'd' | 'g' | 'i' | 'm' | 's' | 'u' | 'v' | 'y') && seen.insert(flag)
    }) && !(seen.contains(&'u') && seen.contains(&'v'))
}

/// Counts `Split`s that can execute again through a control-flow cycle.
///
/// RegExp programs are capped at 4096 instructions, so a small, precise DFS
/// per split is clearer than maintaining a separate SCC representation here.
fn repeatable_split_count(program: &RegExpProgram) -> u32 {
    let instructions = &program.instructions;
    let successors = |pc: usize| -> Vec<usize> {
        let Some(instruction) = instructions.get(pc) else {
            return Vec::new();
        };
        let valid = |target: u64| {
            usize::try_from(target)
                .ok()
                .filter(|target| *target < instructions.len())
        };
        match instruction.opcode {
            REGEXP_OPCODE_SPLIT => [valid(instruction.operand0), valid(instruction.operand1)]
                .into_iter()
                .flatten()
                .collect(),
            REGEXP_OPCODE_JUMP => valid(instruction.operand0).into_iter().collect(),
            REGEXP_OPCODE_ACCEPT => Vec::new(),
            _ if pc + 1 < instructions.len() => vec![pc + 1],
            _ => Vec::new(),
        }
    };

    instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| instruction.opcode == REGEXP_OPCODE_SPLIT)
        .filter(|(split_pc, _)| {
            let mut visited = vec![false; instructions.len()];
            let mut stack = successors(*split_pc);
            while let Some(pc) = stack.pop() {
                if pc == *split_pc {
                    return true;
                }
                if visited[pc] {
                    continue;
                }
                visited[pc] = true;
                stack.extend(successors(pc));
            }
            false
        })
        .count() as u32
}

fn has_non_consuming_cycle(program: &RegExpProgram) -> bool {
    fn is_consuming(instruction: &lila_ir::RegExpInstruction) -> bool {
        matches!(
            instruction.opcode,
            REGEXP_OPCODE_LITERAL_ASCII
                | REGEXP_OPCODE_LITERAL_CODE_POINT
                | REGEXP_OPCODE_NEGATIVE_ASCII_CLASS
                | REGEXP_OPCODE_NOT_WHITESPACE
                | REGEXP_OPCODE_POSITIVE_ASCII_CLASS
                | REGEXP_OPCODE_WHITESPACE
                | REGEXP_OPCODE_DOT
                | REGEXP_OPCODE_UNICODE_PROPERTY
        ) || (instruction.opcode == REGEXP_OPCODE_NUMBERED_BACKREFERENCE
            && instruction.operand1 != 0)
    }

    fn visit(pc: usize, instructions: &[lila_ir::RegExpInstruction], state: &mut [u8]) -> bool {
        if state[pc] == 1 {
            return true;
        }
        if state[pc] == 2 || is_consuming(&instructions[pc]) {
            return false;
        }
        state[pc] = 1;
        let instruction = instructions[pc];
        let valid_target = |target: u64| {
            usize::try_from(target)
                .ok()
                .filter(|target| *target < instructions.len())
        };
        let successors = match instruction.opcode {
            REGEXP_OPCODE_SPLIT => [
                valid_target(instruction.operand0),
                valid_target(instruction.operand1),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>(),
            REGEXP_OPCODE_JUMP => valid_target(instruction.operand0).into_iter().collect(),
            REGEXP_OPCODE_ACCEPT => Vec::new(),
            _ if pc + 1 < instructions.len() => vec![pc + 1],
            _ => Vec::new(),
        };
        if successors
            .into_iter()
            .any(|successor| visit(successor, instructions, state))
        {
            return true;
        }
        state[pc] = 2;
        false
    }

    let mut state = vec![0; program.instructions.len()];
    (0..program.instructions.len()).any(|pc| visit(pc, &program.instructions, &mut state))
}

#[cfg(test)]
mod runtime_error_message_pool_tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::lower;

    /// The pool has no silent miss, and this is what makes the standing
    /// instruction on `emit_runtime_error_object` enforceable rather than
    /// advisory: a message that was never interned must *panic by name*, not
    /// quietly resolve to something plausible.
    ///
    /// The old behaviour -- defining `message` from the error's `name` -- was
    /// exactly a silent fallback, and it survived several batches because
    /// nothing in the tree observed it. Softening `payload` into an
    /// `Option`-returning lookup with a name fallback would restore that,
    /// harder to find. So the panic is asserted.
    #[test]
    #[should_panic(expected = "must exist in pool")]
    fn payload_panics_for_a_message_that_was_never_interned() {
        let pool = StringPool::default();
        let _ = pool.payload("a message that is deliberately absent from the pool");
    }

    /// Build the pool the way emission builds it.
    ///
    /// The empty script is deliberate and is the strongest available form of
    /// the assertion: the interning loop in `collect` is documented as
    /// unconditional, so a pool built from a program that references nothing
    /// must still resolve every literal. Interning the table by hand and then
    /// reading it back -- which this test used to do -- proves only that the
    /// table is *internable*, and stays green with the production loop
    /// (`collect`'s `for value in RUNTIME_ERROR_MESSAGE_LITERALS`) deleted.
    /// Deleting it turns `null.x` into a `must exist in pool` panic on every
    /// program, so that one wire is the one this module most needs under test.
    fn production_pool_for_an_empty_script() -> StringPool {
        let parsed = parse(";", ParseOptions::script()).expect("empty script should parse");
        let script = lower(&parsed).script.expect("empty script should lower");
        StringPool::collect(&script, &BTreeMap::new(), &[])
    }

    #[test]
    fn every_runtime_error_message_literal_resolves_to_a_payload() {
        let pool = production_pool_for_an_empty_script();
        for value in RUNTIME_ERROR_MESSAGE_LITERALS {
            // `payload` panics on a miss, so reaching the assertion is the
            // check; the assertion pins the encoding as well.
            let payload = pool.payload(value);
            let len = (payload as u64 & 0xFFFF_FFFF) as usize;
            assert_eq!(
                len,
                StringPool::runtime_bytes_for_string(value).len(),
                "`{value}` interned with the wrong byte length"
            );
        }
        for failure in RegExpMatcherFailure::ALL {
            let message = failure.message();
            let payload = pool.payload(message);
            let len = (payload as u64 & 0xFFFF_FFFF) as usize;
            assert_eq!(
                len,
                StringPool::runtime_bytes_for_string(message).len(),
                "typed RegExp matcher failure message `{message}` interned with the wrong byte length"
            );
        }
    }

    /// Sorted and unique. Not cosmetic: this table is maintained by hand, it is
    /// appended to under time pressure exactly when a `must exist in pool`
    /// panic has just fired, and a duplicate or an out-of-order insert is how a
    /// hand-maintained list starts drifting from the audit that produced it.
    #[test]
    fn the_runtime_error_message_table_is_sorted_and_unique() {
        let mut previous: Option<&str> = None;
        for &value in RUNTIME_ERROR_MESSAGE_LITERALS {
            assert!(!value.is_empty(), "empty message literal in the table");
            if let Some(previous) = previous {
                assert!(
                    previous < value,
                    "`{previous}` and `{value}` are out of order or duplicated"
                );
            }
            previous = Some(value);
        }
        assert!(
            RUNTIME_ERROR_MESSAGE_LITERALS.len() >= 125,
            "the table shrank; a message removed from it is a `must exist in pool` panic waiting \
             for whichever program still throws it"
        );
    }

    /// A spot check against the reason the table exists: the message
    /// `null.x` throws must be present. It is the single most reachable
    /// runtime-thrown message in the language and it was one of the strings
    /// with zero occurrences in this file before this table landed.
    #[test]
    fn the_most_reachable_runtime_error_message_is_in_the_table() {
        for value in [
            "Cannot read properties of null or undefined",
            "unbound identifier",
            "lexical binding accessed before initialization",
        ] {
            assert!(
                RUNTIME_ERROR_MESSAGE_LITERALS.contains(&value),
                "`{value}` must be interned; it is thrown from the most ordinary programs there are"
            );
        }
    }
}

#[cfg(test)]
mod regexp_program_validation_tests {
    use super::*;
    use lila_ir::{RegExpFlags, RegExpInstruction};

    fn program(instructions: Vec<RegExpInstruction>) -> RegExpProgram {
        RegExpProgram {
            flags: RegExpFlags::default(),
            capture_count: 0,
            named_groups: Vec::new(),
            instructions,
            ranges: Vec::new(),
        }
    }

    #[test]
    fn static_program_named_group_table_is_serialized_and_no_names_use_zero() {
        let unnamed = RegExpProgram::compile("a", "").expect("program should compile");
        let named = RegExpProgram::compile("(?<x>a)(?<y>b)", "").expect("program should compile");
        let mut pool = StringPool::default();
        let unnamed_ref = pool.collect_regexp_program_for_test(&unnamed);
        let named_ref = pool.collect_regexp_program_for_test(&named);

        assert_eq!(unnamed_ref.named_group_table_ptr, 0);
        assert_ne!(named_ref.named_group_table_ptr, 0);
        let offset = (named_ref.named_group_table_ptr - STATIC_DATA_OFFSET) as usize;
        let read = |at: usize| u64::from_le_bytes(pool.bytes[at..at + 8].try_into().unwrap());
        assert_eq!(read(offset), REGEXP_NAMED_GROUP_TABLE_MAGIC_VERSION);
        assert_eq!(read(offset + 8), 2);
        assert_eq!(read(offset + 16), 2);
        assert_eq!(
            read(offset + 24),
            (named_ref.named_group_table_ptr + 32) as u64
        );
        let records = offset + 32;
        assert_eq!(read(records + 16), 1);
        assert_eq!(read(records + 40), 1);
        let candidates = (read(records + 8) - STATIC_DATA_OFFSET as u64) as usize;
        assert_eq!(read(candidates), 1);
        assert_eq!(read(candidates + 8), 2);
        assert_eq!(read(records), pool.payload("x") as u64);
        assert_eq!(read(records + 24), pool.payload("y") as u64);
    }

    #[test]
    fn static_program_dedup_key_includes_named_group_mappings() {
        let base = RegExpProgram::compile("a", "").expect("program should compile");
        let mut first = base.clone();
        first.named_groups.push(lila_ir::RegExpNamedGroup {
            name: "x".into(),
            capture_ids: vec![1],
        });
        let mut second = first.clone();
        second.named_groups[0].capture_ids = vec![2];
        let mut pool = StringPool::default();
        let first_ref = pool.collect_regexp_program_for_test(&first);
        let second_ref = pool.collect_regexp_program_for_test(&second);
        assert_ne!(first_ref.ptr, second_ref.ptr);
    }

    #[test]
    fn rejects_non_consuming_program_cycles_without_rejecting_valid_repetition() {
        let self_jump = program(vec![RegExpInstruction::jump(0)]);
        assert!(has_non_consuming_cycle(&self_jump));

        let failed_consuming_loop = program(vec![
            RegExpInstruction::split(1, 2),
            RegExpInstruction::literal_ascii(b'a'),
            RegExpInstruction::jump(0),
        ]);
        assert!(has_non_consuming_cycle(&failed_consuming_loop));

        let valid_star = RegExpProgram::compile("a*", "").expect("star should compile");
        assert!(!has_non_consuming_cycle(&valid_star));
        let valid_lazy_star = RegExpProgram::compile("a*?b", "").expect("lazy star should compile");
        assert!(!has_non_consuming_cycle(&valid_lazy_star));
    }

    #[test]
    fn counts_repeatable_splits_inside_lookbehind() {
        let program =
            RegExpProgram::compile(r"(?<=\w+)f", "").expect("lookbehind repetition should compile");
        assert_eq!(repeatable_split_count(&program), 1);
    }
}

pub(crate) fn align_heap_start(bytes: usize) -> u64 {
    ((STATIC_DATA_OFFSET as u64 + bytes as u64) + 7) & !7
}

pub(crate) fn initial_memory_pages(static_data_bytes: usize, uses_heap: bool) -> u64 {
    let required = if uses_heap {
        align_heap_start(static_data_bytes) + (WASM_PAGE_SIZE * 16)
    } else {
        STATIC_DATA_OFFSET as u64 + static_data_bytes as u64
    };
    required.div_ceil(WASM_PAGE_SIZE).max(1)
}
