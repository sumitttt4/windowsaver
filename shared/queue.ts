export type Pile = "downloads" | "screenshots";
export type Decision = "keep" | "later" | "bin";
export function rotate<T>(items: readonly T[]): T[] {
  return items.length < 2 ? [...items] : [...items.slice(1), items[0]];
}
export function isShortcutTarget(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable ||
      ["INPUT", "TEXTAREA", "SELECT", "BUTTON", "A", "SUMMARY"].includes(
        target.tagName,
      ))
  );
}
