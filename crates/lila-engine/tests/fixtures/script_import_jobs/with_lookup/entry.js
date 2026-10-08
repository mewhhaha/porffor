let pending;
let specifierLookups = 0;
const replacement = function () { throw 'with object replaced private dispatcher'; };
const scope = new Proxy({
  specifier: './value.js',
  '$lila$module$import$0': replacement
}, {
  has(target, key) {
    if (typeof key === 'string' && key.startsWith('$lila$module$'))
      throw 'with object observed private module owner';
    if (key === 'specifier') specifierLookups++;
    return Reflect.has(target, key);
  }
});
if (false) import('./value.js');
with (scope) {
  pending = import(specifier);
}
if (specifierLookups === 0) throw 'specifier skipped its with environment';
pending.then(ns => {
  if (ns.value !== 42 || specifierLookups === 0) throw 'wrong with import result';
  print('with lookup preserves private dispatcher');
});
true;
