function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var touches=0;
var poison=new Proxy({}, {get() {touches++;throw new Error('unexpected read');}});
throws(TypeError,function() {Intl.DisplayNames(poison,poison);},'NewTarget before reads');same(touches,0,'plain call observes nothing');
var marker={}; function T() {}
var nt=new Proxy(T,{get(t,k) {if(k==='prototype')throw marker;return Reflect.get(t,k);}});
var caught;try {Reflect.construct(Intl.DisplayNames,[poison,poison],nt);}catch(e){caught=e;}
same(caught,marker,'prototype abrupt identity');same(touches,0,'prototype abrupt before locales/options');
try {new Intl.DisplayNames([{toString(){throw marker;}}],poison);}catch(e){caught=e;}
same(caught,marker,'locale abrupt identity');same(touches,0,'locale abrupt before options');
for(var primitive of [null,true,false,7,'long',Symbol('options'),1n]) {
 var log=[];var locale={toString(){log.push('locale');return 'en-US';}};
 throws(TypeError,function(){new Intl.DisplayNames([locale],primitive);},'strict options');same(log.join(','),'locale','locales first');
}
for(var missing of [undefined,{}, {type:undefined}]) throws(TypeError,function(){new Intl.DisplayNames('en-US',missing);},'required type absent');
var order=['localeMatcher','style','type','fallback','languageDisplay'];
var valid={localeMatcher:'lookup',style:'long',type:'region',fallback:'code',languageDisplay:'dialect'};
for(var index=0;index<order.length;index++) {
 var bad=order[index], gets=[];
 var opts=new Proxy({}, {get(t,k){gets.push(k);return k===bad?'invalid':valid[k];}});
 throws(RangeError,function(){new Intl.DisplayNames('en-US',opts);},'closed option '+bad);
 same(gets.join(','),order.slice(0,index+1).join(','),'later option not read '+bad);
 gets=[];opts=new Proxy({}, {get(t,k){gets.push(k);if(k===bad)throw marker;return valid[k];}});
 try {new Intl.DisplayNames('en-US',opts);}catch(e){caught=e;}
 same(caught,marker,'getter abrupt '+bad);same(gets.join(','),order.slice(0,index+1).join(','),'getter short circuit');
 gets=[];opts=new Proxy({}, {get(t,k){gets.push(k);return k===bad?{toString(){throw marker;}}:valid[k];}});
 try {new Intl.DisplayNames('en-US',opts);}catch(e){caught=e;}
 same(caught,marker,'coercion abrupt '+bad);same(gets.join(','),order.slice(0,index+1).join(','),'coercion short circuit');
}
print('ok constructor_short_circuits');
262;
