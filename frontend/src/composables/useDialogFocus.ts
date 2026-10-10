/**
 * Modal dialog focus management: on mount focus moves into the dialog (first form
 * field, else first control), Tab and Shift+Tab stay inside it, Escape closes it,
 * and on unmount focus returns to the element that was focused before it opened.
 */

import { onBeforeUnmount, onMounted } from 'vue';
import type { Ref } from 'vue';

const FOCUSABLE =
	'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

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

	onMounted(() => {
		const target =
			panel.value?.querySelector<HTMLElement>('input, select, textarea') ??
			panel.value?.querySelector<HTMLElement>(FOCUSABLE);
		target?.focus();
	});

	onBeforeUnmount(() => {
		if (previous?.isConnected) previous.focus();
	});

	return { onKeydown };
}
