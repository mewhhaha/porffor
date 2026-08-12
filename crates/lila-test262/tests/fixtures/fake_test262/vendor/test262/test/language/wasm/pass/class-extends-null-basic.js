/*---
flags: [raw]
---*/

class C extends null {
  constructor() {
    return Object.create(new.target.prototype);
  }
}

new C().constructor === C;
