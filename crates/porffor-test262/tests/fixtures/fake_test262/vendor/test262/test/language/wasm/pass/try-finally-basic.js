/*---
flags: [raw]
---*/

var x = 0;
try {
  x = 1;
} finally {
  x = x + 1;
}
x;
