use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::input::PlayerInputState;
use crate::gameplay::combat::components::Projectile;
use crate::gameplay::enemy::components::{Enemy, EnemyKind, EnemyType};
use crate::gameplay::map::room::CurrentRoom;
use crate::gameplay::player::components::{Energy, Gold, Health, Player};
use crate::states::AppState;
use crate::coop::components::CoopPlayer;

pub const COOP_PORT: u16 = 3457;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    #[default]
    None,
    Host,
    Client,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct CoopNetConfig {
    pub mode: NetMode,
    pub host_ip: String,
}

#[derive(Resource, Debug, Default)]
pub struct CoopNetState {
    pub socket: Option<UdpSocket>,
    pub peer: Option<SocketAddr>,
    pub connected: bool,
    pub my_id: Option<u8>,
    pub last_input_from_client: Option<CoopInputMsg>,
    pub last_snapshot: Option<CoopSnapshotMsg>,
}

#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct CoopRemoteInput(pub CoopInputMsg);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoopMsg {
    Hello,
    Welcome { your_id: u8 },
    Input(CoopInputMsg),
    Snapshot(CoopSnapshotMsg),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CoopInputMsg {
    pub move_axis: (f32, f32),
    pub attack_pressed: bool,
    pub ranged_pressed: bool,
    pub ranged_held: bool,
    pub dash_pressed: bool,
    pub skill1_pressed: bool,
    pub heal_held: bool,
    pub aim: (f32, f32),
    pub aim_valid: bool,
    pub interact_pressed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CoopPlayerStateMsg {
    pub id: u8,
    pub pos: (f32, f32),
    pub hp: f32,
    pub hp_max: f32,
    pub energy: f32,
    pub energy_max: f32,
    pub gold: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CoopEnemyStateMsg {
    pub kind: u8,
    pub pos: (f32, f32),
    pub hp: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CoopProjectileStateMsg {
    pub pos: (f32, f32),
    pub vel: (f32, f32),
    pub team: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoopSnapshotMsg {
    pub tick: u32,
    pub floor_room: u32,
    pub p1: CoopPlayerStateMsg,
    pub p2: CoopPlayerStateMsg,
    pub enemies: Vec<CoopEnemyStateMsg>,
    pub projectiles: Vec<CoopProjectileStateMsg>,
}

fn bind_socket(bind: &str) -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind(bind)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

pub fn start_host_socket(state: &mut CoopNetState) -> anyhow::Result<()> {
    let sock = bind_socket(&format!("0.0.0.0:{COOP_PORT}"))?;
    state.socket = Some(sock);
    state.peer = None;
    state.connected = false;
    state.my_id = Some(1);
    state.last_input_from_client = None;
    state.last_snapshot = None;
    Ok(())
}

pub fn start_client_socket(state: &mut CoopNetState) -> anyhow::Result<()> {
    let sock = bind_socket("0.0.0.0:0")?;
    state.socket = Some(sock);
    state.peer = None;
    state.connected = false;
    state.my_id = None;
    state.last_input_from_client = None;
    state.last_snapshot = None;
    Ok(())
}

fn try_send_to(sock: &UdpSocket, peer: SocketAddr, msg: &CoopMsg) {
    let Ok(payload) = bincode::serialize(msg) else { return };
    let _ = sock.send_to(&payload, peer);
}

fn try_send(state: &CoopNetState, msg: &CoopMsg) {
    let Some(sock) = state.socket.as_ref() else { return };
    let Some(peer) = state.peer else { return };
    try_send_to(sock, peer, msg);
}

pub fn coop_net_tick_system(
    config: Res<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    mut remote_input: ResMut<CoopRemoteInput>,
    mut next: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let Some(sock) = net.socket.as_ref().and_then(|s| s.try_clone().ok()) else { return };

    // Client keep pinging Hello while in lobby.
    if *state.get() == AppState::CoopLobby && config.mode == NetMode::Client && !net.connected {
        if let Some(peer) = net.peer {
            try_send_to(&sock, peer, &CoopMsg::Hello);
        }
    }

    let mut buf = [0u8; 65507];
    loop {
        let Ok((n, from)) = sock.recv_from(&mut buf) else { break };
        let Ok(msg) = bincode::deserialize::<CoopMsg>(&buf[..n]) else { continue };
        match msg {
            CoopMsg::Hello => {
                if config.mode == NetMode::Host {
                    net.peer = Some(from);
                    net.connected = true;
                    net.my_id = Some(1);
                    try_send_to(&sock, from, &CoopMsg::Welcome { your_id: 2 });
                }
            }
            CoopMsg::Welcome { your_id } => {
                if config.mode == NetMode::Client {
                    net.peer = Some(from);
                    net.connected = true;
                    net.my_id = Some(your_id);
                    if *state.get() == AppState::CoopLobby {
                        next.set(AppState::CoopGame);
                    }
                }
            }
            CoopMsg::Input(input) => {
                if config.mode == NetMode::Host {
                    net.last_input_from_client = Some(input);
                    remote_input.0 = input;
                }
            }
            CoopMsg::Snapshot(snapshot) => {
                if config.mode == NetMode::Client {
                    net.last_snapshot = Some(snapshot);
                }
            }
        }
    }
}

pub fn coop_send_raw(net: &CoopNetState, msg: &CoopMsg) {
    try_send(net, msg);
}

pub fn coop_host_snapshot_system(
    time: Res<Time>,
    config: Res<CoopNetConfig>,
    mut net: ResMut<CoopNetState>,
    mut tick: Local<u32>,
    mut send_timer: Local<Timer>,
    player_q: Query<(&GlobalTransform, &Health, &Energy, &Gold), With<Player>>,
    coop_player_q: Query<(&GlobalTransform, &Health, &Energy, Option<&Gold>), With<CoopPlayer>>,
    current_room: Option<Res<CurrentRoom>>,
    enemies_q: Query<(&EnemyKind, &GlobalTransform, &crate::gameplay::player::components::Health), With<Enemy>>,
    projectiles_q: Query<(&Projectile, &Transform)>,
) {
    if config.mode != NetMode::Host || !net.connected {
        return;
    }
    if send_timer.duration().as_secs_f32() <= 0.0 {
        *send_timer = Timer::from_seconds(1.0 / 15.0, TimerMode::Repeating);
    }
    send_timer.tick(time.delta());
    if !send_timer.just_finished() {
        return;
    }

    *tick = tick.wrapping_add(1);

    let Some(sock) = net.socket.as_ref() else { return };
    let Some(peer) = net.peer else { return };

    let Ok((p1_tf, p1_hp, p1_energy, p1_gold)) = player_q.get_single() else { return };
    let p1 = CoopPlayerStateMsg {
        id: 1,
        pos: (p1_tf.translation().x, p1_tf.translation().y),
        hp: p1_hp.current,
        hp_max: p1_hp.max,
        energy: p1_energy.current,
        energy_max: p1_energy.max,
        gold: p1_gold.0,
    };

    let p2 = coop_player_q
        .get_single()
        .ok()
        .map(|(tf, hp, energy, gold)| CoopPlayerStateMsg {
            id: 2,
            pos: (tf.translation().x, tf.translation().y),
            hp: hp.current,
            hp_max: hp.max,
            energy: energy.current,
            energy_max: energy.max,
            gold: gold.map(|g| g.0).unwrap_or(p1_gold.0),
        })
        .unwrap_or(CoopPlayerStateMsg {
            id: 2,
            pos: (p1_tf.translation().x + 40.0, p1_tf.translation().y),
            hp: p1_hp.current,
            hp_max: p1_hp.max,
            energy: p1_energy.current,
            energy_max: p1_energy.max,
            gold: p1_gold.0,
        });

    let mut enemies = Vec::new();
    for (kind, tf, hp) in &enemies_q {
        enemies.push(CoopEnemyStateMsg {
            kind: enemy_kind_to_u8(kind.0),
            pos: (tf.translation().x, tf.translation().y),
            hp: hp.current,
        });
    }

    let mut projectiles = Vec::new();
    for (proj, tf) in &projectiles_q {
        projectiles.push(CoopProjectileStateMsg {
            pos: (tf.translation.x, tf.translation.y),
            vel: (proj.velocity.x, proj.velocity.y),
            team: match proj.team {
                crate::gameplay::combat::components::Team::Player => 1,
                crate::gameplay::combat::components::Team::Enemy => 2,
                crate::gameplay::combat::components::Team::Pvp1 => 3,
                crate::gameplay::combat::components::Team::Pvp2 => 4,
            },
        });
    }

    let floor_room = current_room.as_deref().map(|r| r.0 .0).unwrap_or(0);
    let snapshot = CoopSnapshotMsg {
        tick: *tick,
        floor_room,
        p1,
        p2,
        enemies,
        projectiles,
    };
    let msg = CoopMsg::Snapshot(snapshot);
    try_send_to(sock, peer, &msg);
}

fn enemy_kind_to_u8(kind: EnemyType) -> u8 {
    match kind {
        EnemyType::MeleeChaser => 1,
        EnemyType::RangedShooter => 2,
        EnemyType::Charger => 3,
        EnemyType::Boss => 4,
    }
}

pub fn build_client_input(input: &PlayerInputState) -> CoopInputMsg {
    let (aim, aim_valid) = match input.aim_world {
        Some(v) => (v, true),
        None => (Vec2::ZERO, false),
    };
    CoopInputMsg {
        move_axis: (input.move_axis.x, input.move_axis.y),
        attack_pressed: input.attack_pressed,
        ranged_pressed: input.ranged_pressed,
        ranged_held: input.ranged_held,
        dash_pressed: input.dash_pressed,
        skill1_pressed: input.skill1_pressed,
        heal_held: input.heal_held,
        aim: (aim.x, aim.y),
        aim_valid,
        interact_pressed: input.interact_pressed,
    }
}
