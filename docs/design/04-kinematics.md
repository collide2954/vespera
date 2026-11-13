# 4. Body kinematics

## 4.1 Goal

Bodies must move with weight, balance and intent. These rules apply to all characters:

- Feet do not slide on the ground.
- Feet touch the ground on slopes and on steps.
- Hands touch the tools and the objects that they use.
- Heads look at the things that are important to the character.
- The body leans into changes of speed and direction.
- Hair, capes and straps move with the body.

## 4.2 Rigs

| Rig | Users |
|---|---|
| Universal humanoid rig (Quaternius) | All humans: players, NPCs and humanoid followers |
| Quadruped rig | Four-legged animals and tamed creatures |
| Bird rig | Birds |
| Spirit rig | Spirits. The rig has few bones. Shaders and particles make most of the motion. |

One file maps the bone names of each rig to Vespera joint names. All code uses the Vespera joint names. An asset test makes sure that each model contains all joints of its rig.

## 4.3 Animation layers

The client makes the pose of each character from six layers. Each layer changes the result of the layer below it.

| No. | Layer | Description |
|---|---|---|
| 1 | Locomotion | A blend space of idle, walk, jog, run and sprint clips in 8 directions. Clips for start, stop and turn in place. Clips for jump, fall and land. |
| 2 | Actions | Clips for gather actions, craft actions, attacks and spells. A bone mask limits most actions to the upper body. |
| 3 | Additive motion | Breath, lean into acceleration, bank on turns, forward lean on slopes, a tired pose and a carry pose for heavy loads. |
| 4 | Inverse kinematics (IK) | Foot IK, hand IK and look-at. |
| 5 | Secondary motion | Spring bones for hair, capes, straps, tails and ears. |
| 6 | Reactions | Hit flinches in the direction of the hit, stagger and knockdown. A cosmetic ragdoll on death. |

The clips come from Universal Animation Library 1 and 2 (free parts). All clips use the universal humanoid rig.

## 4.4 Locomotion

- Distance matching: the playback speed of each locomotion clip follows the real ground speed of the character. Thus the feet do not slide.
- Foot locks: during the stance phase, each foot stays at a fixed position in the world.
- Momentum: the character controller has curves for acceleration and deceleration. The animation shows the start and the stop of each movement.
- Turns: a fast turn plays a turn clip. A slow turn uses lean and bank.
- Steps and slopes: foot IK puts each foot on the ground. The pelvis goes down when one foot is lower than the other foot.

## 4.5 Inverse kinematics

| Solver | Use | Rules |
|---|---|---|
| Two-bone IK | Legs and arms | Keeps the bone lengths. A pole vector sets the direction of the knee or the elbow. |
| Look-at chain | Head, neck and upper spine | Each joint turns a part of the total angle. Each joint has angle limits. |
| Ground probe | Foot targets | A shape cast below each foot finds the ground position and the ground normal. |

Hand IK uses grip points and contact points. For example, an axe has a grip point, and a tree has a contact point for the axe. The content definition of each object gives these points.

## 4.6 Secondary motion

Spring bones use Verlet integration with a fixed time step. Each spring chain has values for stiffness, damping and gravity. Body colliders (spheres and capsules) prevent the chain from going into the body.

## 4.7 Character controller

The server and the client use the same character controller code. The client predicts the movement of the local player with this code (see 8.5).

| Property | Value |
|---|---|
| Shape | Capsule, radius 0.3 m, height 1.8 m |
| Step height | 0.35 m |
| Maximum walkable slope | 45° |
| Walk speed | 1.6 m/s |
| Jog speed | 3.6 m/s |
| Run speed | 5.4 m/s |
| Sprint speed | 7.2 m/s, with a stamina cost |
| Acceleration | From stop to jog speed in 0.25 s |
| Fixed time step | 1/60 s |

On a slope of more than 45°, the character slides down. The controller uses avian3d shape casts against the static world.

## 4.8 Animation and the network

The server does not calculate poses. The server sends a small kinematic state for each character:

- Position, velocity and facing direction.
- Movement mode: ground, air, swim or knocked down.
- The current action and its start time.
- The current target and the current look-at point.

Each client calculates all poses from this state. Thus the network traffic stays small, and the network does not change the pose quality.

## 4.9 Animation levels of detail

| Level | Distance | Layers | Update rate |
|---|---|---|---|
| Near | 0 m to 20 m | All | Each frame |
| Medium | 20 m to 50 m | 1, 2, 3 and 6, and foot IK | 30 Hz |
| Far | 50 m to 120 m | 1 and 2 | 15 Hz |
| Distant | More than 120 m | 1, with a reduced skeleton | 5 Hz |

On the spec floor, all animation work must use 2 ms of CPU time or less for each frame, with 100 visible characters.

## 4.10 Tests

| Test | Pass condition |
|---|---|
| Two-bone IK reach | The end joint is less than 1 mm from a reachable target. |
| Bone lengths | IK does not change a bone length by more than 0.01 mm. |
| Joint limits | No solver output goes outside the joint limits. |
| Foot slide | During stance, the world speed of a locked foot is less than 2 cm/s. |
| Look-at limits | The head never turns more than its limit. |
| Spring stability | Without input, the energy of a spring chain decreases over time. |
| Controller determinism | On one platform, the same inputs always give the same states. |
| Controller trace | For a recorded input sequence, the output agrees with a stored trace within 1 mm on Windows, Linux and macOS. |

## 4.11 Later work

Motion matching for the local player can come after the alpha. It will use the same layers 2 to 6.
