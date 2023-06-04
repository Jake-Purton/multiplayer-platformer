# Networking

The goal is to allow the user to host a game (run a server), join a game (connect to someone else's server) or to play offline (single player).

Options for networking in the bevy game-engine:
* [bevy_renet] (https://crates.io/crates/bevy_renet)
* [bevy_simple_networking] (https://crates.io/crates/bevy_simple_networking)

I decided to use the renet plugin for bevy because it promises to be fast with a simple api and support for TCP/IP so that users do not have to change port forwarding settings on their router.