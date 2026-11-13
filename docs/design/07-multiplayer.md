# 7. Multiplayer

## 7.1 Shards

A shard is one copy of the open world. Each shard has a capacity of approximately 1,000 players.

- Characters belong to the account, not to a shard.
- A player can hop to a different shard outside combat and outside instances. A hop takes less than 10 seconds.
- Each shard has its own world state: resource nodes, creatures and NPC positions.
- All shards share the game clock, the weather, the exchanges, the vaults, the warehouses, the guilds, the chat channels and the friend lists.

## 7.2 Services

| Service | Purpose |
|---|---|
| Login service | Accounts, passwords and connect tokens for shards |
| World shard | One open world: movement, combat, gathering, NPCs and creatures |
| Instance host | Story missions and dungeons. At first, the shard process also hosts instances. |
| Exchange service | All regional exchanges, for all shards |
| Social service | Chat, friends, parties and guilds across all shards |
| Database | SQLite for development, PostgreSQL for production |

Milestone M1 has only one world shard. The other services come in later milestones.

## 7.3 Social features

- Friend lists and block lists.
- Parties of up to 4 players. Each player can bring one companion.
- Guilds with ranks, a guild chat and a guild message. Guild halls come with the Building skill.
- Chat channels: local, party, guild, private, regional trade and shard.
- Emotes.

## 7.4 Opt-in PvP

- Duels. Both players must accept the duel.
- Arenas. Instanced matches with matchmaking. Arena rules make the gear of all players equal.
- Guild battles. Instanced matches between two guilds.

## 7.5 The Frontier

The Frontier is a region with open PvP.

- A clear warning shows at each border of the Frontier. The player must confirm before entry.
- The resources in the Frontier are rich and rare.
- Camps at the border are safe zones.
- The Frontier has no waystones.
- A player can attack another player only if their combat levels differ by 15 or less.
- On death in the Frontier, the character drops all items in the pack and all carried bulk goods. Equipped gear stays with the character, but loses 25% of its durability.
- For 60 seconds, only the killer can see the dropped items. After that time, all players can see them.

## 7.6 Hosting

The project runs the only Vespera servers. The server code is open source, but community servers are not part of the plan.

## 7.7 Accounts

- An account has an email address and a password.
- The login service keeps each password as an Argon2id hash.
- The login service verifies each email address.
- Two-factor authentication with TOTP is optional.
- The login service gives a short-lived connect token for one shard.
- Rate limits protect the login service.

## 7.8 Security and cheats

The client is open source. Thus the server must not trust the client.

- The client sends only intents, for example "move in this direction" or "use skill 3 on target X".
- The server validates each intent: range, line of sight, recharge time, resources and speed.
- The server is the authority for all game state.
- The server limits the message rate of each client.
- Bot detection on the server comes in a later milestone.
