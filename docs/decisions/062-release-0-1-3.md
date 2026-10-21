# 发布 0.1.3

- 日期：2026-10-21
- 状态：已实施
- 提交：`dc6bdca` chore: release 0.1.3
- Brief：无，主 agent 自己做

## 背景

[061](061-split-cr-tab.md) 拆开 CR 与 TAB 后，eink-rs 依赖 crates.io 上的 0.1.2，拿不到这个修复。Claude 问用户要不要推送两个仓库的提交、要不要发 ink-rs 0.1.3，用户答「1. 提交 2. 发」。

## 决策

- 推送 main，发布 0.1.3。用户定（「1. 提交 2. 发」，答的是 Claude 问的推送和发布）。
- 以下由 Claude 在用户的指令下定：
  - 只发 `ink-rs` 0.1.3。`ink-rs-macros` 自 0.1.1 起没有改动，不发。
  - 升补丁版本：改的是输入解析的行为，API 不变。
  - 顺序同 [060](060-release-0-1-2-and-eink-rs.md)：本地全量、推送 main、等 CI 全部通过、打 `v0.1.3` 标签、`cargo publish -p ink-rs`。不建 GitHub Release。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. 本地 fmt、两套 feature 的 clippy、全量测试。
3. 推送 main，等 CI 的 7 个 job。
4. 打 `v0.1.3` 标签并推送，`cargo publish -p ink-rs`。

## 结果

- 本地全量：`cargo test --workspace` 1330 通过，`--no-default-features` 1296 通过；fmt、两套 feature 的 clippy 通过。`cargo package --list` 不含 `docs/`、`macros/`。
- main 推送到 `dc6bdca`，CI run 39553504722 的 7 个 job 全部通过。
- `v0.1.3` 标签推送，ink-rs 0.1.3 发布到 crates.io。eink-rs 的记录 006 升级了依赖。
