use std::net::{SocketAddr, UdpSocket};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::states::AppState;

pub const PVP_PORT: u16 = 3456;

#[derive(Resource, Debug, Default, Clone)]
pub struct PvpNetConfig {
    pub mode: NetMode,
    pub host_ip: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum NetMode {
    #[default]
    None,
    Host,
    Client,
}

#[derive(Resource, Debug, Default)]
pub struct PvpNetState {
    pub socket: Option<UdpSocket>,
    pub peer: Option<SocketAddr>,
    pub connected: bool,
    pub my_id: Option<u8>,
    pub last_input_from_client: Option<PvpInputMsg>,
    pub last_state: Option<PvpStateMsg>,
    pub fire_events: Vec<PvpFireMsg>,
    pub winner: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PvpMsg {
    Hello,
    Welcome { your_id: u8 },
    Input(PvpInputMsg),
    State(PvpStateMsg),
    Fire(PvpFireMsg),
    Result { winner: u8 },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PvpInputMsg {
    pub move_axis: (f32, f32),
    pub melee: bool,
    pub ranged: bool,
    pub aim: (f32, f32),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PvpPlayerStateMsg {
    pub id: u8,
    pub pos: (f32, f32),
    pub hp: f32,
    pub lives: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PvpStateMsg {
    pub tick: u32,
    pub p1: PvpPlayerStateMsg,
    pub p2: PvpPlayerStateMsg,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct PvpFireMsg {
    pub shooter_id: u8,
    pub origin: (f32, f32),
    pub dir: (f32, f32),
}

fn bind_socket(bind: &str) -> anyhow::Result<UdpSocket> {
    let socket = UdpSocket::bind(bind)?;
    socket.set_nonblocking(true)?;
    Ok(socket)
}

pub fn start_host_socket(state: &mut PvpNetState) -> anyhow::Result<()> {
    let sock = bind_socket(&format!("0.0.0.0:{PVP_PORT}"))?;
    state.socket = Some(sock);
    state.peer = None;
    state.connected = false;
    state.my_id = Some(1);
    state.last_state = None;
    state.fire_events.clear();
    state.winner = None;
    Ok(())
}

pub fn start_client_socket(state: &mut PvpNetState) -> anyhow::Result<()> {
    let sock = bind_socket("0.0.0.0:0")?;
    state.socket = Some(sock);
    state.peer = None;
    state.connected = false;
    state.my_id = None;
    state.last_state = None;
    state.fire_events.clear();
    state.winner = None;
    Ok(())
}

fn try_send(state: &PvpNetState, msg: &PvpMsg) {
    let Some(sock) = state.socket.as_ref() else { return };
    let Some(peer) = state.peer else { return };
    let Ok(payload) = bincode::serialize(msg) else { return };
    let _ = sock.send_to(&payload, peer);
}

fn try_send_to(sock: &UdpSocket, peer: SocketAddr, msg: &PvpMsg) {
    let Ok(payload) = bincode::serialize(msg) else { return };
    let _ = sock.send_to(&payload, peer);
}

pub fn pvp_net_tick_system(
    mut config: ResMut<PvpNetConfig>,
    mut net: ResMut<PvpNetState>,
    mut next: ResMut<NextState<AppState>>,
    state: Res<State<AppState>>,
) {
    let Some(sock) = net
        .socket
        .as_ref()
        .and_then(|s| s.try_clone().ok())
    else {
        return;
    };

    // Client side: ensure we keep pinging Hello until connected.
    if *state.get() == AppState::PvpLobby && config.mode == NetMode::Client && !net.connected {
        if let Some(peer) = net.peer {
            try_send_to(&sock, peer, &PvpMsg::Hello);
        }
    }

    let mut buf = [0u8; 2048];
    loop {
        let Ok((n, from)) = sock.recv_from(&mut buf) else { break };
        let Ok(msg) = bincode::deserialize::<PvpMsg>(&buf[..n]) else { continue };
        match msg {
            PvpMsg::Hello => {
                if config.mode == NetMode::Host {
                    net.peer = Some(from);
                    net.connected = true;
                    net.my_id = Some(1);
                    try_send_to(&sock, from, &PvpMsg::Welcome { your_id: 2 });
                    if *state.get() == AppState::PvpLobby {
                        next.set(AppState::PvpGame);
                    }
                }
            }
            PvpMsg::Welcome { your_id } => {
                if config.mode == NetMode::Client {
                    net.peer = Some(from);
                    net.connected = true;
                    net.my_id = Some(your_id);
                    if *state.get() == AppState::PvpLobby {
                        next.set(AppState::PvpGame);
                    }
                }
            }
            PvpMsg::Input(input) => {
                // Host consumes input in simulation system via net.last_state? store in config? handled elsewhere
                // We'll stash it in net.fire_events? no. We put it in a dedicated resource in systems module.
                // This arm is handled by systems::pvp_host_simulation_system via PvpNetState::last_input_client.
                net.last_input_from_client = Some(input);
            }
            PvpMsg::State(st) => {
                if config.mode == NetMode::Client {
                    net.last_state = Some(st);
                }
            }
            PvpMsg::Fire(ev) => {
                net.fire_events.push(ev);
            }
            PvpMsg::Result { winner } => {
                net.winner = Some(winner);
                if *state.get() != AppState::PvpResult {
                    next.set(AppState::PvpResult);
                }
            }
        }
    }

    // Keep local config sane.
    if config.mode == NetMode::None && net.socket.is_some() {
        net.socket = None;
    }
}

// Additional mutable field for host input capture.
impl PvpNetState {
    pub fn clear_runtime(&mut self) {
        self.last_state = None;
        self.fire_events.clear();
        self.winner = None;
    }

    pub fn send_state(&self, st: &PvpStateMsg) {
        try_send(self, &PvpMsg::State(st.clone()));
    }

    pub fn send_fire(&self, fire: PvpFireMsg) {
        try_send(self, &PvpMsg::Fire(fire));
    }

    pub fn send_result(&self, winner: u8) {
        try_send(self, &PvpMsg::Result { winner });
    }

    pub fn send_input(&self, input: PvpInputMsg) {
        try_send(self, &PvpMsg::Input(input));
    }
}

// Host-only: last received client input (updated in net tick).
// Kept as a free field to avoid extra resources.
impl PvpNetState {
    pub(crate) fn take_client_input(&mut self) -> Option<PvpInputMsg> {
        self.last_input_from_client.take()
    }
}

// Hidden field (Rust requires it declared on struct; keep at end of file with Update File patch in-place).
