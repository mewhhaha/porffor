function sourceFromUnits(units) {
  var source = '';
  for (var i = 0; i < units.length; i++) source += String.fromCharCode(units[i]);
  return source;
}
function freshObjects() {
  var array = new Array(0);
  array[0] = 'fresh';
  var object = {};
  object.value = array;
  if (!Object.isExtensible(array) || !Object.isExtensible(object) ||
      array.length !== 1 || array[0] !== 'fresh' || object.value !== array)
    throw 'released compiler storage poisoned a fresh object';
}
function failure(units, expected) {
  var caught;
  try { new RegExp(sourceFromUnits(units)); } catch (error) { caught = error; }
  if (!(caught instanceof expected)) throw 'wrong compile failure';
  freshObjects();
}
var optional = new RegExp(sourceFromUnits([40,97,41,63,92,49,42]));
var first = optional.exec('');
if (first === null || first[0] !== '' || first[1] !== undefined ||
    !Object.isExtensible(first)) throw 'optional capture array';
freshObjects();
var alternative = new RegExp(sourceFromUnits([40,63,58,40,97,41,124,98,41,92,49,42]));
var second = alternative.exec('b');
if (second === null || second[0] !== 'b' || second[1] !== undefined ||
    !Object.isExtensible(second)) throw 'alternative capture array';
freshObjects();
failure([40,97,98,99], SyntaxError);
failure([91,122,45,97,93], SyntaxError);
failure([97,123,53,48,48,48,125], RangeError);
var indexed = new RegExp(sourceFromUnits([40,97,98,41,43]), 'd');
var third = indexed.exec('abab');
if (third[0] !== 'abab' || third[1] !== 'ab' || third.indices[0][1] !== 4 ||
    third.indices[1][0] !== 2 || third.indices[1][1] !== 4)
  throw 'retained descriptor and index arrays';
freshObjects();
if (optional.exec('')[1] !== undefined || alternative.exec('b')[0] !== 'b' ||
    indexed.exec('ab')[1] !== 'ab') throw 'later compilation changed prior programs';
true;
