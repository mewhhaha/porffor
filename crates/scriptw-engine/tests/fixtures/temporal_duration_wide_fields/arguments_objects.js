function bag() { arguments.nanoseconds=17280000000000000000000; return arguments; }
const fields=bag(1,2);
const duration=Temporal.Duration.from(fields);
if (duration.nanoseconds !== 17280000000000000000000 || new Temporal.Duration().with(fields).nanoseconds !== duration.nanoseconds) throw 'Arguments field bag';
function options() { arguments.largestUnit='nanosecond'; arguments.smallestUnit='nanosecond'; return arguments; }
if (duration.round(options()).nanoseconds !== duration.nanoseconds) throw 'Arguments options';
print('ok');
