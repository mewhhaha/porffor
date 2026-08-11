// `class S extends Intl.DateTimeFormat {}` — a subclass with NO explicit
// constructor. That is the shape whose static instance type is taken from the
// heritage builtin's `constructor_instance`, the fourth member of the tuple in
// `standard_builtin_signature` (`porffor-ir/src/lowering.rs`), consumed by the
// single `inherited_instance` binding in `lower_class`.
//
// # The defect this fixture exists for, MEASURED before it was written
//
// `./target/debug/porf inspect`, script result kind:
//
//   class D extends Intl.DateTimeFormat {} new D();     ->  result=undefined
//   class D extends Intl.Locale {}         new D("en"); ->  result=object
//   class D extends Iterator {}            new D();     ->  result=object
//
// `Iterator` reads `object` because batch 6 fixed exactly this arm for
// `IteratorConstructor`. `Intl.DateTimeFormat` is the one remaining reachable
// arm that still carries `ValueInfo::undefined()` — `Proxy`, `Symbol` and
// `BigInt` are constructable but cannot reach the consumer.
//
// The `undefined` is not a fact about the program: `new S()` really does
// produce an ordinary object at run time. It is a static *mistyping*, and
// `porf run --execution-backend wasm` on the three-class probe below answered:
//
//   dtf.reduceRight=undefined  dtf.take=11
//   loc.reduceRight=2          loc.take=11
//   plain.reduceRight=2        plain.take=11
//
// i.e. a plain user method named `reduceRight`, defined on the subclass itself,
// was NOT CALLED AT ALL on the `Intl.DateTimeFormat` subclass, and the call
// site read `undefined`. The identical class over `Intl.Locale` and over no
// heritage at all answered `2`.
//
// # Why `reduceRight` and not `format`
//
// This is the load-bearing detail, and getting it wrong produces a fixture that
// is green on both sides. `lowering.rs` only builds an `ExprIr::CallMethod` for
// a statically-nullish receiver on ten names — `every`, `some`, `find`,
// `reduce`, `reduceRight`, `map`, `filter`, `flatMap`, `take`, `drop` — because
// their guard is `!receiver_is_array` rather than `possible_kinds.contains
// (Object)`. Every other method name, `format` and `resolvedOptions` included,
// takes the ordinary property-read-then-indirect-call path, which never meets
// the hole. A first version of this fixture asserted only on `format`,
// `resolvedOptions` and `instanceof`, and it PASSED unpatched.
//
// Of those ten, nine are `IteratorHelper` variants and are re-routed by
// `receiver_needs_dynamic_helper_dispatch` (`porffor-aot-wasm/src/functions.rs`),
// which batch 6 widened to cover nullish receivers. `reduceRight` is the tenth
// and has no `IteratorHelper` variant, so it is the one name that still reaches
// `emit_method_call`'s generic tail and its statically-nullish shortcut — the
// arm that writes `undefined` into both destination locals and returns without
// emitting a call. `take` is kept beside it as the control that shows the two
// routes are genuinely different: `take` was already correct.
//
// # The controls, and what each one would catch
//
// * `class L extends Intl.Locale` — the sibling Intl builtin that already types
//   correctly, because its signature arm carries a real instance shape. Must be
//   green on BOTH sides. A change that "fixes" DTF by breaking method dispatch
//   generally takes this half down too.
// * `class P {}` — no heritage at all, so `inherited_instance` is not consulted.
//   Must be green on both sides.
// * `format` / `resolvedOptions` / `instanceof` — green before the patch, and
//   they are the REGRESSION guard for it: the patch gives the instance a real
//   (empty-object) heap shape where it previously had none, so a static
//   property resolution that goes wrong on the new shape shows up here.

// ------------------------------------------------------------- the classes

class S extends Intl.DateTimeFormat {
  reduceRight(a) {
    return a + 1;
  }
  take(a) {
    return a + 10;
  }
  describe(a) {
    return a + 100;
  }
}

class L extends Intl.Locale {
  reduceRight(a) {
    return a + 1;
  }
  take(a) {
    return a + 10;
  }
  describe(a) {
    return a + 100;
  }
}

