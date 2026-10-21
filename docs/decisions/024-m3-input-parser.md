# M3 P1：移植 Ink 输入解析器
- 日期：2026-10-20
- 状态：已实施
- 提交：`c7fc36f` feat(input): port Ink input parsing and key mapping
- Brief：[m3-p1](briefs/m3-p1.md)

## 背景
[023](023-m3-input-architecture.md) 定下移植 Ink 的解析器，不用 crossterm 的事件解析。P1 只做纯函数部分，不读 stdin，不接 App。

## 决策
- 公开类型 `KeyEvent`，字段用 `enter`、`super_key`；事件类型用 `Option`。用户在 023 选定字段名。
- 不引入 regex 依赖，用字符判断和字段扫描替代正则，注释里引用原正则。brief 规定。
- Ink 用 `\p{C}` 排除 meta 字符。review 时 Claude 删掉 Codex 生成的约 750 行 Unicode 类别 C 表，规则改为只排除 Cc。记录中未见对其余类别差异的测试。
- Esc 的截止时间归 App，不归解析器。

## 方案
12:40 在独立 worktree `wt/m3-p1` 中派给 Codex，避免与重排任务的半成品互相影响编译。brief 要点：
- 移植 UTF-8 续接解码、`InputParser` 分词、`parse_keypress` 和到 `KeyEvent` 的映射。
- 模块加 `cfg_attr(not(test), expect(dead_code))`，等调用方接入后去掉。
- 测试：input-parser 65 个，parse-keypress 24 个，kitty-keyboard 63 个。
- 验收命令：`cargo test --lib input`；clippy。

## 结果
- Codex 报告：`cargo test --lib input` 153 个通过（152 个 Ink 移植加 1 个格式错误 UTF-8 的自有测试），clippy 通过。
  - `ParsedKey` 去掉 `raw` 与 `code`，管线不读它们。
  - 断言改写：PK-15 去掉 `raw === undefined`；KK-40、KK-41 的 `isKittyProtocol === undefined` 改为 `false`；输入与期望值不变。
  - 不移植：PK-01..03（字节重载，App 解码后走不到）；KK-24（与 KK-03 重复）；KK-25（`sequence`/`raw` 保留，超出范围）；KK-64..75 推到 App 与 kitty 协商阶段。
- Claude review 的改动：
  - 删除 Unicode 类别表，见上；
  - keypress 测试移到 `keypress/tests/`；
  - dead_code 的原因改为 phase 3；
  - 保留 `From<ParsedKey> for (String, KeyEvent)`，因为改成固有方法会在测试构建中触发 dead_code。
- ink-tests.md 增加 3 行统计、「M3 文件中不移植的测试」一节和输入解析字段的说明；修正 KK-24 的措辞。
- 全量 995 通过、0 失败、3 忽略。worktree 已删除。
- KK-64..75 在 [027](027-m3-paste-kitty.md) 处理。
