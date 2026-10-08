var object = { get tag() { print('tag-get'); return function (strings, value) {
  'use strict'; print('tag:' + (this === object) + ':' + Object.isFrozen(strings) + ':' + Object.isFrozen(strings.raw) + ':' + strings[0] + ':' + strings.raw[1] + ':' + value);
}; } };
function operand() { print('operand'); return Promise.resolve(7).then(function (value) {
  print('replace'); Object.defineProperty(object, 'tag', { value: function () { print('wrong-tag'); } }); return value;
}); }
async function run() { object.tag`head${await operand()}tail`; print('done'); }
run().catch(error => print('error:' + error)); print('called');
