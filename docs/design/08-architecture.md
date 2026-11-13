# 8. Architecture

## 8.1 Overview

Vespera uses a client-server architecture. The server is the authority for all game state. The client shows the world and sends the intents of the player to the server.

| Part | Description | Milestone |
|---|---|---|
| Client | Shows the world, plays audio, reads input and predicts the movement of the local player | M0 |
| World shard server | Runs one open world: movement, rules, NPCs and creatures | M0 |
| Login service | Accounts and connect tokens | M3 |
| Social service | Chat, friends, parties and guilds across shards | M3 |
| Exchange service | Regional exchanges for all shards | M5 |
| Database | SQLite for development and tests, PostgreSQL for production | M2 |

## 8.2 Workspace

The repository is one Cargo workspace. Each crate has one purpose.

| Crate | Type | Purpose |
|---|---|---|
| `vespera_core` | Library | Basic shared types: IDs, units, the game clock and deterministic random numbers |
| `vespera_content` | Library | Content schemas, RON loaders and content validation |
| `vespera_rules` | Library | Game rules as headless Bevy plugins: skills, items, inventory, gathering, crafting, combat, effects, quests and economy rules |
| `vespera_motion` | Library | The kinematic character controller, for the server and the client |
| `vespera_world` | Library | The chunk format, terrain data, world queries and the world compiler |
| `vespera_protocol` | Library | Network messages, replicated components and the replication setup |
| `vespera_persistence` | Library | Store traits, and the SQLite and PostgreSQL stores |
| `vespera_server` | Library and binary | The world shard server |
| `vespera_kinematics` | Library | IK solvers, spring bones, the animation graph and the procedural layers |
| `vespera_render` | Library | The stylized material, terrain, sky, fog, foliage, post-processing, quality presets and the dynamic render scale |
| `vespera_ui` | Library | The widget set and the game screens |
| `vespera_audio` | Library | Spatial audio, ambience and music |
| `vespera_client` | Library and binary | The game client |
| `vespera_testkit` | Library, for tests only | A headless server and headless clients in one process, a simulated network, fixtures and test bots |
| `xtask` | Binary | Developer tasks (see 8.10) |

Later milestones add `vespera_login`, `vespera_social` and `vespera_exchange`.

## 8.3 Layer rules

The crates have four layers. A crate can use only crates in lower layers.

| Layer | Crates |
|---|---|
| 1. Base | `vespera_core` |
| 2. Shared logic | `vespera_content`, `vespera_rules`, `vespera_motion`, `vespera_world` |
| 3. Contracts and storage | `vespera_protocol`, `vespera_persistence`, `vespera_kinematics` |
| 4. Applications and presentation | `vespera_server`, `vespera_client`, `vespera_render`, `vespera_ui`, `vespera_audio`, `vespera_testkit` |

The client crates are `vespera_client`, `vespera_render`, `vespera_ui`, `vespera_audio` and `vespera_kinematics`. The server crates are `vespera_server`, `vespera_persistence` and the later services.

More rules:

- Server crates do not use client crates. Client crates do not use server crates.
- `vespera_testkit` is the only exception. It uses the server and the client logic plugins.
- `vespera_persistence` is only for the server and the services.
- Only `vespera_client`, `vespera_render`, `vespera_ui`, `vespera_audio` and `vespera_kinematics` use the render, window, audio and UI features of Bevy.
- `cargo xtask check-layers` reads `cargo metadata`. It fails if a crate has a forbidden dependency.

The client library has two plugin groups:

- Logic plugins: network, prediction, interpolation and input. The test kit uses only these plugins.
- Presentation plugins: render, UI, audio and kinematics.

## 8.4 Bevy conventions

- The client uses `DefaultPlugins` with only the necessary Bevy features.
- The server uses `MinimalPlugins` and the asset plugin. The server has no render code.
- Each game feature is a plugin. The plugin owns its components, resources, messages and system sets.
- Features communicate through messages, events and observers. A feature does not read the private components of a different feature.
- Systems that can fail return a `Result`. Each error has context.
- Content and assets load through the Bevy asset server, on the client and on the server.
- Shaders use WESL.
- Client settings use Bevy app settings.
- Input uses `bevy_enhanced_input`, with input contexts and actions that the player can bind again.
- The game UI uses `bevy_ui` with the Vespera widget set. Development builds can use `bevy_egui` for debug tools. Release builds do not contain `bevy_egui`.

## 8.5 Network

| Item | Decision |
|---|---|
| Replication | `bevy_replicon` |
| Transport | `bevy_replicon_renet`: UDP with encryption and connect tokens |
| Authority | The server, for all game state |
| Client messages | Intents only, for example a move direction or a skill use |
| Movement step | 60 Hz, fixed, on the server and on the client |
| Rules tick | 20 Hz |
| Replication rate | 20 Hz for near entities, less for far entities |
| Interest management | A grid of 64 m cells. A client receives the entities in the cells near its player. |

Prediction and reconciliation:

1. The client runs the shared character controller for the local player with the local input. The client keeps a history of its inputs.
2. The server runs the same controller with the same inputs. The server sends the result and the number of the last input that it used.
3. The client compares the server result with its history. If they are different, the client starts again from the server result. Then it runs all inputs that the server did not use yet.

The client predicts only the local player. The client shows remote entities 100 ms in the past, and interpolates between two received states.

When a player uses a skill, the client starts the cast animation immediately. If the server refuses the skill, the client stops the animation.

## 8.6 Persistence

- `vespera_persistence` has store traits for accounts, characters, inventories, vaults, warehouses, exchange orders and the ledger.
- Two implementations use `sqlx`: SQLite for development and tests, and PostgreSQL for production.
- Each backend has its own migrations.
- The same test suite runs against both backends. CI runs the PostgreSQL tests in a service container.
- Database work runs on a separate Tokio runtime. Systems send requests through channels and receive the results as messages.
- Each economy operation writes to the database immediately, in one transaction.
- The server saves the state of a character at logout, at a shard hop and every 60 seconds.

## 8.7 Content

- Content files are text files in the `content/` folder: RON data files and Yarn Spinner dialogue files.
- The server and the client load the same content files.
- A validation test loads all content. It checks the IDs, the references and the value ranges.
- `bevy_yarnspinner` runs the dialogue.
- UI text uses Fluent files. Dialogue text uses Yarn Spinner string tables. English is the first language.

## 8.8 Errors and logs

- Library errors use `thiserror`.
- The server must not panic because of client input.
- Logs use `tracing`. Development builds can use the Tracy profiler.
- The server exports metrics in a later milestone.

## 8.9 Configuration

- The client keeps its settings in a settings file, through Bevy app settings.
- The server reads a TOML configuration file and environment variables.

## 8.10 Developer tasks

`cargo xtask` gives these commands:

| Command | Purpose |
|---|---|
| `check-comments` | Find comments that the rules do not permit |
| `check-layers` | Find forbidden crate dependencies |
| `check-text-only` | Find binary files in the repository |
| `check-docs` | Do basic ASD-STE100 checks on Markdown files |
| `fetch-assets` | Download the asset archives and verify their hashes |
| `import-assets` | Convert source assets to runtime assets |
| `compile-world` | Run the world compiler |
| `bench` | Run the client benchmark |
| `dev` | Start a local server and a client |
