function same(a,b,label) { if (!Object.is(a,b)) throw new Error(label); }
function check(v,label) { if (!v) throw new Error(label); }
var rtf = new Intl.RelativeTimeFormat('en-US');
for (var unit of ['year','quarter','month','week','day','hour','minute','second']) {
  same(rtf.format(1000,unit), 'in 1,000 '+unit+'s', 'eight unit grouping');
  same(rtf.format(-1,unit+'s'), '1 '+unit+' ago', 'plural unit accepted and singular grammar');
  same(rtf.format(0,unit), 'in 0 '+unit+'s', 'positive zero');
  same(rtf.format(-0,unit), '0 '+unit+'s ago', 'negative zero');
  var parts = rtf.formatToParts(1000,unit);
  same(parts.length,5,'five grouped parts');
  same(parts.map(function(p) {return p.type;}).join(','),'literal,integer,group,integer,literal','part kinds');
  same(parts.map(function(p) {return p.value;}).join(''),rtf.format(1000,unit),'string concatenates exact parts');
  for (var i=0;i<parts.length;i++) {
    same(Object.prototype.hasOwnProperty.call(parts[i],'unit'),i>0 && i<4,'literal lacks own unit');
    if (i>0 && i<4) same(parts[i].unit,unit,'numeric part singular unit');
    same(Object.keys(parts[i]).join(','), i>0 && i<4 ? 'type,value,unit' : 'type,value','part property order');
  }
  check(parts!==rtf.formatToParts(1000,unit),'fresh parts');
}
same(rtf.format(1.0004,'day'),'in 1 day','rounded cardinal one');
same(rtf.format(1.0005,'day'),'in 1.001 days','rounded cardinal other');
same(rtf.format(-0.0001,'day'),'0 days ago','rounding retains past direction');
print('ok eight_units_parts_rounding_and_signed_zero');
262;
