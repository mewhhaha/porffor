const names = ['days','hours','microseconds','milliseconds','minutes','months','nanoseconds','seconds','weeks','years'];
const marker = {};
let trace = '';
const bag = {};
for (const name of names) {
  Object.defineProperty(bag, name, {get() {
    trace += name + ':get,';
    return {valueOf() { trace += name + ':number,'; return name === 'days' ? 1e100 : 0; }};
  }});
}
let caught;
try { Temporal.Duration.from(bag); } catch (error) { caught = error; }
if (!(caught instanceof RangeError)) throw 'late range error';
let expected = '';
for (const name of names) expected += name + ':get,' + name + ':number,';
if (trace !== expected) throw 'all property reads before range validation';
caught = undefined;
try { Temporal.Duration.from({days:1e100, get hours() { throw marker; }}); } catch (error) { caught = error; }
if (caught !== marker) throw 'range error hid later abrupt completion';
caught = undefined;
try { new Temporal.Duration(4294967296,0,0,0,0,0,0,0,0,{valueOf() { throw marker; }}); } catch (error) { caught = error; }
if (caught !== marker) throw 'constructor range check preceded coercion';
caught = undefined;
try { new Temporal.Duration().with({microseconds:1e100, get milliseconds() { throw marker; }}); } catch (error) { caught = error; }
if (caught !== marker) throw 'partial record range check preceded coercion';
let late = false;
caught = undefined;
try { Temporal.Duration.from({microseconds:1.5, get milliseconds() { late=true; throw marker; }}); } catch (error) { caught = error; }
if (!(caught instanceof RangeError) || late) throw 'integral rejection must be immediate';
print('ok');
