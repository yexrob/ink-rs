# 骨架与 M0 测试移植

- 日期：2026-10-19 至 2026-10-20
- 状态：已实施
- 提交：`c23bb71` feat: scaffold crate and port Ink M0 tests
- Brief：[porting](briefs/porting.md)、[components](briefs/components.md)、[borders](briefs/borders.md)、[overflow](briefs/overflow.md)、[text](briefs/text.md)、[internal](briefs/internal.md)、[layout](briefs/layout.md)、[sgr](briefs/sgr.md)

## 背景

用户认可方案后要求：「先搭建一个骨架 然后把Ink的测试先搬过来吧」。方案里定了以 Ink 测试集为规格（见 [001](001-goal-and-approach.md)），M0 的验收是移植过来的布局和文本测试。

## 决策

- 测试期望值从 Ink 自己的测试运行中抓取，不手算。Claude 决定，依据是 Ink 用 boxen、chalk 算出许多期望值。
- 骨架先定公开 API，算法全部写成 `todo!()`；移植的测试能编译，运行时失败。Claude 决定。
- worker 先后换了两次，都由用户指定：先派 6 个 opus-high；用户说「使用Opus medium来实际编写」，改派 opus-medium；用户再说「使用codex来实际编写吧」，最终由 Codex 写。
- 用 `includes()` 检查 chalk SGR 字节的测试，改成整段语义比较 `assert_ansi_eq`，不锁定某一种编码。Claude 决定。
- `.gitignore` 和首个提交：用户要求「init git ignore and commit」。

## 方案

准备（主 agent）：

- 克隆 Ink 到 commit `26d2c3f`，挂一个抓取用的 preload 跑它的测试：1,646 个，1,641 通过，5 个上游 skip。每个断言的期望值写成 Rust 表达式。
- `tests/common` 提供 `styled()` 标记和 `assert_ansi_eq`（按 SGR 语义比较）。
- 清点 Ink 测试：约 566 个静态渲染、391 个单元、673 个运行时、16 个未归类。M0 范围 40 个文件。
- 骨架 API：`View`、`Text`、`Static`、`Transform`、`Newline`、`Spacer`、`Element`、`Color`、`ColorLevel`、`BorderStyle`、`Style`、`render_to_string(RenderOptions { columns: 80, color_level })`；内部文本模块为 `todo!()`。

派发：6 个 `codex exec -s workspace-write`，共用 [porting](briefs/porting.md)，各加一份分区 brief（components、borders、overflow、text、internal、layout）。porting 的要点：

- 一个 Ink 文件对应一个 Rust 文件，保留测试顺序，函数名取 Ink 测试名的 snake_case。
- JSX 译成 builder，颜色、属性按表映射；期望值照抄抓取文件。
- 运行时测试、concurrent 副本、屏幕阅读器、JS 专属行为不移植，按里程碑或原因列进报告；上游 skip 移植为 `#[ignore]`。
- 缺 API 时不移植，报告「needs API」。不改 `src/`（`mod tests;` 除外）和 `tests/common`。
- 验收：`cargo test --no-run` 加 clippy；不跑测试；每个 worker 用 `rustfmt --edition 2024` 只格式化自己的文件；不提交。

SGR 跟进（[sgr](briefs/sgr.md)）：主 agent 重新抓取用 `includes()` 的测试的完整输出，按调用行映射回所属测试；Codex 把 16 个测试里的 58 处正向 `contains` 换成 `assert_ansi_eq`（background 8、border_backgrounds 4、borders 1、text 3 个测试）。

## 结果

- 各组结果：components 移植 37、留下 56；borders 3 个文件 75 个；overflow 8 个文件 104 个；text 11 个文件 154 个；internal 141 个；layout 11 个文件 108 个，含 4 个上游 skip 标 `#[ignore]`。
- 合计 619 个 `#[test]`，对应 615 个 Ink 测试。140 个没移植：46 个 concurrent 副本，79 个属后续里程碑（M1 37、M2 32、M3 6、M4 2、M5 2），14 个不适用，1 个缺 API（`<Text backgroundColor="">`）。
- 移植记录写进 `docs/ink-tests.md`。
- 验证：fmt、clippy、`cargo test --no-run` 通过；`cargo test` 615 个因 `todo!()` 失败，4 个 ignored。
- 提交 `c23bb71`，61 个文件，`.gitignore` 忽略 `/target` 和 `.DS_Store`。
- 遗留：`<Text backgroundColor="">` 的 API 未定（Claude 提议给 `Color` 加一个终端默认色变体），本范围内未见结论。
