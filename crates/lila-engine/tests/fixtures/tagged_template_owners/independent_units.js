function tag(strings) { return strings; }
var entry = tag`shared`;
// These prepared bodies are distinct successful parses with equal template
// strings and equal parser-local site integers.
var firstBody = Function('tag', 'return tag`shared`;');
var secondBody = Function('tag', 'var distinctBody; return tag`shared`;');
var first = firstBody(tag);
var second = secondBody(tag);
var firstAgain = firstBody(tag);
var fromDirectEval = eval('tag`shared`;');
var fromIndirectEval = (0, eval)('var distinctEval; tag`shared`;');
var invalidCooked = Function('tag', 'return tag`\\xZ`;')(tag);
var rawDescriptor = Object.getOwnPropertyDescriptor(first, 'raw');
entry !== first && first !== second && first === firstAgain
  && fromDirectEval !== entry && fromDirectEval !== first
  && fromIndirectEval !== entry && fromIndirectEval !== fromDirectEval
  && entry[0] === 'shared' && first[0] === 'shared' && second[0] === 'shared'
  && fromDirectEval[0] === 'shared' && fromIndirectEval[0] === 'shared'
  && invalidCooked[0] === undefined && invalidCooked.raw[0] === '\\xZ'
  && Object.isFrozen(first) && Object.isFrozen(first.raw)
  && Object.getPrototypeOf(first) === Array.prototype
  && Object.getPrototypeOf(first.raw) === Array.prototype
  && rawDescriptor.value === first.raw && rawDescriptor.writable === false
  && rawDescriptor.enumerable === false && rawDescriptor.configurable === false;
