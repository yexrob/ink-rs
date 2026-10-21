# M3 P4：use_focus 与 use_focus_manager
- 日期：2026-10-20
- 状态：已实施
- 提交：`3c26a6b` feat(focus): add use_focus and use_focus_manager
- Brief：[m3-p4](briefs/m3-p4.md)

## 背景
[023](023-m3-input-architecture.md) 定下焦点的几条规则：layout effect 中注册，无焦点时 Shift+Tab 跳到最后一项，禁用状态下 `focus(id)` 设置 active，注销重复 id 只删本 hook 的条目。P4 与 P5 并行，14:11 在各自的 worktree 中派给两个 Codex。

## 决策
Claude 写 brief 时决定，派出时告知用户可以改：
- `use_focus` 只返回是否获得焦点。
- `FocusManager` 是可复制的句柄，`active_id()` 读当前最新值，不是上一次渲染时的值。
- Tab 和 Esc 先切换焦点，再交给 `use_input` 的回调。
- 记录中未见用户对这几项提出修改。

## 方案
brief 要点：
- `use_focus(FocusOptions) -> bool`，`FocusOptions` 默认 `active: true`、`auto_focus: false`、`id: None`；`use_focus_manager` 返回 `FocusManager`。
- layout effect 注册，新增 Focus 类持有者；活跃的焦点 hook 在 TTY 上持有 raw mode，非 TTY 上焦点持有者不报错、不开 raw mode。
- 派发顺序：Ctrl+C、焦点键、输入回调。
- 自动生成的 id 为 `ink-rs:focus:{n}`，每个 hook 稳定，同一 runtime 内唯一。
- 测试驱动增加 `TestInstance::{input, advance}`，使用假时钟。

规则见 [输入](../design.md#输入)。

## 结果
- Claude review 删除两处：
  - 测试驱动中关掉 Ctrl+C 退出的一行，没有测试用到；
  - 一个只验证共用守卫的 panic 测试。
- 保留 5 个单元测试，各锁定一处与 Ink 不同、移植测试测不到的行为：首帧自动聚焦、回调读到最新焦点、重复 id 按 hook 区分、非 TTY 下焦点不开 raw mode、卸载后句柄不出错。
- 与 Ink 源码对照一致的三处：Esc 受启用开关控制；`focus(id)` 只聚焦活跃条目；激活 effect 的依赖包含 autoFocus。
- 移植：`focus.tsx` 44 个；不移植 3 个（随机 id 碰撞 F19，两个并发模式的重复用例）。另外三个文件移植 7 个；StrictMode 的 2 个和 1 个并发重复用例不适用。
- 全量 1163 通过、0 失败、3 忽略，clippy 干净；文档更新；worktree 删除。
