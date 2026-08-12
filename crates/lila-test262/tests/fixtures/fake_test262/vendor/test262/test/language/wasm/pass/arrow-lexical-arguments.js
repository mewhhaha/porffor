/*---
flags: [raw]
---*/

function outer() {
  return (() => arguments[0])();
}

outer(3);
