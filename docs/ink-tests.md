# Ink 测试移植状态

基准：Ink 8.0.0（commit 26d2c3f）。移植方法见 [design.md](design.md#移植-ink-测试的方法)。

## 总览

Ink 有 95 个测试文件、1,646 个测试。

- **M0**：40 个文件、755 个测试。移植 617 个，写成 621 个 `#[test]`（`content-offset.tsx` 的一个参数化测试按参数拆成 5 个），上游跳过的 4 个中，3 个标 `#[ignore]`，1 个在 taffy 下通过。未移植 138 个，原因见下表，明细见[下文](#m0-文件中未移植的测试)。
- **其他 55 个文件**（891 个测试）：49 个随 M1–M5 移植，6 个不适用。M1 文件中不移植的测试见[下文](#m1-文件中不移植的测试)。

| 未移植原因 | 数量 | 处理 |
|---|---:|---|
| React 并发模式下的重复测试（`… - concurrent`） | 46 | 不移植 |
| 重渲染、state、effect | 35 | M1 |
| Static 跨帧、非 TTY / CI、备用屏幕 | 32 | M2 |
| 输入、raw mode | 6 | M3 |
| 屏幕阅读器 | 2 | M5 |
| 缺 API | 1 | 见[缺 API](#缺-api) |
| 不适用 | 16 | 不移植 |

## 按文件

里程碑为 M0 的文件，表中是移植的部分，其余见下一节。"—" 表示不适用：`build-output.ts` 检查 npm 包的构建产物，`is-devtools-reachable.tsx` 测 React DevTools 的连接，`style-types.ts` 是 TypeScript 类型检查，Rust 由类型系统保证。`styles.ts` 直接调用 Yoga 的 API，去掉样式后的行为由各布局文件的重渲染测试和 `style-update-consistency.tsx` 覆盖。`static-abandoned-render.tsx` 依赖 Suspense 和 transition。`tests/pty.rs` 不用 libtest，其中每个 PTY 用例按一个 `#[test]` 计数。

| Ink 文件 | Ink 测试数 | 里程碑 | 移植的 `#[test]` | Rust 文件 |
|---|---|---|---|---|
| `absolute-truncation.tsx` | 4 | M0 | 4 | `tests/absolute_truncation.rs` |
| `alternate-screen-example.tsx` | 2 | M4 | 2 | `tests/examples.rs` |
| `ansi-newlines.tsx` | 9 | M0 | 9 | `tests/ansi_newlines.rs` |
| `ansi-tokenizer.ts` | 26 | M0 | 26 | `src/text/ansi/tests.rs` |
| `background.tsx` | 32 | M0 / M1 | 27 | `tests/background.rs` |
| `border-backgrounds.tsx` | 5 | M0 | 5 | `tests/border_backgrounds.rs` |
| `borders.tsx` | 52 | M0 / M1 | 48 | `tests/borders.rs` |
| `build-output.ts` | 4 | — |  |  |
| `c1-rendering.tsx` | 8 | M0 | 8 | `tests/c1_rendering.rs` |
| `clear-rerender.tsx` | 9 | M2 | 3 | `tests/clear_rerender.rs` |
| `clip-wide-background.tsx` | 10 | M0 | 10 | `tests/clip_wide_background.rs` |
| `colon-colors.tsx` | 16 | M0 | 16 | `tests/colon_colors.rs` |
| `colorize.ts` | 16 | M0 | 16 | `tests/colorize.rs` |
| `component-regressions.tsx` | 4 | M1 | 2 | `tests/component_regressions.rs` |
| `components.tsx` | 93 | M0 / M1 / M2 / M3 | 72 | `tests/components.rs`、`src/driver/tests.rs` |
| `content-offset.tsx` | 23 | M0 | 24 | `tests/content_offset.rs` |
| `cursor-exit-position.tsx` | 6 | M2 | 3 | `tests/cursor_exit_position.rs`、`src/output/tests/cursor_exit_position.rs` |
| `cursor-helpers.tsx` | 17 | — | 0 | 不适用，cell diff 没有对应内部函数 |
| `cursor.tsx` | 20 | M2 / M3 | 16 | `tests/cursor.rs` |
| `display.tsx` | 6 | M0 / M1 | 4 | `tests/display.rs` |
| `error-overview.tsx` | 7 | M4 | 6 | `src/error_overview/tests.rs` |
| `errors.tsx` | 8 | M4 | 6 | `tests/errors.rs` |
| `exit-keyboard.tsx` | 7 | M3 | 7 | `tests/exit_keyboard.rs` |
| `exit.tsx` | 15 | M3 / M4 | 13 | `tests/exit.rs`、`tests/pty.rs` |
| `flex-align-content.tsx` | 13 | M0 / M1 | 11 | `tests/flex_align_content.rs` |
| `flex-align-items.tsx` | 9 | M0 | 9 | `tests/flex_align_items.rs` |
| `flex-align-self.tsx` | 9 | M0 | 9 | `tests/flex_align_self.rs` |
| `flex-direction.tsx` | 10 | M0 / M1 | 7 | `tests/flex_direction.rs` |
| `flex-justify-content.tsx` | 12 | M0 | 12 | `tests/flex_justify_content.rs` |
| `flex-wrap.tsx` | 8 | M0 / M1 | 8 | `tests/flex_wrap.rs` |
| `flex.tsx` | 8 | M0 | 8 | `tests/flex.rs` |
| `focus-empty-id-regression.tsx` | 3 | M3 | 3 | `tests/focus_empty_id_regression.rs` |
| `focus-order.tsx` | 3 | M3 | 3 | `tests/focus_order.rs` |
| `focus-strict-mode.tsx` | 4 | M3 | 1 | `tests/focus_strict_mode.rs` |
| `focus.tsx` | 47 | M3 | 29 | `tests/focus.rs` |
| `gap.tsx` | 8 | M0 / M1 | 5 | `tests/gap.rs` |
| `hooks-use-input-kitty.tsx` | 24 | M3 | 24 | `tests/hooks_use_input_kitty.rs` |
| `hooks-use-input-navigation.tsx` | 18 | M3 | 18 | `tests/hooks_use_input_navigation.rs` |
| `hooks-use-input.tsx` | 34 | M3 | 32 | `tests/hooks_use_input.rs` |
| `hooks-use-paste.tsx` | 4 | M3 | 4 | `tests/hooks_use_paste.rs` |
| `hooks.tsx` | 6 | M2 / M3 | 5 | `tests/hooks.rs`、`tests/pty.rs` |
| `input-buffered-ctrl-c.tsx` | 5 | M3 | 5 | `tests/input_buffered_ctrl_c.rs` |
| `input-keypad-enter.tsx` | 2 | M3 | 2 | `tests/input_keypad_enter.rs` |
| `input-parser.ts` | 65 | M3 | 65 | `src/input/parser/tests.rs` |
| `is-devtools-reachable.tsx` | 3 | — |  |  |
| `issue-973-static-commit.tsx` | 1 | M2 | 1 | `tests/issue_973_static_commit.rs` |
| `kitty-keyboard.tsx` | 77 | M3 | 74 | `src/input/keypress/tests/kitty_keyboard.rs`、`tests/kitty_keyboard.rs` |
| `kitty-negotiation.tsx` | 5 | M3 / M4 | 5 | `tests/kitty_negotiation.rs` |
| `line-update.ts` | 29 | — | 0 | 不适用，cell diff 没有对应内部函数 |
| `log-update-blank-growth.ts` | 4 | M2 | 4 | `src/output/tests/log_update_blank_growth.rs` |
| `log-update-columns.tsx` | 8 | M2 | 3 | `src/output/tests/log_update_columns.rs` |
| `log-update-sparse.ts` | 9 | M2 | 8 | `src/output/tests/log_update_sparse.rs` |
| `log-update.tsx` | 52 | M2 | 31 | `src/output/tests/log_update.rs` |
| `margin.tsx` | 15 | M0 / M1 | 13 | `tests/margin.rs` |
| `measure-element.tsx` | 13 | M1 / M2 | 12 | `tests/measure_element.rs` |
| `measure-text.tsx` | 13 | M0 | 11 | `src/text/measure/tests.rs` |
| `overflow.tsx` | 44 | M0 | 40 | `tests/overflow.rs` |
| `overlap-wide-background.tsx` | 7 | M0 | 7 | `tests/overlap_wide_background.rs` |
| `padding.tsx` | 16 | M0 / M1 | 14 | `tests/padding.rs` |
| `parse-keypress.ts` | 27 | M3 | 24 | `src/input/keypress/tests/parse_keypress.rs` |
| `position.tsx` | 13 | M0 / M1 | 11 | `tests/position.rs` |
| `reconciler.tsx` | 14 | M1 | 7 | `tests/reconciler.rs` |
| `render-callback.tsx` | 4 | — | 0 | 缺 API / 不适用，见下文 |
| `render-to-string.tsx` | 37 | M0 / M1 / M4 | 34 | `tests/render_to_string.rs` |
| `render.tsx` | 119 | M2 / M3 / M4 | 52 | `tests/render.rs`、`tests/pty.rs`、`tests/examples.rs` |
| `rendering-regressions.tsx` | 2 | M0 | 2 | `tests/rendering_regressions.rs` |
| `rerender-input.tsx` | 4 | M3 | 3 | `tests/rerender_input.rs` |
| `sanitize-ansi.ts` | 50 | M0 | 50 | `src/text/sanitize/tests.rs` |
| `screen-reader.tsx` | 33 | M5 |  |  |
| `squash-text-nodes.tsx` | 2 | M0 | 2 | `tests/squash_text_nodes.rs` |
| `static-abandoned-render.tsx` | 1 | — |  |  |
| `static-blank-lines.tsx` | 9 | M0 | 9 | `tests/static_blank_lines.rs` |
| `static-runtime-blank-lines.tsx` | 8 | M2 | 7 | `tests/static_runtime_blank_lines.rs` |
| `static-string-replacement.tsx` | 4 | M1 | 4 | `tests/static_string_replacement.rs` |
| `static-trailing-layout.tsx` | 7 | M0 | 7 | `tests/static_trailing_layout.rs` |
| `style-types.ts` | 1 | — |  |  |
| `style-update-consistency.tsx` | 15 | M1 | 15 | `tests/style_update_consistency.rs` |
| `styled-combining-marks.tsx` | 9 | M0 | 9 | `tests/styled_combining_marks.rs` |
| `styles.ts` | 17 | — |  |  |
| `suspend-terminal.tsx` | 14 | M4 | 9 | `tests/suspend_terminal.rs`、`tests/pty.rs` |
| `suspension-exit.tsx` | 2 | M4 | 2 | `tests/suspension_exit.rs` |
| `suspension-handle.tsx` | 2 | — |  |  |
| `suspension-input-disable.tsx` | 7 | M4 | 7 | `tests/suspension_input_disable.rs` |
| `suspension-output.tsx` | 3 | M4 | 3 | `tests/suspension_output.rs` |
| `suspension-resize.tsx` | 2 | M4 | 2 | `tests/suspension_resize.rs` |
| `terminal-resize.tsx` | 28 | M2 | 15 | `tests/terminal_resize.rs` |
| `text-controls.tsx` | 24 | M0 | 24 | `tests/text_controls.rs`, `src/text/sanitize/tests.rs` |
| `text-width.tsx` | 18 | M0 | 18 | `tests/text_width.rs` |
| `text.tsx` | 57 | M0 / M1 | 49 | `tests/text.rs` |
| `truncate-width.tsx` | 9 | M0 | 9 | `tests/truncate_width.rs` |
| `use-animation.tsx` | 45 | M4 | 38 | `tests/use_animation.rs` |
| `use-box-metrics.tsx` | 19 | M4 | 16 | `tests/use_box_metrics.rs` |
| `width-height.tsx` | 29 | M0 / M1 | 25 | `tests/width_height.rs` |
| `wrap-text.tsx` | 17 | M0 / M1 | 15 | `tests/wrap_text.rs`, `src/text/wrap/tests.rs` |
| `write-synchronized.tsx` | 7 | — | 0 | 不适用，决策由 render / components 覆盖 |

## M0 文件中未移植的测试

"并发" 一项列出的是去掉 ` - concurrent` 后缀的测试名。

### `background.tsx`

- M1 已移植：`Box preserves child state when adding a background color`、`Box preserves child state when removing a background color`、`Box background updates on rerender`
- 缺 API：`Mixed text with and without background inheritance`
- 并发：`Text inherits parent Box background color`、`Nested Box background inheritance`、`Box background with hex color`、`Box background updates on rerender`

### `borders.tsx`

- M1 已移植：`render border after update`、`render border edge changes after update when borderStyle is unchanged`
- 并发：`single node - full width box`、`single node - fit-content box`、`nested boxes`、`render border after update`

### `components.tsx`

- M1 已移植：`hooks`
- M1 已移植：`remeasure text dimensions on text change`、`replace child node with text`、`reset prop when it’s removed from the element`
- M2 已移植：非 TTY、interactive 覆盖、备用屏幕和非交互 Static 输出，见 `tests/components.rs`。
- M2 已移植：`render only last frame when run in CI`、`render all frames if CI environment variable equals false`，在 `src/driver/tests.rs` 使用同步循环、给定 CI 值与内存终端。20ms 线程持有 AppHandle，五次 tick 后丢弃，循环自然结束；保留 Counter 0–4 不出现 / Counter 5 出现，以及交互 Counter 0–5 都出现的原断言，补充 Static #1–#5 各一次。
- 不移植与后续里程碑的测试见 [M2 文件中不移植的测试](#m2-文件中不移植的测试)。

### `content-offset.tsx`

- 不适用：`contentOffsetX/Y - fractional offsets are truncated to whole cells`、`contentOffsetX/Y - negative fractional offsets are truncated to whole cells`、`contentOffsetX/Y - non-finite offsets fall back to zero`。偏移的类型是整数格。

### `display.tsx`

- M1 已移植：`removing display="none" restores the default layout`、`removing display="flex" restores the default layout`
- 并发：`display flex`、`display none`

### `flex-align-content.tsx`

- M1 已移植：`clears alignContent on rerender to default flex-start`、`clears alignContent from stretch on rerender to default flex-start`
- 不适用：`clears alignContent when prop is omitted on rerender`。它与第一个用例的差别是 JS 中缺少的键与值为 `undefined` 的键，Rust 里两者是同一个测试。
- 并发：`align content center`

### `flex-direction.tsx`

- M1 已移植：`setting direction to undefined restores the default row layout`
- M5：`undefined direction uses row separators for screen readers`
- 并发：`direction row`、`direction column`

### `flex-wrap.tsx`

- M1 已移植：`setting wrap to undefined restores nowrap`、`setting wrap-reverse to undefined restores nowrap`

### `gap.tsx`

- M1 已移植：`removing columnGap restores gap on rerender`、`removing rowGap restores gap on rerender`
- 并发：`gap`、`column gap`、`row gap`

### `margin.tsx`

- M1 已移植：`removing marginLeft restores marginX on rerender`
- 并发：`margin`、`nested margin`

### `measure-text.tsx`

- 不适用：`returns cached result on repeated calls`、`evicts old cached results`。两者观察 JS 对象身份，Rust 返回的是值。

### `overflow.tsx`

- 并发：`overflowX - single text node in a box inside overflow container`、`overflowY - single text node inside overflow container`、`overflow - single text node inside overflow container`、`nested overflow`

### `padding.tsx`

- M1 已移植：`removing paddingLeft restores paddingX on rerender`、`removing paddingX restores padding on rerender`
- 并发：`padding`、`nested padding`

### `position.tsx`

- M1 已移植：`clears top offset on rerender`、`clears percentage top and left offsets on rerender`、`clears bottom and right offsets on rerender`
- 不适用：`clears percentage top and left offsets when props are omitted on rerender`，原因同 `flex-align-content.tsx`。
- 并发：`absolute position with top and left offsets`

### `render-to-string.tsx`

- M1 已移植：`captures initial render output before effect-driven state updates`、`useLayoutEffect state updates are reflected in output`、`runs effect cleanup on teardown`、`runs effect cleanup when a transform throws`
- M4 已移植：`component that throws propagates the error`、`subsequent calls work after a component error`
- 不适用：`preserves component errors from another realm`（JS realm）、`component that throws undefined does not silently return empty output`（`render_to_string` 不捕获 panic，任何 payload 向上传播，同 `component that throws propagates the error`）、`text outside Text component throws`（Rust 编译期拒绝）

### `text.tsx`

- M1 已移植：`text with empty-to-nonempty sibling does not wrap`、`remeasure text when text is changed`、`remeasure text when text nodes are changed`
- 并发：`<Text> with undefined children`、`<Text> with null children`、`text with standard color`、`text with dim+bold`、`text with hex color`、`text with inversion`、`remeasure text when text is changed`、`remeasure text when text nodes are changed`

### `width-height.tsx`

- M1 已移植：`clears maxWidth on rerender`、`clears maxHeight on rerender`、`clears aspectRatio on rerender`
- 不适用：`set width to a fraction of a column`、`truncated text at a positive fractional width keeps its row`。宽度的类型是整数格。
- 并发：`set width`、`set height`

### `wrap-text.tsx`

- M1 已移植：`changing text wrapping recalculates the container height`
- 不适用：`wraps a fraction of a column like one column`（宽度是整数格）、`evicts old cached results`（缓存的内部状态）
- 移植了一部分：`keeps text at its natural width when there is no room to wrap` 缺宽度 `-1` 的断言，无符号宽度表示不了；`uses separate cache entries for texts that end with digits` 和 `does not reuse a cached result from a wider box` 保留输出断言，去掉清缓存的步骤。

## M1 文件中不移植的测试

- `component-regressions.tsx`：`concurrent: true` 的 2 个是并发模式下的重复测试。
- `measure-element.tsx`：
  - 不适用：`measure element returns zeros for node without yoga`。它构造一个没有 Yoga 节点的假元素，Rust 里没有这种节点。
  - M2 已移植：`calculate layout while rendering is throttled`。
- `reconciler.tsx`：
  - 不适用：`fragment refs remain unsupported while their children render and update`（Fragment ref）、`terminal host updates complete inside a transition`（transition）、`Suspense hides nested text while showing its fallback`、`resuming Suspense preserves display none`、`support suspense`（Suspense）。
  - 并发：`support suspense with concurrent mode`。
  - 重复：`replace child node with text` 与 `components.tsx` 的同名测试相同，移植在 `tests/components.rs`。
- `static-abandoned-render.tsx`、`styles.ts`：整个文件不适用，原因见[按文件](#按文件)。

## M2 文件中不移植的测试

- `log-update.tsx`：
  - 重复，只差 `incrementalRendering`：`incremental rendering - renders and updates output`、`incremental rendering - skips identical output`、`incremental rendering - clear() erases the frame with the cursor at row 0`、`incremental rendering - clear() erases the frame with the cursor at row 5`、`incremental rendering - positions cursor after surgical updates`、`incremental rendering - repositions cursor when only cursor position changes (same output)`、`incremental rendering - cursor-only update with no trailing newline lands on the target line`。
  - 不适用：`incremental rendering - single write call with multiple surgical updates` 检查 write 的次数；名字含 `sync()` 的 12 个测试检查 Ink 的 `sync()` 方法，ink-rs 的输出层自己记录屏幕上的内容，没有这个方法。
  - 不移植：`incremental rendering - no trailing newline: trailing to no-trailing transition`。两帧的行相同，只差末尾换行。ink-rs 由视口高度决定末尾换行，这个变化只在视口变矮时出现，由 `terminal-resize.tsx` 的测试覆盖。
- `log-update-columns.tsx`：
  - 不适用：`falls back after a width change or a frame height change`、`clear invalidates the prefix baseline`、`reset invalidates the prefix baseline`、`done invalidates the prefix baseline`、`sync establishes the prefix baseline for the next frame`。它们检查 Ink 增量模式按行前缀重写的算法，ink-rs 按单元格比较。
- `log-update-sparse.ts`：不适用：`the screen reducer preserves skipped cells until they are erased`，它测的是 Ink 的测试辅助函数 `reconstructTerminalLines`。
- `log-update-blank-growth.ts`、`issue-973-static-commit.tsx`：无，文件中的测试已移植。

### `cursor.tsx`

- 重复，`incrementalRendering` 变体：`memoized cursor survives sibling updates (incremental: true)`、`incremental rendering - fullscreen: cursor lands on the requested row across rerender and cursor-only update`、`incremental rendering - fullscreen: cursor lands on the requested row on the sync path`。
- 不适用，Suspense / concurrent：`cursor position does not leak from suspended concurrent render to fallback`。

### `cursor-exit-position.tsx`

- 重复，`incrementalRendering` 变体：`unmount returns a positioned cursor below the frame (incremental: true)`、`done returns the cursor once without erasing output (incremental: true)`、`alternate-screen exit does not reposition the primary cursor (incremental: true)`。

### `hooks.tsx`

- 不适用，Node EventEmitter 的监听器上限：`useInput - no MaxListenersExceededWarning with many useInput hooks`。

### `cursor-helpers.tsx`、`line-update.ts`

- 不适用，分别全部 17 个和 29 个测试：它们测试 Ink 的光标序列辅助函数和按行前缀更新函数，cell diff 没有对应内部函数。光标位置、显隐、退出与外部写后的屏幕由 Output 和 app 测试覆盖。

### `render.tsx`

- 不适用，`rerender on resize` 的 resize-listener 数量断言：驱动轮询事件，不使用 Node EventEmitter；unmount 消耗 app。内容与尺寸断言已移植。resize 示例归 M4，名称见本节示例清单。
- 不适用，Node stream、JS undefined 或 ref/unref：`accepts standard Node streams without stream-specific assertions`、`handles input without requiring process ref methods`、`undefined stream options use the default streams`。
- 不适用，Node TTY 流缺少 `setRawMode` 方法；ink-rs 由 `Terminal` 判断 stdin：`reports raw mode as unavailable when a TTY input stream lacks raw mode`。
- M4，示例与挂起：`Jest example announces completion only after all results`、`subprocess example retains recent lines across chunks`、`terminal resize example stays open to report resized dimensions`、`suspend example reports the result of false`、`suspend example reports the result of true`、`suspend example reports the result of printf editor-command-ran`、`subprocess example strips ANSI sequences across chunks`、`subprocess example displays command launch errors`、`subprocess example decodes UTF-8 across chunks`。
- 不适用，React transition：`transition example deletes a whole grapheme from both queries`、`transition example ignores Enter in the search query`。
- 不适用，断言 Ink 的整屏清除或逐行擦除次数；ink-rs 使用高度上限和 cell diff：`erase screen`、`erase screen where <Static> exists but interactive part is taller than viewport`、`erase screen where state changes in small viewport`、`#450: full-height rerenders should not repeatedly clear terminal`、`#450: shrink from full-height to rows - 1 should clear exactly once`、`#450: shrink from overflow to rows - 1 should clear exactly once`、`#450: <Static> with shrink from full-height should clear exactly once`、`#450: full-height rerenders with <Static> should not repeatedly clear terminal`。
- 不适用，Windows 不在 v1 范围内：`#969: full-height rerenders on Windows should clear terminal between frames`。
- 重复，incrementalRendering 变体；ink-rs 的输出路径是 cell diff：`preserve terminal history when rerendering after clear - incremental`、`full clears do not replay <Static> output into scrollback - incremental`、`growing into an overflowing frame keeps <Static> rows that are still in the viewport - incremental`、`<Static> items added while the frame overflows are written once - incremental`、`alternate screen replays <Static> output when the frame shrinks after overflowing - incremental`、`alternate screen replays <Static> items added while the frame overflows once - incremental`。
- M5，屏幕阅读器：`preserve terminal history when rerendering after clear - screen-reader`、`clear screen-reader output`、`preserve screen-reader output around stdout writes`、`replace restored screen-reader output after clear and stdout write`、`replace screen-reader output ending in a newline after stdout write`、`preserve static history when screen-reader output resizes`、`rewrap screen-reader output when the terminal gets wider`。
- 不适用，design.md 外部输出不捕获 console：`intercept console methods with omitted patchConsole option`、`intercept console methods with undefined patchConsole option`。
- 缺 API，本阶段不添加 onRender / renderTime：`outputs renderTime when onRender is passed`。
- 不适用，Node 写回调、beforeExit 或测试 helper：`waitUntilExit resolves after stdout write callback`、`createDelayedWriteCallbackStdout delays only the first matching chunk`、`waitUntilExit after unmount does not retain a beforeExit listener`、`issue 596: useEffect can run before the first frame write callback`。
- 缺 API，本阶段不添加 waitUntilRenderFlush；Terminal 写入没有 Promise 或 Node 写回调：`waitUntilRenderFlush waits for generic stdout write callback`、`waitUntilRenderFlush resolves after stdout write callback`、`waitUntilRenderFlush flushes pending throttled render`、`waitUntilRenderFlush resolves when stdout is not writable`、`waitUntilRenderFlush waits for rerender write callback`、`waitUntilRenderFlush waits for concurrent rerender commit`、`waitUntilRenderFlush waits for all concurrent waiters on the same rerender`、`useApp waitUntilRenderFlush resolves after the first frame write callback`、`useApp waitUntilRenderFlush waits for state update frame flush`、`useApp waitUntilRenderFlush waits for state update queued in same effect tick`、`waitUntilRenderFlush resolves after unmount`、`waitUntilRenderFlush waits for unmount write callback`、`waitUntilRenderFlush after unmount does not register beforeExit listener`、`waitUntilRenderFlush resolves after exit with error`。
- 不适用，Node 写回调重入、运行时 debug 或 JS realm：`waitUntilExit resolves first exit value when exit is re-entered during unmount writes`、`waitUntilExit resolves first exit value when exit is re-entered during unmount writes in debug mode`、`waitUntilExit resolves first exit value when exit is re-entered during unmount writes with screen reader`、`exit rejects on cross-realm Error`、`exit with cross-realm Error rejects after stdout write callback`。
- 不适用，Node ended stdout、JS timer 数量或 Yoga 内存释放：`unmount does not write to ended stdout stream`、`unmount cancels pending throttled log writes when stdout is ended`、`unmount cancels pending throttled render when stdout is ended`、`unmount frees the root Yoga node`。

### `terminal-resize.tsx`

- 重复，`incrementalRendering` 变体：`rows shrink with a cursor above the output bottom keeps lines above the frame - cursor row 0, 8 rows (incremental)`、`rows shrink with a cursor above the output bottom keeps lines above the frame - cursor row 4, 7 rows (incremental)`、`width and height shrinking together keeps lines above the frame (incremental)`、`rows shrink right after a commit clears the cursor keeps lines above the frame (incremental)`、`incremental rendering - erases and rewrites the frame when the terminal height shrinks onto it`、`incremental rendering - keeps content above the frame when the terminal height shrinks onto it and a custom cursor sits on it`、`incremental rendering - keeps content above the frame when the terminal height shrinks onto it and the same render moves a custom cursor`、`incremental rendering - writes nothing when the terminal height shrinks and the frame still fits`、`incremental rendering - leaves fullscreen without clearing when the terminal height grows`。
- 不适用，Node EventEmitter；unmount 消耗 app：`useWindowSize removes resize listener on unmount`、`useWindowSize does not crash when resize fires after unmount`。
- 不适用，npm `terminal-size` 经 `LINES` 取行数，ink-rs 由驱动查询尺寸，M2 同步驱动负责：`useWindowSize falls back to terminal-size rows when stdout.rows is missing`。
- 不适用，ink-rs 由 resize 事件获知尺寸，Ink 每帧读取 rows：`standard rendering - keeps content above the frame when the terminal height shrinks onto it and the same render moves a custom cursor`。其 incremental 变体见重复清单。

### `clear-rerender.tsx`

- 重复，incrementalRendering 变体：`rerender restores unchanged content after clear (incremental)`、`state update restores unchanged content after clear (incremental)`、`unmount does not restore cleared content (incremental)`。
- M5，屏幕阅读器：`rerender restores unchanged content after clear (screen-reader)`、`state update restores unchanged content after clear (screen-reader)`、`unmount does not restore cleared content (screen-reader)`。

### `components.tsx`

- 不适用，debug 对应 `testing::render`，没有运行时 debug 选项：`debug mode in CI does not replay final frame during unmount teardown`、`debug mode in CI keeps final newline separation after waitUntilExit`、`alternate screen - debug concurrent teardown restores the cursor before the first commit`。
- 不适用，design.md 外部输出不捕获 console：`primary screen - cleanup console output follows the native console during unmount`、`alternate screen - cleanup console output follows the native console during unmount`。
- 不适用，Rust 没有 stdout 到实例的全局映射：`render warns when stdout is reused before unmount`。
- 缺 API：`render different component based on whether stdin is a TTY or not`，见[缺 API](#缺-api)。
- M5，屏幕阅读器：`unmounting a <Static> ancestor in screen-reader mode does not replay stale output`。
- 重复，React 并发变体：`text - concurrent`、`multiple text nodes - concurrent`、`wrap text - concurrent`、`truncate text in the end - concurrent`、`transform children - concurrent`、`static output - concurrent`、`remeasure text dimensions on text change - concurrent`、`newline - concurrent`、`horizontal spacer - concurrent`、`vertical spacer - concurrent`、`unmounting a <Static> ancestor in concurrent mode does not crash`。
- 不适用，Rust 在编译期拒绝 View 下的字符串：`fail when text nodes are not within <Text> component`、`fail when text node is not within <Text> component`。

### `static-runtime-blank-lines.tsx`

- 重复，incrementalRendering 变体：`a blank Static append survives a later live update (incremental: true)`。

### `measure-element.tsx`

- 不适用：`measure element returns zeros for node without yoga`，构造没有 Yoga 节点的假元素；Rust 没有这种节点。

### `render-callback.tsx`

- 缺 API：`onRender observes the committed frame ({"incrementalRendering":false})`，本阶段不添加 onRender 回调。
- 重复：`onRender observes the committed frame ({"incrementalRendering":true})`，incrementalRendering 变体。
- 不适用：`onRender observes the committed frame ({"debug":true})`，debug 对应 `testing::render`，没有运行时选项。
- M5 / 缺 API：`onRender observes the committed frame ({"isScreenReaderEnabled":true})`，屏幕阅读器及 onRender 回调。

### `write-synchronized.tsx`

- 不适用，文件检查 Ink 内部函数与常量：`bsu is the expected synchronized update sequence`、`esu is the expected synchronized update sequence`、`shouldSynchronize returns true for interactive TTY stream`、`shouldSynchronize returns false for non-interactive TTY stream`、`shouldSynchronize returns false for non-TTY stream`、`shouldSynchronize uses CI detection when interactive is not specified`、`shouldSynchronize returns false for non-TTY stream when interactive is not specified`。
- 覆盖位置：`render.tsx` 的 `no bsu/esu when output is unchanged`、`bsu/esu wraps throttledLog trailing call` 检查交互式 TTY 的序列与无变化分支；`components.tsx` 的 `render all frames when interactive is explicitly true`、`interactive option overrides TTY detection`、`alternate screen - ignored when non-interactive`、`alternate screen - ignored when isTTY is false`、`alternate screen - ignored when isTTY is false even if interactive is true` 检查非 TTY 与非交互分支。CI 环境检测由 `src/driver/environment/tests.rs` 的表测试覆盖；两条 CI fixture 移植由 `src/driver/tests.rs` 覆盖。

## M3 文件中不移植的测试

- `parse-keypress.ts`：不适用：`parsing a single-byte Meta key does not mutate the input`、`Meta byte parsing preserves shared buffers across the full byte range`、`byte parsing preserves ASCII and multibyte UTF-8 input`。它们测 `parseKeypress` 的 `Uint8Array` 重载，App 先按 UTF-8 解码再解析，走不到这个分支。
- `kitty-keyboard.tsx`：
  - 重复：`kitty protocol - Ctrl letters retain their unshifted key code` 与 `kitty protocol - ctrl modifier` 输入相同，断言是后者的子集。
  - 不适用：`kitty protocol - preserves sequence and raw`。ink-rs 的 `ParsedKey` 不保留 `raw`，`sequence` 不经 `use_input` 交给用户。
  - 不适用：`kitty protocol - auto detection handles Uint8Array query response`。ink-rs 的输入总是字节切片，没有字符串与 `Uint8Array` 两条路径。
- `hooks-use-input.tsx`：
  - 不适用，React transition 与 discrete 优先级：`useInput - discrete priority keeps states in sync with useTransition during rapid input`。
  - 重复：`useInput - handle remove (delete)` 与 `useInput - handle delete` 输入相同，fixture 条件相同。
- `rerender-input.tsx`：不适用，Node stdin 的 `readable` 监听器：`rerender() does not detach and reattach the stdin readable listener`。
- `exit.tsx`：
  - 不适用：`exit on exit() with error with value property`。Ink 按对象属性区分错误与结果值；ink-rs 用 `exit_with_error` 与 `exit_with` 两个方法区分。
  - 不适用，React 开发模式：`exit when DEV is set`。
- `focus.tsx`：
  - 不适用：`generated focus IDs remain distinct when random values collide`。生成的 ID 来自 runtime 内计数，没有随机源；Tab 在三个生成 ID 间切换的用例检查它们互不相同。
  - React 并发模式下的重复：`focus component renders in concurrent mode`、`focus component with autoFocus renders in concurrent mode`。
- `focus-strict-mode.tsx`：只移植 `focus updates preserve call order (concurrent: false, StrictMode: false)`。并发模式的一个是重复；StrictMode 的两个不适用，ink-rs 没有 StrictMode。

## M4 文件中不移植的测试

- `use-animation.tsx`：
  - 不适用，`Duration` 无法表示：`treats NaN interval as the default interval`、`treats Infinity interval as the default interval`、`treats negative Infinity interval as the default interval`、`clamps negative interval to 1ms`。
  - 不适用，核心不读时钟：`wall clock changes do not move animations backwards`。
  - 不适用，没有并发渲染与 transition：`concurrent aborted renders do not suppress interval reset`、`suspended transitions do not reset the committed animation before commit`。
- `use-box-metrics.tsx`：
  - 不适用，Node EventEmitter；unmount 消耗 app：`removes resize listener on unmount`、`does not crash when resize fires after unmount`。
  - 并发：`tracks a box mounted by child state - concurrent=true`。
- `errors.tsx`：不适用，Node 的 unhandledRejection 与 JS realm：`does not emit unhandledRejection when render exits with an error and waitUntilExit is unused`、`waitUntilExit preserves a component error from another realm`。
- `suspend-terminal.tsx`：
  - 重复：`suspendTerminal disposable resumes via Symbol.asyncDispose` 与 `suspendTerminal returns a disposable that resumes on resume()` 相同；`suspendTerminal shows <Static> output once after resume re-enters the alternate screen - incremental` 与 standard 相同。
  - M5，屏幕阅读器：`suspendTerminal shows <Static> output once after resume re-enters the alternate screen - screen reader`。
  - 不适用，没有 debug 模式：`suspendTerminal shows <Static> output once after resume re-enters the alternate screen - debug`。
  - 不适用，Terminal 操作不抛错，I/O 错误结束应用：`suspendTerminal rolls back so a later suspend works if handover throws`。
- `suspension-handle.tsx`：不适用，没有挂起句柄对象：`an old suspension cannot resume a later suspension (dispose first: false)`、`an old suspension cannot resume a later suspension (dispose first: true)`。
- `error-overview.tsx`：不适用，React 的 key 警告：`does not emit duplicate key warnings for repeated stack lines`。错误界面是 View/Text 数据树，没有 key。

## 缺 API

- `onRender` / `renderTime`：本阶段不添加，影响 `render-callback.tsx` 和 `render.tsx` 的 `outputs renderTime when onRender is passed`。
- `waitUntilRenderFlush`：本阶段不添加。测试通过 `TestApp::advance` 驱动截止时间，Terminal 的写入为同步调用，没有 Node 回调或 Promise 屏障。相关测试名称见 M2 清单。
- `isRawModeSupported`：本阶段不添加 `use_stdin`。stdin 不是 TTY 时，`use_input` 让应用以 `ErrorKind::Unsupported` 退出，组件无法预先判断。影响 `components.tsx` 的 `render different component based on whether stdin is a TTY or not`。
- `<Text backgroundColor="">`：Ink 用空字符串表示不继承父 `Box` 的背景色。Rust 的 `Text::background_color` 收 `Option<Color>`，`None` 表示继承，没有表达"无背景"的值。影响 `background.tsx` 的 `Mixed text with and without background inheritance`。

## 与 Ink 测试的差异

- **resize 的观测边界。** `terminal-resize.tsx` 的订阅安装前 resize 用例改用 live 测试驱动：layout effect 改 TestTerminal 尺寸，下一步处理事件；初始 `100x40` 与最终 `60x20` 不变。尺寸更新和四个宽度用例把最后一次 write 的文本比较改为 vt100 屏幕比较，保留文本、边框和尺寸变化；宽度收缩用例使用 `Screen::resize` 的主屏重排模型，宽度增长不拼回折行。两种 cursor-row 收缩、宽高同时收缩、提交清除光标后收缩和 custom-cursor resize-event 用例保留原 shell + frame 内容，经过 `Screen::resize` 的 xterm 行收缩模型重建 scrollback 与屏幕；仅去掉末尾空行，不过滤内部空行。`homeAndEraseDown + frame` 的字节 pin 改为六行可见、scrollback 为被收缩推进的 `line 0`、光标在末行行首；`eraseLines(6) + frame + newline` 改为帧保留、光标在帧下一行，不断言增长拉回的行。仍放得下时保留无字节断言。`render.tsx` 的 fullscreen 换行用例改为屏幕两行、scrollback 中的 `First` 与末行行首光标；边框 resize 改为屏幕中的原 10 列和新 8 列边框；viewport shrink 保留一次 `CSI 1;1H` 后的向下擦除，并禁止 `CSI 2J`、`CSI 3J`。所有行收缩用例使用 `tests/common::Screen::resize`。原测试期望未改。移植用例之后增加两个 ink-rs 场景，不计入移植数：三个 shell 行下方的 `68 tests passed` 从 80 列缩到 10 列，以及三个宽行中第二行列偏移 12 的声明光标；两者比较去尾部空行后的完整历史与屏幕，检查旧帧擦除和 shell 历史保留。Ink 按绘制行数擦除，ink-rs 按重排行数及光标格偏移擦除。
- **app 高度上限。** `render.tsx` 的 `growing into an overflowing frame keeps <Static> rows that are still in the viewport - standard` 保留 History、S1–S3 和 `live 2 tick 1` 至 `live 4 tick 1`；顶部被裁掉的 `live 0`、`live 1` 不进入 scrollback。`alternate screen replays <Static> items added while the frame overflows once - standard` 的溢出阶段不输出被裁掉的 Static 行，检查可见屏幕无 Static；收缩后保留原期望 `S1, S2, S3, live 0 tick 3, 空行`。`cursor.tsx` 的 `standard rendering - fullscreen: cursor lands on the requested row on the sync path` 在 5 行终端中裁掉 6 行帧的 `Line 0`，可见 `Line 1!` 至 `Line 5`，声明 `(x=3,y=2)` 落到视口 `(row=1,column=3)`；不清屏，裁掉的行不进 scrollback，代替 Ink 的清屏与 cursorUp 字节断言。这些行为由 design.md 的屏高上限决定。
- **cursor 与 hooks 的观测边界。** `cursor.tsx` 的 memo、stdout/stderr 写入和满屏更新测试把 hide/show 顺序与 cursorUp + cursorTo 字节改为 vt100 的光标行、列、可见性；memo 的卸载分支保留 hide 存在且 show 不存在的字节断言。`cursor-exit-position.tsx` 的 `done returns the cursor once without erasing output` 改为屏幕内容、光标在帧下方和第二次 done 无字节；备用屏退出检查主屏、光标、可见性，并保留退出序列之后只有 show 的字节断言。`hooks.tsx` 在进程内重建 fixture，将两个流按写入顺序喂给同一个 vt100 屏幕，卸载后添加 `exited`；stdout 的 stripAnsi/换行切分改为屏幕行比较。debug 用例保留数据和重放内容的期望。`render.tsx` 的同步序列用例检查初帧包装，重渲染后无新 write。
- **app 的观测边界。** 按移植范围，整屏清除次数、按清屏序列切帧的部分改为 vt100 屏幕观测；零整屏清除断言检查 `CSI 2J`、`CSI 3J` 和 `CSI 1;1H` 后的向下擦除均未出现。`render all frames when interactive is explicitly true` 按每帧屏幕验证 Count 0–2，cell diff 不重写 Count 前缀。`side-by-side public components do not rewrite the unchanged left region` 检查左侧区域未重写及屏幕中的计数；不是锁定 Ink 的 CHA + EL。`no throttled renders after unmount` 的卸载消耗 TestApp，没有卸载后的 tick。以上不改变 Ink 的内容期望。
- **输入的观测边界。** `hooks-use-input*.tsx` 与 `exit-keyboard.tsx` 不跑 PTY fixture，在进程内用 `testing::mount` 重建 fixture：条件匹配时请求退出，不匹配时 panic，`exit_result()` 为 `Some(Ok(None))` 代替 `exited` 与退出码。单独的 Esc 和不完整序列用 `advance(20ms)` 交出；连续方向键在一次读取中送入。`hooks.tsx` 的 `useInput - ignore input if not active` 没有定时器，用保留的 `AppHandle` 代替 fixture 的 100ms 退出，检查屏幕首行为 `x`。`components.tsx` 的 raw mode 用例用 `TestTerminal::raw_mode_calls()` 的完整序列代替 `setRawMode` 的 spy，去掉 ref/unref 与监听器数量断言；关闭在步骤结束时发生，代替 Ink 的微任务。`setRawMode() should throw if raw mode is not supported` 改为应用以 `ErrorKind::Unsupported` 退出，且没有切换调用。`cursor.tsx` 的输入用例检查 vt100 光标位置与可见性；空格用例保留 write 次数增长，滚动用例保留擦除期间的隐藏光标字节。`input-keypad-enter.tsx` 把两个投影数组断言合成 `(input, enter)` 对的比较。`rerender-input.tsx` 用测试时钟代替 FakeTimers。`hooks-use-paste.tsx` 同样重建 fixture。`kitty-negotiation.tsx` 不公开 `resolveFlags`，flags 断言改为检查写出的 `CSI >24u`、`CSI >25u`。`kitty-keyboard.tsx` 的假流改为 `TestTerminal` 的 TTY 设置；同步回复用例在挂载后送入回复，此时 raw mode 已开启；卸载后的迟到回复用例经保留的 `AppHandle` 退出，再送入回复。
- **焦点的观测边界。** `focus*.tsx` 读 stdout spy 的断言改为 `testing::render` 的 `last_frame()`，期望文本不变。`undefined` 改为 `None`，空 ID 为 `Some("")`。`delay`、`act`、`waitUntilRenderFlush` 改为步进或 `settle`；单独 Esc 后的 50ms 等待改为 `advance(20ms)`。在 render 中读取 `activeId` 的用例保留 render 读取；捕获 manager 的用例在步进后读 `active_id()`。`focus.tsx` 移植 44 个用例：Ink 用表循环生成的 16 个 kitty 修饰键用例在 Rust 中是一个表测试，失败信息保留 Ink 用例名，共 29 个 `#[test]`。React 条件渲染的 `null` 改为空 fragment，保留子节点位置。
- **退出的观测边界。** `render.tsx` 的 `waitUntilExit resolves first exit value when duplicate exits happen during teardown` 没有 Node 写回调屏障，改为 effect cleanup 在卸载中请求第二次退出，测试保留的 handle 在退出后再请求一次，结果仍为第一次的值。`exit.tsx` 的 `exit on exit() with error and static output` 不跑 PTY fixture，在进程内用 `testing::mount` 读取退出结果，输出期望不变。`components.tsx` 的两个备用屏退出用例只检查错误消息，Rust 错误没有 JS 调用栈。
- **进程的观测边界。** `exit.tsx` 的 11 个进程用例、`hooks.tsx` 的 ``useInput - handle Ctrl+C when `exitOnCtrlC` is `false` `` 与 `render.tsx` 的 `#725: non-TTY child process output is flushed`、两个 `useAnimation can drive … non-interactive process exit` 在 `tests/pty.rs` 中重建 fixture；后三个走管道，不开 PTY。Node 定时器改为线程加 `AppHandle`；外部 `unmount()` 改为 `Instance::handle().exit()`，`wait_until_exit` 消耗 `Instance`。`exit when app finishes execution` 的 fixture 调用 `wait_until_exit` 驱动更新。`#725` 没有 `use_stdin`，fixture 渲染固定文本，代替 stdin 是否为 TTY 的输出。`exit on exit() with raw mode` 另检查退出后的 termios 与开启前相同。macOS 在 `ICANON` 切换后置 `PENDIN`，检查前用零超时的 poll 让内核处理，不读取输入、不屏蔽标志。`src/driver/tests.rs` 与 `src/driver/stdin.rs` 另有 4 个原生测试，覆盖 Ink fixture 不涉及的规则：相邻输入合并且不越过其他消息、EOF 后在更新完成时自然结束、输入交给最后的持有者并在其释放后交给前一个、读错误进入首个错误槽。
- **挂起的观测边界。** `suspend-terminal.tsx` 的回调用例在 effect 中调用 `suspend_terminal`。回调要求 `Send`，不能捕获 `Rc` 的 TestTerminal，改为检查 `raw_mode_calls()` 为 `[true, false, true]`。返回句柄的用例改用 `TestApp::suspend_terminal` 与 `resume_terminal`。回调抛错的用例改为 panic 向上展开，检查 raw mode 停在关闭、光标已显示。Static 用例从最后一次 `CSI ?1049h` 起交给 vt100 比较屏幕。子进程用例在 `tests/pty.rs` 中重建 fixture：回调写出 `CHILD_OUTPUT`，线程 100ms 后请求退出。`suspension-exit.tsx` 的 `unmount` 消耗 TestApp，没有卸载后的 resume。`src/app/tests.rs` 另有 2 个 Ctrl+Z 原生测试，`tests/pty.rs` 另有 bash 作业控制与会话首进程组 2 个 PTY 用例，不计入移植数。
- **log-update 的测试检查屏幕。** 移植版把输出的字节交给 vt100，检查屏幕、scrollback、光标的位置和可见性，以及没有写出字节。Ink 锁定字节的断言（`eraseLines` 的行数、`ESC E` 等）改为检查这些字节产生的屏幕。vt100 在待换行状态下执行 EL 不擦最后一格，xterm 会擦，所以 `full-width updates do not erase after the last character` 还检查字节：满宽的行之后没有 EL。
- **视口高度代替末尾换行。** Ink 的 log-update 测试为每帧选择是否带末尾换行，ink-rs 由视口高度决定：帧高达到屏高时不带。移植版为每个测试选视口高度，让被断言的帧停在 Ink 的位置。`incremental rendering - no trailing newline: shrink` 从满屏缩短，两帧无法同时对上：缩短后光标按 design.md 停在帧的下一行，Ink 的测试停在帧的最后一行。Ink 的公开 render 退出满屏后，光标也停在帧的下一行。
- **SGR 字节检查改为完整比较。** Ink 有 16 个测试用 `includes()` 检查 chalk 生成的 SGR 字节，如 `\x1b[41m`。移植版改用 `assert_ansi_eq` 比较完整输出，完整输出取自 Ink 测试运行时的记录。这样检查样式语义，不锁定 SGR 的写法。涉及 `background.tsx` 8 个、`border-backgrounds.tsx` 4 个、`borders.tsx` 1 个、`text.tsx` 3 个。
- **非 SGR 序列的检查不变。** `text.tsx` 中检查光标移动、DCS、SOS 等序列被剥离、OSC 超链接被保留的断言，与 Ink 一样用 `contains`。
- **`colorize.ts`**：Rust 版测颜色字符串的解析（`"…".parse::<Color>()`），生成 SGR 的部分由渲染测试覆盖。
- **`borders.tsx` 的竖边框用例**：Ink 的正则含未转义的 `|`，断言弱于测试名的意图。移植版比较 7 行完整输出。
- **`text.tsx` 的 hex、rgb、ansi256 颜色用例（6 个）**：Ink 的期望值由同一个 chalk 实例生成，与色彩级别无关。移植版的期望值是 truecolor 的输出，调用改用 `render_truecolor`。
- **`background.tsx` 中 Static 背景色的用例**：Ink 的测试经 `render()` 的 debug 模式取输出，只有 Static 时末尾带 `\n`。`render_to_string` 按 Ink `renderToString` 的规则去掉这个换行，移植版的期望值随之去掉末尾空行。
- **`overflow.tsx` 的越界写入用例**：`columns: 10` 时，Ink 把右边框写到行宽之外，JS 稀疏数组留下的空洞在拼接时被跳过，内部行宽 11 格。移植版按网格补空格，内部行与边框行同宽，都是 12 格。
- **`c1-rendering.tsx` 宽度 4 的超链接用例**：wrap-ansi 折行时用 BEL 结尾重开超链接，叠在原链接内，第二行出现两次打开、两次关闭。移植版每段只打开、关闭一次，保留原终结符。
- **`flex-align-self.tsx` 的 `row - align self baseline`**：Yoga 在有子项按 baseline 对齐时按内容决定行高，`flex-end` 的子项随之贴在内容行的底部。taffy 按 CSS，单行容器的行高等于容器高度，移植版的期望值按 CSS 改写。

- **layout effect 写状态时少写一帧。** Ink 在提交时写出，之后执行 layout effect，状态变了写第二帧。ink-rs 在 paint 前执行 layout effect，一帧写出一次。按写出次数或第一帧断言的测试随之调整。
- **`use_box_metrics` 首帧给值。** Ink 先写出全 0 的帧，passive effect 加微任务后写出测量值。ink-rs 在提交的每一轮刷新 ref 之后比较读者的度量，变了就在 paint 前重渲染，View 首次布局的那一帧即带测量值。`does not trigger extra re-renders when layout is unchanged` 的渲染次数由 Ink 的 2–3 次改为 2 次（未挂载一次、测量后一次）；两个 resize 用例改用 `testing::mount` 与 vt100 屏幕。
- **`use_animation` 的观测边界。** 假时钟逐个运行截止时间，代替 FakeTimers 的 `tickAsync`，时间下界改为精确值。不断言 JS 定时器的创建与清除次数，共享定时器由 `src/runtime/tests.rs` 的截止时间测试覆盖。`reset` 是句柄，`reset is a stable function reference` 改为用挂载时的副本重置。恢复与改 interval 的归零在 layout effect 中写状态，paint 前重算，不写过时帧。`animations advance in debug mode when interactive is false` 用非交互的 `testing::mount` 与 120 fps，ink-rs 没有运行时 debug 选项。
- **示例的观测边界。** `render.tsx` 的 16 个示例用例与 `alternate-screen-example.tsx` 不在 PTY 中运行示例进程：`tests/examples.rs` 以 `#[path]` 引入 `examples/<name>/app.rs`，用 `testing::mount` 在进程内挂载，终端尺寸同 Ink `term` 的参数（默认 100 列）。`waitForOutput` 改为越过节流窗口的 `advance`，读 vt100 屏幕。`eraseLines`、`cursorTo`、`cursorUp` 的字节检查改为屏幕行、光标位置与可见性；输出流上的正则改为屏幕行比较，`input example` 在重绘块中的行号改为屏幕的物理行。table 用例用 40 列的 `render_to_string` 代替 CI 子进程，JS 字符串长度改为 `text_width`。aria 用例以 `interactive: Some(false)` 代替 `CI=true`，N 个空格在一次 `input` 中送入，卸载后读最终帧。snake 的反向用例用假时钟推进一个 150ms 的 tick，代替等待 `Score:` 重绘。`alternate-screen-example.tsx` 的两个用例调用 `reduce`，代替解析子进程的 JSON，去掉 1 秒的 kill 计时器。incremental-rendering 示例截断长行，根节点取内容高度：ink-rs 按屏高裁掉帧顶部的行（见 app 高度上限），Ink 的测试读输出流，不受裁剪影响；截断后 30 行的终端放得下标题和选中项。期望文本不变。
- **错误界面的观测边界。** 位置取 Rust 的 `file:line:column`，不是 file URL，`renders source excerpts for file URLs containing spaces` 改为含空格的文件路径。调用栈只在设置 `RUST_BACKTRACE` 或 `RUST_LIB_BACKTRACE` 时显示，`catch and display error` 比较去色后的前 12 行（标签与消息、位置、7 行源码摘录），不比较 Ink 的栈行；两个 native 栈帧用例喂入构造的 std 格式 backtrace 文本。`ErrorBoundary catches and displays nested component errors` 把抛错组件放在 `Text` 中。`clean up raw mode when error is thrown` 用 `raw_mode_calls()` 为 `[true, false]` 代替 stdin spy。抛字符串、抛 undefined 改为 `panic_any(String)`、`panic_any(())`，后者的消息为 `Box<dyn Any>`。`waitUntilExit preserves the original component error` 经 `take_exit_result` 与 `into_payload` downcast 出 `Arc`，用 `Arc::ptr_eq` 比较身份。`render.tsx` 的 `should reject waitUntilExit when app exits during synchronous render error handling` 没有用户错误边界，改为首次渲染 panic，结果为 `Err(Panic)`。`exit.tsx` 的 `exit with thrown error` 在 `tests/pty.rs` 中重建 fixture，`main` 打印返回的错误；增加检查错误界面、终端恢复与退出码 0，fixture 关闭 backtrace。`render-to-string.tsx` 的两个抛错用例用外层 `catch_unwind` 接住 panic。
- **Static 追加时写出一次。** Ink 的 Static 靠 index 状态和 layout effect 隐藏已写出的条目，每次追加提交两次、写出两次。ink-rs 的 Static 是 host 节点，追加时写出一次。

## Ink 期望中反直觉的行为

测试以 Ink 的行为为准：

- 输入中的原始 SGR（含 RGB、256 色）保留原序列，不随 `color_level` 降级。
- `truncate-width.tsx`：中间截断遇到宽字符，结果是 `你…`，与末尾截断相同。
- ANSI 分词：单独出现的 `ESC \` 记为 `Esc`，不记为 ST。
- `wrapText("hello world", 5)` 的结果中有一行是单个空格。
- `padding.tsx` 的宽度用例按 UTF-16 长度判断每行不超过 40，对应 Ink 的 `line.length`。
- **输入解析的字段。** `ParsedKey` 不保留 `raw` 与 `code`，App 和 `use_input` 不读这两个字段。`parse-keypress.ts` 的 `application keypad Enter maps to Return with carriage return sequence` 去掉 `raw === undefined` 断言；`kitty-keyboard.tsx` 的 `non-kitty sequences fall back to legacy parsing`、`non-kitty sequences - ctrl+c` 把 `isKittyProtocol === undefined` 改为 `false`，`kitty protocol - malformed modifier 0 does not set all flags` 的 `super ?? false` 改为布尔字段。字节输入改为 UTF-8 字节切片，Ink 的字符串与 paste 对象改为 `InputEvent` 枚举。ESC 加单字符的 meta 规则（`metaKeyCodeRe` 的 `\p{C}`）只排除 Cc：键盘不会在 ESC 后发送格式、私用或未分配字符，Cn 随 JavaScript 引擎的 Unicode 版本变化。`src/input/parser/tests.rs` 另有一个 ink-rs 测试，覆盖非法 UTF-8 的替换、跨读取的最大子部分替换与 reset 清空残留字节，Ink 的测试没有覆盖这些情况。

## 上游跳过的测试

4 个，期望值取自 Ink 源码：`width-height.tsx` 中百分比 `minWidth`、`maxWidth` 各一个，Yoga 不支持；`flex-justify-content.tsx` 中 `space-around` 遇到文本节点的两个，Yoga 有缺陷。百分比 `minWidth` 在 taffy 下通过，照常运行；其余 3 个标 `#[ignore]`，能通过时去掉。
