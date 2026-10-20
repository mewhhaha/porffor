/*---
flags: [raw]
---*/

try {
  "x" in 1;
} catch (e) {
  e instanceof TypeError && TypeError instanceof Function;
}
