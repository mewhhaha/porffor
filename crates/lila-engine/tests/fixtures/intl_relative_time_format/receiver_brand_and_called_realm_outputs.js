function check(v,label){if(!v)throw new Error(label);}
function same(a,b,label){if(!Object.is(a,b))throw new Error(label);}
var foreign=__lilaCreateRealm().global;
var other=new foreign.Intl.RelativeTimeFormat('en-US');
var local=new Intl.RelativeTimeFormat('en-US');
var parts=foreign.Intl.RelativeTimeFormat.prototype.formatToParts.call(local,1,'day');
same(Object.getPrototypeOf(parts),foreign.Array.prototype,'called Realm parts array');
same(Object.getPrototypeOf(parts[0]),foreign.Object.prototype,'called Realm part object');
same(Object.getPrototypeOf(foreign.Intl.RelativeTimeFormat.prototype.resolvedOptions.call(local)),foreign.Object.prototype,'called Realm resolved object');
same(Object.getPrototypeOf(foreign.Intl.RelativeTimeFormat.supportedLocalesOf(['en-US'])),foreign.Array.prototype,'called Realm supported array');
same(Intl.RelativeTimeFormat.prototype.format.call(other,-1,'day'),'1 day ago','cross Realm brand');
for(var receiver of [null,undefined,{},Intl.RelativeTimeFormat.prototype,Object.create(local),new Proxy(local,{})]){
 var calls=0;
 try { foreign.Intl.RelativeTimeFormat.prototype.format.call(receiver,{valueOf(){calls++;return 1;}},'day');throw new Error('missing brand'); } catch(e){ check(e instanceof foreign.TypeError,'called Realm brand TypeError'); }
 same(calls,0,'brand fails before coercion');
}
try { foreign.Intl.RelativeTimeFormat.prototype.format.call(local,Infinity,'day');throw new Error('missing RangeError'); } catch(e){ check(e instanceof foreign.RangeError,'called Realm finite RangeError'); }
print('ok receiver_brand_and_called_realm_outputs');
262;
