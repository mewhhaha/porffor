(function workload() {
var keys = []; for (var i = 0; i < 384; ++i) keys.push("group\ud800:" + "x".repeat(1024) + ":" + i);

var checksum = 0;
for (var round = 0; round < 8; ++round) {
var set = new Set(); for (var i = 0; i < 256; ++i) set.add(keys[i]); for (var i = 0; i < 256; ++i) { if (!set.delete(keys[i])) throw new Error("set delete"); ++checksum; } if (set.size !== 0) throw new Error("set size");
}
print(String(checksum));
})();
