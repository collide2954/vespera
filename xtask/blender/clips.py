import json
import math
import sys

import bpy
from mathutils import Matrix, Quaternion, Vector

FPS = 60
CONTACT_WINDOW = 12
CONTACT_HEIGHT = 0.04
END_SEARCH = 4
IDLE_STEP = 15

MAP = {
    "pelvis": "Hips", "spine_01": "Chest", "spine_02": "Chest2", "spine_03": "Chest4",
    "neck_01": "Neck", "head": "Head",
    "clavicle_r": "RightCollar", "upperarm_r": "RightShoulder", "lowerarm_r": "RightElbow", "hand_r": "RightWrist",
    "clavicle_l": "LeftCollar", "upperarm_l": "LeftShoulder", "lowerarm_l": "LeftElbow", "hand_l": "LeftWrist",
    "thigh_r": "RightHip", "calf_r": "RightKnee", "foot_r": "RightAnkle", "ball_r": "RightToe",
    "thigh_l": "LeftHip", "calf_l": "LeftKnee", "foot_l": "LeftAnkle", "ball_l": "LeftToe",
}
SOURCE_CHILD = {
    "Hips": "Chest", "Chest": "Chest2", "Chest2": "Chest3", "Chest4": "Neck", "Neck": "Head",
    "RightCollar": "RightShoulder", "RightShoulder": "RightElbow", "RightElbow": "RightWrist",
    "LeftCollar": "LeftShoulder", "LeftShoulder": "LeftElbow", "LeftElbow": "LeftWrist",
    "RightHip": "RightKnee", "RightKnee": "RightAnkle", "RightAnkle": "RightToe",
    "LeftHip": "LeftKnee", "LeftKnee": "LeftAnkle", "LeftAnkle": "LeftToe",
}
TARGET_CHILD = {
    "pelvis": "spine_01", "spine_01": "spine_02", "spine_02": "spine_03", "spine_03": "neck_01", "neck_01": "head",
    "clavicle_r": "upperarm_r", "upperarm_r": "lowerarm_r", "lowerarm_r": "hand_r", "hand_r": "middle_01_r",
    "clavicle_l": "upperarm_l", "upperarm_l": "lowerarm_l", "lowerarm_l": "hand_l", "hand_l": "middle_01_l",
    "thigh_r": "calf_r", "calf_r": "foot_r", "foot_r": "ball_r",
    "thigh_l": "calf_l", "calf_l": "foot_l", "foot_l": "ball_l",
}
LOOP_BONES = ["Hips", "Chest", "Chest4", "Neck", "RightShoulder", "RightElbow", "LeftShoulder", "LeftElbow",
              "RightHip", "RightKnee", "RightAnkle", "LeftHip", "LeftKnee", "LeftAnkle"]
FEET = ("RightAnkle", "RightToe", "LeftAnkle", "LeftToe")


def heading(vector):
    return math.atan2(vector.x, -vector.y)


def wrap(angle):
    return (angle + math.pi) % (2.0 * math.pi) - math.pi


def rest_head(armature, name):
    return armature.matrix_world @ armature.data.bones[name].head_local


def rest_tail(armature, name):
    return armature.matrix_world @ armature.data.bones[name].tail_local


def direction(armature, name, child_map):
    child = child_map.get(name)
    if child is not None:
        return (rest_head(armature, child) - rest_head(armature, name)).normalized()
    return (rest_tail(armature, name) - rest_head(armature, name)).normalized()


def facing_spread(window):
    mean = math.atan2(sum(math.sin(item["facing"]) for item in window),
                      sum(math.cos(item["facing"]) for item in window))
    return mean, max(abs(wrap(item["facing"] - mean)) for item in window)


def loop_error(start, end):
    angle = sum(start["pose"][name].rotation_difference(end["pose"][name]).angle for name in LOOP_BONES)
    return math.degrees(angle) + abs(start["hips"].z - end["hips"].z) * 1000.0


def sample(source, first, last):
    scene = bpy.context.scene
    frames = []
    for frame in range(first, last + 1):
        scene.frame_set(frame)
        bones = source.pose.bones
        world = source.matrix_world
        right = (world @ bones["RightHip"].head) - (world @ bones["LeftHip"].head)
        right.z = 0.0
        frames.append({
            "frame": frame,
            "hips": world @ bones["Hips"].head,
            "left": (world @ bones["LeftAnkle"].head).z,
            "facing": heading(Vector((0.0, 0.0, 1.0)).cross(right.normalized())),
            "pose": {name: bones[name].matrix_basis.to_quaternion() for name in LOOP_BONES},
        })
    return frames


