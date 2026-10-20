function same(a,b){if(!Object.is(a,b))throw new Error(a+' != '+b);}
// These exact calendar ties bracket whole 400-year Gregorian cycles. The
// half-even bucket ordinal is 1 even though the raw year count is even.
var base='1970-01-01';var opts={largestUnit:'years',smallestUnit:'years',roundingIncrement:100000,roundingMode:'halfEven',relativeTo:base};
same(Temporal.Duration.from({years:150000}).round(opts).years,200000);
same(Temporal.Duration.from({years:150000,nanoseconds:1}).round(opts).years,200000);
same(Temporal.Duration.from({years:149999,days:364,hours:23,minutes:59,seconds:59,milliseconds:999,microseconds:999,nanoseconds:999}).round(opts).years,100000);
same(Temporal.Duration.from({years:-150000}).round(opts).years,-200000);
same(Temporal.Duration.from({months:1500000}).round({largestUnit:'months',smallestUnit:'months',roundingIncrement:1000000,roundingMode:'halfEven',relativeTo:base}).months,2000000);
same(Temporal.Duration.from({days:50000000}).round({largestUnit:'days',smallestUnit:'days',roundingIncrement:100000000,roundingMode:'halfEven',relativeTo:base}).days,0);
// Independent Python Fraction/BigInt-derived rational rounded once. The old
// chained year f64 projection differed by one ULP for this exact duration.
var d=Temporal.Duration.from({years:150000,seconds:28233600,nanoseconds:242811023});
same(d.total({unit:'years',relativeTo:base}),150000.89528159067);
same(d.total({unit:'years',relativeTo:'1970-01-01T00:00[UTC]'}),150000.89528159067);
var negative=Temporal.Duration.from({years:-150000,seconds:-28233600,nanoseconds:-242811023});
same(negative.total({unit:'years',relativeTo:base}),-150000.89528159067);
same(negative.total({unit:'years',relativeTo:'1970-01-01T00:00[UTC]'}),-150000.89528159067);
// Independent legal near-limit week windows: the actual required endpoints
// remain within the date range, and negative progress preserves its sign.
same(Temporal.Duration.from({days:1}).total({unit:'weeks',relativeTo:'+275760-09-06'}),1/7);
same(Temporal.Duration.from({days:-1}).total({unit:'weeks',relativeTo:'-271821-04-27'}),-1/7);
same(Temporal.Duration.from({days:1}).round({smallestUnit:'weeks',roundingMode:'trunc',relativeTo:'+275760-09-06'}).weeks,0);
same(Temporal.Duration.from({days:-1}).round({smallestUnit:'weeks',roundingMode:'trunc',relativeTo:'-271821-04-27'}).weeks,0);
262;
