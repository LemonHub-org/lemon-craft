# LemonCraft UI 阶段 0 基线报告

> 建立日期：2026-08-09
> 分支：`main`
> 基线提交：`c0e47ca9 Redesign visual direction to Lemon Fresh light theme (v2)`

## 基线状态

当前工作树不是干净状态。阶段 0 保留了原有改动，没有回滚或覆盖：

- `assets/voxygen/element/lc_logo.png`
- `assets/voxygen/logo.ico`
- `docs/visual-design-lemon-fresh.md`
- `voxygen/src/hud/mod.rs`
- `voxygen/src/menu/char_selection/ui.rs`
- `voxygen/src/ui/theme.rs`
- `docs/ui-development-plan.md`

当前主题基线按 `Lemon Fresh v2` 记录，但实际 UI 贴图仍包含旧版深棕/蓝灰 chrome；阶段 1 前不应把截图中的混合状态视为最终设计。

## 分辨率基线

项目默认 UI 缩放基准为 `1920×1080`，默认配置使用 `RelativeToWindow([1920, 1080])`。阶段 0 采用以下测试矩阵：

| 场景 | 分辨率 | 用途 | 当前状态 |
|---|---:|---|---|
| 最小横屏 | 1280×720 | 检查内容拥挤、遮挡和文字溢出 | 待运行时截图 |
| 默认基线 | 1920×1080 | 主要视觉比较基准 | 配置基准；当前显示器未提供该物理尺寸 |
| 高分辨率 | 2560×1440 | 检查相对缩放和留白 | 待运行时截图 |
| 16:10 | 1920×1200 | 检查非 16:9 比例适配 | 待运行时截图 |

当前运行环境客户端窗口实际捕获尺寸为 `1707×1067`，因此现有截图属于运行环境烟雾基线，不替代上述四种正式截图。

## 运行时截图

截图位于 [`docs/ui-baseline/`](ui-baseline/)：

- [`main-menu-window.png`](ui-baseline/main-menu-window.png)：窗口初始绘制，主要是空白/清屏状态，不作为正式视觉基线。
- [`main-menu-window-after-load.png`](ui-baseline/main-menu-window-after-load.png)：加载后背景基线。
- [`main-menu-window-final.png`](ui-baseline/main-menu-window-final.png)：等待约 15 秒后的背景基线。
- [`main-menu-window-30s.png`](ui-baseline/main-menu-window-30s.png)：累计等待约 45 秒后的背景基线。

客户端可以启动并显示 LemonCraft 窗口、背景图和版本信息，但主菜单按钮、logo、登录表单等控件没有出现。由于无法从主菜单进入角色选择、游戏 HUD、背包和设置窗口，这些页面暂未生成可验收截图。

因此截图交付状态为：

- 主菜单背景：已捕获
- 主菜单完整交互界面：未通过
- 角色选择：未捕获
- 默认 HUD：未捕获
- 背包：未捕获
- 设置窗口：未捕获

## 当前视觉观察

- 背景资源能够正常加载，当前随机背景仍明显偏暗紫色，与 `Lemon Fresh` 暖白方向不一致。
- 版本信息能够绘制，说明窗口和部分 UI 绘制链路已工作。
- 当前无法仅凭这次运行判断 Iced 控件是资源加载问题、渲染问题还是已有二进制与工作树不同步；需要在后续运行时 QA 中单独复现。

## 阶段 0 结论

代码状态、分辨率规范、资源清单和运行时限制已经冻结并记录。阶段 0 的正式截图子项仍有运行环境限制，阶段 1 可以继续进行资源和主题系统工作，但在阶段 7 前必须补齐四种分辨率下的完整页面截图。
