(function workload() {
var keys = []; for (var i = 0; i < 384; ++i) keys.push("group\ud800:" + "x".repeat(1024) + ":" + i);
var left = new Set(keys.slice(0,256)); var right = new Set(keys.slice(128));
var checksum = 0;
for (var round = 0; round < 8; ++round) {
var result = left.symmetricDifference(right); if (result.size !== 256 || !result.has(keys[0]) || result.has(keys[128]) || !result.has(keys[383])) throw new Error("symmetric difference"); checksum += result.size;
}
print(String(checksum));
})();
