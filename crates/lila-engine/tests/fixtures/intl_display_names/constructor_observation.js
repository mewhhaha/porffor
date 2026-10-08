function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var log = [];
var locale = {toString() { log.push('locale.toString'); return 'en-US'; }};
var locales = new Proxy({length:1,0:locale}, {
 get(target,key) {log.push('locales.get '+String(key)); return Reflect.get(target,key);},
 has(target,key) {log.push('locales.has '+String(key)); return Reflect.has(target,key);}
});
var values = {localeMatcher:'lookup',style:'short',type:'region',fallback:'none',languageDisplay:'standard'};
var options = new Proxy({}, {get(target,key) {log.push('options.get '+String(key)); return {toString() {log.push(String(key)+'.toString');return values[key];}};}});
function Target() {}
var proto={inherited:42};
var nt=new Proxy(Target,{get(target,key) {if(key==='prototype') {log.push('newTarget.prototype');return proto;} return Reflect.get(target,key);}});
var dn=Reflect.construct(Intl.DisplayNames,[locales,options],nt);
same(log.join('|'),'newTarget.prototype|locales.get length|locales.has 0|locales.get 0|locale.toString|options.get localeMatcher|localeMatcher.toString|options.get style|style.toString|options.get type|type.toString|options.get fallback|fallback.toString|options.get languageDisplay|languageDisplay.toString','complete observation order including nonlanguage languageDisplay');
same(Object.getPrototypeOf(dn),proto,'retained prototype');same(dn.inherited,42,'inherited property');
same(Intl.DisplayNames.prototype.of.call(dn,'US'),'US','configuration retained');
var resolved=Intl.DisplayNames.prototype.resolvedOptions.call(dn);same(resolved.style,'short','observed style');check(!Object.hasOwn(resolved,'languageDisplay'),'nonlanguage property absent');
print('ok constructor_observation');
262;
