# 宽度表生成脚本，改用 Unicode 18.0，去掉 unicode-width

- 日期：2026-10-20
- 状态：已实施
- 提交：`379f75b` build(text): generate width tables from Unicode 18.0 and drop unicode-width
- Brief：[fix-width](briefs/fix-width.md)、[fix-width2](briefs/fix-width2.md)

## 背景

`src/text/width_data.rs` 是 string-width 8.3 移植（`src/text/width.rs`）用的 Unicode 表。文件头写明表由没有入库的临时脚本生成；最后一张 `EAW_WIDE_OVERRIDES` 把 `unicode-width` 的结果修正到 get-east-asian-width。发布需要一个任何人都能重跑的生成器。用户要求遗留项「修完再发」（见 [043](043-release-prep.md)）。

## 决策

- 脚本生成全部表，移除 `unicode-width`。用户在提问中选了推荐项「脚本生成全部表，移除 unicode-width」（选择见会话摘要和 Claude 的回复「四项都按推荐方案做」）。提问中的说明：Python 标准库脚本，带 `--check`；宽字符改用 EAW 的 W/F 表，与 get-east-asian-width 同源；去掉 Khmer 特例；有 8 个不可见字符的宽度会变。
- 数据版本从 17.0 改为 18.0。Claude 决定，派发续作后告诉用户，没有提问。理由：Ink 用的 get-east-asian-width 1.7.0 基于 Unicode 18.0，按 17.0 生成会让 990 个字符从 2 列变 1 列，都是 18.0 新增的女真文字和新 emoji。
- 由 Claude 在 brief 中定：
  - 生成器不用 Python 的 `unicodedata`（本机为 Unicode 16）。
  - 不在脚本里按码点手工补表，除非有成文的规则。
  - 切换前在仓库外的临时脚本里列出 `east_asian_width` 结果变化的码点；有可见宽度变化就停下报告。
  - 包不带生成器，`exclude` 加 `/scripts`。
  - design.md 和 `src/text/mod.rs` 中提醒调用方 `unicode-width` 在 ZWJ emoji、谚文字母上宽度不同的句子保留。

规则见 [布局与文本](../design.md#布局与文本) 的"宽度"。

## 备选与取舍

- 只加脚本，保留 `unicode-width`：脚本重现现有的表，宽字符仍由 `unicode-width` 判断。提问中的另一项，用户没有选。

## 方案

- 调研：Claude 子 agent（opus-medium，"Research NFC and width data"），与 NFC 合在一个任务里（NFC 见 [046](046-nfc-wrap.md)）。结论：10 张表中 6 张可以从本地数据原样重现；另外 4 张（MARK、SPACING_MARK、INVISIBLE、PICTOGRAPHIC）有明确定义；建议标准库 Python 脚本 `scripts/gen_width_data.py` 带 `--check` 按字节比较；可选用 W/F 的 `WIDE` 表替换 `EAW_WIDE_OVERRIDES`，只在 8 个不可见字符上有差异。
- 主 agent 从 unicode.org 下载 17.0.0 的五份数据到 `$S/ucd17`。`emoji-test.txt` 在 `Public/emoji/17.0/` 下返回 404，改用 `Public/17.0.0/emoji/emoji-test.txt`。
- 第一轮 Codex（worktree `wt/fix-width`，brief fix-width），与其余 4 项修复并行。
- 第一轮结果：脚本写成，九张保留表与现有文件逐字节相同（`INVISIBLE` 排除代理码点，Rust `char` 不接受 U+D800–U+DFFF）。枚举发现 990 个可见码点会从 2 列变 1 列，worker 按 brief 停下，没有改 Rust 代码和依赖。
- Claude 查明 get-east-asian-width 1.7.0 基于 18.0，下载 18.0.0 数据到 `$S/ucd18`，派续作（brief fix-width2）。要求先核对两点再改代码：
  - 生成的 `WIDE` 与 get-east-asian-width 的集合相同。
  - 其余九张表的变化只出现在 18.0 新分配的码点上。
- 续作 prompt 补一条：Python 的 mise shim 在沙箱中失败，扩充 PATH。

## 结果

- 续作的审计：
  - `WIDE` 与 get-east-asian-width 1.7.0 的 wide、fullwidth 集合相同，共 183,866 个码点。
  - 九张表的每个差异码点在 17.0 中为 Cn，在 18.0 中有赋值类别。
  - 新旧 EAW 只剩 8 个 1→2 差异（U+302A–U+302D、U+3099–U+309A、U+3164、U+16FE4），都属于 `INVISIBLE`，不到达 `east_asian_width`。990 个差异消失。
- 改动：生成器与生成文件头改为 18.0.0；`east_asian_width` 对 W/F 返回 2，其余返回 1；删除 `unicode_width` 导入和 Khmer 分支；`Cargo.toml` 删除 `unicode-width`，`Cargo.lock` 中 vt100 仍用它；design.md 说明生成器、Unicode 18.0.0 和旧版 Node ICU 对新字符可能造成的差异。
- Codex 验证：`--check` 通过，`text_width` 18 项、`--lib` 342 项、`text` 49 项通过，fmt、clippy 通过。
- Claude 集成时自行复核 `--check`。design.md 与 NFC 的提交冲突，保留已提交的断行一行，取续作的宽度一行。提交 `379f75b`，这是 v0.1.0 的标签所在（见 [050](050-release-0-1-0.md)）。
