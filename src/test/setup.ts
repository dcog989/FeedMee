import { cleanup } from "@testing-library/svelte";
import { afterEach } from "vitest";

class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

// jsdom lacks ResizeObserver (used by modals and the article list) and
// Element.scrollTo (used when restoring scroll position).
if (!globalThis.ResizeObserver) {
  globalThis.ResizeObserver = ResizeObserverStub as unknown as typeof ResizeObserver;
}

if (typeof HTMLElement.prototype.scrollTo !== "function") {
  HTMLElement.prototype.scrollTo = () => {};
}

afterEach(() => {
  cleanup();
});
