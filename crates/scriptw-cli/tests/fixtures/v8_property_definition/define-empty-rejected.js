(function () {
const prefix = "x".repeat(2048);
const keys = []; for (let i = 0; i < 256; ++i) keys.push(prefix + ":" + i);
const descriptor = { value: 0, writable: true, enumerable: true, configurable: true };
let sum = 0;
for (let round = 0; round < 8; ++round) {
  const table = Object.create(null);
  Object.preventExtensions(table);
  for (let i = 0; i < keys.length; ++i) {
    descriptor.value = i + round;
    if (Reflect.defineProperty(table, keys[i], descriptor)) throw new Error("closed definition succeeded");
    sum += 1;
  }
  if (Reflect.ownKeys(table).length !== 0) throw new Error("closed table changed");
}
print(sum);
})();
