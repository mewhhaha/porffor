var marker = Symbol('marker');
var trace = '';
var source = { get p() { trace += 'get;'; return undefined; } };
function defaultValue() { trace += 'default;'; return 8; }
var scope = new Proxy({}, { has(object, key) {
  if (key === 'target') { trace += 'has;'; throw marker; }
  return false;
} });
var caught = false;
try {
  try { with (scope) { var { p: target = defaultValue() } = source; trace += 'body;'; } }
  finally { trace += 'finally;'; }
} catch (error) { caught = error === marker; }
caught && trace === 'has;finally;' && target === undefined;
