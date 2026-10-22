# 发布 0.1.7

- 日期：2026-10-22
- 状态：进行中
- 提交：
- Brief：无，主 agent 自己做

## 背景

[072](072-termination.md) 到 [075](075-mouse.md) 做完了 bingo 要的 ink-rs 改动：终止信号交给应用、`TestApp` 读屏、`use_color_level` 和锚定节点的光标、鼠标。eink-rs 的 `Composer` 用 `use_node_cursor`，`Code` 用 `use_color_level`，两个分支在 worktree 里用 `[patch.crates-io]` 指向 ink-rs 的本地 main，合入 eink-rs 的 main 要等 crates.io 上有 0.1.7。用户在真实终端里核对了鼠标和 `Composer` 的输入法，答「没问题 这两个都 我测试过了」。主 agent 问要不要发 0.1.7、要不要推送，用户选「发布并推送 (Recommended)」。

## 决策

- 用户定：「发布并推送 (Recommended)」：发布 0.1.7，推送 ink-rs 和 eink-rs 的 main。
- 以下由 Claude 按同一授权和 [070](070-release-0-1-6.md) 的先例定：
  - 发 `ink-rs` 0.1.7。`ink-rs-macros` 自 v0.1.6 没有改动，维持 0.1.1。
  - 升补丁版本：自 v0.1.6 的公开 API 改动全是新增（`Termination`、`TerminationLocked`、`set_termination`，`MouseEvent`、`MouseButton`、`MouseKind`、`MouseTracking`、`Scope::use_mouse`、`ElementMetrics::contains`，`Scope::use_color_level`、`Scope::use_node_cursor`，`TestApp::live_frame`、`TestApp::static_output`），现有的签名不变。
  - 顺序同 070：本地全量、`cargo publish --dry-run`、推送 main、等 CI、打 `v0.1.7` 标签、`cargo publish -p ink-rs`。

## 备选与取舍

- **升到 0.2.0**：0.x 下新增 API 不要求升次版本；升了的话，下游不改依赖要求拿不到新版本。
- **eink-rs 留着 `[patch.crates-io]`，不发版**：eink-rs 的 CI 拿不到本地路径，发布到 crates.io 的包不能依赖本地路径。

## 方案

主 agent 自己执行：

1. 升版本号和 `Cargo.lock`，单独提交。
2. fmt、两套 feature 的 clippy、全量测试、`cargo publish --dry-run -p ink-rs`。
3. 推送 main，等 CI。
4. 打 `v0.1.7` 标签并推送，`cargo publish -p ink-rs`。
5. eink-rs 的依赖升级和推送放在它的发布记录里。

## 结果
