# 屏幕阅读器模式延后

- 日期：2026-10-20
- 状态：已延后
- 提交：`76165b7` docs: defer the screen reader mode
- Brief：无

## 背景

M5 方案（见 [041](041-rsx-macro.md)）的第一块是屏幕阅读器模式，对应约 47 个 Ink 测试。Claude 自己读了 Ink 的实现：开关 `isScreenReaderEnabled`，默认取 `INK_SCREEN_READER`；打开后把树转成纯文本，按 `aria-role`、`aria-state`、`aria-label`、`aria-hidden` 加前缀或替换；交互写出按列数折行，整段擦除重写，不节流。

Claude 的 ink-rs 方案分两批：SR-1 做 API、协调阶段的 label/hidden 替换、文本渲染器；SR-2 做按行写出器和各种交互路径。方案里有两个待定点：label 和 hidden 在协调阶段处理（被隐藏的子树不挂载，同 Ink）；读屏模式不设屏高上限（同 Ink）。

用户问「屏幕阅读器 是必须的吗？」。Claude 回答：
- 它服务于用 VoiceOver、NVDA 的盲人和低视力用户，普通模式的局部重写读出来是碎片。
- 不做时其他功能不受影响；从 Ink 迁移、写了 `aria-*` 的应用要删掉这些属性；约 47 个测试记为不移植。
- 做的代价是两批 Codex，新增文本渲染器、按行写出器和 aria API，`app.rs` 的外部写入、`clear`、resize、挂起、卸载都要加分支。
- 它是独立模块，以后补和现在做代价相近。建议：公开发布给别人用就做，主要自用就先跳过。

## 决策

- 屏幕阅读器模式延后，不排期。用户决定：「先延后吧」。
- M5 只剩宏 DSL，验收标准定为"宏和 builder 的渲染结果相同"。Claude 据此改写里程碑。

当前状态见 [里程碑](../design.md#里程碑) 表下的说明。

## 备选与取舍

- 现在做（SR-1、SR-2 两批）：与 Ink 对齐，代价见背景。用户没有选。
- 方案中的两个待定点（协调阶段替换、无屏高上限）随延后搁置，没有定。

## 方案

主 agent 自己改文档和注释，没有派 worker：
- `docs/design.md`：M5 一行只留宏 DSL，表下加"读屏模式延后，不排期"。
- `docs/ink-tests.md`：约 47 个读屏用例的标记从 "M5" 改为"延后"。
- `examples/aria/app.rs`、`examples/select-input.rs`、`tests/suspend_terminal.rs` 中写着 "M5" 的注释改为延后。
- 提交前跑 `cargo fmt --check` 和 `cargo build --offline --examples`。

## 结果

- 提交 `76165b7`。
- 发布用的 README 写明屏幕阅读器模式没有移植（见 [043](043-release-prep.md)）。
