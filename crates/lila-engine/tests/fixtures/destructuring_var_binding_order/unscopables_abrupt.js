var marker = Symbol('marker');
var reads = 0;
var caught = 0;
var source = { get p() { reads++; return 8; } };
function check(scope) {
  try { with (scope) { var { p: target } = source; } }
  catch (error) { if (error === marker) caught++; }
}
check({ target: 1, get [Symbol.unscopables]() { throw marker; } });
check({ target: 1, [Symbol.unscopables]: { get target() { throw marker; } } });
caught === 2 && reads === 0;
