//! One fixture inventory shared by artifact validation and real CLI execution.

pub struct ProductProgram {
    pub name: &'static str,
    pub source: &'static str,
    pub expected_stdout: &'static str,
}

pub const CASES: &[ProductProgram] = &[
    ProductProgram {
        name: "loop-branches",
        source: "let total = 0; for (let i = 0; i < 10; i++) { if (i === 3) continue; if (i === 8) break; total += i; } print(total);",
        expected_stdout: "25\n",
    },
    ProductProgram {
        name: "closure-capture",
        source: "function makeAdder(x) { return function(y) { return x + y; }; } const add = makeAdder(4); print(add(5));",
        expected_stdout: "9\n",
    },
    ProductProgram {
        name: "abrupt-completion",
        source: "function checked() { try { throw 7; } catch (value) { return value + 1; } finally { print('done'); } } print(checked());",
        expected_stdout: "done\n8\n",
    },
    ProductProgram {
        name: "heap-aggregates",
        source: "const values = [1, 2, 3]; const record = { items: values }; print(record.items[1]);",
        expected_stdout: "2\n",
    },
    ProductProgram {
        name: "bigint-and-strings",
        source: "const value = 12345678901234567890n + 1n; print(String(value)); print('x😀'.length);",
        expected_stdout: "12345678901234567891\n3\n",
    },
];
