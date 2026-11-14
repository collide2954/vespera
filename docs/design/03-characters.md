# 3. Characters

## 3.1 Peoples

All player characters are human. Variety comes from customization and from the cultural origin. The origin has no effect on statistics.

| Origin | Home region | Style |
|---|---|---|
| Dalefolk | Vesperdale | Farmers and millers. Warm colors, wool and linen. |
| Woodwardens | Whisperwood | Foresters and spirit-speakers. Green colors, bark and leaf patterns. |
| Saltborn | Saltmere | Sailors and traders. Blue colors and cloth that salt has faded. |
| Fellkin | Emberfell | Miners and smiths. Dark leather, soot and copper. |

The origin sets the start outfit, the name suggestions, the voice set and a story connection to the home region. All players start in Vesperdale.

## 3.2 Customization

All humans use one shared rig, the Vespera human rig from MakeHuman. Thus each outfit and each animation fits each character.

The player can change these properties:

- Body shape: sliders from the MakeHuman morph targets, for example the proportions, the age and the weight.
- Build: a slider from slim to strong.
- Height: up to 6% more or less than the base height.
- Face shape: presets, and sliders from the MakeHuman morph targets.
- Hair style and hair color.
- Skin tone and eye color.
- Starlight markings: patterns on the skin that glow more during the Vesper Hour.
- Voice set.

MakeHuman contains the morph targets for the body, the build and the face. The client bakes the selected shape into the mesh when the character loads. Thus the shape has no cost in each frame.

## 3.3 Equipment

Equipment is visible on the character.

| Slot | Visible |
|---|---|
| Head, chest, legs, hands, feet | Yes |
| Back (cape) | Yes |
| Main hand, off hand | Yes |
| Neck, two rings | No |

Outfits come from the MakeHuman asset packs and from other CC0 or CC-BY-4.0 sources (see 10.2). A tint mask gives each outfit part up to three color channels. Dyes change these channels. Thus a small number of meshes gives many different looks.

## 3.4 Companions

Each player has one active companion at a time. The player selects one of three kinds:

| Kind | Source | Role |
|---|---|---|
| Spirit familiar | The player binds a spirit during the Vesper Hour, with the Spiritbinding skill. | Guardian, mender, scout or gatherer. The spirit kind sets the role. |
| Humanoid follower | The player completes the quest of a named NPC. | A personality, dialogue and gear. The follower uses the human rig. |
| Tamed creature | The player tames a wild animal with the Hunting skill. | Animal skills, for example tracking. Pack animals carry bulk goods. |

Companion rules:

- A companion has a level and 4 skill slots. A humanoid follower also has gear.
- The player gives simple orders: follow, guard, attack, hold position and, for a gatherer spirit, gather.
- In an instance, each player can bring one companion. The instance difficulty includes the companions.
- A companion cannot trade.

Wildlife and tamed creatures share one set of creature assets. Thus their style stays the same.

## 3.5 Controls

The default controls are a keyboard and a mouse. Gamepad controls come in a later milestone. The player can change all bindings.

| Action | Default input |
|---|---|
| Move | W, A, S, D, or a left click on the ground |
| Turn the camera | Hold the right mouse button and move the mouse |
| Zoom | Mouse wheel |
| Select a target | Left click on a creature, or Tab for the next target |
| Use a skill | Keys 1 to 8 |
| Interact | E, or a left click on an object |
| Jump | Space |
| Sprint | Left Shift |

## 3.6 Camera

The camera is a third-person orbit camera.

- The zoom range is from 1.5 m behind the shoulder to 25 m above the character.
- The camera moves nearer when a wall or a tree is between the camera and the character.
- The camera follows the character smoothly, with no sudden jumps.
- Camera shake is off by default. A setting turns it on.

## 3.7 Targets

- Tab selects the nearest hostile creature in front of the character. Each new Tab press selects the next creature.
- A click on a creature selects it.
- The UI shows the target of the current target.
- A skill with a target uses the current target. A skill with an area shows a circle on the ground before the player uses it.
