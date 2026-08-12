/*---
flags: [raw]
---*/

function inc(x) {
  return x + 1;
}

let g = inc;
g(2);
