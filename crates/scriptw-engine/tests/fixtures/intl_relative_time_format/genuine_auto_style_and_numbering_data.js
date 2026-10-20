function same(a,b,label) { if (!Object.is(a,b)) throw new Error(label); }
function check(v,label) { if (!v) throw new Error(label); }
var auto = new Intl.RelativeTimeFormat('en-US',{numeric:'auto'});
same(auto.format(-0,'day'),'today','negative auto zero'); same(auto.format(0,'day'),'today','positive auto zero');
same(auto.format(-1,'day'),'yesterday','past auto'); same(auto.format(1,'day'),'tomorrow','future auto');
same(auto.format(1.25,'day'),'in 1.25 days','nonintegral auto fallback');
var literal=auto.formatToParts(0,'day'); same(literal.length,1,'single auto literal'); same(literal[0].type,'literal','auto type'); same(literal[0].value,'today','auto value'); check(!('unit' in literal[0]),'auto literal unit absent');
same(new Intl.RelativeTimeFormat('en-US',{style:'narrow'}).format(3,'day'),'in 3d','genuine narrow');
var fr = new Intl.RelativeTimeFormat('fr',{numeric:'auto'}); same(fr.format(-2,'day'),'avant-hier','French minus two'); same(fr.format(2,'day'),'après-demain','French plus two');
var ar = new Intl.RelativeTimeFormat('ar');
same(ar.format(1,'day'),'خلال يوم واحد','Arabic literal one'); same(ar.format(2,'day'),'خلال يومين','Arabic literal two');
var arabic = ar.formatToParts(1,'day'); same(arabic.length,1,'Arabic no placeholder'); check(!('unit' in arabic[0]),'Arabic literal no unit');
var nu = new Intl.RelativeTimeFormat('en-US-u-nu-arab');
same(nu.resolvedOptions().numberingSystem,'arab','Unicode numbering selected'); check(nu.format(12,'day').indexOf('١٢')!==-1,'native digits copied unchanged');
var override = new Intl.RelativeTimeFormat('en-US-u-nu-arab',{numberingSystem:'latn'}).resolvedOptions(); same(override.locale,'en-US','option removes different extension'); same(override.numberingSystem,'latn','explicit numbering wins');
print('ok genuine_auto_style_and_numbering_data');
262;
