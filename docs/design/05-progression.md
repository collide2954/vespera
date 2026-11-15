# 5. Progression and combat

## 5.1 Skills

Vespera has no classes. Each character has all skills. Each skill has a level from 1 to 99. A skill receives experience points (XP) when the player uses it.

The combat role of a character comes from its weapon and from the 8 skills on its skill bar. A player can change the role at a shrine.

## 5.2 Experience curve

This formula gives the XP that is necessary to go from level L to level L + 1:

```text
xp_step(L) = round(80 × 1.1^(L − 1))
```

The total XP for level 99 is approximately 9.1 million. Level 92 is approximately half of the total XP for level 99. Content data can change the base value (80) and the growth value (1.1).

## 5.3 Skill list

The full game has 23 skills in four families. The names are temporary.

Combat skills:

| Skill | Description |
|---|---|
| Blades | Swords, axes, daggers and spears |
| Marksmanship | Bows, slings and thrown weapons |
| Starcalling | Attack magic from starlight |
| Warding | Shields, armor and protective wards |
| Mending | Heal magic and support magic |
| Spiritbinding | Bind and command spirit familiars |
| Vitality | Health. Vitality receives XP from all combat. |

Gathering skills:

| Skill | Description |
|---|---|
| Woodcutting | Cut trees for logs and bark |
| Mining | Mine ore, stone and gems |
| Fishing | Catch fish, shellfish and pearls |
| Foraging | Collect herbs, fibers and mushrooms |
| Hunting | Track and trap animals, and tame creatures |

A gathering action continues until the resource is empty or the pack is full. The action stops when the player moves out of reach of the resource. To continue, the player starts the action again.

Artisan skills:

| Skill | Description |
|---|---|
| Smithing | Make metal bars, weapons, armor and tools |
| Woodworking | Make planks, bows, staves, handles and furniture |
| Tailoring | Make cloth items and leather items |
| Cooking | Make food |
| Alchemy | Make potions and dyes |
| Jewelcrafting | Cut gems, and make rings and amulets |
| Starweaving | Put starlight into items. Make teleport charms. Stronger during the Vesper Hour. |

Support skills:

| Skill | Description |
|---|---|
| Agility | Use shortcuts. Increases the stamina. |
| Farming | Grow crops, herbs and trees |
| Building | Build houses and guild halls (later milestone) |
| Merchanting | Lead caravans and accept courier contracts. Increases the exchange order limit. |

## 5.4 Combat

Combat uses tab-target. The player selects a target. A skill hits the target if the target is in range and in the line of sight.

Position is important:

- Some skills hit only from the side or from the back.
- A large enemy attack shows a telegraph on the ground before it hits. The player must move out of the telegraph.
- A shield blocks only attacks from the front.

### 5.4.1 Skill bar

- The skill bar has 8 slots.
- One slot at most can contain an elite skill.
- A weapon skill works only with the related weapon in the hand. For example, a bow skill works only with a bow.
- The player can change the skill bar only at a shrine or in a town. Thus the build is an important decision before a fight.

### 5.4.2 Skill sources

- Skill levels unlock skills. For example, Blades level 10 unlocks a set of sword skills.
- Trainers in towns teach skills for coins.
- Quests give skills.
- Elite enemies have elite skills. During the Vesper Hour, a player can capture the echo of an elite skill from a defeated elite enemy.

### 5.4.3 Resources

| Resource | Description |
|---|---|
| Health | Vitality sets the maximum. Health comes back slowly out of combat. |
| Energy | Spells use energy. Energy comes back over time. The armor weight changes the rate. |
| Momentum | Weapon hits make momentum. Some weapon skills use momentum. |

### 5.4.4 Effects

An effect changes a creature for a period of time. Bad effects include bleed, burn, poison, weakness, slow, root, blind and silence. Good effects include wards and blessings. Some skills remove effects.

### 5.4.5 Armor

Armor has three weights: light, medium and heavy. Heavy armor gives more protection, but less energy return. Any character can use any armor weight. The Warding level sets the maximum armor tier.

## 5.5 Death

Outside the Frontier, death has a soft penalty:

1. The character goes back to the last shrine that it activated.
2. Equipped gear loses 10% of its durability.
3. The character has the Spirit-weary effect for 5 minutes. Spirit-weary decreases the maximum health and the maximum energy by 10%.
4. The character loses no items and no XP.

In the Frontier, death also has an item risk (see 7.5).

A repair at a smith costs coins. This cost is an important coin sink.

## 5.6 Quests and missions

| Type | Description |
|---|---|
| Story mission | A part of the main story, in a private instance. Bonus goals give extra rewards. |
| Quest | A story with puzzles and characters in the open world. Quests unlock areas, skills and items. |
| Task | A short job that a player can do again, for example a bounty or a delivery. |
| Vesper event | A world event during the Vesper Hour. All players near the event can join it. |

Dialogue uses Yarn Spinner files (see 8.7). An automated solo test completes each story mission (see 1.2).
