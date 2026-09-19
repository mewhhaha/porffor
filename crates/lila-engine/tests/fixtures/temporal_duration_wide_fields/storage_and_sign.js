const names = ['years','months','weeks','days','hours','minutes','seconds','milliseconds','microseconds','nanoseconds'];
const values = [1,2,3,4,5,6,7,8,9,10];
const ordinary = new Temporal.Duration(1,2,3,4,5,6,7,8,9,10);
for (let i=0;i<names.length;i++) {
  if (ordinary[names[i]] !== values[i] || ordinary.negated()[names[i]] !== -values[i]) throw 'ordinary field projection';
}
for (const [name, value] of [['nanoseconds',17280000000000000000000],['microseconds',17280000000000000000]]) {
  const bag = {}; bag[name] = value;
  const original = Temporal.Duration.from(bag);
  const copy = Temporal.Duration.from(original);
  const replaced = new Temporal.Duration().with(bag);
  const negative = original.negated();
  if (original[name] !== value || copy[name] !== value || replaced[name] !== value) throw 'wide stored Number';
  if (negative[name] !== -value || negative.abs()[name] !== value) throw 'wide sign transform';
  if (original.sign !== 1 || negative.sign !== -1 || original.blank) throw 'wide sign accessor';
  if (original.toString() !== 'PT17280000000000S' || negative.toJSON() !== '-PT17280000000000S') throw 'wide formatting';
}
const nano = new Temporal.Duration(0,0,0,0,0,0,0,0,0,17280000000000000000000);
const micro = new Temporal.Duration(0,0,0,0,0,0,0,0,17280000000000000000);
if (nano.nanoseconds !== 17280000000000000000000 || micro.microseconds !== 17280000000000000000) throw 'constructor width';
const zero = new Temporal.Duration(-0,-0,-0,-0,-0,-0,-0,-0,-0,-0).negated();
for (const name of names) if (!Object.is(zero[name], 0)) throw 'canonical zero';
if (zero.sign !== 0 || !zero.blank) throw 'blank zero';
print('ok');
