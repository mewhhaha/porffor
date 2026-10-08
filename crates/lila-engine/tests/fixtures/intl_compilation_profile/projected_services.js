if (Intl.getCanonicalLocales(['FR', 'fr']).join() !== 'fr') throw 'retained Locale foundation';
var relative = new Intl.RelativeTimeFormat('fr', { numeric: 'always' });
if (relative.format(2, 'day') !== 'dans 2 jours') throw 'selected RelativeTime source row';
if (relative.resolvedOptions().locale !== 'fr') throw 'selected RelativeTime locale';
if (new Intl.RelativeTimeFormat('fr').formatToParts(2, 'day').map(function (part) { return part.value; }).join('') !== 'dans 2 jours') throw 'retained private Number kernel';
print('intl-services-projection:ok');
262;
