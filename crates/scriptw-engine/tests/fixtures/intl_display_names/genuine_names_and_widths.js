function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
for(var row of [['long','United States','year','week'],['short','US','yr.','wk.'],['narrow','US','yr','wk']]) {
 var style=row[0];
 same(new Intl.DisplayNames('en-US',{type:'region',style:style}).of('us'),row[1],'real region width');
 same(new Intl.DisplayNames('en-US',{type:'dateTimeField',style:style}).of('year'),row[2],'real year width');
 same(new Intl.DisplayNames('en-US',{type:'dateTimeField',style:style}).of('weekOfYear'),row[3],'real week width');
 same(new Intl.DisplayNames('en-US',{type:'currency',style:style}).of('usd'),'US Dollar','currency name not symbol');
 same(new Intl.DisplayNames('en-US',{type:'script',style:style}).of('hANS'),'Simplified Han','standalone script name');
 same(new Intl.DisplayNames('en-US',{type:'calendar',style:style}).of('gregory'),'Gregorian Calendar','calendar width');
 same(new Intl.DisplayNames('fr',{type:'currency',style:style}).of('USD'),'dollar des États-Unis','genuine French currency');
}
same(new Intl.DisplayNames('fr',{type:'region',style:'short'}).of('US'),'É.-U.','genuine French short region');
same(new Intl.DisplayNames('ar',{type:'calendar'}).of('GREGORY'),'التقويم الميلادي','genuine Arabic');
same(new Intl.DisplayNames('ja',{type:'region'}).of('US'),'アメリカ合衆国','genuine Japanese');
same(new Intl.DisplayNames('zh',{type:'currency'}).of('USD'),'美元','genuine Chinese');
var fields=new Intl.DisplayNames('en-US',{type:'dateTimeField',fallback:'none'});
for(var field of ['era','year','quarter','month','weekOfYear','weekday','day','dayPeriod','hour','minute','second','timeZoneName']) check(typeof fields.of(field)==='string','complete field '+field);
var locales=['en','en-US','ar','ar-EG','zh','zh-Hans','zh-Hans-CN','de','fr','it','ja','ko','hi'];
for(var locale of locales) for(var style of ['long','short','narrow']) for(var pair of [['language','fr'],['region','US'],['script','Hans'],['currency','USD'],['calendar','gregory'],['dateTimeField','year']]) check(typeof new Intl.DisplayNames(locale,{type:pair[0],style:style,fallback:'none'}).of(pair[1])==='string','actual association '+locale+style+pair[0]);
print('ok genuine_names_and_widths');
262;
