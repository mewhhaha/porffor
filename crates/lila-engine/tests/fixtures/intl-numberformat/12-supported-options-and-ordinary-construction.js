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
check(result === receiver && Object.getPrototypeOf(result) === NF.prototype, "selected legacy construction returns the eligible receiver");
const symbols = Object.getOwnPropertySymbols(receiver);
check(symbols.length === 1, "one legacy fallback symbol installed");
const fallback = Object.getOwnPropertyDescriptor(receiver, symbols[0]);
check(fallback.value !== receiver && Object.getPrototypeOf(fallback.value) === NF.prototype, "fallback holds a fresh ordinary formatter");
check(!fallback.writable && !fallback.enumerable && !fallback.configurable, "fallback descriptor is immutable and hidden");
check(result.format(1) === "1", "format unwraps the legacy receiver");
check(receiver.resolvedOptions().locale === "en-US", "resolvedOptions unwraps the legacy receiver");
caught = undefined;
try { receiver.formatToParts(1); } catch (error) { caught = error; }
check(caught instanceof TypeError, "parts still requires the receiver's own formatter brand");
const ordinaryReceiver = {};
const ordinary = NF.call(ordinaryReceiver, "en-US");
check(ordinary !== ordinaryReceiver && Object.getPrototypeOf(ordinary) === NF.prototype, "ineligible receiver uses ordinary construction");
check(Object.getOwnPropertySymbols(ordinaryReceiver).length === 0, "ordinary receiver has no legacy fallback symbol");
check(ordinary.format(1) === "1", "ordinary result is branded");
caught = undefined;
try { Object.create(NF.prototype).resolvedOptions(); } catch (error) { caught = error; }
check(caught instanceof TypeError, "unchained receiver has no formatter brand or fallback");
print("ok supported options and ordinary construction");
