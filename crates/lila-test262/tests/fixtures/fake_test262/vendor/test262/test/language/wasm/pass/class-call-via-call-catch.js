/*---
flags: [raw]
---*/

class C {}
try {
  C.call({});
} catch (e) {
  e instanceof TypeError;
}
