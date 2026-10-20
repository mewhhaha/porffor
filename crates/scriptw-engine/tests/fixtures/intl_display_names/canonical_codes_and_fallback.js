function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
for(var row of [['language','ZZZZZZZZ-aBcD-aB','zzzzzzzz-Abcd-AB'],['region','999','999'],['script','aBcD','Abcd'],['currency','aaa','AAA'],['calendar','123-ABC-abc','123-abc-abc']]) {
 var code=new Intl.DisplayNames('en-US',{type:row[0],fallback:'code'}), none=new Intl.DisplayNames('en-US',{type:row[0],fallback:'none'});
 same(code.of(row[1]),row[2],'canonical missing code '+row[0]);same(none.of(row[1]),undefined,'missing name is undefined '+row[0]);
}
same(new Intl.DisplayNames('en-US',{type:'region'}).of('419'),'Latin America','numeric region');
same(new Intl.DisplayNames('en-US',{type:'calendar'}).of('ISLAMICC'),'Hijri Calendar (tabular, civil epoch)','actual deprecated alias name');
same(new Intl.DisplayNames('en-US',{type:'calendar'}).of('ethioaa'),'Ethiopic Amete Alem Calendar','actual calendar alias');
for(var calendar of Intl.supportedValuesOf('calendar')) check(typeof new Intl.DisplayNames('en-US',{type:'calendar',fallback:'none'}).of(calendar)==='string','available calendar has real name '+calendar);
var dn=new Intl.DisplayNames('en-US',{type:'currency',fallback:'none'});
for(var currency of Intl.supportedValuesOf('currency')) check(typeof dn.of(currency)==='string','available currency has real name '+currency);
var unknown=new Intl.DisplayNames('en-US',{type:'language',fallback:'none'});
same(unknown.of('en-Abcd-AB'),undefined,'missing qualifier rejects partial translated name');
print('ok canonical_codes_and_fallback');
262;
