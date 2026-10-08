// Both construction and matching use the computed runtime compiler path.
var units = [94,40,63,60,113,49,62,120,41,92,107,60,113,49,62,36];
var source = '';
for (var i = 0; i < units.length; i++) source += String.fromCharCode(units[i]);
var expression = new RegExp(source, 'dg');
var match = expression.exec('xx');
if (expression.source !== source || match[0] !== 'xx' || match.groups.q1 !== 'x')
  throw 'computed named capture/backreference failed';
if (Object.getPrototypeOf(match.groups) !== null ||
    match.indices.groups.q1 !== match.indices[1] || expression.lastIndex !== 2)
  throw 'named result metadata failed';
true;
