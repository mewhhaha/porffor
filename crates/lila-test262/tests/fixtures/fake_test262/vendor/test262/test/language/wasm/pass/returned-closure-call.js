/*---
flags: [raw]
---*/

function outer() {
  let x = 2;
  return function (y) {
    return x + y;
  };
}

let f = outer();
f(3);
