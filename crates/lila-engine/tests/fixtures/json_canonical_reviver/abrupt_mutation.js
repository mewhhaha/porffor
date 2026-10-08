var marker = {}, visits = 0;
try {
  JSON.parse('{"first":0,"second":[1],"later":2}', function (key, value) {
    print('visit:' + key);
    if (key === 'first') this.second = new Proxy([7], {get:function (object, name, receiver) {
      if (name === 'length') { print('length'); throw marker; }
      return Reflect.get(object, name, receiver);
    }});
    return value;
  });
} catch (error) { print(error === marker); }
try {
  JSON.parse('{"first":0,"second":{},"later":2}', function (key, value) {
    print('own-visit:' + key);
    if (key === 'first') this.second = new Proxy({deep:{value:7}}, {ownKeys:function () { print('ownKeys'); throw undefined; }});
    return value;
  });
} catch (error) { print(error === undefined); }
try {
  JSON.parse('{"a":{"b":1},"later":2}', function (key, value) { visits++; print('callback:' + key); throw marker; });
} catch (error) { print((error === marker) + ':' + visits); }
try {
  JSON.parse({toString:function () { print('input-throw'); throw marker; }}, function () { print('unexpected-reviver'); });
} catch (error) { print(error === marker); }
print(JSON.parse('{"value":7}', 42).value);
