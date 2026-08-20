/*---
flags: [raw]
---*/

var ok;
try {
  class C extends null {
    constructor() {
      return Object.create(new.target.prototype);
    }
    m() {
      return super.x;
    }
  }
  new C().m();
} catch (e) {
  ok = e instanceof TypeError;
}
ok;
