async function run() {
  var events = [], marker = {}, read;
  try { await Promise.reject(marker); }
  catch (error) {
    read = () => error;
    await 0;
    if (read() !== marker) throw "catch capture";
    events.push("catch");
  } finally { await 0; events.push("finally"); }
  if (read() !== marker || events.join(",") !== "catch,finally") throw "standalone continuation";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
