var object = { method: function (value) { return value; } };
async function run() { print((object?.method)(await 7)); }
run().catch(error => print('error:' + error)); print('called');