class P {
  reduceRight(a) {
    return a + 1;
  }
  take(a) {
    return a + 10;
  }
  describe(a) {
    return a + 100;
  }
}

const s = new S();
const l = new L("en");
const p = new P();

// CONTROLS FIRST, DISCRIMINATOR LAST. This ordering is the whole reason the
// controls are worth anything. A JavaScript fixture aborts at its first failing
// assertion, so with `s.reduceRight` on top the pre-patch run died on line one
// and every row below it — the controls included — was never executed. The
// header calls these "green on BOTH sides"; only this order demonstrates it.
// The pre-patch run must now reach the very last check and fail exactly there.

// The two rows that were already green, so a fixture that goes green because
// dispatch changed wholesale is still visible.
if (l.reduceRight(1) !== 2) throw "locale subclass reduceRight was not called";
if (p.reduceRight(1) !== 2) throw "plain subclass reduceRight was not called";

// `take` is one of the nine names batch 6 re-routed, and `describe` is a name
// outside the ten entirely. Both were already correct on all three receivers;
// they pin the boundary of the defect rather than the defect.
if (s.take(1) !== 11) throw "dtf subclass take was not called";
if (l.take(1) !== 11) throw "locale subclass take was not called";
if (p.take(1) !== 11) throw "plain subclass take was not called";
if (s.describe(1) !== 101) throw "dtf subclass describe was not called";
if (l.describe(1) !== 101) throw "locale subclass describe was not called";
if (p.describe(1) !== 101) throw "plain subclass describe was not called";

// ------------------------------------------- DTF instance identity and methods

if (typeof s !== "object") throw "dtf instance is not an object";
if (s === null) throw "dtf instance is null";
if (!(s instanceof S)) throw "dtf instance is not an instance of S";
if (!(s instanceof Intl.DateTimeFormat)) throw "dtf instance is not an Intl.DateTimeFormat";
if (Object.getPrototypeOf(s) !== S.prototype) throw "dtf instance prototype is not S.prototype";

if (typeof s.format !== "function") throw "dtf format is not a function";
if (typeof s.format(0) !== "string") throw "dtf format(0) is not a string";
if (s.format(0).length === 0) throw "dtf format(0) is empty";

const dtfOptions = s.resolvedOptions();
if (typeof dtfOptions !== "object") throw "dtf resolvedOptions is not an object";
if (dtfOptions === null) throw "dtf resolvedOptions is null";
if (typeof dtfOptions.timeZone !== "string") throw "dtf resolvedOptions timeZone is not a string";
if (typeof dtfOptions.locale !== "string") throw "dtf resolvedOptions locale is not a string";

// ---------------------------------------- Intl.Locale instance negative control

if (typeof l !== "object") throw "locale instance is not an object";
if (l === null) throw "locale instance is null";
if (!(l instanceof L)) throw "locale instance is not an instance of L";
if (!(l instanceof Intl.Locale)) throw "locale instance is not an Intl.Locale";
if (Object.getPrototypeOf(l) !== L.prototype) throw "locale instance prototype is not L.prototype";

if (typeof l.toString !== "function") throw "locale toString is not a function";
if (typeof l.toString() !== "string") throw "locale toString() is not a string";
if (l.toString().length === 0) throw "locale toString() is empty";
if (typeof l.baseName !== "string") throw "locale baseName is not a string";
if (l.baseName !== "en") throw "locale baseName is not en";

// ------------------------------------------------------- the discriminator
//
// LAST, deliberately. Every row above was green before the lowering patch, and
// a JavaScript fixture aborts at its first failing assertion — so with this
// check at the top (where it used to be) the pre-patch run died on its first
// line and *none* of the controls, boundary rows or regression guards above
// ever executed. The header calls them "green on BOTH sides"; only this order
// demonstrates it. Pre-patch, this file must reach exactly here and fail; post
// patch it must reach `262`.
if (s.reduceRight(1) !== 2) throw "dtf subclass reduceRight was not called";

262;
