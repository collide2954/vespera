# 1. Vision

## 1.1 Summary

Vespera is a 3D fantasy MMORPG with an open world. Guild Wars and RuneScape are its main inspirations. Vespera is free and open source. It uses the Rust language and the Bevy engine.

The player explores an old, quiet world that is full of hidden magic. The player gathers resources, makes items, trades and fights. The player can play alone with a companion, or with other players.

## 1.2 Pillars

All design decisions must agree with seven pillars. Each pillar has a check. A check is a measurable result. It shows if the game has the quality of the pillar.

| Pillar | Description | Check |
|---|---|---|
| Mystical | The world feels old and full of hidden magic. Light, fog, sound and story make this feeling. | Each region has at least one secret that only the Vesper Hour shows. |
| Alive | People and creatures have reasons to move. They react to time, weather and players. | No named NPC stays in one place for longer than its schedule permits. |
| Open | The player can walk from any point to any other point. | The open world has no load screens. Only the entry to an instance has a load screen. |
| Solo-friendly | A solo player with a companion can complete the story. | An automated solo test completes each story mission. |
| Player economy | Players gather and make most goods. Geography changes prices. | At least 80% of the traded item value comes from goods that players made. |
| Fast | The game runs well on a ThinkPad T590 with integrated graphics. | The benchmark shows 60 FPS on the Low preset. |
| Clean | The code is small, tested and easy to read. | All CI gates pass on each commit. |

## 1.3 Inspirations

Vespera takes these ideas from Guild Wars:

- Skill builds with an 8-slot skill bar and a large skill pool.
- Story missions in private instances.
- Tab-target combat that stays good when the network latency is high.

Vespera takes these ideas from RuneScape:

- A seamless open world.
- Classless skills from level 1 to level 99.
- Quests with personality and humor.
- A player economy with an exchange.
- World shards that players can hop between.

Vespera adds these ideas:

- The Vesper Hour (see 1.4).
- Regional exchanges, and physical transport of bulk goods.
- Bodies that move with weight and balance.
- One companion for each player, from three kinds.

Vespera does not copy names, characters, art, music, maps or rules text from other games.

## 1.4 The Vesper Hour

Vesper is the evening star. When the sun goes down, Vesper comes into the sky. For a short time, the veil between the living world and the spirit world becomes thin. This time is the Vesper Hour.

During the Vesper Hour:

- Spirits walk in the world. Some spirits are friendly and some are dangerous.
- Rare creatures and rare resources come into the world.
- Starlight collects in pools. The Starweaving skill uses this starlight.
- Players can bind a spirit familiar.
- World events start in some regions.

After the Vesper Hour, the night is dark and more dangerous. At dawn, the veil closes.

The effects of the Vesper Hour must not flash or strobe. Flashing light can cause seizures in some players.

## 1.5 Tone

The tone is quiet wonder, with some sadness and much hope. The world is old and has lost much. But its people continue to live, work and laugh. The tone is not grim and not silly. The humor is gentle.

## 1.6 Scope limits

These items are not in the plan:

- Flying mounts and player flight.
- Voice acting.
- Mobile, console, web and VR versions.
- The sale of power or of an advantage for real money.
