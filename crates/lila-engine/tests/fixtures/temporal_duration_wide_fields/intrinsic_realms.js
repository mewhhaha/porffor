const foreign = __lilaCreateRealm().global;
const D = foreign.Temporal.Duration;
const prototype = D.prototype;
const from = D.from;
const negate = prototype.negated;
const absolute = prototype.abs;
foreign.Temporal.Duration = function() { throw 'public binding'; };
const original = Temporal.Duration.from({nanoseconds:17280000000000000000000});
for (const result of [from(original),negate.call(original),absolute.call(original)]) {
  if (Object.getPrototypeOf(result) !== prototype) throw 'intrinsic allocation Realm';
  if (Math.abs(result.nanoseconds) !== original.nanoseconds) throw 'Realm field width';
}
const NewTarget = foreign.Function.bind(null);
const constructed = Reflect.construct(Temporal.Duration,[0,0,0,0,0,0,0,0,0,17280000000000000000000],NewTarget);
if (Object.getPrototypeOf(constructed) !== prototype || constructed.nanoseconds !== original.nanoseconds) throw 'NewTarget width and Realm';
print('ok');
