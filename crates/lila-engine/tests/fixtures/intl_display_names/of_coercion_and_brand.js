function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var dn=new Intl.DisplayNames('en-US',{type:'region'}), method=Intl.DisplayNames.prototype.of;
var reads=0, code=new Proxy({}, {get(){reads++;throw new Error('unexpected code read');}});
for(var receiver of [undefined,null,0,true,'US',{},Intl.DisplayNames.prototype,Object.create(Intl.DisplayNames.prototype),new Proxy(dn,{})]) throws(TypeError,function(){method.call(receiver,code);},'private brand');
same(reads,0,'brand before code coercion');
var hint, calls=0;
same(dn.of({[Symbol.toPrimitive](value){hint=value;calls++;return 'us';}}),'United States','observed code');same(hint,'string','string hint');same(calls,1,'one ToPrimitive');
var log=[];
same(dn.of({toString(){log.push('string');return {};},valueOf(){log.push('value');return 'us';}}),'United States','ordinary primitive fallback');same(log.join(','),'string,value','ToString order');
var marker={}, caught;
try{dn.of({toString(){throw marker;}});}catch(e){caught=e;}same(caught,marker,'abrupt identity');
throws(TypeError,function(){dn.of(Symbol('US'));},'symbol code');
throws(RangeError,function(){dn.of(undefined);},'undefined converted then invalid');
same(dn.of('US',code),'United States','extra argument ignored');same(reads,0,'extra argument not coerced');
throws(RangeError,function(){dn.of('\uD800');},'isolated high surrogate reaches validator');
throws(RangeError,function(){dn.of('\uDC00');},'isolated low surrogate reaches validator');
print('ok of_coercion_and_brand');
262;
