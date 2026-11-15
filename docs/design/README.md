# Vespera design

This folder contains the design of Vespera. The design has two parts:

- Decisions. The project owner accepted these decisions. The decision register in this file lists them.
- Defaults. The chapters give default values for all other topics. A default stays until a person changes it.

## How to change the design

1. Change the related chapter.
2. If the change replaces a decision, change the decision register in the same commit.
3. Write the reason for the change in the commit message.

## Chapters

| No. | Chapter | Contents |
|---|---|---|
| 1 | [Vision](01-vision.md) | Summary, pillars, inspirations, the Vesper Hour, tone and scope limits |
| 2 | [World](02-world.md) | Structure, scale, regions, travel, time, weather, life and world authoring |
| 3 | [Characters](03-characters.md) | Peoples, customization, equipment, companions, controls, camera and targets |
| 4 | [Body kinematics](04-kinematics.md) | Rigs, animation layers, IK, secondary motion and the character controller |
| 5 | [Progression and combat](05-progression.md) | Skills, the XP curve, combat, death and quests |
| 6 | [Economy](06-economy.md) | Items, storage, exchanges, trade, transport, crafting, sources and sinks |
| 7 | [Multiplayer](07-multiplayer.md) | Shards, services, social features, PvP, the Frontier, hosting and security |
| 8 | [Architecture](08-architecture.md) | Workspace, layer rules, Bevy conventions, network, persistence and content |
| 9 | [Performance](09-performance.md) | Spec floor, targets, budgets, render rules, presets and measurement |
| 10 | [Assets](10-assets.md) | Art direction, sources, licenses, style rules, the asset store and audio |
| 11 | [Engineering rules](11-engineering.md) | Toolchain, TDD, test levels, code rules, lints, CI, commits and documentation |
| 12 | [Roadmap](12-roadmap.md) | Milestones M0 to M8 and open questions |

## Decision register

| ID | Topic | Decision | Section |
|---|---|---|---|
| D01 | Engine and language | Rust edition 2024 and Bevy (0.20 at the start). | 11.1 |
| D02 | License | Apache-2.0 for the code. | [LICENSE](../../LICENSE) |
| D03 | Art direction | Semi-realistic: real human proportions, PBR materials from photo scans and natural light, with a light fantasy finish. Vespera lighting, fog, color grading and wind unify the assets. | 10.1 |
| D04 | Lore hook | The Vesper Hour. A full day and night cycle. At dusk, the veil to the spirit world becomes thin for a short time. | 1.4 |
| D05 | Controls | Hybrid: WASD or click-to-move, a third-person orbit camera, and targets by click or Tab. | 3.5 |
| D06 | Combat | Guild Wars style builds: tab-target, an 8-slot skill bar from a large pool, and telegraphed attacks. | 5.4 |
| D07 | Progression | Classless. Skills from level 1 to level 99 increase by use. The weapon and the skill bar set the combat role. | 5.1 |
| D08 | Peoples | Humans only, with deep customization on one shared rig. | 3.1 |
| D09 | Kinematics | Layered procedural animation: clips, an animation graph, IK and procedural layers. | 4.3 |
| D10 | Death | A soft penalty. The Frontier is the exception. | 5.5 |
| D11 | World structure | One seamless open world for each shard, in streamed chunks. Story missions and dungeons are private instances. | 2.1 |
| D12 | Solo play | Online only. Instances and companions make solo play possible. | 2.1 |
| D13 | Companions | One companion at a time. The player selects a spirit familiar, a humanoid follower or a tamed creature. | 3.4 |
| D14 | World authoring | A deterministic world compiler makes the world from painted layer maps and data files. | 2.9 |
| D15 | Market | Regional exchanges, with one order book for each major city. Direct trade and NPC shops also exist. | 6.4 |
| D16 | Storage | Linked vaults for coins and personal items. Bulk goods stay in the warehouse of each city. | 6.3 |
| D17 | Travel | Earned fast travel through discovered waystones, for a fee. Bulk goods cannot go through waystones. | 2.4 |
| D18 | PvP | Opt-in duels, arenas and guild battles. One Frontier region has open PvP and item risk. | 7.4 |
| D19 | Hosting | Only official servers. The server code is open source. | 7.6 |
| D20 | Shards | Shards of approximately 1,000 players. Players can hop between shards. All shards share one economy. | 7.1 |
| D21 | Platforms | Windows, Linux and macOS are first-class. CI builds and tests all three. | 11.6 |
| D22 | Frame rate | 60 FPS at 1080p output on the spec floor (Low preset), with a dynamic render scale. | 9.2 |
| D23 | Netcode | `bevy_replicon` with the renet transport. Vespera code predicts the movement. | 8.5 |
| D24 | Physics | `avian3d` for colliders and spatial queries, and a Vespera kinematic character controller. | 4.7 |
| D25 | Database | SQLite for development and tests. PostgreSQL for production. | 8.6 |
| D26 | Game UI | `bevy_ui` with a Vespera widget set. `bevy_egui` only for debug tools in development builds. | 8.4 |
| D27 | Content | Typed RON data files and Yarn Spinner dialogue. No script language. | 8.7 |
| D28 | Comments | No `//` or `/* */` comments. Rustdoc comments only on public items of library crates, in ASD-STE100. | 11.4 |
| D29 | Documentation | All documentation in ASD-STE100. | 11.8 |
| D30 | Tests | Test-driven development from the top to the bottom. | 11.2 |
| D31 | Quality gates | The core CI gates, a coverage gate of 90%, weekly mutation tests and visual tests. | 11.6 |
| D32 | Repository | A public GitHub repository. Trunk-based development on `main`, with small commits and a push after each commit. | 11.7 |
| D33 | Binary assets | A separate asset store. The repository is text-only. The golden screenshots are also in the asset store. | 10.5 |
| D34 | Asset licenses | CC0-1.0 and CC-BY-4.0. OFL-1.1 only for fonts. | 10.3 |
| D35 | Asset tiers | Only the free tiers of asset packs. | 10.2 |
| D36 | First milestone | The living valley slice. | 12.2 |
| D37 | Human models | MakeHuman and MPFB: CC0 bodies, morph targets and the human rig. | 10.2 |
| D38 | Animation sources | The 100STYLE motion capture data (CC-BY-4.0) for idle, walk and the locomotion styles. A CC0 source for the jog, run and sprint clips at the speeds of 4.7. Universal Animation Library clips (CC0) for actions. | 4.3 |
| D39 | Nature sources | Poly Haven and ambientCG scans (CC0) for rocks, plants and materials. A tree generator makes the trees. | 10.2 |
| D40 | Shader compiler on Windows | DXC from the official release of Microsoft, as `dxcompiler.dll` next to the client. FXC made the main render pass approximately 30 times slower on the reference computer. | 9.1 |
| D41 | Human rig | The `game_engine` rig of MPFB: 53 bones, with fingers and without face bones. | 4.2 |
| D42 | Human LOD 0 | The import step simplifies the MakeHuman meshes to the triangle budget. The source files keep the full detail. | 10.6 |
| D43 | Human sources | Text recipes in the repository. MPFB builds the bodies from the recipes and the MakeHuman pack. | 10.6 |
