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

## Phase 2 incremental 1 — generic menu button slice

Status: complete (2026-08-09).

- Added `style::button::Style::lemon_fresh(...)` as the shared adapter for ordinary menu button image states.
- Applied the `BUTTON_IMAGE_TINT` token to the current button artwork while keeping the three image states swappable for the later asset redraw.
- Migrated the main menu and character-selection generic buttons only; character cards, HUD controls, quality colors, and semantic selection frames remain isolated.
- Verified all three generic button PNGs exist and passed `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check.

## Phase 2 incremental 2 — selection frames and text inputs

Status: complete (2026-08-09).

- Added `style::button::Style::selection(...)` so language, server, world, and map-option lists share one selection-frame resource contract.
- Added theme tokens for input fill, normal border, focused border, and selection highlight.
- Made the custom Iced text-input renderer draw a light input surface with a visible focus ring while preserving cursor, placeholder, and text-selection behavior.
- Kept character-card selection and HUD controls outside this slice because they carry domain-specific visual meaning.
- Verification: `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check passed.

## Phase 2 incremental 3 — generated selection and input textures

Status: complete (2026-08-09).

- Replaced `generic/frames/selection.png`, `selection_hover.png`, and `selection_press.png` with a new Lemon Fresh pixel-art frame set.
- Replaced `generic/textbox.png` with the matching warm-white input texture used by login, world editing, and character naming.
- The generated palette follows the visual spec: warm-white surfaces, amber normal/pressed states, Lime hover/focus, dark ink-safe interiors, no blue-gray or black chrome.
- The focused input variant was used as a reference, while the runtime focus ring remains code-driven so the custom renderer keeps one source of truth.
- Source was generated with the built-in image model, chroma-key cleaned, cropped to the existing contracts (`186x47` selection frames and `169x25` textbox), and validated as RGBA PNGs.

## Phase 2 incremental 4 — scrollbars and settings surface

Status: complete (2026-08-09).

- Added the shared `SCROLLBAR_THUMB` token and migrated settings, group, crafting, and loot scrollbars to the Lemon Fresh amber thumb color. The loot scroller's existing dynamic fade remains intact.
- Replaced the settings window's runtime dependency on the old dark `settings_bg` fill with the themed `PANEL_FILL` surface and `PANEL_BG_ALT` sidebar, while preserving the existing frame geometry and layout IDs.
- Removed the unused `settings_bg` image registration; the legacy asset remains on disk for a later cleanup pass rather than being deleted in this incremental slice.
- Verification: `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check passed.

## Phase 2 incremental 5 — shared close-button states

Status: complete (2026-08-09).

- Replaced the shared `close_btn`, `close_btn_hover`, and `close_btn_press` sprites used across settings, bag, map, quest, trade, diary, prompt, social, and crafting windows.
- The new three-state set keeps the existing `24x25` RGBA contract and uses warm ivory, amber, and lime accents with dark ink glyphs.
- The assets were generated as a single state board, chroma-key cleaned, cropped with nearest-neighbor scaling, and validated after the locked compile check.

## Phase 2 incremental 6 — main-menu background

Status: complete (2026-08-09).

- Replaced `background/bg_main.jpg`, the full-screen background used by normal main-menu screens, while preserving the existing `1920x1080` JPEG contract.
- Regenerated the voxel valley from the original composition: river leading line, central landmark tree, layered cliffs, and foliage remain recognizable.
- Removed the old red-magenta cast and near-black foreground crush; the new grade uses muted sage, moss, umber, warm sand, and controlled amber light so the Lemon Fresh panels and logo remain legible.
- The output was generated with the built-in image model using the original background as a reference, then resized and validated as `1920x1080` RGB JPEG.
- Verification: `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check passed. Live screenshot review remains the next visual QA step.

## Phase 2 incremental 7 — main-menu hierarchy cleanup

Status: complete (2026-08-09).

