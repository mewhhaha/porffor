// The same content participates in independent property and collection tables.
// Include equal strings built separately, exact UTF-16 and collision chains.
(function () {
  function check(value, message) { if (!value) throw new Error(message); }
  const keys = ['', 'prop-98', 'prop-105', 'prop-120', 'prop-142',
                '\ud800', '\ufffd', '\ud83d\ude00\udfff'];
  for (let i = 0; i < 257; ++i) keys.push('x'.repeat(257) + ':' + i);
  for (const propertyFirst of [false, true]) {
    const object = Object.create(null), map = new Map(), set = new Set();
    for (let i = 0; i < keys.length; ++i) {
      const key = ('!' + keys[i]).slice(1);
      if (propertyFirst) object[key] = i;
      map.set(key, i); set.add(key);
      if (!propertyFirst) object[key] = i;
    }
    if (typeof gc === 'function') gc();
    for (let i = 0; i < keys.length; ++i) {
      // Creation routes and property/collection lookup order vary independently.
      const key = keys[i].split('').join('');
      check(object[key] === i && map.get(key) === i && set.has(key), 'cross-owner lookup');
      check(object[key] === i && map.get(key) === i && set.has(key), 'repeated lookup');
      const extended = key + '!';
      check(!Object.hasOwn(object, extended) && !map.has(extended) && !set.has(extended),
            'new string content has its own hash');
      check(delete object[key] && map.delete(key) && set.delete(key), 'delete shared key');
      check(!Object.hasOwn(object, key) && !map.has(key) && !set.has(key), 'deleted key stays absent');
      map.set(key, i + 1); set.add(key);
      Object.defineProperty(object, key, {value: i + 1, configurable: true, enumerable: true});
    }
    if (typeof gc === 'function') gc();
    check(map.size === keys.length && set.size === keys.length, 'reinsertion size');
    for (let i = 0; i < keys.length; ++i) {
      check(object[keys[i]] === i + 1 && map.get(keys[i]) === i + 1 && set.has(keys[i]),
            'original strings after reinsertion and GC');
    }
    check(Object.keys(object).every((key, i) => key === keys[i]), 'property order');
    check([...map.keys()].every((key, i) => key === keys[i]), 'map order');
    check([...set].every((key, i) => key === keys[i]), 'set order');
  }
})();
