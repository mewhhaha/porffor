function fields(d,days,hours,months) { if(d.days!==days || d.hours!==hours || d.months!==(months||0) || d.minutes!==0 || d.seconds!==0 || d.nanoseconds!==0) throw new Error(d.toString()); }
var spring='2024-03-09T12:00[America/New_York]';var fall='2024-11-02T12:00[America/New_York]';
fields(Temporal.Duration.from({hours:24}).round({largestUnit:'days',smallestUnit:'hours',relativeTo:spring}),1,1);
fields(Temporal.Duration.from({days:1}).round({largestUnit:'hours',relativeTo:spring}),0,23);
fields(Temporal.Duration.from({hours:22,minutes:59}).round({largestUnit:'days',smallestUnit:'hours',roundingMode:'ceil',relativeTo:spring}),1,0);
fields(Temporal.Duration.from({hours:24,minutes:59}).round({largestUnit:'days',smallestUnit:'hours',roundingMode:'ceil',relativeTo:fall}),1,0);
fields(Temporal.Duration.from({hours:12}).round({smallestUnit:'days',relativeTo:spring}),1,0);
fields(Temporal.Duration.from({hours:12}).round({smallestUnit:'days',relativeTo:fall}),0,0);
fields(Temporal.Duration.from({hours:12,minutes:30}).round({smallestUnit:'days',roundingMode:'halfEven',relativeTo:fall}),0,0);
fields(Temporal.Duration.from({hours:12,minutes:30}).round({smallestUnit:'days',roundingMode:'halfExpand',relativeTo:fall}),1,0);
fields(Temporal.Duration.from({hours:11,minutes:45}).round({smallestUnit:'days',relativeTo:'2020-10-03T12:00[Australia/Lord_Howe]'}),1,0);
fields(Temporal.Duration.from({days:28,hours:23,minutes:59}).round({largestUnit:'months',smallestUnit:'hours',roundingMode:'ceil',relativeTo:'2024-02-01T12:00[America/New_York]'}),0,0,1);
262;
