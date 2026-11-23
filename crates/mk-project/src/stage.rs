//! MakerCAD's scale classes stage table (doc 05 section 5.5).
//!
//! Written by Kimi (the Crew, campaign mk-core-27, 2026-11-22) from the contract
//! crew/contracts/mk_stage.md (ATLAS-api-harness), which Gordon wrote himself as a PA (the contract, the reference and the hidden tests).

/// Millimetres in one inch.
pub const MM_PER_INCH: f64 = 25.4;
/// Macro's imperial grid and snap: 1/256 in, about 0.0992 mm (v1 starting value, to tune).
pub const MACRO_GRID_IN: f64 = 0.00390625;
/// Grid steps per major line where the doc gives no major value (v1 starting value, to tune).
pub const MAJOR_STEPS: f64 = 10.0;
/// The top of the macro range, exclusive (doc 05 section 5.5: macro is under 10 mm).
pub const MACRO_MAX_MM: f64 = 10.0;
/// The top of the bench range, inclusive (doc 05 section 5.5: bench is 10-500 mm).
pub const BENCH_MAX_MM: f64 = 500.0;
/// The top of the furniture range, inclusive (doc 05 section 5.5: furniture is 0.3-2.5 m).
pub const FURNITURE_MAX_MM: f64 = 2500.0;

/// The unit system used for grid, major-line, and snapping steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Units {
    /// Millimetre-based steps.
    Metric,
    /// Inch-based steps, converted to millimetres by the accessors.
    Imperial,
}

/// A scale class in the stage table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScaleClass {
    /// Room-scale designs.
    Room,
    /// Furniture-scale designs.
    Furniture,
    /// Body-worn or carried designs.
    Body,
    /// Workbench-scale designs.
    Bench,
    /// Macro-scale designs.
    Macro,
}

/// The default stage associated with a scale class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Stage {
    /// A room shell or default room box.
    Space,
    /// A floor plane.
    FloorPlane,
    /// A mannequin or accessory layout mat.
    Mannequin,
    /// A workbench top.
    Workbench,
    /// A cutting mat.
    CuttingMat,
}

/// Sancho's presentation in a scale class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SanchoStance {
    /// Standing at true scale in the room.
    Standing,
    /// Standing at true scale beside the piece.
    Beside,
    /// Wearing the design at true scale.
    Wearing,
    /// Standing at true size on the floor beside the workbench.
    FloorBeside,
    /// A giant eye with a magnifying glass.
    GiantEye,
}

/// One row of the scale-class stage table.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StageRow {
    /// The scale class represented by this row.
    pub class: ScaleClass,
    /// The lower end of the typical size range in millimetres.
    pub size_min_mm: f64,
    /// The upper end of the typical size range in millimetres.
    pub size_max_mm: f64,
    /// The displayed typical size range.
    pub size_text: &'static str,
    /// The default stage.
    pub stage: Stage,
    /// The displayed stage description.
    pub stage_text: &'static str,
    /// Sancho's stance.
    pub sancho: SanchoStance,
    /// The displayed Sancho description.
    pub sancho_text: &'static str,
    /// The metric grid step in millimetres.
    pub grid_mm: f64,
    /// The metric snapping step in millimetres.
    pub snap_mm: f64,
    /// The metric major-grid step in millimetres.
    pub major_mm: f64,
    /// The imperial grid step in inches.
    pub grid_in: f64,
    /// The imperial snapping step in inches.
    pub snap_in: f64,
    /// The imperial major-grid step in inches.
    pub major_in: f64,
}

