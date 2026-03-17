# 迷雾回响（Echoes in the Fog）

一个用 Rust + Bevy 编写的 2D 俯视角轻度 Roguelike 小游戏（课程项目 / MVP）。

## 运行方式

本项目入口是 `src/main.rs`，运行时直接运行它即可。

也可以使用标准 Rust 方式运行（推荐）：

```bash
cargo run
```

## 游戏怎么玩

### 目标

- 依次探索房间、击败敌人并通关 Boss 房。
- 清理 Boss 房后会出现“奖励三选一”，选择后进入结算（胜利）。

### 操作

- `WASD`：移动
- 鼠标左键：近战攻击
- 鼠标右键：远程攻击
- `Space`：冲刺/位移
- `E`：与门交互（切换房间）
- `Esc`：暂停/继续
- 奖励选择界面：按 `1` / `2` / `3` 选择奖励

### 提示

- 进入新房间后可能会锁门；击败房间内敌人即可解锁并继续前进。
- 奖励会永久影响当前局的属性与战斗能力（例如移速、暴击、攻击间隔等）。
- 失败会进入失败界面；胜利会进入胜利界面；两者都可按 `Enter` 返回主菜单。

## 配置（可选）

游戏数值与房间序列由 `assets/configs/*.ron` 驱动，例如：

- `assets/configs/rooms.ron`：房间序列（Start/Normal/Reward/Boss）
- `assets/configs/game_balance.ron`：整体平衡（例如 `boss_room_gives_victory`）

## 目录结构（简述）

- `src/core`：输入、资源加载等基础设施
- `src/gameplay`：战斗/敌人/地图/进度/奖励等核心玩法
- `src/ui`：主菜单、HUD、暂停、奖励选择、结算界面
- `src/data`：配置定义与加载
