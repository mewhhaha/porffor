function Ordinary() {
  return eval('() => new.target');
}
function Alternate() {}
const ordinaryCall = Ordinary();
const ordinaryConstruct = new Ordinary();
const alternateConstruct = Reflect.construct(Ordinary, [], Alternate);
if (ordinaryCall() !== undefined || ordinaryConstruct() !== Ordinary || alternateConstruct() !== Alternate) throw 'ordinary semantic newTarget';

function* syncGenerator() {
  const read = () => new.target;
  const prepared = eval('() => new.target');
  if (new.target !== undefined || read() !== undefined || prepared() !== undefined) throw 'generator entry newTarget';
  yield read;
  if (new.target !== undefined || prepared() !== undefined) throw 'generator resume newTarget';
  return prepared;
}
const sync = syncGenerator();
const syncFirst = sync.next();
const syncLast = sync.next();
if (syncFirst.done || syncFirst.value() !== undefined || !syncLast.done || syncLast.value() !== undefined) throw 'escaped generator newTarget';

async function asynchronous() {
  const read = eval('() => new.target');
  if (new.target !== undefined || read() !== undefined) throw 'async entry newTarget';
  await 0;
  if (new.target !== undefined) throw 'async resume newTarget';
  return read;
}
async function* asyncGenerator() {
  const lexical = () => new.target;
  const prepared = eval('() => new.target');
  if (new.target !== undefined || lexical() !== undefined || prepared() !== undefined) throw 'async generator entry newTarget';
  yield lexical;
  await 0;
  if (new.target !== undefined || prepared() !== undefined) throw 'async generator resume newTarget';
  yield prepared;
  return new.target;
}
function AsyncArrowOwner() {
  this.read = async () => {
    await 0;
    return new.target;
  };
}
async function observe() {
  const asyncRead = await asynchronous();
  if (asyncRead() !== undefined) throw 'escaped async newTarget';
  const iterator = asyncGenerator();
  const first = await iterator.next();
  const second = await iterator.next();
  const last = await iterator.next();
  if (first.done || first.value() !== undefined || second.done || second.value() !== undefined || !last.done || last.value !== undefined) throw 'async generator activation leaked as newTarget';
  if (first.value() !== undefined || second.value() !== undefined) throw 'completed async generator escaped capture';
  const direct = new AsyncArrowOwner();
  const alternate = Reflect.construct(AsyncArrowOwner, [], Alternate);
  if ((await direct.read()) !== AsyncArrowOwner || (await alternate.read()) !== Alternate) throw 'async arrow lexical newTarget';
  print('callable-entry-roles:ok');
}
observe().catch(error => print('unexpected:' + error));
262;
