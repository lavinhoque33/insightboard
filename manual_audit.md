# Manual Audit: Widget Add/Configure Behavior

## Scope
- Issue reported from manual testing:
  - Selecting a widget from "Add Widget" creates it immediately without configuration input.
  - Clicking "Configure" from widget 3-dot menu does nothing.
- This is treated as a separate manual audit/remediation phase, not project progress tracking.

## Expected Behavior
- After adding a widget, user should be able to configure required parameters (URL, username, topic, etc.) immediately.
- Configure action from widget menu should always open the widget configuration modal.

## Progress Log
- 2026-03-03 10:59 CST: Audit started, reproduced issue path in dashboard editor.
- 2026-03-03 11:03 CST: Fixes implemented and verified with live UI smoke automation.

## Root Cause
- Add-widget flow created widget instances and rendered them but did not call configuration modal open logic after creation.
- Widget header actions were competing with GridStack drag behavior; click interactions on menu controls could be swallowed or unreliable.
- Dropdown state handling in `BaseWidget` depended on CSS class behavior without robust outside-click and propagation control in a draggable context.

## Fixes Applied
- `frontend/src/views/DashboardEditorView.vue`
  - Added GridStack drag constraints:
    - `draggable.handle: '.widget-drag-handle'`
    - `draggable.cancel: '.widget-action,button,a,input,textarea,select'`
  - Updated `addWidget()` to call `configureWidget(newWidget.id)` immediately after DOM/grid insertion.
  - Removed brittle manual dropdown DOM class toggling in add flow.
- `frontend/src/components/BaseWidget.vue`
  - Reworked options menu to explicit reactive render (`showMenu`) with controlled toggle/close behavior.
  - Added document click listener to close menu on outside click.
  - Added click/mousedown stop-propagation protections on widget action controls.
  - Marked drag handle and action zones with dedicated classes (`widget-drag-handle`, `widget-action`).

## Verification Evidence
- Build verification:
  - `cd frontend && npm run build` succeeded.
- Live smoke verification (Playwright against running app):
  - Created account and dashboard.
  - Added `Status Monitor` widget from Add Widget menu.
  - Confirmed `Configure Status Monitor` modal opens immediately after selection.
  - Saved URL config and confirmed widget loaded status data.
  - Opened widget 3-dot menu and clicked `Configure`; confirmed modal opens.
  - Repeated add flow with `GitHub Activity`; confirmed immediate config modal.
  - Repeated 3-dot `Configure` on GitHub widget; confirmed modal opens.

## Checklist
- [x] Reproduce/confirm root causes in widget add/config flow.
- [x] Fix add-widget flow to prompt configuration reliably.
- [x] Fix configure menu click handling to reliably open modal.
- [x] Ensure GridStack drag behavior does not swallow action clicks.
- [x] Validate by build/test and functional smoke.
- [x] Document result and close audit.

## Audit Status
- Closed on 2026-03-03 11:03 CST.

## Follow-up Hardening (2026-03-03 11:14 CST)

### Additional Issues Reported
- Configure action in widget menu looked visually disabled/grey.
- Canceling config right after adding a widget still left an empty widget on the dashboard.
- Widget header color contrast was poor (purple background with unreadable dark text).

### Additional Fixes Applied
- `frontend/src/components/BaseWidget.vue`
  - Changed widget header to high-contrast neutral styling (`bg-base-200`, `text-base-content`, border separator).
  - Updated action button text color for readability in header.
  - Updated Configure/Remove menu item classes to explicit interactive/visible styles.
  - Added stable test IDs for widget options/configure/remove menu controls.
- `frontend/src/views/DashboardEditorView.vue`
  - Added new-widget modal tracking (`pendingNewWidgetId`, `configSavedForCurrentModal`).
  - Added `closeConfigModal()` to discard a newly created widget when user cancels initial configuration.
  - Kept existing-widget cancel behavior unchanged.

### Regression Tests Added
- Vitest setup added for frontend component tests.
- `frontend/src/components/__tests__/BaseWidget.spec.ts`
  - Verifies 3-dot menu Configure click emits `configure`.
- `frontend/src/views/__tests__/DashboardEditorView.spec.ts`
  - Verifies canceling the initial config modal removes the newly added widget.

### Validation Results
- `cd frontend && npm run test` passed (2 tests).
- `cd frontend && npm run build` passed.
- Live Playwright smoke validation passed:
  - Add widget -> config modal opens.
  - Cancel initial config -> widget is not persisted.
  - 3-dot Configure opens config modal correctly.

## Follow-up Behavior Correction (2026-03-03 11:19 CST)

### Reported UX Gap
- Selecting a widget type from Add Widget still showed a newly created empty widget in the background before config save.

### Behavior Change Implemented
- Updated add flow to use a draft widget only in modal state.
- Selecting widget type now opens configuration modal only.
- Widget is created and inserted into grid/dashboard only after clicking `Save Changes` with valid config.
- Clicking `Cancel` now simply closes the modal with no widget creation/removal side effects.

### Files Updated
- `frontend/src/views/DashboardEditorView.vue`
  - Added modal draft state flag (`isNewWidget`) on `configModalWidget`.
  - Moved actual `widgets.push(...)` + GridStack `addWidget(...)` into `saveWidgetConfig()` for new widgets only.
  - Simplified modal close behavior to avoid mutation on cancel.
- `frontend/src/views/__tests__/DashboardEditorView.spec.ts`
  - Updated regression to assert widget count stays unchanged while modal is open and after cancel.
  - Added regression asserting widget is created only after successful save.

### Validation Results
- `cd frontend && npm run test` passed (3 tests).
- `cd frontend && npm run build` passed.
- Live Playwright validation on real UI:
  - Before selection: widget count `2`.
  - After selecting Status (modal open): widget count still `2`.
  - After cancel: widget count still `2`.
  - After save with URL: widget count becomes `3`.
