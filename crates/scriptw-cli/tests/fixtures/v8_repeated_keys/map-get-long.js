(function () {
const prefix = "x".repeat(2048);
const keys = [];
for (let i = 0; i < 64; ++i) keys.push(prefix + ":" + i);
const table = new Map();
for (let i = 0; i < keys.length; ++i) { table.set(keys[i], i); }
let sum = 0;
for (let round = 0; round < 128; ++round) {
  for (let i = 0; i < keys.length; ++i) sum += table.get(keys[i]);
}
if (sum !== 258048) throw new Error("lookup checksum");
print(sum);
})();
