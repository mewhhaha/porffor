/*---
flags: [raw]
---*/

var ok;
try {
  class C extends null {
    constructor() {
      super();
    }
  }
  new C();
} catch (e) {
  ok = e instanceof TypeError;
}
ok;
