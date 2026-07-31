#!/usr/bin/env python3
"""Probe which Test262 feature tags the Wasm-AOT path can execute at all.

Test262 labels every case with `features: [...]`. This runs one minimal snippet
per feature through the real compiler and records whether it executes, which
turns "how many features are left" into a measurement instead of an estimate.

A probe proves presence, not conformance: passing means the feature is not
categorically missing, and says nothing about edge cases. A failing probe is the
stronger signal - it means every Test262 case carrying that tag is out of reach.

usage: python3 scripts/feature-probe.py [--jobs N]
"""
import argparse
import concurrent.futures
import pathlib
import subprocess
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
PORF = REPO / "target/release/porf"
OUT = REPO / "target/probe"

# (feature tag, test262 case count, snippet, module?, expected first line of output)
PROBES = [
        ("Temporal", 6661, 'print(typeof Temporal);', False, 'object'),
        # Simple binding patterns work; only nested ones are refused, so these
        # are probed separately rather than as one pass/fail for 6,637 cases.
        ("destructuring-binding", 6637, 'const [a, b] = [1, 2]; const {c} = {c: 3}; print(a + b + c);', False, '6'),
        ("destructuring-binding/nested", 6637, 'const {a, b: [c]} = {a: 1, b: [2]}; print(a + c);', False, '3'),
        ("async-iteration", 4968, 'async function* g(){ yield 1; }\n(async () => { for await (const v of g()) print(v); })();', False, '1'),
        ("class", 4765, 'class A { m(){ return 1; } }\nprint(new A().m());', False, '1'),
        ("generators", 4085, 'function* g(){ yield 1; }\nprint(g().next().value);', False, '1'),
        ("TypedArray", 2513, 'const a = new Uint8Array(2); a[0] = 7; print(a[0]);', False, '7'),
        ("default-parameters", 2269, 'function f(a = 5){ return a; }\nprint(f());', False, '5'),
        ("class-fields-public", 2055, 'class A { x = 3; }\nprint(new A().x);', False, '3'),
        ("Symbol.iterator", 1846, 'const o = { *[Symbol.iterator](){ yield 25; } };\nprint([...o][0]);', False, '25'),
        ("class-methods-private", 1709, 'class A { #m(){ return 4; } run(){ return this.#m(); } }\nprint(new A().run());', False, '4'),
        ("Intl.Era-monthcode", 1543, 'print(typeof Intl);', False, 'object'),
        ("class-static-methods-private", 1513, 'class A { static #m(){ return 5; } static run(){ return A.#m(); } }\nprint(A.run());', False, '5'),
        ("BigInt", 1499, 'print((2n ** 64n).toString());', False, '18446744073709551616'),
        ("Symbol", 1478, 'print(typeof Symbol("x"));', False, 'symbol'),
        ("class-fields-private", 1131, 'class A { #x = 6; get(){ return this.#x; } }\nprint(new A().get());', False, '6'),
        ("arrow-function", 949, 'const f = (a) => a * 2; print(f(4));', False, '8'),
        ("dynamic-import", 946, 'import("./dep.mjs").then(m => print(m.v));', True, '10'),
        ("async-functions", 705, 'async function f(){ return 8; }\nf().then(print);', False, '8'),
        ("Reflect.construct", 696, 'class A { constructor(){ this.x = 9; } }\nprint(Reflect.construct(A, []).x);', False, '9'),
        ("regexp-unicode-property-escapes", 681, 'print(/\\p{L}/u.test("a"));', False, 'true'),
        ("Symbol.asyncIterator", 538, 'print(typeof Symbol.asyncIterator);', False, 'symbol'),
        ("computed-property-names", 478, 'const k = "a"; print({ [k]: 11 }.a);', False, '11'),
        ("explicit-resource-management", 477, '{ using r = { [Symbol.dispose](){ print("disposed"); } }; }', False, 'disposed'),
        ("Reflect", 468, 'print(Reflect.has({a:1}, "a"));', False, 'true'),
        ("Proxy", 468, 'print(new Proxy({}, { get(){ return 12; } }).anything);', False, '12'),
        ("SharedArrayBuffer", 463, 'print(new SharedArrayBuffer(8).byteLength);', False, '8'),
        ("resizable-arraybuffer", 463, 'const b = new ArrayBuffer(2, { maxByteLength: 8 }); b.resize(4); print(b.byteLength);', False, '4'),
        ("iterator-helpers", 393, 'print([1,2,3].values().map(x => x * 2).toArray().length);', False, '3'),
        ("Atomics", 376, 'const a = new Int32Array(new SharedArrayBuffer(8)); Atomics.store(a, 0, 13); print(Atomics.load(a, 0));', False, '13'),
        ("object-rest", 355, 'const { a, ...rest } = { a: 1, b: 2, c: 3 }; print(Object.keys(rest).length);', False, '2'),
        ("class-static-fields-private", 345, 'class A { static #x = 14; static get(){ return A.#x; } }\nprint(A.get());', False, '14'),
        ("Symbol.species", 276, 'print(typeof Symbol.species);', False, 'symbol'),
        ("top-level-await", 271, 'const v = await Promise.resolve(15); print(v);', True, '15'),
        ("ArrayBuffer", 268, 'print(new ArrayBuffer(16).byteLength);', False, '16'),
        ("Symbol.toPrimitive", 233, 'print(+{ [Symbol.toPrimitive](){ return 17; } });', False, '17'),
        ("import-defer", 229, 'import defer * as ns from "./dep.mjs"; print(ns.v);', True, '10'),
        ("source-phase-imports", 228, 'import source src from "./dep.mjs"; print(typeof src);', True, 'object'),
        ("regexp-modifiers", 224, 'print(/(?i:a)b/.test("Ab"));', False, 'true'),
        ("class-static-fields-public", 213, 'class A { static x = 18; }\nprint(A.x);', False, '18'),
        ("cross-realm", 201, 'print(typeof $262.createRealm);', False, 'function'),
        ("set-methods", 192, 'print(new Set([1,2]).union(new Set([3])).size);', False, '3'),
        ("DataView", 190, 'const d = new DataView(new ArrayBuffer(8)); d.setInt32(0, 19); print(d.getInt32(0));', False, '19'),
        ("regexp-v-flag", 187, 'print(/[\\p{L}]/v.test("a"));', False, 'true'),
        ("numeric-separator-literal", 159, 'print(1_000_000);', False, '1000000'),
        ("Intl.Locale", 156, 'print(new Intl.Locale("en").language);', False, 'en'),
        # The object exists - `print(globalThis)` gives [object Object] - but
        # `typeof` on it answers undefined. A defect, not a missing feature.
        ("globalThis", 148, 'print(typeof globalThis);', False, 'object'),
        ("destructuring-assignment", 141, 'let a, b; [a, b] = [20, 21]; print(a + b);', False, '41'),
        ("object-spread", 135, 'print(Object.keys({ ...{a:1, b:2} }).length);', False, '2'),
        ("change-array-by-copy", 132, 'print([3,1,2].toSorted()[0]);', False, '1'),
        ("Symbol.toStringTag", 130, 'print(Object.prototype.toString.call({ [Symbol.toStringTag]: "X" }));', False, '[object X]'),
        ("Array.fromAsync", 95, 'Array.fromAsync([1,2]).then(a => print(a.length));', False, '2'),
        ("Promise.any", 92, 'Promise.any([Promise.resolve(22)]).then(print);', False, '22'),
        ("rest-parameters", 96, 'function f(...a){ return a.length; }\nprint(f(1,2,3));', False, '3'),
        ("logical-assignment-operators", 108, 'let a = null; a ??= 23; print(a);', False, '23'),
        ("optional-chaining", 0, 'print(({a:{b:24}})?.a?.b);', False, '24'),
]


