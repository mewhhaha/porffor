function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label); }
var foreign = __lilaCreateRealm().global;
var Other = foreign.Intl.DisplayNames;
function Target() {}
function functionProto() {}
var arrayProto = ['inherited-index'];
var proxyReads = [];
var proxyTarget = {marker: 'proxy-marker'};
var proxyProto = new Proxy(proxyTarget, {get(t, k, r) { proxyReads.push(String(k)); return Reflect.get(t, k, r); }});
for (var proto of [functionProto, arrayProto, proxyProto]) {
  var receiver;
  Object.defineProperty(proto, 'inherited', {configurable: true, get() { receiver = this; return 'inherited-value'; }});
  Target.prototype = proto;
  for (var C of [Intl.DisplayNames, Other]) {
    var lf = Reflect.construct(C, ['en-US',{type:'region'}], Target);
    same(Object.getPrototypeOf(lf), proto, 'exact tagged NewTarget prototype');
    same(lf.inherited, 'inherited-value', 'inherited accessor'); same(receiver, lf, 'accessor receiver');
    if (proto === arrayProto) same(lf[0], 'inherited-index', 'Array tagged prototype indexed access');
    if (proto === proxyProto) same(lf.marker, 'proxy-marker', 'Proxy prototype path');
    same(C.prototype.of.call(lf, 'US'), 'United States', 'private brand with exotic prototype');
    same(C.prototype.resolvedOptions.call(lf).locale, 'en-US', 'private record retained');
  }
}
check(proxyReads.indexOf('marker') !== -1, 'Proxy inherited Get actually called');
var bound = foreign.Array.bind(null);
for (var primitive of [undefined, null, false, 0, 'prototype', Symbol('prototype')]) {
  bound.prototype = primitive;
  var lf = Reflect.construct(Intl.DisplayNames, ['en-US',{type:'region'}], new Proxy(new Proxy(bound, {}), {}));
  same(Object.getPrototypeOf(lf), Other.prototype, 'primitive fallback uses actual NewTarget Realm');
  same(Other.prototype.of.call(lf, 'US'), 'United States', 'fallback remains branded');
}
print('ok tagged_newtarget_prototypes');
262;
