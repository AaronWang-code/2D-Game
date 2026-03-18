use bevy::prelude::*;

use crate::core::assets::GameAssets;
use crate::core::input::PlayerInputState;
use crate::core::achievements::ShopPurchaseEvent;
use crate::data::registry::GameDataRegistry;
use crate::gameplay::map::room::{CurrentRoom, FloorLayout, RoomId, RoomType};
use crate::gameplay::map::InGameEntity;
use crate::gameplay::player::components::{
    AttackCooldown, AttackPower, CritChance, DashCooldown, Energy, Gold, Health, MoveSpeed, Player, RewardModifiers,
};
use crate::states::AppState;
use crate::utils::rng::GameRng;

pub struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShopOffers>()
            .init_resource::<ShopSeenRooms>()
            .add_systems(
                Update,
                (
                    spawn_shop_kiosk_if_needed,
                    maybe_enter_shop_state,
                    open_shop_hotkey,
                    handle_shop_purchase_input.run_if(in_state(AppState::Shop)),
                ),
            );
    }
}

#[derive(Component)]
pub struct ShopKiosk;

#[derive(Component)]
pub struct ShopUiLine;

#[derive(Resource, Debug, Default, Clone)]
pub struct ShopOffers {
    pub room: Option<RoomId>,
    pub lines: Vec<ShopLine>,
}

#[derive(Resource, Debug, Default)]
pub struct ShopSeenRooms {
    pub rooms: bevy::utils::HashSet<RoomId>,
}

#[derive(Debug, Clone)]
pub struct ShopLine {
    pub title: String,
    pub description: String,
    pub cost: u32,
    pub item: ShopItem,
}

#[derive(Debug, Clone, Copy)]
pub enum ShopItem {
    Heal,
    IncreaseMaxHealth,
    IncreaseAttackPower,
    ReduceDashCooldown,
    IncreaseMoveSpeed,
    IncreaseEnergyMax,
    IncreaseCritChance,
    IncreaseAttackSpeed,
}

pub fn spawn_shop_kiosk_if_needed(
    mut commands: Commands,
    assets: Option<Res<GameAssets>>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    existing: Query<(), With<ShopKiosk>>,
) {
    if existing.iter().next().is_some() {
        return;
    }
    let (Some(assets), Some(layout), Some(current)) = (assets, layout, current) else { return };
    let Some(room) = layout.room(current.0) else { return };
    if room.room_type != RoomType::Shop {
        return;
    }

    commands.spawn((
        SpriteBundle {
            texture: assets.textures.white.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 15.0)),
            sprite: Sprite {
                color: Color::srgb(0.20, 0.70, 0.25),
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            ..default()
        },
        ShopKiosk,
        InGameEntity,
        Name::new("ShopKiosk"),
    ));

    commands.spawn((
        Text2dBundle {
            text: Text::from_section(
                "商店 (E)",
                TextStyle {
                    font: assets.font.clone(),
                    font_size: 22.0,
                    color: Color::WHITE,
                },
            ),
            transform: Transform::from_translation(Vec3::new(0.0, -56.0, 16.0)),
            ..default()
        },
        ShopKiosk,
        InGameEntity,
        Name::new("ShopKioskLabel"),
    ));
}

pub fn maybe_enter_shop_state(
    input: Res<PlayerInputState>,
    layout: Option<Res<FloorLayout>>,
    current: Option<Res<CurrentRoom>>,
    mut seen: ResMut<ShopSeenRooms>,
    mut offers: ResMut<ShopOffers>,
    mut next: ResMut<NextState<AppState>>,
    data: Option<Res<GameDataRegistry>>,
    mut rng: ResMut<GameRng>,
) {
    let (Some(layout), Some(current)) = (layout, current) else { return };
    let Some(room) = layout.room(current.0) else { return };
    if room.room_type != RoomType::Shop {
        return;
    }

    let auto_open = !seen.rooms.contains(&current.0);
    let manual_open = input.interact_pressed;
    if !(auto_open || manual_open) {
        return;
    }
    seen.rooms.insert(current.0);

    generate_shop_offers(&mut offers, data.as_deref(), &mut rng, current.0);
    next.set(AppState::Shop);
}

pub fn open_shop_hotkey(
    input: Res<PlayerInputState>,
    mut offers: ResMut<ShopOffers>,
    mut next: ResMut<NextState<AppState>>,
    data: Option<Res<GameDataRegistry>>,
    mut rng: ResMut<GameRng>,
    current: Option<Res<CurrentRoom>>,
) {
    if !input.shop_pressed {
        return;
    }
    let room = current.as_deref().map(|c| c.0).unwrap_or(RoomId(0));
    generate_shop_offers(&mut offers, data.as_deref(), &mut rng, room);
    next.set(AppState::Shop);
}

