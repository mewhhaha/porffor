/*---
flags: [raw]
---*/

function outer() {
  let x = 1;

  function inner() {
    return x + 1;
  }

  return inner();
}

outer();
