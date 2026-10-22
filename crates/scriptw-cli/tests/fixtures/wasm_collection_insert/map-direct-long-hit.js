(function () {
const prefix = "x".repeat(2048);
const keys = []; for (let i = 0; i < 128; ++i) keys.push(prefix + ":" + i);
let sum = 0;
for (let round = 0; round < 16; ++round) {
  const table = new Map();
  for (let i = 0; i < keys.length; ++i) table.set(keys[i], i);
  for (let i = 0; i < keys.length; ++i) {
    if (table.getOrInsert(keys[i], i + round) !== i) throw new Error("insertion value");
  }
  sum += table.size + table.get(keys[127]);
}
if (sum !== 4080) throw new Error("insertion checksum");
print(sum);
})();
