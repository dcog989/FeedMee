// Global state for the tooltip
export const tooltipState = $state({
    visible: false,
    text: '',
    x: 0,
    y: 0,
});

export function tooltip(node: HTMLElement, text: string) {
    if (!text) return;

    function onMouseEnter(e: MouseEvent) {
        if (!text) return;
        tooltipState.text = text;
        tooltipState.visible = true;
        updatePosition(e);
    }

    function onMouseMove(e: MouseEvent) {
        updatePosition(e);
    }

    function onMouseLeave() {
        tooltipState.visible = false;
    }

    function updatePosition(e: MouseEvent) {
        // Offset from cursor
        tooltipState.x = e.clientX + 10;
        tooltipState.y = e.clientY + 15;
    }

    node.addEventListener('mouseenter', onMouseEnter);
    node.addEventListener('mousemove', onMouseMove);
    node.addEventListener('mouseleave', onMouseLeave);

    return {
        update(newText: string) {
            text = newText;
        },
        destroy() {
            node.removeEventListener('mouseenter', onMouseEnter);
            node.removeEventListener('mousemove', onMouseMove);
            node.removeEventListener('mouseleave', onMouseLeave);
            tooltipState.visible = false;
        },
    };
}
