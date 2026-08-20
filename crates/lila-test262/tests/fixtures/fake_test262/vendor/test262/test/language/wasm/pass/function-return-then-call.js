/*---
flags: [raw]
---*/

function inc(x) {
  return x + 1;
}

function pick() {
  return inc;
}

let g = pick();
g(2);
