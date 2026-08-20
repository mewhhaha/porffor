/*---
flags: [raw]
---*/

let f = function fact(n) {
  if (n === 0) {
    return 1;
  }

  return n * fact(n - 1);
};

f(4);
