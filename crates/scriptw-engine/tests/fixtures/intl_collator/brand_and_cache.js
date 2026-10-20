function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var getter = Object.getOwnPropertyDescriptor(Intl.Collator.prototype, 'compare').get;
for (var value of [undefined, null, 1, 'x', {}, Intl.Collator.prototype, Object.create(Intl.Collator.prototype)]) {
 throws(TypeError, function() { getter.call(value); }, 'getter private brand'); throws(TypeError, function() { Intl.Collator.prototype.resolvedOptions.call(value); }, 'resolved private brand');
}
var c = new Intl.Collator('en-US'); var proxyGets = 0; var proxy = new Proxy(c, {get() { proxyGets++; throw new Error('proxy observed'); }});
throws(TypeError, function() { getter.call(proxy); }, 'Proxy lacks private brand'); same(proxyGets, 0, 'brand no public reads');
var cmp = c.compare; same(cmp, c.compare, 'cache identity'); same(cmp.call({fake: true}, 'a', 'b'), cmp('a', 'b'), 'receiver ignored');
Object.defineProperty(c, 'compare', {configurable: true, value: function() { throw new Error('public dispatch'); }});
same(getter.call(c), cmp, 'public compare cannot replace private cache'); same(cmp('a', 'a'), 0, 'captured record');
print('ok brand_and_cache');
262;
