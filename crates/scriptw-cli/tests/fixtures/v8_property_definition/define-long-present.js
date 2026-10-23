(function () {
const prefix = "x".repeat(2048);
const keys = []; for (let i = 0; i < 256; ++i) keys.push(prefix + ":" + i);
const descriptor = { value: 0, writable: true, enumerable: true, configurable: true };
let sum = 0;
for (let round = 0; round < 8; ++round) {
  const table = Object.create(null);
  for (let i = 0; i < keys.length; ++i) table[keys[i]] = -1;
  for (let i = 0; i < keys.length; ++i) {
    descriptor.value = i + round;
    Object.defineProperty(table, keys[i], descriptor);
  }
  sum += Object.keys(table).length + table[keys[255]];
}
print(sum);
})();