fn generate_shop_offers(
    offers: &mut ShopOffers,
    data: Option<&GameDataRegistry>,
    rng: &mut GameRng,
    room: RoomId,
) {
    offers.room = Some(room);
    offers.lines.clear();

    let mut pool = vec![
        ShopItem::Heal,
        ShopItem::IncreaseMaxHealth,
        ShopItem::IncreaseAttackPower,
        ShopItem::ReduceDashCooldown,
        ShopItem::IncreaseMoveSpeed,
        ShopItem::IncreaseEnergyMax,
        ShopItem::IncreaseCritChance,
        ShopItem::IncreaseAttackSpeed,
    ];
    rng.shuffle(&mut pool);
    pool.truncate(3);

    let floor = data.map(|d| d.balance.floor_rooms).unwrap_or(4) as u32;
    let base_cost = 25 + floor * 3;

    for item in pool {
        let (title, desc, cost) = describe_item(item, base_cost);
        offers.lines.push(ShopLine {
            title: title.to_string(),
            description: desc.to_string(),
            cost,
            item,
        });
    }
}

fn describe_item(item: ShopItem, base_cost: u32) -> (&'static str, &'static str, u32) {
    match item {
        ShopItem::Heal => ("治疗", "立刻恢复 35 点生命", base_cost),
        ShopItem::IncreaseMaxHealth => ("强健", "最大生命 +20", base_cost + 10),
        ShopItem::IncreaseAttackPower => ("锋刃", "攻击力 +5", base_cost + 12),
        ShopItem::ReduceDashCooldown => ("迅捷", "冲刺冷却 -15%", base_cost + 12),
        ShopItem::IncreaseMoveSpeed => ("轻盈", "移动速度 +30", base_cost + 10),
        ShopItem::IncreaseEnergyMax => ("充能", "最大能量 +25", base_cost + 8),
        ShopItem::IncreaseCritChance => ("锐眼", "暴击率 +8%", base_cost + 14),
        ShopItem::IncreaseAttackSpeed => ("连击", "攻速 +15%", base_cost + 14),
    }
}

pub fn handle_shop_purchase_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    offers: Res<ShopOffers>,
    mut next: ResMut<NextState<AppState>>,
    mut shop_purchase: EventWriter<ShopPurchaseEvent>,
    mut player_q: Query<(
        &mut Gold,
        &mut Health,
        &mut Energy,
        &mut MoveSpeed,
        &mut AttackPower,
        &mut CritChance,
        &mut DashCooldown,
        &mut AttackCooldown,
        &mut RewardModifiers,
    ), With<Player>>,
) {
    let idx = if keyboard.just_pressed(KeyCode::Digit1) || keyboard.just_pressed(KeyCode::Numpad1) {
        Some(0)
    } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2) {
        Some(1)
    } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3) {
        Some(2)
    } else {
        None
    };
    let Some(i) = idx else { return };
    let Some(line) = offers.lines.get(i) else { return };

    let Ok((mut gold, mut hp, mut energy, mut move_speed, mut power, mut crit, mut dash_cd, mut atk_cd, mut mods)) =
        player_q.get_single_mut()
    else {
        return;
    };

    if gold.0 < line.cost {
        warn!("金币不足：需要 {}，当前 {}", line.cost, gold.0);
        return;
    }
    gold.0 -= line.cost;

    apply_item(
        line.item,
        &mut hp,
        &mut energy,
        &mut move_speed,
        &mut power,
        &mut crit,
        &mut dash_cd,
        &mut atk_cd,
        &mut mods,
    );

    shop_purchase.send(ShopPurchaseEvent);
    next.set(AppState::InGame);
}

fn apply_item(
    item: ShopItem,
    hp: &mut Health,
    energy: &mut Energy,
    move_speed: &mut MoveSpeed,
    power: &mut AttackPower,
    crit: &mut CritChance,
    dash_cd: &mut DashCooldown,
    atk_cd: &mut AttackCooldown,
    mods: &mut RewardModifiers,
) {
    match item {
        ShopItem::Heal => {
            hp.current = (hp.current + 35.0).min(hp.max);
        }
        ShopItem::IncreaseMaxHealth => {
            hp.max += 20.0;
            hp.current = hp.current.min(hp.max);
            mods.max_hp_add += 20.0;
        }
        ShopItem::IncreaseAttackPower => {
            power.0 += 5.0;
        }
        ShopItem::ReduceDashCooldown => {
            let d = dash_cd.timer.duration().as_secs_f32() * 0.85;
            dash_cd.timer.set_duration(std::time::Duration::from_secs_f32(d.max(0.1)));
            mods.dash_cooldown_mult += 0.15;
        }
        ShopItem::IncreaseMoveSpeed => {
            move_speed.0 += 30.0;
        }
        ShopItem::IncreaseEnergyMax => {
            energy.max += 25.0;
            energy.current = (energy.current + 25.0).min(energy.max);
        }
        ShopItem::IncreaseCritChance => {
            crit.0 = (crit.0 + 0.08).clamp(0.0, 1.0);
            mods.crit_add += 0.08;
        }
        ShopItem::IncreaseAttackSpeed => {
            let d = atk_cd.timer.duration().as_secs_f32() * 0.85;
            atk_cd.timer.set_duration(std::time::Duration::from_secs_f32(d.max(0.05)));
            mods.attack_speed_mult += 0.15;
        }
    }
}
