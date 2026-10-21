# 外部写入以换行加转义序列结尾时多出空行

- 日期：2026-10-20
- 状态：已实施
- 提交：`f2e2514` fix(app): skip the extra line break after data ending in a newline and escapes
- Brief：[fix-crlf](briefs/fix-crlf.md)

## 背景

遗留项"外部写入在 CRLF 下多出一个空行"，当时没有记下触发条件。用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。

## 决策

- 判断要不要补 `\r\n` 时看最后一个文本字符，不看最后一个字节：跳过末尾的转义序列和 `\r`；最后的文本字符是 `\n` 或没有文本时不补。Claude 决定，向用户说明「CRLF 那项方案已确定，不需要你决策」。
- `"abc\r"` 仍补 `\r\n`：光标在数据本行的第 0 列，重画会盖掉 `abc`。
- 数据里的换行照原样写出，由 TTY 的 ONLCR 转换。非交互路径不变。

规则见 [终端输出](../design.md#终端输出) 的"外部输出的换行"。

## 方案

- 调研：Claude 子 agent（opus-medium，"Research external-write blank line"）。要求读写入路径，对照 Ink 的 `writeToStdout` 和 log-update，在仓库外建临时 crate，用 `TestTerminal` 和 vt100 实测候选输入，提出修法。
- 调研结论：
  - 触发：数据以 `\n` 结尾、后面跟着颜色复位等控制序列，如 owo-colors、chalk 产生的 `"\x1b[31mabc\n\x1b[0m"`，或 `"abc\n\r"`。
  - 原因：`src/app.rs` 只检查 `data.ends_with('\n')`。
  - raw mode 保留 OPOST|ONLCR，ink-rs 自己不转换 LF。
  - 没有现有测试断言错误行为。
- 派 Codex（worktree `wt/fix-crlf`），与其余 4 项修复并行。Brief 要点：
  - 用 `text::ansi::tokenize` 倒序跳过非文本 token 和 `\r`，写成一个私有 helper。
  - 更新 design.md "外部输出的换行"一段。
  - `tests/hooks.rs` 加 1 个测试，覆盖 stdout 和 stderr，用 `Screen` 检查 `abc` 紧贴帧上方。
- 验收：`--test hooks`、`--lib`，加 fmt 和 workspace clippy。

## 结果

- Codex 在 `src/app.rs` 加 `needs_external_newline`。修复前新测试失败（前两行为 `["abc", ""]`），修复后通过。无偏差。
- Claude 审查时发现 design.md 里"没有文本字符时不补，Ink 不补"一句有歧义，集成时改清楚。
- 提交 `f2e2514`。发布前的全量测试见 [050](050-release-0-1-0.md)。
