(function workload() {
var keys = []; for (var i = 0; i < 384; ++i) keys.push("group\ud800:" + "x".repeat(1024) + ":" + i);

var checksum = 0;
for (var round = 0; round < 8; ++round) {
var map = new Map(); for (var i = 0; i < 256; ++i) map.set(keys[i],i); for (var i = 0; i < 256; ++i) { if (!map.delete(keys[i])) throw new Error("map delete"); ++checksum; } if (map.size !== 0) throw new Error("map size");
}
print(String(checksum));
})();
