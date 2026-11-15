# Vespera

Vespera is a 3D fantasy MMORPG with an open world. Guild Wars and RuneScape are its main inspirations.

Each day at dusk, the veil to the spirit world becomes thin. Spirits walk, rare creatures come into the world and starlight magic becomes strong. The people of the world call this time the Vesper Hour.

## Status

Milestone M0 (foundations) is complete. Work on milestone M1, the living valley slice, has started. A player can walk on flat ground and cut oak trees for logs and Woodcutting experience.

Read the design in [docs/design](docs/design/README.md).

## Play the development version

Do these steps from the root of the repository:

1. Build the shard and the client: `cargo build --workspace`.
2. On Windows, download the DXC shader compiler: `cargo xtask fetch-dxc`.
3. Start a local shard: `target/debug/vespera-server`.
4. In a second terminal, start the client: `target/debug/vespera`.
5. Move with W, A, S and D. Hold Left Shift to sprint.
6. Go near a tree and push E to cut it.

Do not use `cargo run -p` for these crates. A build of one package selects other Bevy features than a build of the workspace. Thus Cargo builds Bevy again, and this takes a long time.

On Windows, the client uses Direct3D 12. To select a different graphics backend, set the `WGPU_BACKEND` variable, for example to `vulkan`.

To measure the frame rate, set the `VESPERA_BENCHMARK_SECONDS` variable for the client, for example to `20`. After that time, the client writes the frame times (p50, p95, p99 and the 1% low) to the log and stops. The log also shows the GPU time of each render pass. The benchmark uses the full screen without vsync. Measure on mains power, because the battery limits the GPU.

The shard listens on `127.0.0.1:5000`. To use a different address, set the `VESPERA_SHARD_ADDRESS` variable for the shard and the `VESPERA_SERVER` variable for the client.

## Goals

- A seamless open world with a full day and night cycle.
- Classless progression with skills from level 1 to level 99.
- Combat builds with an 8-slot skill bar.
- A player economy with regional exchanges.
- Solo play with one companion, and group play with other players.
- Good performance on a ThinkPad T590 with integrated graphics.
- Clean code with full tests.

## Technology

| Area | Choice |
|---|---|
| Language | Rust, edition 2024 |
| Engine | Bevy 0.20 |
| Network | `bevy_replicon` with renet |
| Physics | `avian3d` |
| Database | SQLite for development, PostgreSQL for production |
| Platforms | Windows, Linux and macOS |

## Contributions

Read [CONTRIBUTING.md](CONTRIBUTING.md) before you make a change.

## License

The code uses the [Apache License 2.0](LICENSE). Assets have their own licenses. The asset manifest records the license of each asset.
