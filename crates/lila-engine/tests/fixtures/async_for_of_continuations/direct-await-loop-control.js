async function run() {
  var events = [];
  try {
    for (const value of [1, 2]) {
      events.push("before" + value);
      await 0;
      events.push("middle" + value);
      await 0;
      events.push("after" + value);
    }
  } catch (error) { throw error; }
  if (events.join(",") !== "before1,middle1,after1,before2,middle2,after2") throw events.join(",");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
