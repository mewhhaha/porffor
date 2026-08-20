/*---
flags: [raw]
---*/

function outer(x) {
  return y => x + y;
}

let f = outer(2);

f(3);
