class C extends null {
  constructor() {
    return Object.create(new.target.prototype);
  }
}

let x = new C();
Object.getPrototypeOf(x) === C.prototype && x instanceof C;
