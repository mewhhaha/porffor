const foreign = __lilaCreateRealm().global;
const I = foreign.Temporal.Instant;
const D = foreign.Temporal.Duration;
const instantPrototype = I.prototype;
const durationPrototype = D.prototype;
const typeErrorPrototype = foreign.TypeError.prototype;
const rangeErrorPrototype = foreign.RangeError.prototype;
const receiver = new Temporal.Instant(5n);
const foreignReceiver = new I(5n);
const add = instantPrototype.add;
const subtract = instantPrototype.subtract;
const round = instantPrototype.round;
const until = instantPrototype.until;
const since = instantPrototype.since;
foreign.Temporal.Instant = function(){throw 'public Instant';};
foreign.Temporal.Duration = function(){throw 'public Duration';};
foreign.TypeError = function(){throw 'public TypeError';};
foreign.RangeError = function(){throw 'public RangeError';};
I.from = function(){throw 'public from';};
for(const result of [add.call(receiver,{nanoseconds:1}),subtract.call(receiver,{nanoseconds:-1}),round.call(receiver,'nanosecond')]) {
  if(Object.getPrototypeOf(result)!==instantPrototype)throw 'foreign Instant result';
}
for(const result of [until.call(receiver,'1970-01-01T00:00Z'),since.call(receiver,new Temporal.Instant(6n))]) {
  if(Object.getPrototypeOf(result)!==durationPrototype)throw 'foreign Duration result';
}
if(Object.getPrototypeOf(Temporal.Instant.prototype.add.call(foreignReceiver,{nanoseconds:0}))!==Temporal.Instant.prototype)throw 'receiver Realm does not select result';
for(const operation of [()=>add.call({},{}),()=>round.call(receiver),()=>until.call(receiver,1n)]){
 let caught=false;try{operation();}catch(error){if(Object.getPrototypeOf(error)!==typeErrorPrototype)throw 'TypeError Realm';caught=true;}if(!caught)throw 'missing TypeError';
}
for(const operation of [()=>add.call(receiver,{days:1}),()=>round.call(receiver,{smallestUnit:'hour',roundingIncrement:25}),()=>since.call(receiver,receiver,{largestUnit:'day'})]){
 let caught=false;try{operation();}catch(error){if(Object.getPrototypeOf(error)!==rangeErrorPrototype)throw 'RangeError Realm';caught=true;}if(!caught)throw 'missing RangeError';
}
const NewTarget = foreign.Function.bind(null);
if(Object.getPrototypeOf(Reflect.construct(Temporal.Instant,[0n],NewTarget))!==instantPrototype)throw 'Instant NewTarget Realm';
if(Object.getPrototypeOf(Reflect.construct(Temporal.Duration,[],NewTarget))!==durationPrototype)throw 'Duration NewTarget Realm';
if(Object.getPrototypeOf(D.from({nanoseconds:1}))!==durationPrototype)throw 'Duration shared allocator';
print('ok');
