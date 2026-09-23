const foreign = __lilaCreateRealm().global;
const create = new foreign.Function('return class { static error = new TypeError(); static { this.make = () => new TypeError(); } field = new TypeError(); };');
const ForeignClass = create();
const first = new ForeignClass();
const second = Reflect.construct(ForeignClass, []);
for (const error of [ForeignClass.error, ForeignClass.make(), first.field, second.field]) {
  if (Object.getPrototypeOf(error) !== foreign.TypeError.prototype) throw 'element Realm';
}
if (first.field === second.field) throw 'fresh invocation';
print('ok');
