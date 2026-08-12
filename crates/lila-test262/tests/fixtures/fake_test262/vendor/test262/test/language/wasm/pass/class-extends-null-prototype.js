/*---
flags: [raw]
---*/

class C extends null {
  constructor() {
    return Object.create(new.target.prototype);
  }
}

Object.getPrototypeOf(C.prototype) === null;