- Removed the redundant main-menu logo and version column; the background now carries the scene identity without competing branding inside the action flow.
- Replaced the old three-column composition with a focused left navigation rail and a centered login/language panel.
- Increased the left action rail width for readable labels and replaced the old banner-gradient intro surface with a shared Lemon Fresh panel overlay.
- Added a framed central panel so inputs, server state, and primary actions have a stable readable surface over the new background.
- Removed unused main-menu image registrations for the deleted logo and banner-gradient dependency; source assets remain untouched for later global cleanup.
- Verification: `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check passed.

## Phase 2 incremental 8 — main-menu button sprites

Status: complete (2026-08-09).

- Replaced the shared `button`, `button_hover`, and `button_press` sprites used by the main-menu action rail and primary login actions.
- Preserved the existing sprite contracts (`106x26` normal and `212x52` hover/pressed) while removing the old dark-brown leather treatment.
- The new states use the same warm ivory, lime, amber, and dark-ink language as the close buttons, selection frames, and input textures.
- The assets were generated as a blank three-state board, chroma-key cleaned, cropped with nearest-neighbor scaling, and validated as RGBA PNGs.
- Verification: `cargo fmt --all`, `git diff --check`, and the locked `voxygen` compile check passed.

## Phase 2 incremental 9 — main-menu visual redesign

Status: complete (2026-08-09).

- Removed the obsolete top version strip and the remaining logo/banner competition from the main login flow.
- Reworked the composition around one translucent primary content surface, with a restrained utility action panel anchored at the lower left.
- Replaced the old framed center panel and language gradient banner with open spacing, a small lime accent rule, dark-ink typography, and a shared warm surface.
- Preserved login, server selection, language selection, error confirmation, and singleplayer/multiplayer actions; this slice changes composition and hierarchy only.
- Verification: `cargo fmt --all`, `git diff --check`, and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed.

## Phase 2 incremental 10 — menu typography foundation

Status: complete (2026-08-09).

- Replaced the main-menu Iced default font from the legacy pixel face with the localization-safe universal font (`GoNotoCurrent`).
- Assigned display hierarchy to `alkhemi` for menu headings and `universal` for inputs, server names, loading messages, credits, and body copy.
- Updated Simplified Chinese font metadata from WenQuanYi Zen Hei to `NotoSansTC-Regular` for cleaner glyph proportions and stronger menu readability.
- Kept language-specific font manifests intact for other locales; unsupported scripts continue to use their existing localized coverage.
- Verification: `cargo fmt --all`, `git diff --check`, and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed. The windowed client restarted successfully with `fullscreen.enabled: false`.

## Phase 2 incremental 11 — main-menu density pass

Status: complete (2026-08-09).

- Reduced the login surface from `500×560` to `560×480` so its proportions follow the actual content instead of creating a tall empty shell.
- Expanded login inputs from `230px` to `300px`, giving the form a stronger horizontal rhythm and reducing side voids.
- Reduced the primary action stack from `200px` to `130px`, tightened button spacing, and compacted the lower-left utility panel.
- Kept the explanatory copy, login flow, server selection, language selection, and singleplayer entry intact.
- Verification: `cargo fmt --all`, `git diff --check`, and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed. The windowed client restarted successfully.

## Phase 2 incremental 12 — surface color and input cleanup

Status: complete (2026-08-09).

- Replaced near-white shared menu surfaces with a muted moss-sand palette so panels no longer read as isolated white cards over the game scene.
- Added a theme-driven input surface and removed the white textbox sprite from login and singleplayer world-edit fields.
- Kept the existing focus ring, cursor, selection, and localized text behavior while removing the legacy white field background from these flows.
- Verification: `cargo fmt --all`, `git diff --check`, and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed. The windowed client restarted successfully.

## Phase 2 incremental 13 — main-menu contrast correction

Status: complete (2026-08-09).

- Darkened the shared menu surfaces again to a muted moss palette with enough separation from the bright background art.
- Added a dedicated main-menu button treatment that tints the legacy button state masks dark olive instead of exposing their ivory artwork.
- Switched main-menu button labels to warm light text, including a readable disabled state; ordinary menus keep their existing light button treatment during the incremental migration.
- Verification: `cargo fmt --all`, `git diff --check`, and `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish` passed.

## Phase 2 incremental 14 — shared scene surface

Status: complete (2026-08-09).

- Removed the individual solid backgrounds from the login utility group, login form, and main-menu input wrappers.
- Removed framed panel fills from the server list, credits, connection prompt, disclaimer, and singleplayer confirmation overlays.
- Removed the internal text-input fill from the custom renderer while preserving its four-edge focus/active border.
- Kept the root scene background, the lime hierarchy rule, and interaction-state artwork so the menu remains readable and operable without stacked cards.
- Verification: `cargo fmt --all`, `git diff --check`, `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish`, and `cargo build --bin lemoncraft-voxygen --locked --no-default-features --features default-publish` passed. The windowed client restarted successfully.

## Phase 2 incremental 15 — temporary singleplayer landing flow

Status: complete (2026-08-09).

- Temporarily removed the multiplayer action, server selector, server input, and multiplayer account notice from the landing screen.
- Recentered the remaining singleplayer, language, credits, and quit actions into one vertical composition.
- Kept the multiplayer implementation marked as dormant code so it can be restored without rebuilding the connection flow.
- Verification: `cargo fmt --all`, `git diff --check`, `cargo check -p lemoncraft-voxygen --locked --no-default-features --features default-publish`, and `cargo build --bin lemoncraft-voxygen --locked --no-default-features --features default-publish` passed. The windowed client restarted successfully.

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
