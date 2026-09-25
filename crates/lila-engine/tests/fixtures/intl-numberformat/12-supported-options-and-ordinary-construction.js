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
check(result === receiver, "normative-optional constructor mode chains onto an inheriting receiver");
const symbols = Object.getOwnPropertySymbols(receiver);
check(symbols.length === 1 && symbols[0].description === "IntlLegacyConstructedSymbol", "fallback symbol installed");
const chained = receiver[symbols[0]];
check(chained !== receiver && Object.getPrototypeOf(chained) === NF.prototype, "chained formatter is ordinary");
check(typeof chained.format(1) === "string" && receiver.format(1) === chained.format(1), "format unwraps the receiver");
check(receiver.resolvedOptions().locale === "en-US", "resolvedOptions unwraps the receiver");
caught = undefined;
try { NF.prototype.formatToParts.call(receiver, 1); } catch (error) { caught = error; }
check(caught instanceof TypeError, "formatToParts requires the internal slot itself");
const plain = NF.call({}, "en-US");
check(Object.getPrototypeOf(plain) === NF.prototype && typeof plain.format(1) === "string", "unrelated receiver gets a new formatter");
print("ok supported options and ordinary construction");
