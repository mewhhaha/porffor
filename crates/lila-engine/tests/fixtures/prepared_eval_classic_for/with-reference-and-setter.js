var count = 91;
var scope = {count: 37};
var observed;
function initialize() { delete scope.count; return 7; }
with (scope) {
  eval("for (var count = initialize();;) { observed = count; break; }");
}
if (count !== 91 || scope.count !== 7 || observed !== 7)
  throw new Error('initializer changed the resolved Reference');
var trace = [];
var writes = {get count() { throw new Error('unexpected prior read'); }, set count(value) { trace.push('set:' + value); }};
function value() { trace.push('initialize'); return 11; }
with (writes) {
  eval("for (var count = value();;) { break; }");
}
if (trace.join(',') !== 'initialize,set:11' || count !== 91)
  throw new Error('declaration must only write the selected object environment');
print('ok');
true;
