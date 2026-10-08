var worker = "var date = new Date(); if (date.getTime() !== 1234 || date.getHours() !== 19 || Temporal.Now.timeZoneId() !== 'America/New_York' || Temporal.Now.zonedDateTimeISO().epochNanoseconds !== 1234000000n || new Intl.DateTimeFormat('en', {year:'numeric'}).resolvedOptions().timeZone !== 'America/New_York') throw 'worker default'; __lilaAgentReport('America/New_York:19:1234'); __lilaAgentLeaving();";
__lilaAgentStart(worker);
var report = null;
for (var attempt = 0; attempt < 20000 && report === null; attempt++) {
  __lilaAgentSleep(1);
  report = __lilaAgentGetReport();
}
if (report !== 'America/New_York:19:1234') throw 'worker primary, projection and clock report';
262;
