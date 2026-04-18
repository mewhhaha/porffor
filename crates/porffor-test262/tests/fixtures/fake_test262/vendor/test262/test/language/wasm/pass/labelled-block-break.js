/*---
flags: [raw]
---*/

let x = 0;
outer: {
  x = 1;
  break outer;
  x = 2;
}
x;
