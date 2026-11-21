"""Sancho's placeholder (decision D26): BodyKun, rebuilt as a clean rigged glTF.

    blender -b --factory-startup --disable-autoexec bodychan-bodykun.blend \
        --python tools/blender/export_bodykun.py -- OUT.glb [HEIGHT_M] [REPORT.json]

Source: "BodyChan - BodyKun | Posable Drawing Figures" by vinchau, BlendSwap 23521, CC0.

What this does, so the result does not depend on Rigify or on any script inside the .blend
(run it with --disable-autoexec; nothing in the file is executed):
1. A new armature, "Sancho", holding only BodyKun's 71 deform bones (DEF-*), each copied exactly
   (head, tail, roll) from the Rigify rig. Rigify parents some DEF bones to hidden ORG bones; each
   one's parent here is the nearest ancestor that has a DEF twin (ORG-shoulder.R -> DEF-shoulder.R).
2. BodyKun's 12 meshes moved onto it (their vertex groups are already named DEF-*), joined into
   one mesh with one wood material, the colour of the code-built Sancho.
3. Scaled to HEIGHT_M (default 1.68 m) with the feet on the floor, transforms applied.
4. Two clips as NLA tracks: Wave (right arm, 2.4 s) and Grip (both hands, 1.5 s). Rotation axes
   come from the bones' own geometry: the arm turns toward "up"; each finger joint bends toward
   the palm side of its hand (the side its thumb tip is on).
5. Exported as GLB with skins and the two animations.
"""

import json
import math
import sys

import bpy
from mathutils import Quaternion, Vector

args = sys.argv[sys.argv.index("--") + 1:] if "--" in sys.argv else []
OUT = args[0] if args else "sancho.glb"
HEIGHT_M = float(args[1]) if len(args) > 1 else 1.68
REPORT = args[2] if len(args) > 2 else None

SOURCE_RIG = "body_kun_rig"
MESHES = ["bust", "core", "fingers", "foot", "hands", "head", "lowerarms", "neck", "shins", "thigh", "trunk", "upperarms"]
WOOD = (0.658, 0.366, 0.171, 1.0)  # linear; sRGB #D4A373, as mk-sancho's code-built figure
FPS = 30

scene = bpy.context.scene
scene.render.fps = FPS
view_layer = bpy.context.view_layer


def reveal(collection):
    collection.exclude = False
    collection.hide_viewport = False
    for child in collection.children:
        reveal(child)


reveal(view_layer.layer_collection)
for c in bpy.data.collections:
    c.hide_select = False
    c.hide_viewport = False
for o in bpy.data.objects:
    o.hide_viewport = False
    o.hide_select = False
    try:
        o.hide_set(False)
    except RuntimeError:
        pass

src = bpy.data.objects[SOURCE_RIG]
deform = [b for b in src.data.bones if b.use_deform]
names = {b.name for b in deform}


def twin(name):
    """The DEF bone that stands for `name` (ORG-x, MCH-x or DEF-x), if there is one."""
    base = name.split("-", 1)[1] if name.split("-", 1)[0] in ("ORG", "MCH", "DEF") and "-" in name else name
    return "DEF-" + base


def deform_parent(bone):
    p = bone.parent
    while p is not None:
        for candidate in (p.name, twin(p.name)):
            if candidate in names and candidate != bone.name:
                return candidate
        p = p.parent
    return None


# 1. The clean skeleton.
arm_data = bpy.data.armatures.new("Sancho")
rig = bpy.data.objects.new("Sancho", arm_data)
scene.collection.objects.link(rig)
rig.matrix_world = src.matrix_world.copy()
bpy.ops.object.select_all(action="DESELECT")
view_layer.objects.active = rig
rig.select_set(True)
bpy.ops.object.mode_set(mode="EDIT")
for b in deform:
    eb = arm_data.edit_bones.new(b.name)
    eb.head = (0.0, 0.0, 0.0)
    eb.tail = (0.0, 1.0, 0.0)
    eb.matrix = b.matrix_local.copy()
    eb.length = b.length
    eb.use_deform = True
