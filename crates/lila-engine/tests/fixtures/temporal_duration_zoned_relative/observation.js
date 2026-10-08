var log=[];var one={get days(){log.push('one');return 1;}};var two={get hours(){log.push('two');return 24;}};
var relative=Temporal.ZonedDateTime.from('2024-03-09T12:00[America/New_York]');
for (var name of ['year','month','day','hour','epochNanoseconds','timeZoneId','calendarId']) Object.defineProperty(relative,name,{get:function(){throw new Error('public slot getter');}});
var options={get relativeTo(){log.push('relative');return relative;}};
if(Temporal.Duration.compare(one,two,options)!==-1 || log.join(',')!=='one,two,relative')throw new Error(log.join(','));
log=[];var roundOptions={get largestUnit(){log.push('largest');return 'days';},get relativeTo(){log.push('relative');return relative;},get roundingIncrement(){log.push('increment');return 1;},get roundingMode(){log.push('mode');return 'ceil';},get smallestUnit(){log.push('smallest');return 'hours';}};
var rounded=Temporal.Duration.from({hours:22,minutes:59}).round(roundOptions);
if(rounded.days!==1 || log.join(',')!=='largest,relative,increment,mode,smallest')throw new Error(log.join(','));
log=[];var totalOptions={get relativeTo(){log.push('relative');return relative;},get unit(){log.push('unit');return 'hours';}};
if(Temporal.Duration.from({days:1}).total(totalOptions)!==23 || log.join(',')!=='relative,unit')throw new Error(log.join(','));
var marker={};var seen;log=[];
try{Temporal.Duration.from({days:1}).round({get relativeTo(){throw marker;},get roundingIncrement(){log.push('late');return 1;}});}catch(e){seen=e;}
if(seen!==marker || log.length!==0)throw new Error('abrupt relativeTo');
262;
