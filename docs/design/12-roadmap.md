# 12. Roadmap

Each milestone has goals and acceptance criteria. Where possible, acceptance tests in `vespera_testkit` check the criteria.

## 12.1 M0: Foundations

Goals:

- A Cargo workspace with all crates from 8.2, as empty skeletons.
- The files `rust-toolchain.toml`, `deny.toml`, `typos.toml`, `rustfmt.toml` and `comment-allowlist.toml`, and the workspace lints.
- The xtask commands `check-comments`, `check-layers`, `check-text-only`, `check-docs` and `fetch-assets`.
- `vespera_testkit` with a headless server and a headless client in one process.
- All CI jobs from 11.6 that apply to empty crates.
- The `vespera-assets` repository with a first empty release.

Acceptance criteria:

- CI is green on Windows, Linux and macOS.
- An acceptance test starts a server and a client, and the client connects to the server.

## 12.2 M1: Living valley slice

Goals:

- The world compiler makes a valley of 1 km × 1 km from layer maps.
- Terrain chunks stream around the camera.
- The sky, the sun, the moon, the day and night cycle, and the Vesper Hour effects.
- Grass, trees and rocks from the Stylized Nature MegaKit, with wind.
- One player character with locomotion, foot IK, look-at and spring bones.
- The hybrid controls and the orbit camera.
- Client prediction against a local server through renet.
- The Woodcutting skill. The player cuts a tree and receives logs and XP. The tree falls, and later it grows again.
- A pack with 28 slots.
- The benchmark from 9.7.

Acceptance criteria:

- A player walks across the valley with no load screen and no visible foot slide.
- The acceptance test `player_receives_oak_logs_after_cutting_an_oak_tree` passes.
- The benchmark shows 60 FPS on the Low preset on the reference computer.

## 12.3 M2: Gather and make

Goals:

- The Mining, Fishing and Foraging skills.
- The Smithing, Cooking and Woodworking skills, with recipes and stations.
- The inventory UI.
- A vault and a warehouse in one city.
- Persistence with SQLite and PostgreSQL.

Acceptance criteria:

- A player gathers, crafts, stores items, logs out and logs in again. The state of the character stays the same.
- All persistence tests pass on both databases.

## 12.4 M3: Shared world

Goals:

- The login service, accounts and connect tokens.
- Many clients on one shard, with interest management.
- Chat, parties and friend lists.
- Two shards, and shard hops.

Acceptance criteria:

- With 200 bot clients, the rules tick stays at 25 ms p99 or less.
- A player hops to a different shard in less than 10 seconds.

## 12.5 M4: Combat and companions

Goals:

- The skill bar, the combat skills and the first 30 combat skills on the bar.
- Effects, creatures with AI, telegraphs, death and shrines.
- Spirit familiars and the Spiritbinding skill.

Acceptance criteria:

- In an automated test, a solo player with a spirit familiar defeats an elite creature.

## 12.6 M5: Economy

Goals:

- Direct trade and NPC shops.
- Regional exchanges in two cities.
- Couriers, basic caravans, waystones and fees.
- Economy metrics and the ledger.

Acceptance criteria:

- Property tests show that no operation makes or destroys coins or items, except the declared sources and sinks.
- In a bot simulation, the prices for the same goods are different in the two cities.

## 12.7 M6: Living world

Goals:

- NPC schedules, wildlife, weather and ambient audio.
- Vesper Hour events.
- Humanoid followers, tamed creatures and the Hunting skill.

Acceptance criteria:

- The checks of the pillars Alive and Mystical pass (see 1.2).

## 12.8 M7: Story and the Frontier

Goals:

- The first story missions in instances.
- A dungeon with party scaling.
- Quests in Yarn Spinner.
- The Frontier with its PvP rules, duels and guilds.

Acceptance criteria:

- An automated solo test completes each story mission.

## 12.9 M8: Alpha

Goals:

- The five regions of the first continent.
- A performance pass and an accessibility pass.
- Gamepad controls and all quality presets.
- Deployment of the official servers.

Acceptance criteria:

- A closed alpha test with external players starts.

## 12.10 Open questions

| No. | Question | Default |
|---|---|---|
| 1 | Which source for the music: CC0 or CC-BY-4.0 libraries, a composer, or both? | None |
| 2 | Which CC0 creature packs agree with the art style? | Test the Quaternius animal packs first. |
| 3 | What are the details of the Frontier death rules? | The rules in 7.5. |
| 4 | Which protocol do the internal services use? | Decide in M3. |
| 5 | Is 4 players the correct party size? | 4 players |
| 6 | Is 96 minutes the correct length for a day cycle? | 96 minutes |
| 7 | What are the combat values: damage, health, energy and recharge times? | Decide in M4. |
| 8 | What is the gamepad layout? | Decide in M8. |
