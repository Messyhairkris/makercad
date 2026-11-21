"""Details of one Rigify rig: its pose-library action and the controls that bend fingers and arms.

    blender -b --factory-startup --disable-autoexec FILE.blend --python tools/blender/inspect_rig.py -- RIG OUT.json
"""

import json
import sys

import bpy

args = sys.argv[sys.argv.index("--") + 1:]
rig_name, out = args[0], args[1]
rig = bpy.data.objects[rig_name]

report = {"rig": rig_name, "matrix_world": [list(r) for r in rig.matrix_world], "actions": []}
for a in bpy.data.actions:
    paths = sorted({fc.data_path for fc in a.fcurves}) if hasattr(a, "fcurves") else []
    report["actions"].append({
        "name": a.name, "frame_range": list(a.frame_range),
        "markers": [{"name": m.name, "frame": m.frame} for m in a.pose_markers] if hasattr(a, "pose_markers") else [],
        "bones": sorted({p.split('"')[1] for p in paths if p.startswith("pose.bones[")}),
    })
names = [b.name for b in rig.pose.bones]
report["masters"] = [n for n in names if "master" in n]
report["fk"] = [n for n in names if "_fk" in n]
report["fingers"] = [n for n in names if n.startswith(("f_", "thumb")) and not n.startswith(("ORG", "DEF", "MCH"))]
report["ik_fk"] = {pb.name: pb["IK_FK"] for pb in rig.pose.bones if "IK_FK" in pb.keys()}
report["anim"] = rig.animation_data.action.name if rig.animation_data and rig.animation_data.action else None
with open(out, "w", encoding="utf-8") as f:
    json.dump(report, f, indent=1)
print("wrote", out)