parents = {}
for b in deform:
    p = deform_parent(b)
    parents[b.name] = p
    if p:
        arm_data.edit_bones[b.name].parent = arm_data.edit_bones[p]
        arm_data.edit_bones[b.name].use_connect = False
bpy.ops.object.mode_set(mode="OBJECT")
roots = [n for n, p in parents.items() if p is None]

# 2. The body: BodyKun's meshes on the new skeleton, joined, one material.
parts = []
for name in MESHES:
    o = bpy.data.objects[name]
    for md in list(o.modifiers):
        if md.type == "ARMATURE":
            o.modifiers.remove(md)
    world = o.matrix_world.copy()
    o.parent = rig
    o.matrix_world = world
    parts.append(o)
body = bpy.data.objects["trunk"]
bpy.ops.object.select_all(action="DESELECT")
for o in parts:
    o.select_set(True)
view_layer.objects.active = body
with bpy.context.temp_override(active_object=body, object=body, selected_objects=parts, selected_editable_objects=parts):
    bpy.ops.object.join()
body.name = "Sancho body"
body.data.name = "Sancho body"
missing_groups = sorted({g.name for g in body.vertex_groups} - names)
# The joined mesh fails Blender's validity check (the exporter warns "not valid"); fix what it finds.
fixed_mesh = body.data.validate(verbose=False, clean_customdata=False)
mod = body.modifiers.new("Armature", "ARMATURE")
mod.object = rig
mat = bpy.data.materials.new("Sancho wood")
mat.use_nodes = True
bsdf = next(n for n in mat.node_tree.nodes if n.type == "BSDF_PRINCIPLED")
bsdf.inputs["Base Color"].default_value = WOOD
bsdf.inputs["Roughness"].default_value = 0.65
body.data.materials.clear()
body.data.materials.append(mat)
for poly in body.data.polygons:
    poly.material_index = 0

# 3. Real size, feet on the floor.
zs = [(body.matrix_world @ v.co).z for v in body.data.vertices]
source_height = max(zs) - min(zs)
scale = HEIGHT_M / source_height
rig.scale = (scale, scale, scale)
bpy.context.view_layer.update()
rig.location.z -= min((body.matrix_world @ v.co).z for v in body.data.vertices)
bpy.context.view_layer.update()
bpy.ops.object.select_all(action="DESELECT")
rig.select_set(True)
body.select_set(True)
view_layer.objects.active = rig
with bpy.context.temp_override(active_object=rig, object=rig, selected_objects=[rig, body], selected_editable_objects=[rig, body]):
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)
bpy.context.view_layer.update()
zs = [(body.matrix_world @ v.co).z for v in body.data.vertices]

# 4. The clips.
bones = rig.data.bones
pose = rig.pose.bones
for pb in pose:
    pb.rotation_mode = "QUATERNION"


def head(n):
    return bones[n].head_local.copy()


def tail(n):
    return bones[n].tail_local.copy()


def direction(n):
    return (tail(n) - head(n)).normalized()


def local_rotation(n, axis_armature, degrees):
    """A rotation by `degrees` about an armature-space axis, as bone `n`'s pose rotation."""
    rest = bones[n].matrix_local.to_3x3()
    axis = (rest.inverted() @ axis_armature).normalized()
    return Quaternion(axis, math.radians(degrees))


UP = Vector((0.0, 0.0, 1.0))


def reset_pose():
    for pb in pose:
        pb.rotation_quaternion = Quaternion()


def key(n, q, frame):
    pose[n].rotation_quaternion = q
    pose[n].keyframe_insert("rotation_quaternion", frame=frame)


