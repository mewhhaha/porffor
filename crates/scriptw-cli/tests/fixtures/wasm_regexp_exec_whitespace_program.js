const whitespace = "\u0009\u000A\u000B\u000C\u000D\u0020\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A\u2028\u2029\u202F\u205F\u3000\uFEFF";
const everyMemberMatch = /\s{25}/.exec(whitespace);
const everyMember = everyMemberMatch !== null && everyMemberMatch[0] === whitespace && everyMemberMatch.index === 0;
const noFalsePositives = /\s/.exec("\u180E") === null && /\s/.exec("\u200B") === null && /\s/.exec("q") === null;

const quantified = /([Nn]?ever|([Nn]othing\s{1,}))more/g;
const source = "Nevermore Nothing\tmore nothing\u2003more";
const first = quantified.exec(source);
const second = quantified.exec(source);
const third = quantified.exec(source);
const exhausted = quantified.exec(source);
const captures = first !== null && first[0] === "Nevermore" && first[1] === "Never" && first[2] === undefined
  && second !== null && second[0] === "Nothing\tmore" && second[1] === "Nothing\t" && second[2] === "Nothing\t"
  && third !== null && third[0] === "nothing\u2003more" && third[1] === "nothing\u2003" && third[2] === "nothing\u2003";

everyMember && noFalsePositives && captures && exhausted === null && quantified.lastIndex === 0;
