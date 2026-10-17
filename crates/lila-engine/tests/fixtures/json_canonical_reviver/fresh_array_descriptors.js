function check(condition, label) { if (!condition) throw new Error(label); }
var setterHits = 0;
for (var key of ['0', '3', '9']) {
  Object.defineProperty(Array.prototype, key, {
    get: function () { throw new Error('inherited getter'); },
    set: function () { setterHits++; }, configurable: true
  });
}
var result = JSON.parse('[null,true,false,0,-0,1e+0,"text","\\uD800",[],{"same":0,"same":1}]');
for (var key of ['0', '3', '9']) delete Array.prototype[key];
check(setterHits === 0, 'prototype hooks');
check(result.length === 10 && result[0] === null && result[1] === true && result[2] === false, 'primitive tags');
check(Object.is(result[4], -0) && result[5] === 1 && result[6] === 'text', 'primitive payloads');
check(result[7].length === 1 && result[7].charCodeAt(0) === 0xd800, 'UTF16 value');
check(Object.getPrototypeOf(result) === Array.prototype && Array.isArray(result[8]) && result[8].length === 0, 'array prototypes');
check(result[9].same === 1 && Object.getPrototypeOf(result[9]) === Object.prototype, 'nested object duplicate');
for (var index = 0; index < result.length; index++) {
  var descriptor = Object.getOwnPropertyDescriptor(result, String(index));
  check(descriptor.writable && descriptor.enumerable && descriptor.configurable, 'data flags ' + index);
  check(!('get' in descriptor) && !('set' in descriptor), 'absent accessors ' + index);
}
var length = Object.getOwnPropertyDescriptor(result, 'length');
check(length.value === 10 && length.writable && !length.enumerable && !length.configurable, 'length descriptor');
Object.defineProperty(result, '0', {value: 'changed', writable: false, configurable: false});
Object.defineProperty(result, '1', {get: function () { return 22; }});
check(result[0] === 'changed' && result[1] === 22 && result[2] === false, 'descriptor independence');
check(Object.getOwnPropertyDescriptor(result, '2').writable, 'unmodified sibling descriptor');
print('fresh-array-descriptors:ok');