def clip(name, keys, last):
    """`keys`: {bone: [(frame, quaternion), ...]}; the clip becomes an NLA track named `name`."""
    reset_pose()
    rig.animation_data_create()
    action = bpy.data.actions.new(name)
    rig.animation_data.action = action
    for n, frames in keys.items():
        for frame, q in frames:
            key(n, q, frame)
    track = rig.animation_data.nla_tracks.new()
    track.name = name
    strip = track.strips.new(name, 0, action)
    strip.action_frame_end = last
    rig.animation_data.action = None
    reset_pose()


# Wave: the right arm goes up and the forearm waves three times.
I = Quaternion()
arm_axis = direction("DEF-upper_arm.R").cross(UP).normalized()
fore_axis = direction("DEF-forearm.R").cross(UP).normalized()
up = local_rotation("DEF-upper_arm.R", arm_axis, 95)
f = lambda deg: local_rotation("DEF-forearm.R", fore_axis, deg)
clip("Wave", {
    "DEF-upper_arm.R": [(0, I), (15, up), (60, up), (72, I)],
    "DEF-forearm.R": [(0, I), (15, f(40)), (23, f(70)), (30, f(25)), (38, f(70)), (45, f(25)), (53, f(70)), (60, f(40)), (72, I)],
}, 72)

# Grip: both hands close and open.
CURL = {"01": 55, "02": 75, "03": 55}
THUMB = {"01": 15, "02": 30, "03": 35}
palm_side = {}
grip = {}
for side in ("L", "R"):
    finger_bases = [head(f"DEF-f_{k}.01.{side}") for k in ("index", "middle", "ring", "pinky")]
    hand = head(f"DEF-hand.{side}")
    centre = sum(finger_bases, Vector()) / 4.0
    along = (centre - hand).normalized()
    across = (finger_bases[0] - finger_bases[3]).normalized()
    normal = across.cross(along).normalized()
    if normal.dot(tail(f"DEF-thumb.03.{side}") - (hand + centre) / 2.0) < 0:
        normal = -normal
    palm_side[side] = [round(c, 3) for c in normal]
    for finger, angles in [("f_index", CURL), ("f_middle", CURL), ("f_ring", CURL), ("f_pinky", CURL), ("thumb", THUMB)]:
        for seg, deg in angles.items():
            n = f"DEF-{finger}.{seg}.{side}"
            axis = direction(n).cross(normal).normalized()
            q = local_rotation(n, axis, deg)
            grip[n] = [(0, I), (15, q), (30, q), (45, I)]
clip("Grip", grip, 45)
scene.frame_start, scene.frame_end = 0, 72

# 5. Export.
bpy.ops.object.select_all(action="DESELECT")
rig.select_set(True)
body.select_set(True)
view_layer.objects.active = rig
bpy.ops.export_scene.gltf(
    filepath=OUT,
    export_format="GLB",
    use_selection=True,
    export_animations=True,
    export_animation_mode="NLA_TRACKS",
    export_force_sampling=True,
    export_skins=True,
    export_def_bones=False,
    export_apply=False,
    export_yup=True,
    export_materials="EXPORT",
    export_cameras=False,
    export_lights=False,
    export_extras=False,
)

report = {
    "out": OUT, "height_m": HEIGHT_M, "source_height": round(source_height, 4), "scale": scale,
    "bones": len(bones), "roots": roots, "mesh_fixed_by_validate": bool(fixed_mesh),
    "vertices": len(body.data.vertices), "triangles": sum(len(p.vertices) - 2 for p in body.data.polygons),
    "z_range_m": [round(min(zs), 4), round(max(zs), 4)], "groups_without_bone": missing_groups,
    "palm_side": palm_side, "tracks": [t.name for t in rig.animation_data.nla_tracks],
    "parents_sample": {n: parents[n] for n in ["DEF-shoulder.R", "DEF-upper_arm.R", "DEF-f_index.01.R", "DEF-thigh.L", "DEF-pelvis.L", "DEF-spine.004"]},
}
print(json.dumps(report))
if REPORT:
    with open(REPORT, "w", encoding="utf-8") as fh:
        json.dump(report, fh, indent=1)
