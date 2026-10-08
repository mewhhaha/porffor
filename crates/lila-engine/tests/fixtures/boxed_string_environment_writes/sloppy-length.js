var box = new String('globglob'), rhsCalls = 0, trace = '';
var length = 40, assigned, updated, compounded;
function rhs() { rhsCalls++; trace += 'rhs;'; return -1; }
with (box) {
  assigned = length = rhs();
  updated = length++;
  compounded = length += 1;
}
if (box.length !== 8 || length !== 40 || assigned !== -1 ||
    updated !== 8 || compounded !== 9 || rhsCalls !== 1 || trace !== 'rhs;') {
  throw 'sloppy Object Environment write';
}
var descriptor = Object.getOwnPropertyDescriptor(box, 'length');
if (descriptor.value !== 8 || descriptor.writable || descriptor.enumerable ||
    descriptor.configurable) throw 'length descriptor changed';
var empty = new String('');
with (empty) { length = 7; }
if (empty.length !== 0 || length !== 40) throw 'empty String length write';
print('ok');
