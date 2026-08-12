/*---
flags: [raw]
---*/

function make(x) {
  return { f() { return x; } };
}

make(2).f();
