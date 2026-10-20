/*---
flags: [raw]
---*/

function f(x, y = x + 1) {
  return y;
}

f(2);
