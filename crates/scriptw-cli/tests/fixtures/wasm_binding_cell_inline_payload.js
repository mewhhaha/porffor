(function () {
  function check(actual, expected, label) { if (actual !== expected) throw new Error(label); }
  function cells() {
    let value = { n: 1 };
    const first = value;
    function read() { return value; }
    function write(next) { value = next; }
    const snapshot = [value];
    write('text'); check(read(), 'text', 'string write');
    write(37); check(read(), 37, 'scalar write');
    write(undefined); check(read(), undefined, 'reference cleared');
    check(first.n, 1, 'retained old object'); check(snapshot[0], first, 'immutable snapshot');
    const second = { n: 2 }; write(second); check(read(), second, 'reference restored');
    return [read, write, first, second];
  }
  const escaped = cells(); escaped[1](false); check(escaped[0](), false, 'escaped alias');
  function mapped(value) {
    const args = arguments;
    function read() { return value; }
    args[0] = 'mapped'; check(read(), 'mapped', 'argument writes captured cell');
    value = { n: 3 }; check(args[0], value, 'cell writes mapped argument');
    Object.defineProperty(args, '0', { writable: false });
    const frozen = args[0]; value = 71; check(args[0], frozen, 'frozen mapping retired');
    delete args[0]; value = 73; check(args[0], undefined, 'deleted mapping retired');
    return read;
  }
  check(mapped(1)(), 73, 'captured parameter survives');
  function strict(value) { 'use strict'; arguments[0] = 9; return function () { return value; }; }
  check(strict(5)(), 5, 'strict unmapped');
  let readers = []; for (let i = 0; i < 4; i++) readers.push(() => i);
  for (let i = 0; i < readers.length; i++) check(readers[i](), i, 'iteration cells');
  function* sequence(value) { const get = () => value; yield get; value = 'resumed'; yield get; value = 83; return get; }
  const generator = sequence({ n: 4 }); const get = generator.next().value;
  check(get().n, 4, 'suspended reference'); check(generator.next().value, get, 'same reader');
  check(get(), 'resumed', 'resumed string'); check(generator.next().value, get, 'completion reader');
  check(get(), 83, 'completed scalar');
  let threw = false; try { const fixed = 1; (() => { fixed = 2; })(); } catch (error) { threw = error instanceof TypeError; }
  check(threw, true, 'const captured binding');
  threw = false; try { (() => hidden)(); let hidden = 1; } catch (error) { threw = error instanceof ReferenceError; }
  check(threw, true, 'TDZ captured binding');
  class Base { constructor(value) { this.value = value; } }
  class Derived extends Base { constructor(value) { const get = () => this; super(value); check(get(), this, 'derived this'); check(new.target, Derived, 'new target'); } }
  check(new Derived(97).value, 97, 'derived result');
  print('binding-cell-controls:ok');
})();
