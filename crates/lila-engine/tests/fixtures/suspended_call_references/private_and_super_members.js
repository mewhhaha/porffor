function operand() { print('operand'); return Promise.resolve(7); }
class Parent { get method() { print('super-get'); return function (value) { print('super:' + (this === child) + ':' + value); }; } }
class Child extends Parent {
  get #method() { print('private-get'); return function (value) { print('private:' + (this === child) + ':' + value); }; }
  async run() { this.#method(await operand()); super.method(await operand()); print('done'); }
}
var child = new Child(); child.run().catch(error => print('error:' + error)); print('called');
