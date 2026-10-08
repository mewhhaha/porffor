// Unicode and erased export syntax before callable spans: 🟣 café
import { café } from './other.js';
export function read() { void import.meta; return import('./other.js'); }
export default function () { return import('./other.js'); }
export class Class { load() { return import('./other.js'); } }
