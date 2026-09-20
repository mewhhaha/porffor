async function run() {
  var closures = [], total = 0;
  for (let index of [1, 2]) {
    let local = index * 10;
    const body = () => local;
    try { await Promise.reject(index); }
    catch (error) {
      const read = () => index + ":" + local + ":" + error;
      closures.push(read);
      await 0;
      if (read() !== index + ":" + local + ":" + error) throw "resumed catch chain";
      local++;
    } finally { await 0; }
    total += body();
  }
  if (closures[0]() !== "1:11:1" || closures[1]() !== "2:21:2" || total !== 32) throw "iteration/catch capture";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
