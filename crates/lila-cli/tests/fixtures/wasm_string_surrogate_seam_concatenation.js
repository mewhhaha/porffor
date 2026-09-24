// ECMAScript strings are sequences of UTF-16 code units, so appending a lone
// low surrogate to a string that ends with a lone high surrogate yields the
// same String value as the literal surrogate pair. Every concatenation route
// must agree with the literal: equality, property keys, code points, string
// iteration and /u RegExp matching all observe the pair.
const high = "\uD83D";
const low = "\uDCA9";
const pair = "💩";
const failures = [];
function check(label, condition) {
  if (!condition) failures.push(label);
}

check("literal escape pair", "💩" === pair);
check("binary +", high + low === pair);
check("+ length", (high + low).length === 2);
let compound = high;
compound += low;
check("+=", compound === pair);
check("template", `${high}${low}` === pair);
check("String.prototype.concat", high.concat(low) === pair);
check("Array.prototype.join", [high, low].join("") === pair);
check("split and join", pair.split("").join("") === pair);
check("slice halves", pair.slice(0, 1) + pair.slice(1) === pair);
check("charAt halves", pair.charAt(0) + pair.charAt(1) === pair);
check("String.fromCharCode", String.fromCharCode(0xd83d, 0xdca9) === pair);
check(
  "String.fromCharCode mixed",
  String.fromCharCode(0x61, 0xd83d, 0xdca9, 0x62) === "a" + pair + "b",
);
check("inner seam", ("a" + high) + (low + "b") === "a" + pair + "b");
check("repeat boundary", (low + high).repeat(2) === low + pair + high);
check("repeat boundary length", (low + high).repeat(3).length === 6);
check("padStart filler", low.padStart(2, high) === pair);
check("padStart truncated filler", "x".padStart(2, high + low) === high + "x");
check("padStart repeated filler", low.padStart(4, low + high) === low + pair + low);
check("padStart truncated high meets receiver", low.padStart(3, low + high) === low + pair);
check("padEnd receiver", high.padEnd(2, low) === pair);
check("padEnd repeated filler", "x".padEnd(5, low + high) === "x" + low + pair + high);
check("JSON.parse escaped pair", JSON.parse('"\\ud83d\\udca9"') === pair);
check("JSON.parse escaped halves apart", JSON.parse('"\\ud83d-\\udca9"') === high + "-" + low);
check("JSON.parse escaped key", Object.keys(JSON.parse('{"\\ud83d\\udca9":1}'))[0] === pair);
check("two seams", "x" + high + low + high + low === "x" + pair + pair);
check("codePointAt", (high + low).codePointAt(0) === 0x1f4a9);
check("string iterator", [...(high + low)].length === 1);
check("/u RegExp", /^\u{1F4A9}$/u.test(high + low));
const keyed = {};
keyed[high + low] = 1;
check("property key", keyed[pair] === 1);

// Only a high-then-low seam pairs.
check("low then high stays unpaired", (low + high).length === 2 && low + high !== pair);
check("high then high", (high + high).charCodeAt(1) === 0xd83d);
check("high then BMP", (high + "x").charCodeAt(1) === 0x78);
check("lone halves", String.fromCharCode(0xdca9, 0xd83d) === low + high);
check("low, high, low", (low + high + low).slice(1) === pair);

if (failures.length !== 0) throw failures.join("; ");
true;
