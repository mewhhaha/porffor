/*---
flags: [raw]
---*/

try {
  class C {
    #x = 1;
    read(obj) {
      return obj.#x;
    }
  }
  new C().read({});
} catch (e) {
  e.name;
}
