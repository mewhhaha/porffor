/*---
flags: [raw]
---*/

var ok;
try {
  class C extends null {}
  new C();
} catch (e) {
  ok = e instanceof TypeError;
}
ok;
