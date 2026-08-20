/*---
flags: [raw]
---*/

var x = 0;
try {
  throw 1;
} catch (e) {
  x = 1;
} finally {
  x = x + 1;
}
x;
