# 决策记录与记录规则

- 日期：2026-10-21
- 状态：已实施
- 提交：`42f88a9` docs: record the basis of past decisions；`b4a1b84` docs: require a decision record before each change；`9f31b97` docs: describe the test screen pulling rows back on height growth
- Brief：[decision-history](briefs/decision-history.md)

## 背景

原语集成期间，用户提出：「对了 吧项目立项到现在每次你写或者拍worker 的依据 也就是决策和方案写到docs/下面吧 并且把这个规则也写到CLAUDE.md」。

## 决策

- 从立项起每次写代码、派 worker 的依据补写到 `docs/`，以后写进 CLAUDE.md 作为规则。用户决定（见上一句引文）。
- 下面几项由 Claude 定，用户没有另行表态：
  - 记录放在 `docs/decisions/`，一个决策一个文件，按时间编号，格式和索引见 [README](README.md)。design.md 写设计现状，记录写来历，链接 design.md 的小节，不复述。
  - 派给 worker 的 brief 原样存进 `briefs/`，本机路径换成占位符；scratchpad 下的克隆、期望值、调研笔记和 worker 报告不入库，记录写它们的结论。
  - 账号、额度重置流程、管理后台地址、邮箱不进记录。
  - 规则写成 CLAUDE.md 第 4 条：写代码或派 worker 之前开记录，和改动一起提交或先于改动提交，落地后补结果。
- 文档探索派 Sonnet 或 Codex，不派 Opus。用户决定（「文档探索任务以后不要拍Opus， sonnet就够了」）。Codex 可以接任何委派的任务，不只写代码。用户决定（「codex也是受你管辖的 子agent也可以派它不只是你自己的agent」）。

## 方案

- 主 agent 从两次会话的 JSONL 导出时间线（用户、助手、派发、提问、提交），把 60 份 brief 拷进 `briefs/` 并替换路径，按时间切成 a–f 六段。
- 六段并行派给六个 Claude 子 agent（opus-high），共用 [decision-history](briefs/decision-history.md)，各写一段的记录，只读不改仓库。派出后用户指出这类任务不该用 Opus；六个已在运行，跑完没有重派，规则用于以后的派发。
- 主 agent 合并：统一编号、生成索引；检查记录间链接、design.md 锚点和敏感字符串。

## 结果

- 第一次导出漏掉了会话中途插入的用户消息（transcript 里存为 `queued_command` 附件，不是普通的用户条目）。补导出后发给 d、e 两段的 agent，f 段改用用户原话。
- 段边界的重复合并两处：写控制的两条合成 [038](038-write-control.md)，同步驱动与 tmux 的两条合成 [021](021-m2-p2c-sync-driver-decisions.md)。
- 得到 001–057 共 57 条记录。各 agent 查不到的依据在记录里写明「记录中未见」。
- design.md 测试一节说行增长不从 scrollback 拉回内容，与 `tests/common::Screen` 自 `6921715` 起的行为不符，单独提交修正（`9f31b97`）。
