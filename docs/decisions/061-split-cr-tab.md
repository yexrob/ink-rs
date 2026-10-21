# 输入解析拆开 CR 与 TAB

- 日期：2026-10-21
- 状态：实施中
- 提交：无
- Brief：无，主 agent 自己做

## 背景

eink-rs 移植 Ink UI 的输入组件时（eink-rs 的记录 002）发现：一次读取送进 `"\r\r"`，TextInput 的值变成 `"ab\r\r"`。ink-rs 照 Ink 的 `input-parser.ts`，只把 BS、DEL、Ctrl+C 拆成单独的事件，CR 和 TAB 留在文本里；Ink 的理由是粘贴的文本里会有这两个字节。两个键被一次读到时（按住不放、程序忙时快速输入、SSH 延迟），Enter 和 Tab 成了文本：TextInput 把 `\r` 插进值里，不提交；ConfirmInput、Select 丢掉 Enter；按住 Tab 时焦点不动。

用户让 Claude「处理， 和做Pastel 并行」。Claude 给了三个方案，用户问「这个影响是什么」；Claude 说明影响后，用户答「拆键的那个按照你的来」。

## 决策

- CR（0x0d）和 TAB（0x09）同 BS、DEL、Ctrl+C，各成一个事件；其余字符仍连成文本。用户定（「拆键的那个按照你的来」，Claude 推荐的是这一项）。
- 以下由 Claude 按同一授权定：
  - 只加这两个字节。raw mode 下 Enter 发 CR，Tab 发 TAB，这次的问题只涉及这两个键；LF 等其他控制字节不变。
  - 移植自 Ink 的四个测试改名、改期望，注释写明偏离 Ink 的原因：解析器的 `does_not_split_pasted_carriage_return_from_text`、`does_not_split_pasted_tab_from_text`，`hooks_use_input` 的 `use_input_pasted_carriage_return`、`use_input_pasted_tab`。
  - `docs/design.md` 的输入一节写明偏离。
  - 推送和发布 0.1.3 等用户批准。eink-rs 等 0.1.3 发布后升级依赖。

## 备选与取舍

- **paste 模式开着时才拆**：只修好输入框。ConfirmInput、Select 和焦点切换在没有 `use_paste` 时照样丢键。
- **只拆整块都是 CR 或 TAB 的读取**：只修好按住不放，`"a\r"` 这类快速输入仍是文本。
- 选定方案的代价：不用 `use_paste` 的程序，终端没包粘贴标记时，粘贴的多行文本变成「一行 + Enter」。开了 `use_paste` 的程序靠 bracketed paste 保持粘贴的文本完整，eink-rs 的输入框都开了。

## 方案

主 agent 自己做：改 `src/input/parser.rs` 的 `split_control_bytes` 和四个测试，更新 `docs/design.md`，跑全量测试、clippy、fmt，提交。
