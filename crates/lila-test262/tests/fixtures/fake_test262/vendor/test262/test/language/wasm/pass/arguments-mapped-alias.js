/*---
flags: [raw]
---*/

function f(x) {
  arguments[0] = 3;
  return x;
}

f(1);
