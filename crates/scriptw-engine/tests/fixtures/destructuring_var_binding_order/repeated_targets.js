var trace = '';
var scope = new Proxy({}, { has(object, key) { if (key === 'target') trace += 'h'; return false; } });
var source = { get a() { trace += 'a'; return 1; }, get b() { trace += 'b'; return 2; } };
with (scope) { var { a: target, b: target } = source; }
trace === 'hahb' && target === 2;
