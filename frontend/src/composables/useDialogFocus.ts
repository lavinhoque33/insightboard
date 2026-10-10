/**
 * Modal dialog focus management: on mount focus moves into the dialog (first form
 * field, else first control), Tab and Shift+Tab stay inside it, Escape closes it,
 * and on unmount focus returns to the element that was focused before it opened.
 *
 * The opening transition can keep the dialog unfocusable (visibility: hidden)
 * for a few frames, so the initial focus is retried each frame until it lands,
 * for at most MAX_FOCUS_FRAMES.
 */

import { onBeforeUnmount, onMounted } from 'vue';
import type { Ref } from 'vue';

const FOCUSABLE =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';
const MAX_FOCUS_FRAMES = 60;

export function useDialogFocus(
	panel: Ref<HTMLElement | null>,
	onClose: () => void,
) {
	const previous =
		document.activeElement instanceof HTMLElement
			? document.activeElement
			: null;

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			onClose();
			return;
		}
		if (event.key !== 'Tab' || !panel.value) return;
		const items = Array.from(
			panel.value.querySelectorAll<HTMLElement>(FOCUSABLE),
		);
		const first = items[0];
		const last = items[items.length - 1];
		if (!first || !last) return;
		if (event.shiftKey && document.activeElement === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && document.activeElement === last) {
			event.preventDefault();
			first.focus();
		}
	}

	let frame = 0;

	function focusInitial(attempt: number) {
		const root = panel.value;
		if (!root || root.contains(document.activeElement)) return;
		const target =
			root.querySelector<HTMLElement>('input, select, textarea') ??
			root.querySelector<HTMLElement>(FOCUSABLE);
		target?.focus();
		if (
			target &&
			document.activeElement !== target &&
			attempt < MAX_FOCUS_FRAMES
		)
			frame = requestAnimationFrame(() => focusInitial(attempt + 1));
	}

	onMounted(() => focusInitial(0));

	onBeforeUnmount(() => {
		cancelAnimationFrame(frame);
		if (previous?.isConnected) previous.focus();
	});

	return { onKeydown };
}
