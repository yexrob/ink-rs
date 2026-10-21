# 核心组件范围

- 日期：2026-10-20
- 状态：已实施
- 提交：无
- Brief：无

## 背景

用户问「有哪些component呢 宏观来看」，接着问「这些组件够不够」。这一轮只讨论，没有写代码。

## 决策

以下由 Claude 提出；用户下一句是「可以 推进M0吧」，没有提出异议，视为接受。

- 核心保留 6 个组件，各对应 Ink 导出的一个：`View`（Ink 的 `Box`）、`Text`、`Newline`、`Spacer`、`Transform`、`Static`。
- 判断核心组件的标准：需要 host tree、paint 或 terminal 层专门处理，没法用别的组件拼出来。按这个标准原语有 4 个：`View`、`Text`、`Static`、`Transform`。`Newline`、`Spacer` 是语法糖，为和 Ink 对齐保留。
- 输入框、选择列表、spinner 这类控件不进核心，以后放进单独的 crate，只用公开 API 写，用来检验 API 够不够。
- 错误处理只做应用级 panic 界面（M4），不做子树级错误边界。
- v1 不做鼠标输入，和 Ink 一致。

## 备选与取舍

- **子树级错误边界**：要靠 `catch_unwind`，它在 `panic = "abort"` 下不起作用，被打断的组件 state 可能停在改了一半的状态。
- **控件进核心**：Ink 核心也没有，它们在社区包里（ink-text-input、ink-select-input、ink-spinner）。

## 方案

Claude 拿 design.md 和 Ink 的导出列表对照，找出设计文档漏掉的三项：

- context：Ink 靠 React context 传 app、焦点等状态，ink-rs 需要自己的 `provide` / `use_context`，放 M1。
- `useStdin` / `useStdout` / `useStderr`：`useStdout().write` 和 `Static` 有重叠，做不做未定。
- `useIsScreenReaderEnabled`：放 M5，和屏幕阅读器模式一起。

常见控件用公开 API 的组合方式：滚动区域用 `overflow` 加 `content_offset` 加 `use_box_metrics`；输入框用 `use_input` 加 `use_paste` 加光标；spinner 用 `use_animation`；弹窗用 absolute 定位。

## 结果

- 没有提交。context 的落地和 `useStdout` 类 hooks 的取舍在后续里程碑的记录里，不在本范围。
