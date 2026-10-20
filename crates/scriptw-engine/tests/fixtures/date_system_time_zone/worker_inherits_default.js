var worker = "var date = new Date(); if (date.getTime() !== 1234 || date.getHours() !== 19 || Temporal.Now.timeZoneId() !== 'America/New_York' || Temporal.Now.zonedDateTimeISO().epochNanoseconds !== 1234000000n || new Intl.DateTimeFormat('en', {year:'numeric'}).resolvedOptions().timeZone !== 'America/New_York') throw 'worker default'; __scriptwAgentReport('America/New_York:19:1234'); __scriptwAgentLeaving();";
__scriptwAgentStart(worker);
var report = null;
for (var attempt = 0; attempt < 20000 && report === null; attempt++) {
  __scriptwAgentSleep(1);
  report = __scriptwAgentGetReport();
}
if (report !== 'America/New_York:19:1234') throw 'worker primary, projection and clock report';
262;
