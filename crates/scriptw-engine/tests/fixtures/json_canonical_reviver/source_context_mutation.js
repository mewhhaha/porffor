var root;
var result = JSON.parse('{"change":0,"zero":-0,"same":1e+0,"different":2,"nest":{"n":3},"dup":4,"dup":5,"__proto__":{"own":6},"":7}', function (key, value, context) {
  var name = key === '' ? (this === root ? '<empty>' : '<root>') : key;
  print(name + ':' + (Object.prototype.hasOwnProperty.call(context, 'source') ? context.source : 'none'));
  if (key === 'change') { root = this; this.zero = +0; this.same = 1; this.different = 9; this.nest = {n:3}; }
  return key === '' && this === root ? undefined : value;
});
print(Object.is(result.zero, +0) + ':' + Object.is(result.zero, -0) + ':' + result.different + ':' + Object.prototype.hasOwnProperty.call(result, '') + ':' + Object.prototype.hasOwnProperty.call(result, '__proto__') + ':' + (Object.getPrototypeOf(result) === Object.prototype));
