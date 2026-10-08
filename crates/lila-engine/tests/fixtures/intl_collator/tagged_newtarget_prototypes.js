function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var getter = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, 'compare').get;
var fn = function Tagged() {}; var array = ['marker']; var proxy = new Proxy({marker: 42}, {});
for (var proto of [fn, array, proxy]) { function Target() {} Target.prototype = proto; var c = Reflect.construct(Intl.Collator, ['en-US'], Target); same(Object.getPrototypeOf(c), proto, 'representation tag preserved'); same(getter.call(c)('a', 'a'), 0, 'brand with tagged prototype'); }
var foreign = __lilaCreateRealm().global;
var frozenPrototypeGets = 0;
var frozenTarget = new Proxy(foreign.Function, {get(t, k) { if (k === 'prototype') { frozenPrototypeGets++; return 1; } return Reflect.get(t, k); }});
throws(TypeError, function() { Reflect.construct(Intl.Collator, ['en-US'], frozenTarget); }, 'frozen Function.prototype Proxy invariant');
same(frozenPrototypeGets, 1, 'frozen prototype observed once');
// A bound constructor has no own frozen prototype, so this primitive Get is valid.
var Target = foreign.Function.bind(null); var proxyTarget = new Proxy(Target, {get(t, k) { if (k === 'prototype') return 1; return Reflect.get(t, k); }});
var c = Reflect.construct(Intl.Collator, ['en-US'], proxyTarget); same(Object.getPrototypeOf(c), foreign.Intl.Collator.prototype, 'primitive fallback actual NewTarget Realm'); same(getter.call(c)('a', 'b') < 0, true, 'foreign fallback record');
print('ok tagged_newtarget_prototypes');
262;