def find_cycle(frames, clip):
    lows = [item["left"] for item in frames]
    floor = min(lows)
    contacts = [index for index in range(CONTACT_WINDOW, len(frames) - CONTACT_WINDOW)
                if lows[index] < floor + CONTACT_HEIGHT
                and lows[index] == min(lows[index - CONTACT_WINDOW:index + CONTACT_WINDOW + 1])]
    best = None
    for first, second in zip(contacts, contacts[1:]):
        for end in range(second - END_SEARCH, min(second + END_SEARCH, len(frames) - 1) + 1):
            if not clip["min_seconds"] <= (end - first) / FPS <= clip["max_seconds"]:
                continue
            mean, spread = facing_spread(frames[first:end + 1])
            if math.degrees(spread) > clip["max_turn_degrees"]:
                continue
            error = loop_error(frames[first], frames[end])
            if best is None or error < best[0]:
                best = (error, first, end, mean)
    return best, len(contacts)


def find_idle(frames, clip):
    best = None
    shortest = int(clip["min_seconds"] * FPS)
    longest = int(clip["max_seconds"] * FPS)
    for length in range(shortest, longest + 1, IDLE_STEP):
        for first in range(0, len(frames) - length, IDLE_STEP):
            window = frames[first:first + length + 1]
            travel = max((item["hips"] - window[0]["hips"]).xy.length for item in window)
            if travel > clip["max_travel_meters"]:
                continue
            mean, _ = facing_spread(window)
            error = loop_error(window[0], window[-1])
            if best is None or error < best[0]:
                best = (error, first, first + length, mean)
    return best


def import_source(path):
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    bpy.ops.import_anim.bvh(filepath=path, target="ARMATURE", global_scale=0.01, frame_start=1,
                            use_fps_scale=False, rotate_mode="NATIVE", axis_forward="-Z", axis_up="Y")
    return bpy.context.view_layer.objects.active


def cut(clip):
    source = import_source(clip["bvh"])
    frames = sample(source, clip["first"], clip["last"])
    if clip["kind"] == "idle":
        found, contacts = find_idle(frames, clip), 0
    else:
        found, contacts = find_cycle(frames, clip)
    if found is None:
        raise SystemExit(f"error: the clip {clip['name']} has no loop that obeys its cut rules ({contacts} contacts)")
    error, first, end, mean_facing = found
    scene = bpy.context.scene
    world_rotations = []
    hips = []
    for index in range(first, end + 1):
        scene.frame_set(frames[index]["frame"])
        world = source.matrix_world
        world_rotations.append({name: (world @ source.pose.bones[name].matrix).to_quaternion() for name in set(MAP.values())})
        hips.append(world @ source.pose.bones["Hips"].head)
    floor = min(min(rest_head(source, name).z, rest_tail(source, name).z) for name in FEET)
    return {
        "name": clip["name"],
        "error": error,
        "contacts": contacts,
        "source_frames": [frames[first]["frame"], frames[end]["frame"]],
        "facing": mean_facing,
        "rest": {name: source.matrix_world.to_quaternion() @ source.data.bones[name].matrix_local.to_quaternion()
                 for name in set(MAP.values())},
        "directions": {name: direction(source, name, SOURCE_CHILD) for name in set(MAP.values())},
        "hips_height": rest_head(source, "Hips").z - floor,
        "rotations": world_rotations,
        "hips": hips,
    }


def bone_order(target):
    order = []

    def visit(bone):
        order.append(bone.name)
        for child in bone.children:
            visit(child)

    for bone in target.data.bones:
        if bone.parent is None:
            visit(bone)
    return order


