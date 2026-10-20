/*---
flags: [raw]
---*/

function outer() {
  let x = 1;

  function inc() {
    x = x + 1;
    return x;
  }

  inc();
  return inc();
}

outer();
