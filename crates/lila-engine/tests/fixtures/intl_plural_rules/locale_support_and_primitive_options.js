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

var rules = new Intl.PluralRules('en-US-u-nu-arab'); same(rules.resolvedOptions().locale,'en-US','irrelevant extension removed');
same(Intl.PluralRules.supportedLocalesOf(['en-US-u-nu-arab']).join(','),'en-US-u-nu-arab','matched requested extension preserved');
var log = [], options = new Proxy({}, {get(t,key) {log.push(key); check(key === 'localeMatcher','supported locales only reads matcher'); return 'lookup';}});
same(Intl.PluralRules.supportedLocalesOf(['en','en'],options).join(','),'en','canonical duplicate removal'); same(log.join(','),'localeMatcher','supported options order');
throws(TypeError,function () {new Intl.PluralRules('en',null);},'null constructor options');
throws(TypeError,function () {Intl.PluralRules.supportedLocalesOf('en',null);},'null supported options');
var boxed = false;
Object.defineProperty(Number.prototype,'type',{configurable:true,get() {boxed = Object.getPrototypeOf(this) === Number.prototype; return 'ordinal';}});
try {same(new Intl.PluralRules('en',1).select(3),'few','primitive constructor options'); check(boxed,'Number option boxed receiver');}
finally {delete Number.prototype.type;}
var touched = false;
Object.defineProperty(Object.prototype,'localeMatcher',{configurable:true,get() {touched=true; throw new Error('default prototype taint');}});
try {same(new Intl.PluralRules('en').select(1),'one','default null prototype options'); same(touched,false,'undefined options no prototype read');}
finally {delete Object.prototype.localeMatcher;}
log = [];
var locales = new Proxy({length:1,0:{toString() {log.push('locale string');return 'en';}}},{get(t,key) {log.push('get:'+key);return Reflect.get(t,key);},has(t,key) {log.push('has:'+key);return Reflect.has(t,key);}});
new Intl.PluralRules(locales,{localeMatcher:'lookup'});
same(log.join(','),'get:length,has:0,get:0,locale string','locale list observation');

print("ok locale_support_and_primitive_options");
262;
