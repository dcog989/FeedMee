import { tooltipState } from "./actions/tooltip.svelte";

/** Open state and position for a tag popover anchored to a button. */
export function createTagPopover() {
  let articleId = $state<number | null>(null);
  let x = $state(0);
  let y = $state(0);

  function toggle(event: MouseEvent, id: number) {
    if (articleId === id) {
      articleId = null;
      return;
    }
    tooltipState.visible = false;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    x = rect.left;
    y = rect.bottom + 4;
    articleId = id;
  }

  function close() {
    articleId = null;
  }

  return {
    get articleId() {
      return articleId;
    },
    get x() {
      return x;
    },
    get y() {
      return y;
    },
    toggle,
    close,
  };
}
