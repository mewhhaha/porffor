var targetPrivate = 5;
export const rootThis = this;
export const directArrow = () => this;
export const nestedArrow = () => () => this;
export function ordinary() { return this; }
export function arrowFromCall() { return () => this; }
