export function randomInt(n: number): number {
  return Math.floor(Math.random() * n);
}

export function range(start: number, end: number): number {
  return randomInt(end - start) + start;
}
