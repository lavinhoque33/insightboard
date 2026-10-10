/**
 * Disclosure menu behaviour (a button that shows a list of actions): the trigger
 * exposes `aria-expanded`; Escape closes and returns focus to the trigger; a click
 * outside, or focus leaving the menu, closes it.
 */

import { nextTick, onBeforeUnmount, ref, watch } from 'vue';

export function useDisclosure() {
	const open = ref(false);
	// The element wrapping trigger and panel, and the trigger button itself
	const root = ref<HTMLElement | null>(null);
	const trigger = ref<HTMLElement | null>(null);

	function close(options: { returnFocus?: boolean } = {}) {
		if (!open.value) return;
		open.value = false;
		if (options.returnFocus) trigger.value?.focus();
	}

	function onDocumentClick(event: MouseEvent) {
		if (!root.value?.contains(event.target as Node | null)) close();
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.stopPropagation();
			close({ returnFocus: true });
		}
	}

	function onFocusOut(event: FocusEvent) {
		const next = event.relatedTarget as Node | null;
		if (next && !root.value?.contains(next)) close();
	}

	function listen(add: boolean) {
		if (add) {
			document.addEventListener('click', onDocumentClick, true);
			root.value?.addEventListener('keydown', onKeydown);
			root.value?.addEventListener('focusout', onFocusOut);
		} else {
			document.removeEventListener('click', onDocumentClick, true);
			root.value?.removeEventListener('keydown', onKeydown);
			root.value?.removeEventListener('focusout', onFocusOut);
		}
	}

	watch(open, async (isOpen) => {
		listen(isOpen);
		if (!isOpen) return;
		// Move focus into the panel so keyboard users land on the first action
		await nextTick();
		root.value
			?.querySelector<HTMLElement>(
				'[data-menu-panel] button, [data-menu-panel] a',
			)
			?.focus();
	});

	onBeforeUnmount(() => listen(false));

	return {
		open,
		root,
		trigger,
		toggle: () => {
			open.value = !open.value;
		},
		close,
	};
}
