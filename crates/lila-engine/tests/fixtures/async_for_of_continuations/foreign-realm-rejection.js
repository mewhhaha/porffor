var foreign = __lilaCreateRealm().global;
var marker = new foreign.RangeError("foreign marker");
async function run() {
  var catches = 0;
  for (const value of [1, 2]) {
    try { await Promise.reject(marker); }
    catch (error) {
      await value;
      if (error !== marker || error.constructor !== foreign.RangeError) throw "rejection Realm or identity";
      catches++;
    } finally { await 0; }
  }
  if (catches !== 2) throw "missing rejection";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
