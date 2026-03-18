# Echoes in the Fog（迷雾回响）

Rust + Bevy 的 2D 俯视角动作 / 轻度 Roguelike 小游戏课程项目（占位美术，玩法与架构优先）。

## 技术栈
- Rust
- Bevy
- serde + ron（配置驱动）
- rand（随机）

## 运行方式
```bash
运行main.rs
```

## 操作
- `WASD` / 方向键：移动
- 鼠标左键：普通攻击
- `Space` / 鼠标右键：冲刺
- `E`：与门交互切房间
- `Esc`：暂停/继续

## 目录结构（按需求文档）
见 `rust_game_codex_requirements.txt` 中的建议结构；代码已按 `src/core`、`src/gameplay`、`src/ui`、`src/data` 等模块拆分。

## 当前实现功能（MVP）
- 状态：Loading / MainMenu / InGame / Paused / RewardSelect / GameOver / Victory
- 玩家：移动、朝向、近战攻击、冲刺、无敌帧、死亡
- 战斗：Hitbox/Hurtbox、伤害事件、击退、受击闪白、粒子
- 敌人：近战追击、远程射击、冲锋、Boss（三阶段弹幕差异）
- 房间：房间序列、门交互、锁门清怪开门、淡入淡出切换
- 奖励：三选一（8 种）并真实影响数值/行为
- UI：主菜单、HUD、暂停、奖励选择、失败/胜利

## 配置驱动
`assets/configs/*.ron`：
- `player.ron`、`enemies.ron`、`boss.ron`、`rewards.ron`、`rooms.ron`、`game_balance.ron`

## 后续可扩展方向
- 真随机房间图生成（RoomGraph）
- 更完善的机关/解谜房
- 存档/读档
- 替换美术与音频资源、加入更丰富动画与 Shader

