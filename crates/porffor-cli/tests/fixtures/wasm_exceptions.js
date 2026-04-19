var second;

try {
  throw "x";
} catch (e) {
  e;
}

try {
  class A {}
  class B extends A {
    constructor() {}
  }
  new B();
} catch (e) {
  second = e.name;
}

second;
