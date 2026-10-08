// Each lexical iteration and each nested body/catch has its own cells.
// Resumption must reattach those cells without sharing another generator's owner.
function same(actual, expected, label) {
  if (actual !== expected) throw label + ": " + actual + " !== " + expected;
}
function read(readers) {
  return [readers[0](), readers[1](), readers[2](), readers[3]()].join(",");
}
function nextReaders(consumer, label) {
  var result = consumer.next();
  same(result.done, false, label + " is suspended");
  return result.value;
}
function* walk(values) {
  for (let iteration of values) {
    let body = iteration * 10;
    let readBody = () => body;
    try {
      throw iteration;
    } catch (error) {
      let caught = error * 100;
      let readers = [() => iteration, readBody, () => error, () => caught];
      yield readers;
      iteration += 10;
      body += 1;
      caught += 1;
      yield readers;
    }
  }
  return "finished";
}

var first = walk([1, 2]);
var second = walk([3, 4]);
var firstInitial = nextReaders(first, "first owner");
var secondInitial = nextReaders(second, "second owner");
same(read(firstInitial), "1,10,1,100", "first initial cells");
same(read(secondInitial), "3,30,3,300", "second initial cells");
same(nextReaders(first, "first resume"), firstInitial, "same readers across resume");
same(read(firstInitial), "11,11,1,101", "first mutated cells");
same(read(secondInitial), "3,30,3,300", "second owner remains suspended");
same(nextReaders(second, "second resume"), secondInitial, "second readers retain identity");
same(read(secondInitial), "13,31,3,301", "second mutated cells");

var firstLater = nextReaders(first, "first next iteration");
var secondLater = nextReaders(second, "second next iteration");
same(firstLater === firstInitial, false, "fresh first iteration readers");
same(secondLater === secondInitial, false, "fresh second iteration readers");
same(read(firstLater), "2,20,2,200", "fresh first iteration cells");
same(read(secondLater), "4,40,4,400", "fresh second iteration cells");
same(read(firstInitial), "11,11,1,101", "earlier first cells survive advancement");
same(read(secondInitial), "13,31,3,301", "earlier second cells survive advancement");
same(nextReaders(first, "first later resume"), firstLater, "later first readers identity");
same(nextReaders(second, "second later resume"), secondLater, "later second readers identity");
same(read(firstLater), "12,21,2,201", "later first mutated cells");
same(read(secondLater), "14,41,4,401", "later second mutated cells");
var firstDone = first.next();
var secondDone = second.next();
same(firstDone.done, true, "first loop exhausted");
same(firstDone.value, "finished", "first completion");
same(secondDone.done, true, "second loop exhausted");
same(secondDone.value, "finished", "second completion");
same(read(firstInitial), "11,11,1,101", "first capture after completion");
same(read(secondLater), "14,41,4,401", "second capture after completion");
print("ok");
262;
