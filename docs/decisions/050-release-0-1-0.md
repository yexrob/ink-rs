# 发布 0.1.0

- 日期：2026-10-20
- 状态：已实施
- 提交：无新提交；标签 `v0.1.0` 打在 `379f75b`
- Brief：无（主 agent 自己完成）

## 背景

用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。5 项修复依次提交：`f2e2514`（[045](045-external-write-newline.md)）、`5c17d3d`（[046](046-nfc-wrap.md)）、`0d05344`（[047](047-color-reset.md)）、`6921715`（[048](048-resize-scrollback-erase.md)）、`379f75b`（[049](049-width-tables-unicode-18.md)）。

## 决策

- 发布步骤全部执行。用户在提问中选「全部执行 (Recommended)」（回答只见于会话摘要）。步骤：`master` 改名 `main`，建公开仓库 `yexrob/ink-rs` 并推送，打 `v0.1.0` 标签并推送，`cargo publish` 先发 `ink-rs-macros` 再发 `ink-rs`。
- 外部操作前再问一次，由 Claude 在 [043](043-release-prep.md) 中定：发布不可撤回（只能 yank），推送按 CLAUDE.md 需要用户批准。

## 备选与取舍

提问中的另两项，用户没有选：
- 只推 GitHub：改名、建仓库、推送代码和标签，暂不发 crates.io。
- 先不发布：保持本地状态，不做外部操作。

## 方案

提问前，主 agent 做了以下检查：
- 检查打包清单，docs 和 `CLAUDE.md` 不在包内。
- 跑全量测试。
- 跑 `cargo publish --workspace --dry-run`，不上传。
- 查 crates.io 上已有的 `ink-rs 0.0.0` 归属于用户的账号。

用户确认后，主 agent 按顺序执行发布步骤。之后检查残留的 `master` 引用和临时文件占用。

## 结果

- GitHub 公开仓库的默认分支为 `main`，代码和 `v0.1.0` 标签已推送。
- crates.io 上 `ink-rs-macros 0.1.0` 和 `ink-rs 0.1.0` 已发布。
- 发布前的验证：
  - 全量测试通过：默认 features 1323 个，`--no-default-features` 1291 个，PTY 测试通过。
  - fmt、clippy、doc 和 examples 构建无警告。
  - dry run 通过。
- docs.rs 会自动构建文档，Claude 没有看结果。
- 记录中未见 GitHub Release。
- 本地遗留：
  - 临时目录占 471M，其中 Ink 源码克隆 284M，research 目录 126M。
  - tmux 会话 `ink-rs` 里还开着 demo、kitty、player 三个窗口。
  - Claude 提出可以清理，记录中未见用户回复。
- 这之后到会话结束，用户只让 Claude 在 `~/tmp` 下写 demo，没有 ink-rs 改动。Windows 支持和 0.1.1 在第二个会话，见 [055](055-release-0-1-1.md)。
