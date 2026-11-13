# Vespera

Vespera is a 3D fantasy MMORPG with an open world. Guild Wars and RuneScape are its main inspirations.

Each day at dusk, the veil to the spirit world becomes thin. Spirits walk, rare creatures come into the world and starlight magic becomes strong. The people of the world call this time the Vesper Hour.

## Status

Milestone M0 (foundations) is complete. The repository has the workspace, the repository checks, the CI gates and a test kit with a headless server and a headless client. Milestone M1, the living valley slice, comes next.

Read the design in [docs/design](docs/design/README.md).

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
