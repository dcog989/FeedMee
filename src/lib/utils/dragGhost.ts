export function createDragGhost(e: DragEvent, label: string) {
  const dt = e.dataTransfer;
  if (!dt) return;
  const root = document.documentElement;
  const style = getComputedStyle(root);
  const bg = style.getPropertyValue('--bg-content').trim() || '#333';
  const text = style.getPropertyValue('--text-primary').trim() || '#fff';
  const pink = style.getPropertyValue('--bg-selected').trim() || '#ec4899';
  const img = document.createElement('div');
  img.textContent = label;
  img.style.cssText = `padding:2px 8px;background:${bg};color:${text};border:1px solid ${pink};border-radius:4px;font:8px/1.3 sans-serif;white-space:nowrap;position:absolute;top:-1000px;left:-1000px;pointer-events:none;`;
  document.body.appendChild(img);
  dt.setDragImage(img, 0, 0);
  requestAnimationFrame(() => document.body.removeChild(img));
}
