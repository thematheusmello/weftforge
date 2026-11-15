What this change does: It reshapes the native studio into fixed menu, tool, inspector, scene and transport panels. It embeds redistributable typefaces, includes their notices in package outputs, and isolates popup keyboard handling while retaining pending numeric edits.

Review scope: `f8be9b77c6b308fddd1a75bf5ba29687d560147e` → `e121cde85c63e26fa82450a2bdbade3e770ca687` in `.local/photocraft-studio-ui-20261115`. Single-user native desktop use. Source review only; the root agent owns workspace gates and uncommitted evidence metadata.

**Must fix**

1. **Remaining selected choices have low text contrast** (`apps/weftforge/src/main.rs:3514`; affected callers at `:1780` and `:3174`).
   - **What this is:** The global selection stroke also supplies selected button text in egui 0.35 (`widget_style.rs:150–154`). Boundary choices and sewing-span curve choices use these default selected buttons.
   - **Problem:** Selecting a boundary and reopening its list draws the selected choice in RGB 74/143/208 on RGB 53/73/93 at 13 px. Their calculated contrast is 2.71:1; the Scene and Snap overrides leave these other selected choices unchanged.
   - **Fix:** Give the global selected text stroke a light color, or apply the same explicit light text override to the remaining selected choices. Keep custom blue outlines separate where needed.
   - **If we skip it:** People with reduced vision can have difficulty reading the currently selected curve in either list.

No other concrete findings in the reviewed changes. Traced numeric focus/text/caret retention, popup dismissal and global shortcuts, menu command reachability, custom tool accessibility labels, selection identity, inspector creation ordering, command transaction paths, and package notice copies. All five font/notice files match their recorded SHA-256 and byte counts and are Git-tracked. The whitespace check reports only the retained upstream JetBrains Mono notice's trailing space on line 21.

Verdict: fix 1 first.

Not checked: No builds, test execution, native app control, screen-reader session, packaged launch, or reference parity assessment; the root agent's reported 49 native tests were read as context, not independently rerun. Source review cannot establish runtime platform or assistive-technology behavior.


Root resolution at `7ab2107aaa51f2c0fa9c9fd2ada972a08c433882`: changed the shared selection text stroke to RGB222. Explicit tool/Scene blue outlines remain. The actual selected boundary popup in `captures/selected-boundary-dropdown.jpg` has readable light text, reviewed directly by root. All five final gates pass at this source. This closes finding 1 for this increment; formal accessibility conformance and other-platform testing remain unverified.
