(function workload() {
var keys = []; for (var i = 0; i < 384; ++i) keys.push("group\ud800:" + "x".repeat(1024) + ":" + i);
var left = new Set(keys.slice(0,256)); var right = new Set(keys.slice(128,256));
var checksum = 0;
for (var round = 0; round < 8; ++round) {
var result = left.intersection(right); if (result.size !== 128 || !result.has(keys[128])) throw new Error("intersection"); checksum += result.size;
}
print(String(checksum));
})();
