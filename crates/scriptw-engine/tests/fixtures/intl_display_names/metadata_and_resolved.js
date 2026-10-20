function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
function descriptor(object, key, value, writable, enumerable, configurable) {
  var d = Object.getOwnPropertyDescriptor(object, key);
  same(d.value, value, String(key) + ' value');
  same(d.writable, writable, String(key) + ' writable');
  same(d.enumerable, enumerable, String(key) + ' enumerable');
  same(d.configurable, configurable, String(key) + ' configurable');
}
var C = Intl.DisplayNames;
same(typeof C, 'function', 'constructor installed');
same(Object.getPrototypeOf(C), Function.prototype, 'constructor prototype');
descriptor(Intl, 'DisplayNames', C, true, false, true);
descriptor(C, 'name', 'DisplayNames', false, false, true);
descriptor(C, 'length', 2, false, false, true);
descriptor(C, 'prototype', C.prototype, false, false, false);
same(Object.getPrototypeOf(C.prototype), Object.prototype, 'ordinary prototype');
descriptor(C.prototype, 'constructor', C, true, false, true);
descriptor(C.prototype, Symbol.toStringTag, 'Intl.DisplayNames', false, false, true);
for (var row of [[C, 'supportedLocalesOf', 1], [C.prototype, 'resolvedOptions', 0], [C.prototype, 'of', 1]]) {
  var method = row[0][row[1]];
  descriptor(row[0], row[1], method, true, false, true);
  descriptor(method, 'name', row[1], false, false, true);
  descriptor(method, 'length', row[2], false, false, true);
  check(!Object.hasOwn(method, 'prototype'), row[1] + ' nonconstructor');
}
var dn = new C('en-US', {type: 'language'});
same(Object.getPrototypeOf(dn), C.prototype, 'instance prototype');
same(Object.prototype.toString.call(dn), '[object Intl.DisplayNames]', 'tag');
same(Reflect.ownKeys(dn).length, 0, 'private slots have no own keys');
check(Object.isExtensible(dn), 'extensible instance');
var first = dn.resolvedOptions();
same(Object.getPrototypeOf(first), Object.prototype, 'resolved ordinary object');
same(Reflect.ownKeys(first).join(','), 'locale,style,type,fallback,languageDisplay', 'resolved property order');
for (var row of [['locale','en-US'],['style','long'],['type','language'],['fallback','code'],['languageDisplay','dialect']]) descriptor(first,row[0],row[1],true,true,true);
first.type = 'region'; delete first.locale;
var second = dn.resolvedOptions(); check(first !== second, 'fresh options');
same(second.type, 'language', 'private type retained'); same(second.locale, 'en-US', 'private locale retained');
for (var type of ['region','script','currency','calendar','dateTimeField']) {
  var options = new C('en-US-u-ca-buddhist', {type:type, languageDisplay:'standard', extra:7}).resolvedOptions();
  same(Reflect.ownKeys(options).join(','), 'locale,style,type,fallback', 'nonlanguage options ' + type);
  same(options.locale, 'en-US', 'irrelevant extension removed'); check(!Object.hasOwn(options,'languageDisplay'), 'no nonlanguage slot');
}
class Derived extends C {}
var derived = new Derived('en-US', {type:'region',style:'short',fallback:'none'});
same(Object.getPrototypeOf(derived), Derived.prototype, 'actual subclass prototype'); same(derived.of('US'), 'US', 'subclass record');
print('ok metadata_and_resolved');
262;
