var box = new String('ab'), strictAssign, strictUpdate, trace = '', caught = 0;
with (box) {
  strictAssign = function () {
    'use strict';
    return length = (trace += 'rhs;', 7);
  };
  strictUpdate = function () {
    'use strict';
    return length++;
  };
}
try { strictAssign(); throw 'strict write returned'; }
catch (error) {
  if (!(error instanceof TypeError)) throw error;
  caught++;
}
try { strictUpdate(); throw 'strict update returned'; }
catch (error) {
  if (!(error instanceof TypeError)) throw error;
  caught++;
}
if (caught !== 2 || trace !== 'rhs;' || box.length !== 2) {
  throw 'strict Object Environment write order';
}
var descriptor = Object.getOwnPropertyDescriptor(box, 'length');
if (descriptor.value !== 2 || descriptor.writable || descriptor.configurable) {
  throw 'strict write changed length descriptor';
}
print('ok');
