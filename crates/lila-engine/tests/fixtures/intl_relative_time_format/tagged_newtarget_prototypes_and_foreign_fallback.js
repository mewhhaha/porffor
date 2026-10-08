function check(v,label){if(!v)throw new Error(label);}
function same(a,b,label){if(!Object.is(a,b))throw new Error(label);}
var foreign=__lilaCreateRealm().global;
function Target(){}
function functionPrototype(){}
var arrayPrototype=['index'];
var proxyPrototype=new Proxy({marker:'proxy'},{get(t,k,r){return Reflect.get(t,k,r);}});
for(var prototype of [functionPrototype,arrayPrototype,proxyPrototype]){
 Target.prototype=prototype;
 var rtf=Reflect.construct(Intl.RelativeTimeFormat,['en-US'],Target);
 same(Object.getPrototypeOf(rtf),prototype,'exact tagged NewTarget prototype');
 same(Intl.RelativeTimeFormat.prototype.format.call(rtf,1,'day'),'in 1 day','private brand retained');
}
var bound=foreign.Array.bind(null);
for(var primitive of [undefined,null,false,0,'prototype',Symbol('prototype')]){
 bound.prototype=primitive;
 var rtf=Reflect.construct(Intl.RelativeTimeFormat,['en-US'],new Proxy(new Proxy(bound,{}),{}));
 same(Object.getPrototypeOf(rtf),foreign.Intl.RelativeTimeFormat.prototype,'bound proxy primitive fallback Realm');
 same(rtf.format(-1,'day'),'1 day ago','fallback private state');
}
try {Reflect.construct(Intl.RelativeTimeFormat,['en-US'],new Proxy(foreign.Function,{get(t,k,r){if(k==='prototype')return 1;return Reflect.get(t,k,r);}}));throw new Error('missing frozen invariant');}catch(e){check(e instanceof TypeError,'frozen prototype Proxy invariant remains TypeError');}
print('ok tagged_newtarget_prototypes_and_foreign_fallback');
262;
