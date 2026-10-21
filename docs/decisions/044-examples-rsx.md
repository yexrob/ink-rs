# 示例改写为 `rsx!`

- 日期：2026-10-20
- 状态：已实施
- 提交：`536267e` refactor(examples): build trees with rsx!
- Brief：[rsx-common](briefs/rsx-common.md)、[rsx-ex-a](briefs/rsx-ex-a.md)、[rsx-ex-b](briefs/rsx-ex-b.md)

## 背景

`rsx!` 合入后（[041](041-rsx-macro.md)），用户先让 Claude 把 demo（`~/tmp/ink-rs-demo`）改写成 rsx。发布提问答完、遗留项调研开始时，用户追加「另外 Example里面的例子改写成宏的形式吧」。Claude 回复「收到，examples 也改成 rsx」。

## 决策

- 25 个示例全部改用 `rsx!`。用户决定（「另外 Example里面的例子改写成宏的形式吧」）。
- 以下由 Claude 定：
  - 行为不变：同一棵树、同样的样式、key 和输出；不改状态、hook、输入处理。现有示例测试不改期望，不加新测试。
  - 宏由 `macros` feature 提供，每个示例和 `examples`、`examples_pty` 两个测试目标声明 `required-features = ["macros"]`；jest、subprocess-output 为 `["tokio", "macros"]`。用 `required-features`，不加 `cfg` 属性。
  - rustfmt 不格式化宏内部，宏体按 rustfmt 的调用风格手工排版。
  - 用法限制写进共用 brief：没有裸开关；字符串只捕获标识符；变量作子节点要加花括号；宏内不能写 `let`。后两条来自改写 demo 时遇到的问题。

## 方案

- 派两个 Codex 并行，worktree `wt/rsx-ex-a`、`wt/rsx-ex-b`，各约一半示例。两份 brief 都以 `rsx-common` 为共用部分。
- part A 负责 alternate-screen 到 render-throttle 的 12 个示例，并独占 `Cargo.toml`：按字母序声明 25 个 `[[example]]`，补 `[[test]] name = "examples"`。
- part B 负责 scroll 到 use-stdout 的 13 个示例，不碰 `Cargo.toml`。
- 验收：`cargo build --examples`、`--test examples --test examples_pty`、workspace clippy、`cargo fmt --all --check`；part A 另跑 `--no-default-features` 的构建和 clippy，确认示例被跳过。
- 同时段主 agent 做发布元数据（[043](043-release-prep.md)）。

## 结果

- 两个 worker 的共同偏差：`.into()` 放在宏闭括号的下一行，因为 rustfmt 不接受 brief 要求的同一行写法。Claude 接受。
- 宏不支持 `let`，borders、incremental-rendering、select-input、use-animation 把循环内的计算移到宏外的迭代器里。static 在 `Static` 闭包里嵌套 `rsx!`，两个根合成 fragment。
- Claude 审查两份 diff，行为与原版一致。集成时简化 borders：去掉 `(row, last)` 预计算，改为在 `row.iter()` 上用 `i + 1 == row.len()` 判断。
- `docs/design.md` 示例一节写明示例用 `rsx!` 建树和 `required-features` 的声明。
- 全量 1317 个测试和 28 个 PTY 用例通过，clippy、doc 和 `--no-default-features` 各项检查通过。worktree 已清理。
