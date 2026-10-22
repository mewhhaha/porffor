(function () {
const prefix = "x".repeat(8);
const keys = [];
for (let i = 0; i < 64; ++i) keys.push(prefix + ":" + i);
const table = new Set();
for (let i = 0; i < keys.length; ++i) { table.add(keys[i]); }
let sum = 0;
for (let round = 0; round < 128; ++round) {
  for (let i = 0; i < keys.length; ++i) sum += (table.has(keys[i]) ? 1 : 0);
}
if (sum !== 8192) throw new Error("lookup checksum");
print(sum);
})();
