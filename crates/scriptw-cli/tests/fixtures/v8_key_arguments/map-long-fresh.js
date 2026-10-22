(function () {
const prefix = "x".repeat(2048);
const keys = []; for (let i = 0; i < 256; ++i) keys.push(prefix + ":" + i);
let sum = 0;
for (let round = 0; round < 8; ++round) {
  const table = new Map();
  for (let i = 0; i < keys.length; ++i) { table.set(keys[i], i + round); }
  sum += table.size + table.get(keys[255]);
}
print(sum);
})();
