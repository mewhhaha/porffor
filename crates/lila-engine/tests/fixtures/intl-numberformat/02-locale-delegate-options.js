function check(value, message) { if (!value) throw new Error(message); }
for (const value of [12.5, 12n]) {
  let reads = 0;
  const options = { get minimumFractionDigits() { reads++; return 2; }, maximumFractionDigits: 2, useGrouping: false };
  const text = value.toLocaleString("en-US", options);
  check(text === (typeof value === "bigint" ? "12.00" : "12.50"), "locale digits " + text);
  check(reads === 1, "option read exactly once");
  let error;
  try { value.toLocaleString(null); } catch (caught) { error = caught; }
  check(error instanceof TypeError, "null locales must throw");
}
const original = Intl.NumberFormat;
Object.defineProperty(Intl, "NumberFormat", { configurable: true, get() { throw new Error("mutable namespace lookup"); } });
check((1234).toLocaleString("en-US") === "1,234", "intrinsic Number route");
check((1234n).toLocaleString("en-US") === "1,234", "intrinsic BigInt route");
Object.defineProperty(Intl, "NumberFormat", { value: original, writable: true, configurable: true });
print("ok locale delegates");
