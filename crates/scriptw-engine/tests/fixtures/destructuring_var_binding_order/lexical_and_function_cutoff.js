var lookups = 0;
var scope = new Proxy({ target: 40 }, { has(object, key) { if (key === 'target') lookups++; return Reflect.has(object, key); } });
var lexical = false;
with (scope) { let { p: target } = { p: 6 }; lexical = target === 6; }
var lexicalOrder = lexical && lookups === 0;
var local;
with (scope) { local = function() { var { p: target } = { p: 7 }; return target; }; }
var localOrder = local() === 7 && scope.target === 40 && lookups === 0;
var tdz = false;
try { with (scope) { let { p: target = target } = {}; } }
catch (error) { tdz = error instanceof ReferenceError; }
lexicalOrder && localOrder && tdz && lookups === 0;
