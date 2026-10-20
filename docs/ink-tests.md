# Ink 测试移植状态

基准：Ink 8.0.0（commit 26d2c3f）。移植方法见 [design.md](design.md#移植-ink-测试的方法)。

## 总览

Ink 有 95 个测试文件、1,646 个测试。

- **M0**：40 个文件、755 个测试。移植 615 个，写成 619 个 `#[test]`（`content-offset.tsx` 的一个参数化测试按参数拆成 5 个），上游跳过的 4 个中，3 个标 `#[ignore]`，1 个在 taffy 下通过。未移植 140 个，原因见下表，明细见[下文](#m0-文件中未移植的测试)。
- **其他 55 个文件**（891 个测试）：50 个随 M1–M5 移植，5 个不适用。M1 文件中不移植的测试见[下文](#m1-文件中不移植的测试)。

| 未移植原因 | 数量 | 处理 |
|---|---:|---|
| React 并发模式下的重复测试（`… - concurrent`） | 46 | 不移植 |
| 重渲染、state、effect | 35 | M1 |
| Static 跨帧、非 TTY / CI、备用屏幕 | 32 | M2 |
| 输入、raw mode | 6 | M3 |
| 组件抛错 | 2 | M4 |
| 屏幕阅读器 | 2 | M5 |
| 缺 API | 1 | 见[缺 API](#缺-api) |
| 不适用 | 16 | 不移植 |

## 按文件

里程碑为 M0 的文件，表中是移植的部分，其余见下一节。"—" 表示不适用：`build-output.ts` 检查 npm 包的构建产物，`is-devtools-reachable.tsx` 测 React DevTools 的连接，`style-types.ts` 是 TypeScript 类型检查，Rust 由类型系统保证。`styles.ts` 直接调用 Yoga 的 API，去掉样式后的行为由各布局文件的重渲染测试和 `style-update-consistency.tsx` 覆盖。`static-abandoned-render.tsx` 依赖 Suspense 和 transition。

| Ink 文件 | Ink 测试数 | 里程碑 | 移植的 `#[test]` | Rust 文件 |
|---|---|---|---|---|
| `absolute-truncation.tsx` | 4 | M0 | 4 | `tests/absolute_truncation.rs` |
| `alternate-screen-example.tsx` | 2 | M4 |  |  |
| `ansi-newlines.tsx` | 9 | M0 | 9 | `tests/ansi_newlines.rs` |
| `ansi-tokenizer.ts` | 26 | M0 | 26 | `src/text/ansi/tests.rs` |
| `background.tsx` | 32 | M0 / M1 | 27 | `tests/background.rs` |
| `border-backgrounds.tsx` | 5 | M0 | 5 | `tests/border_backgrounds.rs` |
| `borders.tsx` | 52 | M0 / M1 | 48 | `tests/borders.rs` |
| `build-output.ts` | 4 | — |  |  |
| `c1-rendering.tsx` | 8 | M0 | 8 | `tests/c1_rendering.rs` |
| `clear-rerender.tsx` | 9 | M2 |  |  |
| `clip-wide-background.tsx` | 10 | M0 | 10 | `tests/clip_wide_background.rs` |
| `colon-colors.tsx` | 16 | M0 | 16 | `tests/colon_colors.rs` |
| `colorize.ts` | 16 | M0 | 16 | `tests/colorize.rs` |
| `component-regressions.tsx` | 4 | M1 | 2 | `tests/component_regressions.rs` |
| `components.tsx` | 93 | M0 / M1 | 41 | `tests/components.rs` |
| `content-offset.tsx` | 23 | M0 | 24 | `tests/content_offset.rs` |
| `cursor-exit-position.tsx` | 6 | M2 |  |  |
| `cursor-helpers.tsx` | 17 | M2 |  |  |
| `cursor.tsx` | 20 | M2 |  |  |
| `display.tsx` | 6 | M0 / M1 | 4 | `tests/display.rs` |
| `error-overview.tsx` | 7 | M4 |  |  |
| `errors.tsx` | 8 | M4 |  |  |
| `exit-keyboard.tsx` | 7 | M3 |  |  |
| `exit.tsx` | 15 | M3 |  |  |
| `flex-align-content.tsx` | 13 | M0 / M1 | 11 | `tests/flex_align_content.rs` |
| `flex-align-items.tsx` | 9 | M0 | 9 | `tests/flex_align_items.rs` |
| `flex-align-self.tsx` | 9 | M0 | 9 | `tests/flex_align_self.rs` |
| `flex-direction.tsx` | 10 | M0 / M1 | 7 | `tests/flex_direction.rs` |
| `flex-justify-content.tsx` | 12 | M0 | 12 | `tests/flex_justify_content.rs` |
| `flex-wrap.tsx` | 8 | M0 / M1 | 8 | `tests/flex_wrap.rs` |
| `flex.tsx` | 8 | M0 | 8 | `tests/flex.rs` |
| `focus-empty-id-regression.tsx` | 3 | M3 |  |  |
| `focus-order.tsx` | 3 | M3 |  |  |
| `focus-strict-mode.tsx` | 4 | M3 |  |  |
| `focus.tsx` | 47 | M3 |  |  |
| `gap.tsx` | 8 | M0 / M1 | 5 | `tests/gap.rs` |
| `hooks-use-input-kitty.tsx` | 24 | M3 |  |  |
| `hooks-use-input-navigation.tsx` | 18 | M3 |  |  |
| `hooks-use-input.tsx` | 34 | M3 |  |  |
| `hooks-use-paste.tsx` | 4 | M3 |  |  |
| `hooks.tsx` | 6 | M3 |  |  |
| `input-buffered-ctrl-c.tsx` | 5 | M3 |  |  |
| `input-keypad-enter.tsx` | 2 | M3 |  |  |
| `input-parser.ts` | 65 | M3 |  |  |
| `is-devtools-reachable.tsx` | 3 | — |  |  |
| `issue-973-static-commit.tsx` | 1 | M2 |  |  |
| `kitty-keyboard.tsx` | 77 | M3 |  |  |
| `kitty-negotiation.tsx` | 5 | M3 |  |  |
| `line-update.ts` | 29 | M2 |  |  |
| `log-update-blank-growth.ts` | 4 | M2 | 3 | `src/output/tests/log_update_blank_growth.rs` |
| `log-update-columns.tsx` | 8 | M2 | 2 | `src/output/tests/log_update_columns.rs` |
| `log-update-sparse.ts` | 9 | M2 | 8 | `src/output/tests/log_update_sparse.rs` |
| `log-update.tsx` | 52 | M2 | 31 | `src/output/tests/log_update.rs` |
| `margin.tsx` | 15 | M0 / M1 | 13 | `tests/margin.rs` |
| `measure-element.tsx` | 13 | M1 | 11 | `tests/measure_element.rs` |
| `measure-text.tsx` | 13 | M0 | 11 | `src/text/measure/tests.rs` |
| `overflow.tsx` | 44 | M0 | 40 | `tests/overflow.rs` |
| `overlap-wide-background.tsx` | 7 | M0 | 7 | `tests/overlap_wide_background.rs` |
| `padding.tsx` | 16 | M0 / M1 | 14 | `tests/padding.rs` |
| `parse-keypress.ts` | 27 | M3 |  |  |
| `position.tsx` | 13 | M0 / M1 | 11 | `tests/position.rs` |
| `reconciler.tsx` | 14 | M1 | 7 | `tests/reconciler.rs` |
| `render-callback.tsx` | 4 | M2 |  |  |
| `render-to-string.tsx` | 37 | M0 / M1 | 32 | `tests/render_to_string.rs` |
| `render.tsx` | 119 | M2 |  |  |
| `rendering-regressions.tsx` | 2 | M0 | 2 | `tests/rendering_regressions.rs` |
| `rerender-input.tsx` | 4 | M3 |  |  |
| `sanitize-ansi.ts` | 50 | M0 | 50 | `src/text/sanitize/tests.rs` |
| `screen-reader.tsx` | 33 | M5 |  |  |
| `squash-text-nodes.tsx` | 2 | M0 | 2 | `tests/squash_text_nodes.rs` |
| `static-abandoned-render.tsx` | 1 | — |  |  |
| `static-blank-lines.tsx` | 9 | M0 | 9 | `tests/static_blank_lines.rs` |
| `static-runtime-blank-lines.tsx` | 8 | M2 |  |  |
| `static-string-replacement.tsx` | 4 | M1 | 4 | `tests/static_string_replacement.rs` |
| `static-trailing-layout.tsx` | 7 | M0 | 7 | `tests/static_trailing_layout.rs` |
| `style-types.ts` | 1 | — |  |  |
| `style-update-consistency.tsx` | 15 | M1 | 15 | `tests/style_update_consistency.rs` |
| `styled-combining-marks.tsx` | 9 | M0 | 9 | `tests/styled_combining_marks.rs` |
| `styles.ts` | 17 | — |  |  |
| `suspend-terminal.tsx` | 14 | M4 |  |  |
| `suspension-exit.tsx` | 2 | M4 |  |  |
| `suspension-handle.tsx` | 2 | M4 |  |  |
| `suspension-input-disable.tsx` | 7 | M4 |  |  |
| `suspension-output.tsx` | 3 | M4 |  |  |
| `suspension-resize.tsx` | 2 | M4 |  |  |
| `terminal-resize.tsx` | 28 | M2 |  |  |
| `text-controls.tsx` | 24 | M0 | 24 | `tests/text_controls.rs`, `src/text/sanitize/tests.rs` |
| `text-width.tsx` | 18 | M0 | 18 | `tests/text_width.rs` |
| `text.tsx` | 57 | M0 / M1 | 49 | `tests/text.rs` |
| `truncate-width.tsx` | 9 | M0 | 9 | `tests/truncate_width.rs` |
| `use-animation.tsx` | 45 | M4 |  |  |
| `use-box-metrics.tsx` | 19 | M4 |  |  |
| `width-height.tsx` | 29 | M0 / M1 | 25 | `tests/width_height.rs` |
| `wrap-text.tsx` | 17 | M0 / M1 | 15 | `tests/wrap_text.rs`, `src/text/wrap/tests.rs` |
| `write-synchronized.tsx` | 7 | M2 |  |  |

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
- M2，Static 跨帧：`static padding is not emitted again when there are no new items`、`skip previous output when rendering new static output`、`static output stops accumulating after Static unmounts (#904)`、`fullStaticOutput is reset when <Static> unmounts so stale items are not replayed`、`unmounting an ancestor of <Static> clears staticNode and does not crash the renderer`、`removing a <Static> ancestor that is a direct child of the root does not crash`、`separate Ink instances do not clobber each other’s staticNode`、`updating <Static> in one instance after another instance mounted <Static> sets the dirty flag on the correct root`、`remounting <Static> via key change emits the new items (nested under <Box>)`、`remounting <Static> via key change emits the new items (root-level — removeChildFromContainer)`、`render only new items in static output on final render`
- M2，非 TTY / CI：`render only last frame when run in CI`、`render all frames if CI environment variable equals false`、`debug mode in CI does not replay final frame during unmount teardown`、`debug mode in CI keeps final newline separation after waitUntilExit`、`render only last frame when stdout is not a TTY`、`render all frames when interactive is explicitly true`、`interactive option overrides TTY detection`、`static output is written immediately in non-interactive mode`
- M2，备用屏幕与卸载：`alternate screen - enters on mount and exits on unmount`、`primary screen - cleanup console output follows the native console during unmount`、`alternate screen - does not replay exit(Error) output on the primary screen during unmount`、`alternate screen - does not replay teardown output on the primary screen during unmount`、`alternate screen - cleanup console output follows the native console during unmount`、`alternate screen - cleanup() exits the alternate screen`、`alternate screen - debug concurrent teardown restores the cursor before the first commit`、`render warns when stdout is reused before unmount`、`alternate screen - ignored when non-interactive`、`alternate screen - disabled by default`、`alternate screen - content is rendered between enter and exit`、`alternate screen - ignored when isTTY is false`、`alternate screen - ignored when isTTY is false even if interactive is true`
- M3：`disable raw mode when all input components are unmounted`、`do not disable raw mode when swapping components that use useInput`、`clear pending input parser state when swapping components that use useInput`、`re-ref stdin when input is used after previous unmount`、`setRawMode() should throw if raw mode is not supported`、`render different component based on whether stdin is a TTY or not`
- M5：`unmounting a <Static> ancestor in screen-reader mode does not replay stale output`
- 并发：`text`、`multiple text nodes`、`wrap text`、`truncate text in the end`、`transform children`、`static output`、`remeasure text dimensions on text change`、`newline`、`horizontal spacer`、`vertical spacer`，以及 `unmounting a <Static> ancestor in concurrent mode does not crash`
- 不适用：`fail when text nodes are not within <Text> component`、`fail when text node is not within <Text> component`。Rust 在编译期拒绝 `View` 下的字符串。

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
- M4：`component that throws propagates the error`、`subsequent calls work after a component error`
- 不适用：`preserves component errors from another realm`（JS realm）、`component that throws undefined does not silently return empty output`（JS 能抛任意值）、`text outside Text component throws`（Rust 编译期拒绝）

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
  - M2：`calculate layout while rendering is throttled`，依赖节流和非 debug 输出。
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
  - M2，需要实例：`side-by-side public components do not rewrite the unchanged left region`。
- `log-update-sparse.ts`：不适用：`the screen reducer preserves skipped cells until they are erased`，它测的是 Ink 的测试辅助函数 `reconstructTerminalLines`。
- `log-update-blank-growth.ts`：M2，需要实例：`growing a rendered Box with an empty row preserves terminal history`。

## 缺 API

- `<Text backgroundColor="">`：Ink 用空字符串表示不继承父 `Box` 的背景色。Rust 的 `Text::background_color` 收 `Option<Color>`，`None` 表示继承，没有表达"无背景"的值。影响 `background.tsx` 的 `Mixed text with and without background inheritance`。

## 与 Ink 测试的差异

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
- **Static 追加时写出一次。** Ink 的 Static 靠 index 状态和 layout effect 隐藏已写出的条目，每次追加提交两次、写出两次。ink-rs 的 Static 是 host 节点，追加时写出一次。

## Ink 期望中反直觉的行为

测试以 Ink 的行为为准：

- 输入中的原始 SGR（含 RGB、256 色）保留原序列，不随 `color_level` 降级。
- `truncate-width.tsx`：中间截断遇到宽字符，结果是 `你…`，与末尾截断相同。
- ANSI 分词：单独出现的 `ESC \` 记为 `Esc`，不记为 ST。
- `wrapText("hello world", 5)` 的结果中有一行是单个空格。
- `padding.tsx` 的宽度用例按 UTF-16 长度判断每行不超过 40，对应 Ink 的 `line.length`。

## 上游跳过的测试

4 个，期望值取自 Ink 源码：`width-height.tsx` 中百分比 `minWidth`、`maxWidth` 各一个，Yoga 不支持；`flex-justify-content.tsx` 中 `space-around` 遇到文本节点的两个，Yoga 有缺陷。百分比 `minWidth` 在 taffy 下通过，照常运行；其余 3 个标 `#[ignore]`，能通过时去掉。
