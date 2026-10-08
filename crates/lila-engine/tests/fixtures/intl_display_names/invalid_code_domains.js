function check(value, label) { if (!value) throw new Error(label); }
function same(actual, expected, label) { if (!Object.is(actual, expected)) throw new Error(label + ': ' + String(actual)); }
function throws(C, body, label) { var caught; try { body(); } catch (e) { caught = e; } check(caught && caught.constructor === C && caught instanceof C, label); return caught; }
var vectors=[
 ['language',['','a','abcdefghi','root','abcd','abcd-GB','en_GB','en-u-hebrew','en--GB','en-US-','aa-aaaa-bbbb','aa-aaaaa-AAAAA','aa-bb-cc']],
 ['region',['U','USA','12','1234','U1','éé',' US']],
 ['script',['Lat','Latin','L4tn','l_at','éabc']],
 ['currency',['US','USDD','U1D','u$d',' USD']],
 ['calendar',['','ab','abcdefghi','abc--def','-abc','abc-','abc_de','ébc']],
 ['dateTimeField',['Year','weekofyear','dayperiod','timezoneName','millisecond','', 'calendar']]
];
for(var row of vectors) for(var fallback of ['code','none']) {
 var dn=new Intl.DisplayNames('en-US',{type:row[0],fallback:fallback});
 for(var code of row[1]) throws(RangeError,function(){dn.of(code);},'invalid '+row[0]+' '+code+' '+fallback);
 throws(RangeError,function(){dn.of('\uD800');},'invalid UTF16 '+row[0]);
}
print('ok invalid_code_domains');
262;