def run(probe):
    tag, count, snippet, is_module, want = probe
    safe = tag.replace(".", "_").replace("-", "_").replace("/", "__")
    ext = "mjs" if is_module else "js"
    path = OUT / f"{safe}.{ext}"
    path.write_text(snippet + "\n")
    try:
        result = subprocess.run(
            [str(PORF), "run", "--execution-backend", "wasm", str(path)],
            capture_output=True, text=True, timeout=120,
        )
        combined = result.stdout + result.stderr
        first = next((l for l in combined.splitlines() if l and not l.startswith("run outcome")), "")
        # A probe must produce the RIGHT answer. Checking only for the absence
        # of an error marks `typeof Intl -> undefined` as working.
        ok = result.returncode == 0 and first.strip() == want
        return tag, count, ok, (first[:70] if first.strip() != want else "")
    except subprocess.TimeoutExpired:
        return tag, count, False, "TIMEOUT"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--jobs", type=int, default=4)
    args = parser.parse_args()

    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "dep.mjs").write_text("export const v = 10;\n")
    if not PORF.exists():
        print(f"missing {PORF}; run: cargo build --release -p porffor-cli", file=sys.stderr)
        return 2

    with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as pool:
        results = list(pool.map(run, PROBES))

    results.sort(key=lambda r: -r[1])
    works = [r for r in results if r[2]]
    broken = [r for r in results if not r[2]]

    print(f"{'':2} {'feature':38} {'cases':>6}  detail")
    for tag, count, ok, detail in results:
        print(f"{'ok' if ok else 'XX'} {tag:38} {count:6}  {detail}")

    print()
    print(f"probed:      {len(results)} features")
    print(f"executes:    {len(works)} ({sum(r[1] for r in works)} tagged cases)")
    print(f"unsupported: {len(broken)} ({sum(r[1] for r in broken)} tagged cases)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