/// Returns the stage-table row for `class` (rules A1-A5).
pub fn row(class: ScaleClass) -> StageRow {
    match class {
        ScaleClass::Room => StageRow {
            class: ScaleClass::Room,
            size_min_mm: 500.0,
            size_max_mm: 10000.0,
            size_text: "0.5-10 m",
            stage: Stage::Space,
            stage_text: "The Space (room shell or a default room box 3.6 x 3 x 2.5 m)",
            sancho: SanchoStance::Standing,
            sancho_text: "True scale, 1.68 m (5'6\"), standing in the room",
            grid_mm: 100.0,
            snap_mm: 100.0,
            major_mm: 1000.0,
            grid_in: 1.0,
            snap_in: 1.0,
            major_in: 12.0,
        },
        ScaleClass::Furniture => StageRow {
            class: ScaleClass::Furniture,
            size_min_mm: 300.0,
            size_max_mm: 2500.0,
            size_text: "0.3-2.5 m",
            stage: Stage::FloorPlane,
            stage_text: "A floor plane with a rug-sized patch",
            sancho: SanchoStance::Beside,
            sancho_text: "True scale beside the piece",
            grid_mm: 10.0,
            snap_mm: 10.0,
            major_mm: 10.0 * MAJOR_STEPS,
            grid_in: 0.25,
            snap_in: 0.25,
            major_in: 0.25 * MAJOR_STEPS,
        },
        ScaleClass::Body => StageRow {
            class: ScaleClass::Body,
            size_min_mm: 100.0,
            size_max_mm: 1000.0,
            size_text: "0.1-1 m, worn or carried",
            stage: Stage::Mannequin,
            stage_text: "A mannequin or an accessory layout mat",
            sancho: SanchoStance::Wearing,
            sancho_text: "True scale; can wear the design",
            grid_mm: 5.0,
            snap_mm: 5.0,
            major_mm: 5.0 * MAJOR_STEPS,
            grid_in: 0.125,
            snap_in: 0.125,
            major_in: 0.125 * MAJOR_STEPS,
        },
        ScaleClass::Bench => StageRow {
            class: ScaleClass::Bench,
            size_min_mm: 10.0,
            size_max_mm: 500.0,
            size_text: "10-500 mm",
            stage: Stage::Workbench,
            stage_text: "A workbench top (cutting mat texture, 600 x 450 mm)",
            sancho: SanchoStance::FloorBeside,
            sancho_text: "True size on the floor beside a small part on the workbench",
            grid_mm: 1.0,
            snap_mm: 1.0,
            major_mm: 1.0 * MAJOR_STEPS,
            grid_in: 0.03125,
            snap_in: 0.03125,
            major_in: 0.03125 * MAJOR_STEPS,
        },
        ScaleClass::Macro => StageRow {
            class: ScaleClass::Macro,
            size_min_mm: 0.0,
            size_max_mm: 10.0,
            size_text: "< 10 mm",
            stage: Stage::CuttingMat,
            stage_text: "A cutting mat with mm grid",
            sancho: SanchoStance::GiantEye,
            sancho_text: "A giant eye with a magnifying glass",
            grid_mm: 0.1,
            snap_mm: 0.1,
            major_mm: 0.1 * MAJOR_STEPS,
            grid_in: MACRO_GRID_IN,
            snap_in: MACRO_GRID_IN,
            major_in: MACRO_GRID_IN * MAJOR_STEPS,
        },
    }
}

/// Returns the scale class selected by the finite, positive-size ladder (rules A9-A11).
pub fn class_for_size(longest_mm: f64) -> Option<ScaleClass> {
    if !longest_mm.is_finite() || longest_mm <= 0.0 {
        return None;
    }

    if longest_mm < MACRO_MAX_MM {
        Some(ScaleClass::Macro)
    } else if longest_mm <= BENCH_MAX_MM {
        Some(ScaleClass::Bench)
    } else if longest_mm <= FURNITURE_MAX_MM {
        Some(ScaleClass::Furniture)
    } else {
        Some(ScaleClass::Room)
    }
}

/// Returns the class's grid step in millimetres (rule A6).
pub fn grid_for(class: ScaleClass, units: Units) -> f64 {
    let row = row(class);
    match units {
        Units::Metric => row.grid_mm,
        Units::Imperial => row.grid_in * MM_PER_INCH,
    }
}

/// Returns the class's major-grid step in millimetres (rule A6).
pub fn major_for(class: ScaleClass, units: Units) -> f64 {
    let row = row(class);
    match units {
        Units::Metric => row.major_mm,
        Units::Imperial => row.major_in * MM_PER_INCH,
    }
}

/// Rounds `mm` to the nearest snapping step, with ties away from zero (rules A7-A8).
pub fn snap_for(class: ScaleClass, units: Units, mm: f64) -> f64 {
    let row = row(class);
    let step = match units {
        Units::Metric => row.snap_mm,
        Units::Imperial => row.snap_in * MM_PER_INCH,
    };
    (mm / step).round() * step
}

/// Returns the next larger class in the stage menu, excluding Body (rule A12).
pub fn next_larger(class: ScaleClass) -> Option<ScaleClass> {
    match class {
        ScaleClass::Macro => Some(ScaleClass::Bench),
        ScaleClass::Bench => Some(ScaleClass::Furniture),
        ScaleClass::Furniture => Some(ScaleClass::Room),
        ScaleClass::Room | ScaleClass::Body => None,
    }
}

/// Returns the next smaller class in the stage menu, excluding Body (rule A13).
pub fn next_smaller(class: ScaleClass) -> Option<ScaleClass> {
    match class {
        ScaleClass::Room => Some(ScaleClass::Furniture),
        ScaleClass::Furniture => Some(ScaleClass::Bench),
        ScaleClass::Bench => Some(ScaleClass::Macro),
        ScaleClass::Macro | ScaleClass::Body => None,
    }
}
