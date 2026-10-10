const intrinsicTypePrototype = TypeError.prototype;
TypeError = undefined;
const strictMode = (function() { return this === undefined; })();
const whole = {marker: 71};
function check(ok) { if (!ok) throw 'parameter named self policy'; }
function checkType(error) { check(Object.getPrototypeOf(error) === intrinsicTypePrototype); }
async function checkNamedSelf(original, read, write) {
  check(read() === original);
  let calls = 0;
  function rhs() { calls++; gc(); return whole; }
  let caught = false;
  let result;
  try { result = await write(rhs); } catch (error) { checkType(error); caught = true; }
  check(calls === 1 && caught === strictMode && read() === original);
  check(strictMode || result === whole);
}
const variableShadow = async function self(read = () => self, write = async rhs => self = await rhs()) {
  var self = 0;
  const bodyRead = () => self;
  check((self = await whole) === whole && bodyRead() === whole);
  await checkNamedSelf(variableShadow, read, write);
  check(self === whole);
};
const lexicalShadow = async function self(read = () => self, write = async rhs => self = await rhs()) {
  let self = 0;
  const bodyRead = () => self;
  check((self = await whole) === whole && bodyRead() === whole);
  await checkNamedSelf(lexicalShadow, read, write);
  check(self === whole);
};
const constantShadow = async function self(read = () => self, write = async rhs => self = await rhs()) {
  const self = 0;
  let calls = 0;
  function rhs() { calls++; return whole; }
  let caught = false;
  try { self = await rhs(); } catch (error) { checkType(error); caught = true; }
  check(caught && calls === 1 && self === 0);
  await checkNamedSelf(constantShadow, read, write);
  check(self === 0);
};
const parameterShadow = async function self(self = 0, read = () => self, write = async rhs => self = await rhs()) {
  var self = 1;
  const bodyRead = () => self;
  check(read() === 0 && bodyRead() === 1);
  let calls = 0;
  function rhs() { calls++; gc(); return whole; }
  check((await write(rhs)) === whole);
  check(calls === 1 && read() === whole && bodyRead() === 1);
  check((self = await 2) === 2 && bodyRead() === 2 && read() === whole);
};
const simpleParameters = async function self(parameter) {
  await checkNamedSelf(simpleParameters, () => self, async rhs => self = await rhs());
};
const simpleParameterShadow = async function self(self) {
  check((self = await whole) === whole && self === whole);
};
async function run() {
  await variableShadow();
  await lexicalShadow();
  await constantShadow();
  await parameterShadow();
  await simpleParameters(0);
  await simpleParameterShadow(0);
  print('parameter-named-self-policies:ok');
}
run().catch(error => print('unexpected:' + error));
