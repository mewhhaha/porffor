function same(a,b){if(!Object.is(a,b))throw new Error(a+' != '+b);}
var before='2011-12-29T12:00[Pacific/Apia]';var after='2011-12-31T12:00[Pacific/Apia]';
same(Temporal.Duration.from({days:1}).total({unit:'hours',relativeTo:before}),24);
same(Temporal.Duration.from({days:1}).total({unit:'hours',relativeTo:after}),24);
same(Temporal.Duration.compare({days:1},{hours:24},{relativeTo:before}),0);
same(Temporal.Duration.from({hours:1}).round({largestUnit:'hours',smallestUnit:'minutes',relativeTo:after}).hours,1);
var later='2012-01-02T12:00[Pacific/Apia]';
same(Temporal.Duration.from({days:-1}).total({unit:'days',relativeTo:later}),-1);
same(Temporal.Duration.from({hours:-25}).round({smallestUnit:'days',relativeTo:later}).days,-1);
same(Temporal.Duration.from({hours:-25}).total({unit:'days',relativeTo:later}),-25/24);
262;
