function check(value, message) { if (!value) throw new Error(message); }
const nf = new Intl.NumberFormat("en-US", { useGrouping: false });
for (const method of ["formatRange", "formatRangeToParts"]) {
  const events = [];
  const marker = {};
  const start = { [Symbol.toPrimitive](hint) { events.push("start:" + hint); return NaN; } };
  const end = { [Symbol.toPrimitive](hint) { events.push("end:" + hint); throw marker; } };
  let caught;
  try { nf[method](start, end); } catch (error) { caught = error; }
  check(caught === marker, method + " second throw precedes NaN rejection");
  check(events.join(",") === "start:number,end:number", method + " conversion order");
  events.length = 0;
  caught = undefined;
  try { nf[method](start, undefined); } catch (error) { caught = error; }
  check(caught instanceof TypeError && events.length === 0, method + " undefined before conversion");
  caught = undefined;
  try { nf[method](NaN, Symbol()); } catch (error) { caught = error; }
  check(caught instanceof TypeError, method + " Symbol before NaN rejection");
  events.length = 0;
  const negativeZero = { [Symbol.toPrimitive](hint) { events.push("zero:" + hint); return -0; } };
  caught = undefined;
  try { nf[method](negativeZero, end); } catch (error) { caught = error; }
  check(caught === marker && events.join(",") === "zero:number,end:number", method + " negative zero observes end");
  events.length = 0;
  caught = undefined;
  try { nf[method].call({}, start, end); } catch (error) { caught = error; }
  check(caught instanceof TypeError && events.length === 0, method + " brand before endpoints");
}
print("ok range abrupt order");
