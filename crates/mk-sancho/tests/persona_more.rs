//! Three more tests for mk_sancho::persona, written by Claude after mutation testing showed
//! gaps in the Crew's 59: no test fed a value that ends right after `[` or `{`, and none told
//! the point arm's forearm from its hand.

use mk_sancho::persona::{
    check_model, read_persona, ModelFacts, Persona, PersonaError, Problem, MAX_INFLUENCES,
};

const MINIMAL: &str = "\
[persona]
name = \"S\"
model = \"s.glb\"
height_m = 1.7

[clips]
idle = \"Idle\"
wave = \"Wave\"
point = \"Point\"
think = \"Think\"
worried = \"Worried\"
celebrate = \"Celebrate\"
walk_to = \"Walk\"
sit = \"Sit\"
carry_board = \"Carry\"
";

#[test]
fn a_value_that_stops_right_after_an_opening_bracket_is_a_bad_value() {
    // The reader must stop at the end of the text, not index past it.
    let array = MINIMAL.replace("[clips]", "[bones]\npoint_arm = [\n\n[clips]");
    assert!(read_persona(&array) == Err(PersonaError::BadValue(7, "bones.point_arm".to_string())));

    let table = MINIMAL.replace("height_m = 1.7", "height_m = 1.7\nlod = {");
    assert!(read_persona(&table) == Err(PersonaError::BadValue(5, "persona.lod".to_string())));
}

/// The persona with a point arm named in its own words, and a model that has the three
/// bones of the rest of the rig under Mixamo names but only some of the arm's own.
fn arm_persona() -> Persona {
    let text = MINIMAL.replace(
        "[clips]",
        "[bones]\npoint_arm = [\"X_UPPER\", \"X_FORE\", \"X_HAND\"]\n\n[clips]",
    );
    read_persona(&text).expect("a persona with a point arm reads")
}

fn model_with(arm_bones: &[&str]) -> ModelFacts {
    let mut bone_names: Vec<String> = ["Hips", "Spine", "Neck", "Head"]
        .iter()
        .map(|b| format!("mixamorig:{b}"))
        .collect();
    bone_names.extend(arm_bones.iter().map(|b| b.to_string()));
    ModelFacts {
        bone_names,
        max_influences: MAX_INFLUENCES,
        triangles: 1_000,
        clip_names: [
            "Idle",
            "Wave",
            "Point",
            "Think",
            "Worried",
            "Celebrate",
            "Walk",
            "Sit",
            "Carry",
        ]
        .iter()
        .map(|c| c.to_string())
        .collect(),
        rest_height_m: 1.7,
    }
}

#[test]
fn the_point_arm_names_the_upper_arm_forearm_and_hand_in_that_order() {
    let persona = arm_persona();
    let missing = |bones: &[&str]| check_model(&persona, &model_with(bones), 30_000);

    // Only the forearm is not in the model.
    assert!(
        missing(&["X_UPPER", "X_HAND"])
            == vec![Problem::MissingBone {
                bone: "RightForeArm".to_string()
            }]
    );
    // Only the hand is not in the model.
    assert!(
        missing(&["X_UPPER", "X_FORE"])
            == vec![Problem::MissingBone {
                bone: "RightHand".to_string()
            }]
    );
    // Only the upper arm is not in the model.
    assert!(
        missing(&["X_FORE", "X_HAND"])
            == vec![Problem::MissingBone {
                bone: "RightArm".to_string()
            }]
    );
    // All three are there.
    assert!(missing(&["X_UPPER", "X_FORE", "X_HAND"]).is_empty());
}
