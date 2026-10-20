function exercise(methodKey, readKey, writeKey) {
  class C {
    [methodKey]() { return this.marker + 6; }
    get [readKey]() { return this.marker + 1; }
    set [writeKey](next) { this.marker = next; }
    static [methodKey + "Static"]() { return 21; }
  }

  let target = new C();
  target.marker = 5;

  let read = Object.getOwnPropertyDescriptor(C.prototype, "read").get;
  let write = Object.getOwnPropertyDescriptor(C.prototype, "write").set;
  write.call(target, 9);

  return target.dyn() === 15 && read.call(target) === 10 && C.dynStatic() === 21;
}

exercise("dyn", "read", "write");
