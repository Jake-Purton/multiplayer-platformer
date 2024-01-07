use crate::{MultiplayerSetting, main_menu::HostClient};

use bevy::prelude::*;

pub fn run_if_online(host: Res<MultiplayerSetting>) -> bool {
    !matches!(host.0, HostClient::Play)
}

// for systems that run if we are in client mode
pub fn run_if_client(host_or_join: Res<MultiplayerSetting>) -> bool {
    matches!(host_or_join.0, HostClient::Client | HostClient::Host)
}

// allows systems to run if the host setting is on
pub fn run_if_host(host: Res<MultiplayerSetting>) -> bool {
    matches!(host.0, HostClient::Host)
}