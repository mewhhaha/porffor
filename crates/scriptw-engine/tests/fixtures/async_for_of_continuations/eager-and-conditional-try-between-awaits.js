async function run() {
  var events = [], marker = {};
  for (const value of [1, 2]) {
    try { events.push("eager" + value); } catch (error) { throw "eager catch"; }
    await 0;
    try {
      if (value === 1) { await Promise.reject(marker); }
      else { await 0; events.push("fulfilled" + value); }
    } catch (error) {
      if (error !== marker) throw "conditional rejection";
      await 0;
      events.push("caught" + value);
    }
    await 0;
    events.push("after" + value);
  }
  if (events.join(",") !== "eager1,caught1,after1,eager2,fulfilled2,after2") throw events.join(",");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
