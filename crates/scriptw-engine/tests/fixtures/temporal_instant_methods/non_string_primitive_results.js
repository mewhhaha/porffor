const receiver = new Temporal.Instant(0n);
const from = Temporal.Instant.from;
for (const primitive of [undefined, null, false, 1, 1n, Symbol('instant')]) {
  for (const convert of [input => Temporal.Instant.from(input), input => from(input), input => receiver.until(input), input => receiver.since(input)]) {
    let calls = 0;
    let caught;
    try {
      convert({[Symbol.toPrimitive](hint) {
        if (hint !== 'string') throw 'primitive hint';
        calls++;
        return primitive;
      }});
    } catch (error) { caught = error; }
    if (!(caught instanceof TypeError) || calls !== 1) throw 'non-string primitive must throw TypeError';
  }
}
for (const value of [{toString(){return 1;}}, {toString(){return {};}, valueOf(){return 1;}}]) {
  let caught;
  try { Temporal.Instant.from(value); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw 'ordinary primitive type';
}
print('ok');
