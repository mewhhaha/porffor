var marker = {}, lengths = [Infinity, 1e300, 9007199254740992];
for (var i = 0; i < lengths.length; i++) {
  var source = new Proxy({ length: lengths[i] }, {
    get: function (object, key, receiver) { print('get:' + key); return Reflect.get(object, key, receiver); },
    has: function (object, key) { print('has:' + key); throw marker; }
  });
  try { Array.prototype.flat.call(source, 1e300); } catch (error) { print(error === marker); }
}
var nested = new Proxy([1], {
  get: function (object, key, receiver) {
    if (key === 'length') { print('nested-get'); return { [Symbol.toPrimitive]: function (hint) { print('nested-' + hint); return Infinity; } }; }
    return Reflect.get(object, key, receiver);
  },
  has: function (object, key) { print('nested-has:' + key); throw marker; }
});
try { [nested].flat(Infinity); } catch (error) { print(error === marker); }
var emptyLengths = [-Infinity, -3, NaN, -0, 0.5];
for (var j = 0; j < emptyLengths.length; j++) print(Array.prototype.flat.call({ length: emptyLengths[j] }).length);
