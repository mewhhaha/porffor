function same(a,b,message){if(a!==b)throw new Error(message);}
function check(value,message){if(!value)throw new Error(message);}
function* shared(view){for(var key in view){yield ()=>key;}}
var varIterator=shared({a:1,b:1});
var firstVar=varIterator.next().value;
same(firstVar(),'a','original shared var first key');
var secondVar=varIterator.next().value;
gc();
same(firstVar(),'b','var remains shared across iterations');
same(secondVar(),'b','same var storage');
same(varIterator.next().done,true,'var cursor completes');
function* assigned(view){let key;for(key in view){yield key;}}
var assignment=assigned({a:1,b:1});
same(assignment.next().value,'a','bare Identifier original PutValue');
gc();
same(assignment.next().value,'b','bare Identifier next Reference');
same(assignment.next().done,true,'bare Identifier completes');
var target={};
var bases=0,keys=0,coercions=0;
function base(){bases++;return target;}
function property(){keys++;return {[Symbol.toPrimitive](){coercions++;return 'slot';}};}
function* properties(){for(base()[property()] in {a:1,b:1}){yield target.slot;}}
var propertyIterator=properties();
same(propertyIterator.next().value,'a','computed member original PutValue');
check(bases===1&&keys===1&&coercions===1,'head Reference evaluated once before first body');
gc();
same(propertyIterator.next().value,'b','second member PutValue');
check(bases===2&&keys===2&&coercions===2,'body resume does not replay previous Reference');
same(propertyIterator.next().done,true,'member cursor completes');
same(bases,2,'no head after exhausted cursor');
class Holder{#value;*values(view){for(this.#value in view){yield this.#value;}}get value(){return this.#value;}}
var holder=new Holder();
var privateIterator=holder.values({a:1,b:1});
same(privateIterator.next().value,'a','original private brand/write owner');
gc();
same(privateIterator.next().value,'b','private Reference repeated only for next key');
same(privateIterator.next().done,true,'private cursor completes');
same(holder.value,'b','same actual private field');
var readers=[];
function* lexical(){for(let [first,...rest] in {ab:1,cd:1}){readers.push(()=>first+rest.join(''));yield first;}}
var lexicalIterator=lexical();
same(lexicalIterator.next().value,'a','eager Array binding initialization');
same(lexicalIterator.next().value,'c','new Array pattern iteration');
gc();
same(readers[0](),'ab','captured first original scoped pattern cells');
same(readers[1](),'cd','captured second original scoped pattern cells');
same(lexicalIterator.next().done,true,'Array binding cursor completes');
function* objectPattern(){for(const {length:size} in {ab:1,cde:1})yield size;}
var objectIterator=objectPattern();
same(objectIterator.next().value,2,'raw String Object pattern GetV');
same(objectIterator.next().value,3,'same original Object pattern owner');
same(objectIterator.next().done,true,'Object binding cursor completes');
var originalIterator=String.prototype[Symbol.iterator];
var closes=0;
String.prototype[Symbol.iterator]=function(){var iterator=originalIterator.call(this);return {next(){return iterator.next();},return(){closes++;return {done:true};}};};
function* destructured(){for([target.first] in {ab:1,cd:1})yield target.first;}
var destructuring=destructured();
same(destructuring.next().value,'a','eager assignment pattern original Put');
same(closes,1,'original IteratorClose finishes head before body suspension');
gc();
same(destructuring.next().value,'c','next assignment pattern');
same(closes,2,'one close per original eager head');
same(destructuring.next().done,true,'assignment pattern cursor completes');
var closeToken={close:true};closeToken.self=closeToken;
String.prototype[Symbol.iterator]=function(){var iterator=originalIterator.call(this);return {next(){return iterator.next();},return(){throw closeToken;}};};
var closeError;
try{destructured().next();}catch(error){closeError=error;}
same(closeError,closeToken,'whole original IteratorClose Throw precedes body');
String.prototype[Symbol.iterator]=originalIterator;
var frozen='outer';
function* immutable(){const frozen='constant';for(frozen in {a:1})yield 'body';}
var immutableError;
try{immutable().next();}catch(error){immutableError=error;}
check(immutableError instanceof TypeError,'original immutable PutValue throws');
same(frozen,'outer','outer binding unchanged');
var strict=(function(){return this===undefined;})();
function* absent(){for(forInMissingName in {a:1})yield 'body';}
if(strict){
  var missingError;
  try{absent().next();}catch(error){missingError=error;}
  check(missingError instanceof ReferenceError,'strict unresolvable Reference throws');
}else{
  var sloppy=absent();
  same(sloppy.next().value,'body','sloppy unresolvable Reference creates global');
  same(forInMissingName,'a','original global PutValue');
  same(sloppy.next().done,true,'sloppy cursor completes');
  delete globalThis.forInMissingName;
}
function* tdz(){for(let view in yield ()=>view)yield view;}
var head=tdz();
var readHead=head.next().value;
gc();
var tdzError;
try{readHead();}catch(error){tdzError=error;}
check(tdzError instanceof ReferenceError,'captured original head TDZ before target completes');
same(head.next({a:1}).value,'a','head TDZ left before original iteration creation');
var tdzAfter;
try{readHead();}catch(error){tdzAfter=error;}
check(tdzAfter instanceof ReferenceError,'escaped head TDZ is distinct from iteration binding');
same(head.next().done,true,'TDZ cursor completes');
print('generator-forin-assignments:ok');
