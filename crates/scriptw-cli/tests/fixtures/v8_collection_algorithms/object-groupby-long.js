(function workload() {
var keys = []; for (var i = 0; i < 384; ++i) keys.push("group\ud800:" + "x".repeat(1024) + ":" + i);
var inputs = keys.slice(0,256).concat(keys.slice(0,256));
var checksum = 0;
for (var round = 0; round < 8; ++round) {
var groups = Object.groupBy(inputs, function (key) { return key; }); checksum += Reflect.ownKeys(groups).length + groups[keys[round]].length;
}
print(String(checksum));
})();
