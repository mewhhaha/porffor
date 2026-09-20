function check(value, message) { if (!value) throw new Error(message); }
const NF = Intl.NumberFormat;
for (const options of [undefined, true, 3, "lookup"]) {
  const result = NF.supportedLocalesOf(["en-US"], options);
  check(result.length === 1 && result[0] === "en-US", "supportedLocalesOf primitive options");
}
let caught;
try { NF.supportedLocalesOf(["en-US"], null); } catch (error) { caught = error; }
check(caught instanceof TypeError, "supportedLocalesOf null options");
check(new NF("en-US", true).format(1) === "1", "constructor also coerces non-null primitive options");
caught = undefined;
try { new NF("en-US", null); } catch (error) { caught = error; }
check(caught instanceof TypeError, "constructor null options rejected");
const receiver = Object.create(NF.prototype);
const result = NF.call(receiver, "en-US");
check(result !== receiver && Object.getPrototypeOf(result) === NF.prototype, "selected ordinary construction policy");
check(Object.getOwnPropertySymbols(receiver).length === 0, "no legacy fallback symbol installed");
check(typeof result.format(1) === "string", "ordinary result is branded");
caught = undefined;
try { receiver.resolvedOptions(); } catch (error) { caught = error; }
check(caught instanceof TypeError, "receiver was not initialized through legacy chaining");
print("ok supported options and ordinary construction");
