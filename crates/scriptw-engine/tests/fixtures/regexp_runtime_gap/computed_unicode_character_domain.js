function computed(units, flags) { return new RegExp(fromUnits(units), fromUnits(flags)); }
var face = fromUnits([0xd83d, 0xde00]);
var nextFace = fromUnits([0xd83d, 0xde01]);
var outsideFaceRange = fromUnits([0xd83d, 0xde02]);
var kelvin = fromUnits([0x212a]);
var longS = fromUnits([0x017f]);
var deseretUpper = fromUnits([0xd801, 0xdc00]);
var deseretLower = fromUnits([0xd801, 0xdc28]);
for (var mode = 117; mode <= 118; mode++) {
  for (var marker of [68, 83, 87]) {
    var complement = computed([94, 92, marker, 36], [100, 103, mode]);
    var match = complement.exec(face);
    require(match && match[0] === face && match.indices[0][1] === 2, 'complement includes complete astral character');
    require(complement.lastIndex === 2, 'Unicode global advancement is UTF-16 length');
    require(computed([94, 91, 92, marker, 93, 36], [mode]).test(face), 'pooled class complement includes astral character');
  }
  require(computed([94, 107, 36], [105, mode]).test(kelvin), 'Unicode literal simple fold');
  require(computed([94, 115, 36], [105, mode]).test(longS), 'Unicode long-s fold');
  require(computed([94, 0xd801, 0xdc00, 36], [105, mode]).test(deseretLower), 'astral simple fold fits character domain');
  require(computed([94, 91, 107, 93, 36], [105, mode]).test(kelvin), 'Unicode class simple fold');
  require(!computed([94, 91, 94, 107, 93, 36], [105, mode]).test(kelvin), 'negated class complements after case closure');
  require(computed([94, 92, 119, 36], [105, mode]).test(kelvin), 'word class includes fold-to-ASCII character');
  require(computed([94, 92, 119, 36], [105, mode]).test(longS), 'word class includes long s');
  require(!computed([94, 92, 87, 36], [105, mode]).test(kelvin), 'nonword complements closed WordCharacters');
  require(!computed([94, 91, 92, 87, 93, 36], [105, mode]).test(longS), 'class nonword excludes closed word members');
  require(computed([94, 92, 98, 107, 92, 98, 36], [105, mode]).test(kelvin), 'word boundary uses Unicode closed word range');
  require(!computed([94, 92, 66, 107, 92, 66, 36], [105, mode]).test(kelvin), 'nonboundary retains polarity');
  require(computed([40, 63, 105, 58, 94, 92, 119, 36, 41], [mode]).test(kelvin), 'scoped i closes word class');
  require(!computed([40, 63, 45, 105, 58, 94, 92, 119, 36, 41], [105, mode]).test(kelvin), 'scoped removal restores sensitive word class');
  require(computed([40, 63, 45, 105, 58, 94, 92, 87, 36, 41], [105, mode]).test(kelvin), 'scoped removal restores nonword complement');
  require(computed([40, 63, 105, 58, 94, 92, 98, 107, 92, 98, 36, 41], [mode]).test(kelvin), 'scoped i closes boundaries');
  require(!computed([40, 63, 45, 105, 58, 94, 92, 98, 107, 92, 98, 36, 41], [105, mode]).test(kelvin), 'scoped removal restores boundary membership');
  require(computed([94, 0xd83d, 0xde00, 43, 36], [mode]).test(face + face), 'raw pair is one quantified Unicode atom');
  require(computed([94, 92, 117, 68, 56, 51, 68, 92, 117, 68, 69, 48, 48, 43, 36], [mode]).test(face + face), 'two fixed escapes form one quantified Unicode atom');
  require(!computed([94, 0xd83d, 92, 117, 68, 69, 48, 48, 36], [mode]).test(face), 'raw lead cannot merge with escaped trail');
  require(!computed([94, 92, 117, 68, 56, 51, 68, 0xde00, 36], [mode]).test(face), 'escaped lead cannot merge with raw trail');
  require(!computed([94, 91, 0xd83d, 92, 117, 68, 69, 48, 48, 93, 36], [mode]).test(face), 'raw class lead cannot merge with escaped trail');
  require(!computed([94, 91, 92, 117, 68, 56, 51, 68, 0xde00, 93, 36], [mode]).test(face), 'escaped class lead cannot merge with raw trail');
  require(computed([94, 91, 0xd83d, 0xde00, 93, 36], [mode]).test(face), 'raw class pair denotes one code point');
  require(computed([94, 91, 92, 117, 68, 56, 51, 68, 92, 117, 68, 69, 48, 48, 93, 36], [mode]).test(face), 'fixed class pair denotes one code point');
  var range = computed([94, 91, 0xd83d, 0xde00, 45, 0xd83d, 0xde01, 93, 36], [mode]);
  require(range.test(face) && range.test(nextFace) && !range.test(outsideFaceRange), 'astral range endpoints remain scalars');
  require(computed([94, 91, 0xdbff, 0xdfff, 93, 36], [mode]).test(fromUnits([0xdbff, 0xdfff])), 'maximum code point remains in bitmap');
  var loneLeadClass = computed([94, 91, 0xd83d, 93, 36], [mode]);
  require(loneLeadClass.test(fromUnits([0xd83d])) && !loneLeadClass.test(face), 'lone surrogate is distinct from paired code point');
  var reverse = computed([40, 63, 60, 61, 0xd83d, 0xde00, 41, 40, 46, 41], [100, mode]).exec(face + 'x');
  require(reverse && reverse[1] === 'x' && reverse.indices[0][0] === 2 && reverse.indices[1][1] === 3, 'reverse Unicode atom and capture indices');
  var reference = computed([94, 40, 46, 41, 92, 49, 36], [105, mode]);
  require(reference.test(kelvin + 'k') && reference.test(deseretUpper + deseretLower), 'computed references retain Unicode folding table');
}
require(!computed([94, 107, 36], [105]).test(kelvin), 'legacy non-ASCII-to-ASCII folding remains excluded');
require(!computed([94, 92, 119, 36], [105]).test(longS), 'legacy word membership remains ASCII');
require(!computed([94, 0xd83d, 0xde00, 43, 36], []).test(face + face), 'legacy quantifier remains attached to trail unit');
require(computed([94, 0xd83d, 0xde00, 43, 36], []).test(face + fromUnits([0xde00])), 'legacy trail repetition still matches');
print('ok');
262;
