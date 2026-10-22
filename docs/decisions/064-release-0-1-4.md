# 发布 0.1.4

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：无，主 agent 自己做

## 背景

[063](063-frame-grid.md) 让 `max_fps` 按固定节拍排帧后，eink-rs 依赖 crates.io 上的 0.1.3，拿不到这个改动。Claude 问要不要准备发布 0.1.4，用户答「发布 ， 发布前让我验收一下」。

## 决策

- 发布 0.1.4，推送前由用户验收。用户定（「发布 ， 发布前让我验收一下」）。
- 以下由 Claude 在用户的指令下定：
  - 只发 `ink-rs` 0.1.4。`ink-rs-macros` 自 0.1.1 起没有改动，不发。
  - 升补丁版本：API 不变，改的是 `max_fps` 的节拍与动画交付的时机，同 0.1.3 改输入解析的处理。
  - 顺序同 [062](062-release-0-1-3.md)，在推送前加用户验收：本地全量、`cargo publish --dry-run`、用户验收、推送 main、等 CI 全部通过、打 `v0.1.4` 标签、`cargo publish -p ink-rs`。不建 GitHub Release。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. 本地 fmt、两套 feature 的 clippy、全量测试、`cargo publish --dry-run -p ink-rs`。
3. 交给用户验收：改动清单、tmux 里的 demo 与测帧程序。用户通过前不推送。
4. 推送 main，等 CI 的 7 个 job。
5. 打 `v0.1.4` 标签并推送，`cargo publish -p ink-rs`。

## 结果

（发布后补）
