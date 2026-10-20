const events = [];
const outer = 11;
class Parent {
  static label() { return this.name; }
  value() { return outer; }
}
class Child extends Parent {
  #private = outer + 1;
  [(events.push('key'), 'field')] = super.value() + this.#private;
  static #staticPrivate = outer;
  static value = super.label() + this.#staticPrivate;
  static {
    events.push('block');
    this.read = () => this.#staticPrivate;
  }
  read() { return this.#private; }
}
const first = new Child();
const second = new Child();
if (first.field !== 23 || second.field !== 23 || first.read() !== 12) throw 'instance context';
if (Child.value !== 'Child11' || Child.read() !== 11) throw 'static context';
if (events.join(',') !== 'key,block') throw events.join(',');
print('ok');
