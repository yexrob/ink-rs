# 发布 0.1.6

- 日期：2026-10-22
- 状态：实施中
- 提交：无
- Brief：无，主 agent 自己做

## 背景

[069](069-terminal-graphics.md) 加了终端图形原语：字符串序列的解析、`Scope::use_graphics()`、`OutputWriter::write_passthrough`。eink-rs 的记录 023 要在这些原语上做 `Image` 组件和 Markdown 的图片，依赖 crates.io 上的 ink-rs。069 的手测在 Ghostty、tmux、herdr 里通过后，主 agent 问要不要发 0.1.6、要不要推送，用户答「发布吧 发版」「都推送吧」。

## 决策

- 用户定：
  - 「发布吧 发版」：发布 0.1.6。
  - 「都推送吧」：推送 ink-rs 和 eink-rs 的 main。
- 以下由 Claude 按同一授权和 [067](067-release-0-1-5.md) 的先例定：
  - 发 `ink-rs` 0.1.6。`ink-rs-macros` 自 v0.1.5 没有改动，维持 0.1.1。
  - 升补丁版本：新增 `Graphics`、`GraphicsSupport`、`use_graphics`、`write_passthrough`，现有的公开 API 不变。`Terminal` 和 `WriteDestination` 的改动在 crate 内部。
  - 顺序同 067：本地全量、`cargo publish --dry-run`、推送 main、等 CI、打 `v0.1.6` 标签、`cargo publish -p ink-rs`。用户验收是 069 的手测，用户看过结果后同意发布。

## 备选与取舍

- **升到 0.2.0**：0.x 下新增 API 不要求升次版本；升了的话，下游不改依赖要求拿不到新版本。
- **eink-rs 用 `[patch.crates-io]` 指向本地，不发版**：eink-rs 的 CI 拿不到本地路径，发布到 crates.io 的包不能依赖 git。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. fmt、两套 feature 的 clippy、全量测试、`cargo publish --dry-run -p ink-rs`。
3. 推送 main，等 CI。
4. 打 `v0.1.6` 标签并推送，`cargo publish -p ink-rs`。
5. 推送 eink-rs 的 main。
