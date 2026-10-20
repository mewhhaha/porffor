var hooks = 0, result;
try {
  result = JSON.parse('{"first":0,"object":{"keep":1}}', function (key, value, context) {
    print((key === '' ? '<root>' : key) + ':' + (Object.prototype.hasOwnProperty.call(context, 'source') ? context.source : 'none'));
    if (key === 'first') {
      Object.defineProperty(this.object, 'z', {get:function () { print('current-get:z'); return 10; }, enumerable:true, configurable:true});
      Object.defineProperty(Object.prototype, 'z', {get:function () { hooks++; print('unexpected-metadata-get'); throw {}; }, configurable:true});
    }
    return value;
  });
} finally { delete Object.prototype.z; }
print(result.object.z + ':' + Object.prototype.hasOwnProperty.call(result.object, 'z') + ':' + hooks);
