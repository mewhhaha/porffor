function run() {
  var count = 37;
  var preserved = 91;
  var trace = [];
  var read = function () { return count; };
  function initialize() { count = 99; trace.push('initialize'); return 0; }
  var completion = eval("for (var count = initialize(), preserved, next = (trace.push('next:' + read()), count + 3); count < next; count++) { trace.push('body:' + read()); }");
  if (read() !== 3 || next !== 3 || preserved !== 91
      || trace.join(',') !== 'initialize,next:0,body:0,body:1,body:2'
      || completion !== 5) throw new Error('caller publication or initializer order');
}
run();
var count = 37;
var seen;
eval("for (var count = 0;;) { seen = count; break; }");
if (count !== 0 || seen !== 0) throw new Error('global caller publication');
print('ok');
true;
