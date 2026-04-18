/*---
flags: [raw]
---*/

function inc(x) {
  return x + 1;
}

let a = [inc];
a[0](2);
