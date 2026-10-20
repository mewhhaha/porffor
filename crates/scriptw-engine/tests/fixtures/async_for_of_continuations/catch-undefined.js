async function run() {
  let caught = 0;
  for (const index of [1, 2]) {
    try { await Promise.reject(undefined); }
    catch (error) { if (error !== undefined) throw "replaced undefined"; caught += index; }
  }
  if (caught !== 3) throw "missing catch or repeated iteration";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
