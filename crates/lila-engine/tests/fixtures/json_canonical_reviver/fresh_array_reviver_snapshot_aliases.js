var result = JSON.parse('[1e+0,2e0,-0,[4e0],{"dup":5,"dup":6e0}]', function (key, value, context) {
  if (Array.isArray(this) && this.length === 5 && key === '0') {
    this[1] = 20;
    Object.defineProperty(this, '2', {value: -0, writable: true, enumerable: true, configurable: true});
  }
  print((key === '' ? '<root>' : key) + ':' + (context.source === undefined ? 'none' : context.source));
  return value;
});
print('values:' + result[0] + ':' + result[1] + ':' + Object.is(result[2], -0) + ':' + result[3][0] + ':' + result[4].dup);
