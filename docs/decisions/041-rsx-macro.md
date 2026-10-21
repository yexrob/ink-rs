# M5：宏 DSL `rsx!`

- 日期：2026-10-20
- 状态：已实施
- 提交：`b7c323b` feat(macros): add the rsx! macro
- Brief：[m5-rsx](briefs/m5-rsx.md)

## 背景

M4 完成后用户说「好的 接着M5吧」。M5 原计划包含屏幕阅读器模式和宏 DSL。Claude 自己读了 Ink 的屏幕阅读器实现（没有派调研 agent），给出两块的方案，列出 4 个待定点，其中两个属于宏：

- 语法：A 是 rsx 风格 `Name(位置参数, 方法: 值) { 子节点 }`，B 是 JSX 风格 `<Name prop={..}>`。Claude 推荐 A。
- 名字：`ui!`、`element!` 或 `rsx!`。Claude 推荐 `ui!`。

design.md 定过两条边界：宏是 builder 的语法糖，能力不超出 builder；加过程宏时拆成 workspace。

用户答「屏幕阅读器 是必须的吗？\n用rsx」。屏幕阅读器部分见 [042](042-defer-screen-reader.md)。

## 决策

- 语法用 A，名字用 `rsx!`。Claude 把「用rsx」理解为同时回答语法和名字两问，记录中未见用户对这一理解的确认或异议。
- 没有裸开关：写 `bold: true`，不能只写 `bold`。Claude 决定。原因：`Text(content)` 里的 `content` 可能是 `Text::new` 的参数，也可能是开关，宏无法区分，所以裸标识符一律当位置参数；dioxus 也这样处理。这一点和给用户看的预览不同，派发后向用户说明。
- 每个元素展开为一个块，builder 值逐条语句传递，`if` / `for` 的各分支不需要同一类型。builder 变量用 `Span::mixed_site()`，用户代码看不到。
- 字符串字面量含 `{` 时展开为 `format!`，只捕获标识符。
- 展开里自带 `use ::ink_rs::Styled as _;`，调用方不用导入 `Styled`。
- 不做组件语法糖，组件仍写 `{component(f, props)}`。
- 新 workspace 成员 `macros/`（`ink-rs-macros`），由默认开启的 feature `macros` 引入。

结果见 [写法](../design.md#写法)。

## 备选与取舍

- B（JSX 风格）：Ink 和 React 用户熟悉，但需要位置参数的元素（`Static(items, ..)`、`Transform(..)`）要另设属性，还要写闭合标签。
- 名字 `ui!`：Claude 原推荐，用户选了 rsx。

## 方案

- 派 Codex，worktree `wt/m5-rsx`。方案中宏和屏幕阅读器第一批改的文件不重叠，可以并行。派发时屏幕阅读器还在等用户决定，宏先行。
- Brief 要点：
  - 语法规则（`Node`、`Element`、`Args`），`if` / `for` 的条件用 `parse_without_eager_brace`，位置参数在命名参数之后报错。
  - 展开规则，`key` 最后调用；一个根返回 builder 值，多个根合成 `Element::fragment`；根为字符串、`if`、`for` 时报错。
  - 保留用户 span，拼错的方法名在原位置报 "no method named"。
  - 依赖只用 Cargo.lock 里已有的 syn、quote、proc-macro2。
  - `tests/rsx.rs` 5 个测试：同一棵树用宏和 builder 各写一遍，比较 `render_to_string`，并对照一个写死的期望字符串。不加 trybuild。
- 验收：两种 feature 配置的 build 与 clippy、`--test rsx`、doc 测试、`cargo doc -D warnings`。

## 结果

- Codex 无实现偏差。集成后全量 1317 个测试通过，另有 1 个文档测试，PTY 28 个通过；`--no-default-features` 下 1303 个通过。
- Claude 按实现改写 `docs/design.md` 的 workspace 段和宏 DSL 段。
- 用户随后要求把 demo（`~/tmp/ink-rs-demo`）改写成 rsx。改写中暴露两条用法限制：字符串里只能写变量名，字段访问要先存成局部变量；变量作子节点要加花括号，否则被当成组件名。两条写进示例改写的共用 brief，见 [044](044-examples-rsx.md)。
