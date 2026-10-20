function fromUnits(units) {
  var source = '';
  for (var i = 0; i < units.length; i++) source += String.fromCharCode(units[i]);
  return source;
}
function require(value, label) { if (!value) throw new Error(label); }
