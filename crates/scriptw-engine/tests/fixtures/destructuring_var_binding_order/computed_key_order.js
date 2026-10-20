var trace = '';
var sourceKey = { toString() { trace += 'key;'; return 'p'; } };
var source = { get p() { trace += 'get;'; return undefined; } };
var scope = new Proxy({}, { has(target, key) { trace += 'has:' + key + ';'; return false; } });
var fallback = 7;
with (scope) { var { [sourceKey]: target = fallback } = source; }
trace === 'has:source;has:sourceKey;key;has:target;get;has:fallback;' && target === 7;
