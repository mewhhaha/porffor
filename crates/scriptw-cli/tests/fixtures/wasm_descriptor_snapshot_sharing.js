(function () {
  function check(actual, expected, label) {
    if (actual !== expected) throw new Error(label);
  }
  const object = {a: 1, b: 2};
  const original = Object.getOwnPropertyDescriptor(object, 'a');
  check(original.get, undefined, 'data getter is absent');
  check(original.set, undefined, 'data setter is absent');
  Object.defineProperty(object, 'a', {get: function () { return 7; }});
  check(object.a, 7, 'data converts to accessor');
  check(object.b, 2, 'other data descriptor remains independent');
  check(original.value, 1, 'public snapshot stays unchanged');
  let stored = 0;
  function both(value) {
    if (arguments.length) stored = value;
    return stored;
  }
  Object.defineProperty(object, 'a', {get: both, set: both});
  object.a = 11;
  check(object.a, 11, 'same function in separate accessor fields');
  Object.defineProperty(object, 'a', {get: function () { return 19; }});
  object.a = 23;
  check(object.a, 19, 'getter replacement');
  check(stored, 23, 'setter survives getter replacement');
  Object.defineProperty(object, 'a', {value: 29, writable: true});
  check(object.a, 29, 'accessor converts back to data');
  check(Object.getOwnPropertyDescriptor(object, 'a').get, undefined, 'getter retired');
  Object.freeze(object);
  check(object.a, 29, 'freeze preserves data');
  check(Object.getOwnPropertyDescriptor(object, 'b').writable, false, 'freeze independent field');
  const values = [];
  for (let i = 0; i < 512; i++) values.push({value: i, other: -i});
  if (typeof gc === 'function') gc();
  for (let i = 0; i < values.length; i++) {
    Object.defineProperty(values[i], 'value', {get: function () { return 31; }});
    check(values[i].value, 31, 'fresh getter after movement');
    check(values[i].other, -i, 'unrelated value after movement');
    delete values[i].value;
    values[i].value = i + 1;
    check(values[i].value, i + 1, 'delete and recreate data');
  }
  check(Object.keys(values[0]).join(','), 'other,value', 'insertion order');
  print('descriptor-snapshot-controls:ok');
})();
