import json
import sys

import bpy

from bl_ext.blender_org.mpfb.services.assetservice import AssetService
from bl_ext.blender_org.mpfb.services.humanservice import HumanService
from bl_ext.blender_org.mpfb.services.meshservice import MeshService
from bl_ext.blender_org.mpfb.services.objectservice import ObjectService
from bl_ext.blender_org.mpfb.services.targetservice import TargetService

RIG = "game_engine"


def asset_path(subdir, kind, label):
    entry = AssetService.get_asset_list(subdir, kind).get(label)
    if entry is None:
        raise SystemExit(f"error: MPFB has no {subdir} asset with the name {label}")
    for key in ("full_path", "path", "absolute_path"):
        if key in entry:
            return str(entry[key])
    raise SystemExit(f"error: the MPFB entry for {label} has no path")


def body_height(basemesh):
    depsgraph = bpy.context.evaluated_depsgraph_get()
    evaluated = basemesh.evaluated_get(depsgraph)
    mesh = evaluated.to_mesh()
    heights = [(evaluated.matrix_world @ vertex.co).z for vertex in mesh.vertices]
    evaluated.to_mesh_clear()
    return max(heights) - min(heights)


def triangle_count():
    depsgraph = bpy.context.evaluated_depsgraph_get()
    total = 0
    for obj in bpy.data.objects:
        if obj.type != "MESH":
            continue
        evaluated = obj.evaluated_get(depsgraph)
        mesh = evaluated.to_mesh()
        mesh.calc_loop_triangles()
        total += len(mesh.loop_triangles)
        evaluated.to_mesh_clear()
    return total


def build(recipe):
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    macro = TargetService.get_default_macro_info_dict()
    for key, value in recipe["shape"].items():
        macro[key] = value
    basemesh = HumanService.create_human(
        mask_helpers=True,
        detailed_helpers=True,
        extra_vertex_groups=True,
        feet_on_ground=True,
        scale=0.1,
        macro_detail_dict=macro,
    )
    basemesh.use_shape_key_edit_mode = True
    bpy.ops.object.select_all(action="DESELECT")
    bpy.context.view_layer.objects.active = basemesh
    basemesh.select_set(True)
    bpy.ops.object.mode_set(mode="EDIT", toggle=False)
    MeshService.select_all_vertices_in_vertex_group_for_active_object("body", deselect_other=True)
    bpy.ops.object.mode_set(mode="OBJECT", toggle=False)

    rig = HumanService.add_builtin_rig(basemesh, RIG, import_weights=True)
    if rig is None:
        raise SystemExit(f"error: MPFB cannot add the {RIG} rig")
    HumanService.set_character_skin(
        asset_path("skins", "mhmat", recipe["skin"]),
        basemesh,
        bodyproxy=None,
        skin_type="MAKESKIN",
        material_instances=False,
    )
    height = body_height(basemesh)
    for kind in ("eyes", "eyebrows", "eyelashes", "hair"):
        HumanService.add_mhclo_asset(
            asset_path(kind, "mhclo", recipe[kind]),
            basemesh,
            asset_type=kind,
            subdiv_levels=0,
            material_type="MAKESKIN",
        )
    for garment in recipe["clothes"]:
        HumanService.add_mhclo_asset(
            asset_path("clothes", "mhclo", garment),
            basemesh,
            asset_type="Clothes",
            subdiv_levels=0,
            material_type="MAKESKIN",
        )
    local_lowest = ObjectService.get_lowest_point(basemesh)
    matrix = basemesh.matrix_world
    rig.location.z -= matrix.translation.z + matrix.to_scale().z * local_lowest
    bpy.context.view_layer.update()
    return rig, height


def main():
    arguments = sys.argv[sys.argv.index("--") + 1:]
    recipe_path, blend_path = arguments[0], arguments[1]
    with open(recipe_path, encoding="utf-8-sig") as recipe_file:
        recipe = json.load(recipe_file)
    rig, height = build(recipe)
    bpy.ops.wm.save_as_mainfile(filepath=blend_path)
    print(f"HUMAN height {height:.3f} m, triangles {triangle_count()}, bones {len(rig.data.bones)}")


main()
