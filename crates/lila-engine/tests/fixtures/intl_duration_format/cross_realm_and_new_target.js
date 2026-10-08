function check(value, message) { if (!value) throw new Error(message); }
function throws(kind, fn, message) { var caught; try { fn(); } catch (error) { caught = error; } check(caught instanceof kind, message); }
var foreign = __lilaCreateRealm().global, Constructor = foreign.Intl.DurationFormat;
var formatter = new Constructor('en');
check(Object.getPrototypeOf(formatter) === Constructor.prototype, 'foreign formatter prototype');
check(Object.getPrototypeOf(formatter.resolvedOptions()) === foreign.Object.prototype, 'foreign resolved method realm');
var localParts = Intl.DurationFormat.prototype.formatToParts.call(formatter,{hours:1});
check(Object.getPrototypeOf(localParts) === Array.prototype && Object.getPrototypeOf(localParts[0]) === Object.prototype, 'local called method owns results');
var foreignParts = Constructor.prototype.formatToParts.call(new Intl.DurationFormat('en'),{hours:1});
check(Object.getPrototypeOf(foreignParts) === foreign.Array.prototype && Object.getPrototypeOf(foreignParts[0]) === foreign.Object.prototype, 'foreign called method owns results');
var target = foreign.Function.bind(null); Object.defineProperty(target,'prototype',{value:1,configurable:true});
var fallback = Reflect.construct(Intl.DurationFormat,['sr'],target);
check(Object.getPrototypeOf(fallback) === Constructor.prototype, 'GetFunctionRealm primitive prototype fallback');
function Sub() {} var prototype = Object.create(null); Sub.prototype = prototype;
check(Object.getPrototypeOf(Reflect.construct(Intl.DurationFormat,['en'],Sub)) === prototype, 'custom NewTarget prototype');
print('ok cross_realm_and_new_target'); 262;
