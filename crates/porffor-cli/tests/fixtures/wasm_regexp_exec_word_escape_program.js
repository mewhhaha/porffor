const asciiWord = /\w+/.exec("AZaz09_");
const asciiNonWord = /\W+/.exec("!-");
const noAsciiWordFalsePositive = /\w/.exec("!") === null && /\w/.exec("é") === null;
const captured = /(\w+)(\W+)/.exec("word!?");
const captures = captured !== null && captured[0] === "word!?" && captured[1] === "word" && captured[2] === "!?";

const astral = "😀";
const unicodeAstral = /\W/u.exec(astral);
const unicodeConsumesScalar = unicodeAstral !== null && unicodeAstral[0] === astral && unicodeAstral[0].length === 2;
const nonUnicodeCodeUnits = /\W{2}/.exec(astral);
const nonUnicodeConsumesCodeUnits = nonUnicodeCodeUnits !== null && nonUnicodeCodeUnits[0] === astral;
const excessiveCodeUnitsDoNotMatch = /\W{3}/.exec(`${astral}A`) === null;
const explicitCodeUnitsThenAscii = /\W\WA/.exec(`${astral}A`);
const explicitCodeUnitsAdvanceToAscii = explicitCodeUnitsThenAscii !== null
  && explicitCodeUnitsThenAscii[0] === `${astral}A`;
const quantifiedCodeUnitsThenAscii = /\W{2}A/.exec(`${astral}A`);
const quantifiedCodeUnitsAdvanceToAscii = quantifiedCodeUnitsThenAscii !== null
  && quantifiedCodeUnitsThenAscii[0] === `${astral}A`;
const individualCodeUnits = /\W/g;
const high = individualCodeUnits.exec(astral);
const highLastIndex = individualCodeUnits.lastIndex;
const low = individualCodeUnits.exec(astral);
const lowLastIndex = individualCodeUnits.lastIndex;
const highAndLowAreNonWord = high !== null && high[0].charCodeAt(0) === 0xD83D
  && highLastIndex === 1
  && low !== null && low[0].charCodeAt(0) === 0xDE00
  && lowLastIndex === 2;
const loneSurrogate = String.fromCharCode(0xD800);
const loneSurrogateNonWord = /\W/.exec(loneSurrogate);
const loneSurrogateMatches = loneSurrogateNonWord !== null
  && loneSurrogateNonWord[0].charCodeAt(0) === 0xD800;

asciiWord !== null && asciiWord[0] === "AZaz09_"
  && asciiNonWord !== null && asciiNonWord[0] === "!-"
  && noAsciiWordFalsePositive && captures && unicodeConsumesScalar
  && nonUnicodeConsumesCodeUnits && excessiveCodeUnitsDoNotMatch
  && explicitCodeUnitsAdvanceToAscii && quantifiedCodeUnitsAdvanceToAscii
  && highAndLowAreNonWord && loneSurrogateMatches;
