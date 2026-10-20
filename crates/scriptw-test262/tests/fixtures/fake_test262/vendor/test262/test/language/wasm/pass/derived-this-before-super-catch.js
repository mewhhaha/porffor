/*---
flags: [raw]
---*/

try {
  class A {}
  class B extends A {
    constructor() {
      this.x = 1;
      super();
    }
  }
  new B();
} catch (e) {
  e.name;
}
