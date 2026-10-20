liveEvents.push('first');
export let value = 'first';
export function replace(next) { value = next; }
export function loadOther() { return import('./other.js'); }
