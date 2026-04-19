/*---
flags: [raw]
---*/

try {
  class A {}
  class B extends A {
    constructor() {}
  }
  new B();
} catch (e) {
  e.name;
}
