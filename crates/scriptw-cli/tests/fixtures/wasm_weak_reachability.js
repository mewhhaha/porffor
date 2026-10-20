const foreign = __scriptwCreateRealm().global;
const Constructor = foreign.WeakMap;
foreign.WeakMap = null;
const map = Reflect.construct(Constructor, []);
const key = {};
map.set(key, 42);
if (map.get(key) !== 42 || !map.has(key) || !map.delete(key) || map.has(key)) throw new Error('created Realm weak storage');
print('weak storage:ok');
true;
