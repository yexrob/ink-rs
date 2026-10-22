# 发布 0.1.4

- 日期：2026-10-22
- 状态：已实施
- 提交：`854f030` chore: release 0.1.4；`2f92fdd` chore(deps): update tokio to 1.54.1
- Brief：无，主 agent 自己做

## 背景

[063](063-frame-grid.md) 让 `max_fps` 按固定节拍排帧后，eink-rs 依赖 crates.io 上的 0.1.3，拿不到这个改动。Claude 问要不要准备发布 0.1.4，用户答「发布 ， 发布前让我验收一下」。

## 决策

- 发布 0.1.4，推送前由用户验收。用户定（「发布 ， 发布前让我验收一下」）。
- 以下由 Claude 在用户的指令下定：
  - 只发 `ink-rs` 0.1.4。`ink-rs-macros` 自 0.1.1 起没有改动，不发。
  - 升补丁版本：API 不变，改的是 `max_fps` 的节拍与动画交付的时机，同 0.1.3 改输入解析的处理。
  - 顺序同 [062](062-release-0-1-3.md)，在推送前加用户验收：本地全量、`cargo publish --dry-run`、用户验收、推送 main、等 CI 全部通过、打 `v0.1.4` 标签、`cargo publish -p ink-rs`。不建 GitHub Release。
- 验收中用户加了两项，推送前再交给用户验收一次：
  - 帧率示例随 0.1.4 发布，见 [065](065-frame-rate-example.md)。用户定（「两个都要」）。
  - `Cargo.lock` 里的 tokio 从 1.54.0 升到 1.54.1，单独提交。用户定（选「升到 1.54.1」）。起因：`cargo publish --dry-run` 警告 1.54.0 被撤回。`Cargo.toml` 要求 `1.54`，下游解析到 1.54.1，受影响的是本仓库的 CI 和测试。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. 本地 fmt、两套 feature 的 clippy、全量测试、`cargo publish --dry-run -p ink-rs`。
3. 交给用户验收：改动清单、tmux 里的 demo 与测帧程序。用户通过前不推送。
4. 用户加的两项：tokio 升级、帧率示例（065）。本地全量与 dry-run 重跑，再交给用户验收。
5. 推送 main，等 CI 的 7 个 job。
6. 打 `v0.1.4` 标签并推送，`cargo publish -p ink-rs`。

## 结果

- `854f030` 升版本号；验收中加了 `2f92fdd`（tokio 1.54.1）和 `a329d34`（帧率示例，见 [065](065-frame-rate-example.md)）。
- 本地：fmt、两套 feature 的 clippy 通过；`--all-features` 1331 个、`--no-default-features` 1297 个测试通过；`cargo publish --dry-run` 打包 198 个文件，不含 `docs/` 和 `macros/`，没有警告。
- 用户在 tmux 里看了 Markdown demo 和帧率示例，答「批准」。
- 推送 main（`bf77756..656b6c8`），CI 运行 39722353083 的 7 个 job 通过。
- `v0.1.4` 标签打在 `656b6c8`，同 v0.1.3 为轻量标签；`cargo publish -p ink-rs` 发布 0.1.4。`ink-rs-macros` 维持 0.1.1。
- eink-rs 升级依赖见 eink-rs 的记录 017。
