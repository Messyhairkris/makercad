"""What is inside a .blend: objects, armatures and their bones, meshes and how they are attached.

    blender -b --factory-startup --disable-autoexec FILE.blend --python tools/blender/inspect_blend.py -- OUT.json

Run with --disable-autoexec: a downloaded .blend can carry Python (Rigify rigs carry a UI script),
and nothing in it is run. This script only reads.
"""

import json
import sys

import bpy

out = sys.argv[sys.argv.index("--") + 1] if "--" in sys.argv else "inspect.json"


def v3(v):
    return [round(float(x), 4) for x in v]


report = {
    "blender": bpy.app.version_string,
    "file_version": list(bpy.data.version),
    "scenes": [s.name for s in bpy.data.scenes],
    "collections": {c.name: {"objects": [o.name for o in c.objects], "children": [k.name for k in c.children],
                             "hide_render": c.hide_render} for c in bpy.data.collections},
    "texts": [t.name for t in bpy.data.texts],
    "actions": [{"name": a.name, "fcurves": len(a.fcurves) if hasattr(a, "fcurves") else None} for a in bpy.data.actions],
    "materials": [],
    "objects": [],
    "armatures": [],
}

for m in bpy.data.materials:
    color = None
    if m.use_nodes and m.node_tree:
        for n in m.node_tree.nodes:
            if n.type == "BSDF_PRINCIPLED":
                color = v3(n.inputs["Base Color"].default_value)
                break
    report["materials"].append({"name": m.name, "diffuse": v3(m.diffuse_color), "principled_base": color,
                                "users": m.users})

for o in bpy.data.objects:
    item = {
        "name": o.name, "type": o.type,
        "parent": o.parent.name if o.parent else None,
        "parent_type": o.parent_type, "parent_bone": o.parent_bone or None,
        "collections": [c.name for c in o.users_collection],
        "hide_viewport": o.hide_viewport, "hide_render": o.hide_render,
        "location": v3(o.matrix_world.translation), "dimensions": v3(o.dimensions),
        "modifiers": [{"type": md.type, "name": md.name,
                       "object": getattr(getattr(md, "object", None), "name", None)} for md in o.modifiers],
        "constraints": [c.type for c in o.constraints],
    }
    if o.type == "MESH":
        me = o.data
        item["mesh"] = {"vertices": len(me.vertices), "polygons": len(me.polygons),
                        "triangles": sum(len(p.vertices) - 2 for p in me.polygons),
                        "materials": [m.name if m else None for m in me.materials]}
        item["vertex_groups"] = [g.name for g in o.vertex_groups]
    report["objects"].append(item)

for o in bpy.data.objects:
    if o.type != "ARMATURE":
        continue
    arm = o.data
    bones = []
    for b in arm.bones:
        bones.append({"name": b.name, "parent": b.parent.name if b.parent else None, "deform": b.use_deform,
                      "head": v3(b.head_local), "tail": v3(b.tail_local), "length": round(b.length, 4)})
    deform = [b["name"] for b in bones if b["deform"]]
    fingers = [n for n in deform if any(k in n.lower() for k in ("f_", "thumb", "finger", "palm"))]
    report["armatures"].append({
        "object": o.name, "data": arm.name, "bones": len(bones), "deform_bones": len(deform),
        "deform": deform, "finger_deform": fingers,
        "rigify_layers": hasattr(arm, "rigify_layers"),
        "pose_props": sorted({k for pb in o.pose.bones for k in pb.keys()})[:60],
        "all_bones": bones,
    })

with open(out, "w", encoding="utf-8") as f:
    json.dump(report, f, indent=1)
print("wrote", out)
