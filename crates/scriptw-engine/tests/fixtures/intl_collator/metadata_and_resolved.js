function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + actual); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
same(Intl.Collator.length, 0, 'constructor length');
same(Intl.Collator.name, 'Collator', 'constructor name');
var proto = Intl.Collator.prototype;
same(Object.getPrototypeOf(proto), Object.prototype, 'ordinary prototype');
var d = Object.getOwnPropertyDescriptor(proto, 'compare');
same(typeof d.get, 'function', 'real getter'); same(d.set, undefined, 'absent setter'); same(d.enumerable, false, 'getter not enumerable'); same(d.configurable, true, 'getter configurable');
same(d.get.length, 0, 'getter length'); same(d.get.name, 'get compare', 'getter name');
var c = Intl.Collator('en-US'); var other = Intl.Collator('en-US');
check(c !== other, 'plain calls allocate'); same(Object.getPrototypeOf(c), proto, 'plain prototype');
var o = c.resolvedOptions();
same(Object.keys(o).join('|'), 'locale|usage|sensitivity|ignorePunctuation|collation|numeric|caseFirst', 'resolved key order');
same(o.usage, 'sort', 'default usage'); same(o.sensitivity, 'variant', 'sort sensitivity'); same(o.collation, 'default', 'default co'); same(o.numeric, false, 'default numeric'); same(o.caseFirst, 'false', 'default first');
check(c.resolvedOptions() !== o, 'fresh resolved object'); same(Object.getPrototypeOf(o), Object.prototype, 'resolved prototype');
same(Object.prototype.toString.call(c), '[object Intl.Collator]', 'tag');
same(c.compare.length, 2, 'bound length'); same(c.compare.name, '', 'anonymous bound name');
throws(TypeError, function() { new c.compare('a', 'b'); }, 'nonconstructible compare');
print('ok metadata_and_resolved');
262;
