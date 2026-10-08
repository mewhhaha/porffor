function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var log=[];
var locale={toString(){log.push('locale');return 'en-us';}};
var options={get localeMatcher(){log.push('matcher');return {toString(){log.push('matcher.string');return 'lookup';}};}};
var result=Intl.DisplayNames.supportedLocalesOf.call(null,[locale,'en-US','en-US-u-ca-buddhist'],options);
same(log.join(','),'locale,matcher,matcher.string','canonicalize before options');
same(result.join(','),'en-US,en-US-u-ca-buddhist','deduplicate and retain supported extension');
same(Object.getPrototypeOf(result),Array.prototype,'fresh supported array');
var gets=0;
throws(RangeError,function(){Intl.DisplayNames.supportedLocalesOf(['not_a_tag'],new Proxy({},{get(){gets++;return 'lookup';}}));},'invalid locale');same(gets,0,'locale failure before options');
var boxed=0;
Object.defineProperty(Number.prototype,'localeMatcher',{configurable:true,get(){boxed++;same(Object.getPrototypeOf(this),Number.prototype,'called Realm primitive boxing');return 'lookup';}});
same(Intl.DisplayNames.supportedLocalesOf('en-US',7)[0],'en-US','static primitive options accepted');same(boxed,1,'one boxed matcher Get');delete Number.prototype.localeMatcher;
throws(TypeError,function(){Intl.DisplayNames.supportedLocalesOf('en-US',null);},'null options');
same(Intl.DisplayNames.supportedLocalesOf(['en-US','es','he','fr','ja','ar-EG'],{localeMatcher:'lookup'}).join(','),'en-US,fr,ja,ar-EG','actual finite locale domain');
print('ok supported_locale_order_and_boxing');
262;
