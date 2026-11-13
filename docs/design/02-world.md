# 2. World

## 2.1 Structure

Each world shard has one seamless open world. The client loads the world in chunks while the player moves. The open world has no load screens.

Story missions and dungeons are private instances. An instance is a separate copy of an area for one party. An instance party has 1 to 4 players, and each player can bring one companion. The difficulty of an instance changes with the size of the party.

## 2.2 Scale and coordinates

| Item | Value |
|---|---|
| World unit | 1 unit is 1 meter |
| Chunk size | 64 m × 64 m |
| Height map resolution | 1 sample for each meter |
| First playable area (M1) | A valley of 1 km × 1 km |
| First continent (target) | 8 km × 8 km |
| Maximum coordinate | 16 km from the world origin |

At 16 km from the origin, a 32-bit float has a precision of approximately 2 mm. This precision is sufficient. Thus the world does not use a floating origin.

## 2.3 Regions

The first continent has five regions. The names are temporary.

| Region | Biome | Purpose |
|---|---|---|
| Vesperdale | Meadows, a river and a village | The start region (M1) |
| Whisperwood | An old forest with giant trees | Woodcutting, spirits and the first story missions |
| Saltmere | A coast with cliffs and a port town | Fishing and trade by boat |
| Emberfell | Highlands with mines | Mining and smithing |
| The Frontier | Wild border lands | Open PvP and rich resources (see 7.5) |

## 2.4 Travel

Players travel on foot by default. Thus the world must be interesting to cross on foot.

Waystones:

- A waystone becomes active for a player when the player touches it for the first time.
- A player can travel between active waystones. Each travel has a coin fee. The fee increases with the distance.
- Bulk goods cannot go through a waystone (see 6.3).
- The Frontier has no waystones. A player must walk out of the Frontier.

Other travel:

- The Starweaving skill makes teleport charms. Each charm goes to one fixed place. Bulk goods cannot go with a teleport charm.
- The Agility skill opens shortcuts.
- Mounts and boats come in a later milestone.

## 2.5 Time of day

The server has one game clock. All shards use the same clock. Thus the time of day is the same on all shards.

| Phase | Length in real time |
|---|---|
| Dawn | 6 minutes |
| Day | 48 minutes |
| Dusk (the Vesper Hour) | 12 minutes |
| Night | 30 minutes |
| Full cycle | 96 minutes |

A player who plays for 96 minutes sees the Vesper Hour at least one time.

The moon has 8 phases. Each phase lasts for one full cycle. The moon phase changes the strength and the kinds of spirits.

## 2.6 Weather

Each region has its own weather. A state machine with probabilities controls the weather. The states are clear, cloudy, rain, storm, fog and snow. Snow occurs only in high regions.

The weather comes from a seed and the game clock. Thus all shards and all clients calculate the same weather, and the weather causes no network traffic.

Weather changes the world:

- Rain makes the ground wet, and fish bite more often.
- Fog decreases the view distance. During the Vesper Hour, fog brings more spirits.
- Storms send NPCs into buildings.

## 2.7 Life in the world

The world must feel alive. These systems make the life of the world:

| System | Description |
|---|---|
| NPC schedules | Each named NPC has a daily schedule: sleep, work, meals, talk and rest. Weather and the Vesper Hour change the schedule. |
| Wildlife | Animals live in herds or alone. Predators hunt prey. Animals run from danger and go back to their territory. |
| Spirits | Spirits come into the world during the Vesper Hour. Each spirit kind has its own behavior. |
| Ambient life | Birds, insects, fish and fireflies. The client makes ambient life only for visual effect. Ambient life has no game rules. |
| Reactions | NPCs and animals react to players, to fights and to noise. |

## 2.8 Simulation levels

The server cannot simulate all of the world at full detail all of the time. Thus the simulation has three levels:

| Level | Where | Update rate |
|---|---|---|
| Full | Chunks near a player | Each rules tick (20 Hz) |
| Reduced | Chunks at a medium distance from all players | 2 Hz, with simple movement |
| Schedule | Chunks far from all players | No movement. When a player comes near, each NPC moves to the correct place in its schedule. |

The schedule level makes sure that a player always sees each NPC at the correct place for the time of day.

## 2.9 World authoring

The world compiler makes the world from source data. The compiler is deterministic: the same input always gives the same output.

Inputs:

- Layer maps. These are painted images for height, biome, water, roads and region borders. The layer maps are in the asset store.
- Landmark files. These RON files give the position and the rotation of each hand-placed object, for example a house, a bridge or a shrine.
- Scatter rules. These RON files give the plants, rocks and trees of each biome, and their density.

Outputs, for each chunk:

- Terrain height and splat data.
- Colliders.
- Scatter instances for trees, plants and rocks.
- Navigation meshes for NPCs.
- Low-detail meshes and impostors for long distances.

A test calculates a hash of the compiler output. If the output changes, the test fails until a person accepts the new hash.

An in-game editor for fine placement comes in a later milestone. The editor writes the same landmark files.
