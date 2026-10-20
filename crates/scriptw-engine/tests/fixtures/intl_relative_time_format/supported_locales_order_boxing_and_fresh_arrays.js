function same(a,b,label) { if (!Object.is(a,b)) throw new Error(label); }
function check(v,label) { if (!v) throw new Error(label); }
var log=[];
var locales={get length(){log.push('length');return 3;}, get 0(){log.push('zero');return 'fr';},get 1(){log.push('one');return 'en-GB';},get 2(){log.push('two');return 'zz-ZZ';}};
var options={get localeMatcher(){log.push('matcher');return {toString(){log.push('string');return 'lookup';}};}};
var a=Intl.RelativeTimeFormat.supportedLocalesOf(locales,options);
same(log.join(','),'length,zero,one,two,matcher,string','supported order');
same(a.join(','),'fr,en-GB','captured service fallback');
var b=Intl.RelativeTimeFormat.supportedLocalesOf(['fr','en-GB','zz-ZZ'],1);
same(b.join(','),'fr,en-GB','primitive option boxing');check(a!==b,'fresh array');
a[0]='de'; same(Intl.RelativeTimeFormat.supportedLocalesOf(['fr'])[0],'fr','no shared mutable result');
same(new Intl.RelativeTimeFormat('en-GB').resolvedOptions().locale,'en','service inventory selected before full NF catalogue');
same(new Intl.RelativeTimeFormat('zz-ZZ').resolvedOptions().locale,'en-US','explicit default profile');
print('ok supported_locales_order_boxing_and_fresh_arrays');
262;
