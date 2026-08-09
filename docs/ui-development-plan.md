# LemonCraft UI 开发后续计划

## 目标

以 `Lemon Fresh v2` 为统一方向，保留现有功能和像素 RPG 气质，逐步消除“新主题颜色 + 旧 UI 贴图”的混合状态。

## 阶段 0：冻结现状与建立基线

> 状态：已完成基线冻结。运行时完整页面截图受当前客户端仅显示背景/版本信息的限制，待后续运行时 QA 补齐。

- 保留当前未提交改动，先确认主题方向不再反复。
- 选定测试分辨率：1280×720、1920×1080、2560×1440、16:10。
- 为主菜单、角色选择、游戏 HUD、背包、设置窗口制作现状截图。
- 清点所有 UI 图片资源，区分继续使用、需要重新绘制和仅保留为语义色资源的内容。
- 确认 Git LFS 资源完整，避免缺失资源影响视觉判断。

交付物：UI 现状截图集、资源清单、目标分辨率规范。

## 阶段 1：统一主题系统

重点文件：

- `voxygen/src/ui/theme.rs`
- `voxygen/src/hud/mod.rs`
- `voxygen/src/ui/ice/renderer/`
- `voxygen/src/menu/`

工作内容：

- 完善背景、面板、边框、文字、hover、pressed、disabled、tooltip 和 overlay token。
- 为 Conrod 和 Iced 分别提供统一转换函数。
- 移除普通 UI 中的硬编码颜色。
- 保留战斗、装备品质、聊天等语义颜色，不强行品牌化。
- 明确深色区域只用于 tooltip、聊天和战斗反馈，不再作为普通窗口默认背景。

验收标准：同一种按钮、面板和选中状态在主菜单、HUD、设置页中视觉一致。

## 阶段 2：重做通用 UI 资源

优先处理：

- 普通按钮及 hover / pressed 状态
- 选中框
- 输入框
- 设置窗口背景和边框
- 暂停菜单框架
- 滚动条
- 标签页选中态
- tooltip 外框
- 通用窗口角落和分隔线

设计原则：

- 统一暖白面板、深色文字、柠檬黄边框。
- 不再依赖深棕色旧按钮作为默认基础组件。
- 组件内部使用轻量层级变化，避免每个元素都有厚重边框。
- 保持像素风，但减少旧版蓝灰描边与深色填充。

验收标准：不看具体功能，也能判断所有界面属于同一个 LemonCraft UI 系统。

## 阶段 3：优化高频游戏 HUD

处理顺序：

1. 血量、能量、架势条
2. 技能栏与快捷栏
3. 小地图与右上角按钮
4. 聊天窗口
5. buff / debuff
6. 拾取提示和通知
7. 准星、伤害数字、头顶名称

重点：

- 减少 HUD 对游戏画面的遮挡。
- 重新校准底部技能栏和左下聊天的视觉重量。
- 统一数字字体和数值对齐。
- 为重要反馈增加清晰的状态层级，而不是单纯增加颜色。
- 检查窗口化、超宽屏和 UI 缩放后的重叠问题。

验收标准：正常探索时 HUD 不喧宾夺主，战斗时关键状态能够快速识别。

## 阶段 4：统一大型功能窗口

处理：

- 背包
- 制作
- 角色/技能书
- 世界地图
- 交易
- 社交
- 任务对话
- 设置

建立共用窗口规范：

- 标题栏高度
- 关闭按钮位置
- 内容边距
- 标签页样式
- 滚动区域
- 搜索框
- 确认/取消按钮
- 空状态、错误状态、加载状态

不建议立即重写 Conrod 或迁移到 Iced。先通过现有组件和主题层完成视觉统一，避免同时引入框架迁移风险。

## 阶段 5：统一主菜单与角色选择

- 登录页、服务器页、连接页使用同一套按钮和输入框视觉。
- 角色选择页统一背景、角色卡、选择态和创建角色入口。
- 检查错误、服务器为空、加载中、没有角色等状态。
- 确认长文本、多语言和低分辨率下不会溢出。
- 保持随机背景，但建立背景筛选规则，避免明暗和色温差异过大。

## 阶段 6：响应式、可访问性和本地化

- 为 UI 建立最小可用窗口尺寸。
- 检查 `ui_scale` 在不同 DPI 下的表现。
- 补充键盘焦点、Tab 顺序、按下反馈和关闭行为。
- 检查色盲模式、低对比度和透明度设置。
- 用长文本语言测试按钮、标签页和设置项。
- 对数字、时间、百分比等数据统一使用稳定的排版方式。

## 阶段 7：视觉与功能 QA

每个 UI 模块都需要验证：

- 默认状态
- hover 状态
- pressed 状态
- disabled 状态
- 加载状态
- 空状态
- 错误状态
- 长文本状态
- UI 缩放状态
- 键盘/手柄操作状态

建议建立截图回归目录，至少覆盖：

- 主菜单
- 角色选择
- 游戏默认 HUD
- 背包
- 制作
- 地图
- 设置
- 暂停菜单
- 聊天输入
- 死亡界面

代码验证继续执行现有的 `cargo fmt`、clippy 和相关测试。

## 长期架构计划

## Phase 1 handoff — theme system

Status: complete (2026-08-09).

- Added shared Lemon Fresh interaction tokens and a floating-point `vek::Rgba` adapter in `voxygen/src/ui/theme.rs`.
- Unified ordinary Iced button, scrollbar, slider, text-input, menu-panel, overlay, and frame colors.
- Migrated ordinary main-menu, server, connection, disclaimer, credits, world-selector, and character-selection surfaces away from hardcoded dark chrome.
- Preserved semantic combat, item-quality, chat, error, and debug colors for later domain-specific review.
- Verification: `cargo fmt --all` and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed; the initial check emitted one unused-import warning, which was removed afterward.
- `cargo check -p lemoncraft-voxygen --tests --locked --no-default-features --features default-publish` also passed. Running the linked test binary was blocked by Windows `link.exe` LNK1104 while another user-side `cargo bench --offline` process was active; no process was terminated.

视觉统一完成后，再评估是否将 Conrod HUD 逐步迁移到 Iced 或其他统一层。迁移前需要先解决：

- 自定义渲染器兼容
- 游戏内坐标与世界投影
- tooltip 和物品拖拽
- 性能
- 手柄输入
- 多语言布局

这不应作为当前阶段的前置条件。

## 当前最优先的三项任务

1. 完成通用按钮、窗口框、设置背景等旧资源的 Lemon Fresh 重制。
2. 将 Conrod 和 Iced 的颜色、状态、边距规范统一。
3. 在 4 种目标分辨率下完成主菜单、HUD、背包和设置窗口的截图回归。

最终目标不是把所有界面变成浅色，而是让 LemonCraft 的世界、HUD、菜单和功能窗口拥有统一、清晰且可持续维护的视觉语言。
