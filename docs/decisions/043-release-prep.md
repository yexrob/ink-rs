# 发布准备：许可证、仓库、README 与包元数据

- 日期：2026-10-20
- 状态：已实施
- 提交：`675eaa7` chore: add package metadata, README and licenses
- Brief：无（主 agent 自己完成）

## 背景

rsx 宏合入、kitty demo 的变音符改好后，用户说「没问题了 准备发布ink-rs」。crates.io 上 `ink-rs` 0.0.0 是用户先前占名发布的版本，许可证为 MIT OR Apache-2.0。

Claude 用一次提问让用户定四件事：许可证、GitHub 仓库与分支名、5 个遗留项是否修完再发、README 语言。5 个遗留项是：tmux 挂上后缩放留下旧帧、`<Text backgroundColor="">`、NFC 偏差、宽度表生成脚本、外部写入的 CRLF 空行。

## 决策

用户在提问中的选择（回答只见于会话摘要）：
- 许可证 MIT OR Apache-2.0，与占名的 0.0.0 一致。用户选推荐项。
- 建公开仓库 `yexrob/ink-rs`，`master` 改名 `main` 后推送。用户选推荐项。
- 遗留项「修完再发」。Claude 推荐的是先发 0.1.0 再修，用户没有采纳。
- README 用英文。用户选推荐项。
- Claude 开始调研时，用户追加「另外 Example里面的例子改写成宏的形式吧」，见 [044](044-examples-rsx.md)。

Claude 在实施中定的：
- 最低 Rust 版本 1.88：代码用了 let-chain。
- 两个 crate 都加 `#![forbid(unsafe_code)]`。
- 打包排除 `docs/`、`CLAUDE.md`、`AGENTS.md`。
- 加 `LICENSE-INK`，保留 Ink 的版权声明。
- crates.io 发布、建仓库推送、打标签都要用户再次确认，因为发布不可撤回（只能 yank），推送按 CLAUDE.md 需要批准。发布见 [050](050-release-0-1-0.md)。

## 方案

- 主 agent 自己写元数据和文档：
  - `Cargo.toml`、`macros/Cargo.toml`：license、repository `github.com/yexrob/ink-rs`、keywords、categories、`rust-version`、docs.rs 配置。
  - `README.md`（英文）：功能、相对 Ink 的扩展（tokio 驱动与 `use_future`、`write_control`、按格 diff 与帧高上限）、屏幕阅读器未移植、feature 表、示例、平台（当时只支持 Unix）、许可证与 Ink 致谢。
  - `LICENSE-MIT`、`LICENSE-APACHE`、`LICENSE-INK`；`macros/` 下的许可证文件是指向根目录的软链接。
- 遗留项并行派 4 个调研 agent，结果用于 [045](045-external-write-newline.md)–[049](049-width-tables-unicode-18.md)。

## 结果

- 验证：README 示例编译并运行；临时装 1.88 工具链编译通过，验完卸载；本地按 docs.rs 的方式构建文档，tokio 和 macros 条目显示所需 feature；`cargo package --list` 不含 docs 和 CLAUDE.md。
- 提交 `675eaa7`。
