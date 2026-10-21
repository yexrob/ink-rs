# 发布 0.1.2，建 eink-rs 仓库

- 日期：2026-10-21
- 状态：已实施
- 提交：`f1a3ae9` docs: list context, callbacks and Props in the README；`85e06ad` chore: release 0.1.2
- Brief：无，主 agent 自己做

## 背景

原语（[057](057-runtime-primitives.md)）和 `#[derive(Props)]`（[059](059-rsx-props-derive.md)）合进 main 后，Claude 问用户两件事：要不要在 GitHub 上建 `yexrob/eink-rs`；0.1.1 之后的提交要不要推送，推送后要不要发 0.1.2。用户答「1.要建 2. 推送 && 发」。

## 决策

- 推送 main，发布新版本。用户决定（「推送 && 发」）。
- 建 GitHub 仓库 `yexrob/eink-rs`。用户决定（「要建」）。
- 以下由 Claude 在用户的指令下定：
  - `ink-rs` 升到 0.1.2，`ink-rs-macros` 升到 0.1.1，`ink-rs` 对 macros 的版本要求改成 0.1.1。新增的都是 API：context、`Callback`、`use_callback`、`Element: PartialEq`、`Props`。`rsx!` 对不带泛型参数的路径改用 `Path::default()`，展开结果对非泛型类型不变。
  - README 补上新 API：hooks 列表加 `provide`、`use_context`、`use_callback`；feature 表的 `macros` 加 `#[derive(Props)]`；组件一条写明 props 可以放 children 和 `Callback`。
  - 顺序同 [055](055-release-0-1-1.md)：本地全量、推送 main、等 CI 全部通过、打 `v0.1.2` 标签、先发 `ink-rs-macros` 再发 `ink-rs`。不建 GitHub Release。
  - `eink-rs` 设为公开，同 ink-rs；先建空仓库，代码等 Ink UI 移植方案讨论后再加。

## 方案

主 agent 自己执行：

1. README 和版本号各一个提交，`cargo package --list` 核对两个包的内容。
2. 本地 fmt、clippy、全量测试。
3. 推送 main，等 CI 的 7 个 job。
4. 打标签，`cargo publish -p ink-rs-macros`，等索引可见后 `cargo publish -p ink-rs`。
5. `gh repo create yexrob/eink-rs --public`。

## 结果

- 本地全量：`cargo test --workspace` 1330 通过，`--no-default-features` 1296 通过；fmt、两套 feature 的 clippy 通过。`cargo package --list` 核对：ink-rs 不含 `docs/`、`macros/`，macros 包含 `src/props.rs`。
- main 从 `a6cea3a` 快进到 `85e06ad` 并推送，CI run 39546019768 的 7 个 job 全部通过。
- `v0.1.2` 标签推送；ink-rs-macros 0.1.1 和 ink-rs 0.1.2 发布到 crates.io。
- `yexrob/eink-rs` 建好，公开，空仓库。
