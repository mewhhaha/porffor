var parse = JSON.parse, receiverHits = 0, reviverHits = 0;
var receiver = {get parse() { print('get-parse'); return parse; }};
var input = {[Symbol.toPrimitive]:function (hint) { print('coerce:' + hint); return '{"value":1}'; }};
function acquireReceiver() { receiverHits++; print('receiver'); return receiver; }
function makeReviver() {
  print('reviver');
  var localCalls = 0;
  return function (key, value, context) {
    localCalls++; reviverHits++;
    print('call:' + (key === '' ? '<root>' : key) + ':' + localCalls + ':' + (context.source === undefined ? 'none' : context.source));
    return value;
  };
}
var result = acquireReceiver().parse((print('input'), input), makeReviver(), (print('ignored'), 0));
print(result.value + ':' + receiverHits + ':' + reviverHits);
var text = '[1]';
var first = (print('callee'), parse)(text, function (key, value) { text = '[2]'; return value; });
var second = parse(text, function (key, value) { return value; });
print(first[0] + ':' + second[0]);
