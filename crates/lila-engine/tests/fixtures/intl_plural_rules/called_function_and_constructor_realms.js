// Source/data-derived expectations; authored control, not an executed result.
function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { check(Object.is(actual, expected), label); }
function throws(ctor, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  check(caught instanceof ctor && caught.constructor === ctor, label);
}
function abrupt(marker, action, label) {
  var caught;
  try { action(); } catch (error) { caught = error; }
  same(caught, marker, label);
}

var foreign = __lilaCreateRealm().global, Other = foreign.Intl.PluralRules;
check(Other !== Intl.PluralRules && Other.prototype !== Intl.PluralRules.prototype,'fresh Realm intrinsics');
same(Object.getPrototypeOf(Other),foreign.Function.prototype,'foreign constructor Realm');
var local = new Intl.PluralRules('en'), other = new Other('en');
var foreignResult = Other.prototype.resolvedOptions.call(local), localResult = Intl.PluralRules.prototype.resolvedOptions.call(other);
same(Object.getPrototypeOf(foreignResult),foreign.Object.prototype,'called foreign result Realm'); same(Object.getPrototypeOf(foreignResult.pluralCategories),foreign.Array.prototype,'called foreign array Realm');
same(Object.getPrototypeOf(localResult),Object.prototype,'called local result Realm'); same(Object.getPrototypeOf(localResult.pluralCategories),Array.prototype,'called local array Realm');
throws(foreign.TypeError,function () {Other.prototype.select.call({},1);},'foreign brand error Realm');
throws(foreign.TypeError,function () {Other.prototype.select.call(local,Symbol());},'foreign conversion error Realm');
throws(foreign.RangeError,function () {Other.prototype.selectRange.call(local,NaN,1);},'foreign range error Realm');
throws(TypeError,function () {Intl.PluralRules.prototype.select.call(other,Symbol());},'local conversion error Realm');
var supported = Other.supportedLocalesOf('en'); same(Object.getPrototypeOf(supported),foreign.Array.prototype,'foreign supported array Realm');
var bound = foreign.Array.bind(null);
for (var prototype of [undefined,null,false,0,'prototype',Symbol('prototype')]) {
  Object.defineProperty(bound,'prototype',{value:prototype,writable:true,configurable:true});
  var selected = Reflect.construct(Intl.PluralRules,['en'],new Proxy(new Proxy(bound,{}),{}));
  same(Object.getPrototypeOf(selected),Other.prototype,'NewTarget fallback Realm'); same(selected.select(1),'one','fallback branded');
}
var boxed = false;
Object.defineProperty(foreign.Number.prototype,'type',{configurable:true,get() {boxed=Object.getPrototypeOf(this)===foreign.Number.prototype;return 'ordinal';}});
same(new Other('en',1).select(3),'few','foreign primitive options'); check(boxed,'boxing uses active Realm');

print("ok called_function_and_constructor_realms");
262;
