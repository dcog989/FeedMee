export function withAdded<T>(current: ReadonlySet<T>, ...items: T[]): Set<T> {
  const next = new Set(current);
  for (const item of items) next.add(item);
  return next;
}

export function withRemoved<T>(current: ReadonlySet<T>, ...items: T[]): Set<T> {
  const next = new Set(current);
  for (const item of items) next.delete(item);
  return next;
}
