/*---
flags: [raw]
---*/

let x = 0;
outer: while (x < 3) {
  x += 1;
  continue outer;
  x = 9;
}
x;
