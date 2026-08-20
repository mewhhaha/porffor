/*---
flags: [raw]
---*/

try {
  class C {}
  C();
} catch (e) {
  e.name;
}