def retarget(target, order, loop):
    bones = target.data.bones
    world_q = target.matrix_world.to_quaternion()
    inverse = target.matrix_world.inverted()
    forward = Vector((0.0, 0.0, 1.0)).cross((rest_head(target, "thigh_r") - rest_head(target, "thigh_l")).normalized())
    turn_q = Quaternion((0.0, 0.0, 1.0), wrap(heading(forward) - loop["facing"]))
    turn_m = turn_q.to_matrix().to_4x4()
    ratio = rest_head(target, "pelvis").z / loop["hips_height"]
    align = {}
    for name, source_name in MAP.items():
        target_rest = world_q @ bones[name].matrix_local.to_quaternion()
        align[name] = direction(target, name, TARGET_CHILD).rotation_difference(loop["directions"][source_name]) @ target_rest
    count = len(loop["rotations"]) - 1
    start = turn_m @ (loop["hips"][0] * ratio)
    velocity = (turn_m @ (loop["hips"][-1] * ratio) - start) / (count / FPS)
    velocity.z = 0.0
    poses = []
    for index in range(count + 1):
        pose_arm = {}
        basis_of = {}
        for name in order:
            bone = bones[name]
            parent_pose = Matrix.Identity(4) if bone.parent is None else pose_arm[bone.parent.name]
            rest_relative = bone.matrix_local if bone.parent is None else bone.parent.matrix_local.inverted() @ bone.matrix_local
            if name not in MAP:
                pose_arm[name] = parent_pose @ rest_relative
                continue
            source_name = MAP[name]
            delta = loop["rotations"][index][source_name] @ loop["rest"][source_name].inverted()
            desired = (world_q.inverted() @ turn_q @ delta @ align[name]).to_matrix().to_4x4()
            if name == "pelvis":
                position = turn_m @ (loop["hips"][index] * ratio)
                drift = start + velocity * (index / FPS)
                desired.translation = inverse @ Vector((position.x - drift.x, position.y - drift.y, position.z))
            else:
                desired.translation = (parent_pose @ rest_relative).translation
            basis = rest_relative.inverted() @ parent_pose.inverted() @ desired
            basis_of[name] = basis
            pose_arm[name] = parent_pose @ rest_relative @ basis
        poses.append(basis_of)
    return poses, velocity.length


def keyframes(target, poses, name):
    count = len(poses) - 1
    action = bpy.data.actions.new(name)
    target.animation_data.action = action
    correction = {}
    for bone in MAP:
        rotation = poses[0][bone].to_quaternion() @ poses[-1][bone].to_quaternion().inverted()
        if rotation.w < 0.0:
            rotation.negate()
        correction[bone] = rotation
    shift = poses[0]["pelvis"].translation - poses[-1]["pelvis"].translation
    previous = {}
    for index in range(count + 1):
        weight = index / count
        for bone in MAP:
            rotation = Quaternion().slerp(correction[bone], weight) @ poses[index][bone].to_quaternion()
            if bone in previous and previous[bone].dot(rotation) < 0.0:
                rotation.negate()
            previous[bone] = rotation
            pose_bone = target.pose.bones[bone]
            pose_bone.rotation_quaternion = rotation
            pose_bone.keyframe_insert("rotation_quaternion", frame=index)
            if bone == "pelvis":
                pose_bone.location = poses[index][bone].translation + shift * weight
                pose_bone.keyframe_insert("location", frame=index)
    track = target.animation_data.nla_tracks.new()
    track.name = name
    track.strips.new(name, 0, action)
    target.animation_data.action = None


def main():
    arguments = sys.argv[sys.argv.index("--") + 1:]
    body_path, spec_path, glb_path, report_path = arguments[0], arguments[1], arguments[2], arguments[3]
    with open(spec_path, encoding="utf-8-sig") as spec_file:
        spec = json.load(spec_file)
    loops = [cut(clip) for clip in spec["clips"]]

    bpy.ops.wm.open_mainfile(filepath=body_path)
    scene = bpy.context.scene
    scene.render.fps = FPS
    scene.render.fps_base = 1.0
    target = next(obj for obj in bpy.data.objects if obj.type == "ARMATURE")
    for pose_bone in target.pose.bones:
        pose_bone.rotation_mode = "QUATERNION"
    target.animation_data_create()
    order = bone_order(target)
    report = {}
    for loop in loops:
        poses, speed = retarget(target, order, loop)
        keyframes(target, poses, loop["name"])
        count = len(poses) - 1
        report[loop["name"]] = {
            "seconds": round(count / FPS, 4),
            "speed": round(speed, 4),
            "loop_error": round(loop["error"], 2),
            "source_frames": loop["source_frames"],
        }
        print(f"CLIP {loop['name']}: {count / FPS:.2f} s, {speed:.2f} m/s, loop error {loop['error']:.1f}, "
              f"source frames {loop['source_frames'][0]}-{loop['source_frames'][1]}, contacts {loop['contacts']}")
    for pose_bone in target.pose.bones:
        pose_bone.rotation_quaternion = Quaternion()
        pose_bone.location = Vector()
    bpy.ops.export_scene.gltf(filepath=glb_path, export_format="GLB", use_selection=False, export_apply=True,
                              export_skins=True, export_animations=True, export_animation_mode="NLA_TRACKS",
                              export_yup=True)
    with open(report_path, "w", encoding="utf-8") as report_file:
        json.dump({"clips": report}, report_file, indent=2)
    print(f"EXPORTED {glb_path}")


main()
