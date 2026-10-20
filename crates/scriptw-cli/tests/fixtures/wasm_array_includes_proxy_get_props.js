var calls = "";
var p = new Proxy({}, {
  get: function(_, key) {
    calls = calls + key + ",";
    if (key === "length") return 4;
    return key * 10;
  }
});

var miss = [].includes.call(p, 42);
var missCalls = calls === "length,0,1,2,3,";

calls = "";
var hit = [].includes.call(p, 10);
var hitCalls = calls === "length,0,1,";

miss === false && hit === true && missCalls && hitCalls && p[1] === 10;
