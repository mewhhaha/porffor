var units = [40,63,60,61,97,41,98];
var source = '';
for (var i = 0; i < units.length; i++) source += String.fromCharCode(units[i]);
var expression = new RegExp(source);
expression.exec('ab')[0] === 'b' && expression.exec('xb') === null;
