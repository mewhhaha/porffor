function same(a, b, label) { if (!Object.is(a, b)) throw new Error(label); }
function check(v, label) { if (!v) throw new Error(label); }
var log = [];
function Target() {}
var target = new Proxy(Target, {get(t, key, receiver) { if (key === 'prototype') log.push('prototype'); return Reflect.get(t,key,receiver); }});
var locales = {get length() { log.push('length'); return 1; }, get 0() { log.push('locale'); return 'en-US'; }};
var options = {};
for (var key of ['localeMatcher','numberingSystem','style','numeric']) {
  Object.defineProperty(options, key, {get: (function(k) { return function() { log.push(k); return {toString() { log.push('string:'+k); return {localeMatcher:'lookup',numberingSystem:'latn',style:'long',numeric:'auto'}[k]; }}; }; })(key)});
}
var rtf = Reflect.construct(Intl.RelativeTimeFormat, [locales,options], target);
same(log.join(','), 'prototype,length,locale,localeMatcher,string:localeMatcher,numberingSystem,string:numberingSystem,style,string:style,numeric,string:numeric', 'constructor order');
same(Intl.RelativeTimeFormat.prototype.format.call(rtf, 1, 'day'), 'tomorrow', 'fully initialized');
log = [];
var sentinel = {marker:1};
try { Reflect.construct(Intl.RelativeTimeFormat, [{get length() { log.push('unexpected'); return 0; }}], new Proxy(Target, {get() { throw sentinel; }})); throw new Error('missing prototype throw'); } catch(e) { same(e, sentinel, 'prototype abrupt'); }
same(log.length, 0, 'prototype abrupt stops locales');
log = [];
try { new Intl.RelativeTimeFormat('en-US', {get numberingSystem() { log.push('nu'); return 'bad!'; }, get style() { log.push('style'); return 'long'; }}); throw new Error('missing nu throw'); } catch(e) { check(e instanceof RangeError, 'nu RangeError'); }
same(log.join(','), 'nu', 'invalid nu stops later Gets');
try { Intl.RelativeTimeFormat('en-US'); throw new Error('missing new required'); } catch(e) { check(e instanceof TypeError, 'requires new'); }
same(new Intl.RelativeTimeFormat('en-US', 1).resolvedOptions().numeric, 'always', 'primitive options boxing');
try { new Intl.RelativeTimeFormat('en-US', null); throw new Error('missing null throw'); } catch(e) { check(e instanceof TypeError, 'null options'); }
print('ok constructor_observation_and_short_circuits');
262;
