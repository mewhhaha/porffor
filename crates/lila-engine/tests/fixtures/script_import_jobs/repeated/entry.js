globalThis.targetCalls = 0;
const first = import('./target.js');
const second = import('./target.js');
if (first === second || targetCalls !== 0) throw 'fresh promises or eager evaluation';
Promise.all([first, second]).then(values => {
  if (values[0] !== values[1] || values[0].value !== 7 || targetCalls !== 1)
    throw 'namespace identity or repeated evaluation';
  return import('./target.js').then(again => {
    if (again !== values[0] || targetCalls !== 1) throw 'later cache identity';
    print('one module namespace');
  });
});
print('imports queued');
true;
