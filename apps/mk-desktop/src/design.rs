//! The Design workspace: SolidCraft's engine and kernel modelling inside MakerCAD's shell (P1).
//!
//! A design is an engine [`Session`]; every change goes through its command surface
//! (`Session::execute`), the same commands SolidCraft's app, its MCP server and the Crew use.
//!
//! **Drawing.** SolidCraft's viewport pipelines are installed once in the window's own renderer
//! (`solidcraft_viewport_egui::install`) and every frame hands them a [`Frame`]: the camera and the
//! scene's key, in an egui paint callback, so the model is drawn straight into the window's render
//! pass. The scene (design, stage, Sancho) is built and uploaded only when it changes: the design
//! changes, Sancho is shown or hidden, or a camera move ends on the floor stage, whose ground grid
//! follows the camera. Turning the view builds and uploads nothing (2026-11-21; it used to draw a
//! picture, read it back and upload it as a texture on every change, about 80–140 ms a frame
//! here). Without a GPU render state the CPU rasterizer draws a picture as before.
//!
//! **Stage.** Sancho stands beside the design at true size (decision D27). A small part sits on a
//! workbench with Sancho on the floor beside it; a big one stands on the floor with him
//! ([`crate::stage`]). The opening view frames all of it; "Fit view" frames the design.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use eframe::egui_wgpu;
use egui::{pos2, Color32, PointerButton, Rect, Sense, TextureOptions};
use serde_json::Value;
use solidcore_record::Vec3;
use solidcraft_engine::Session;
use solidcraft_viewport::hatch::HATCH_COLOURS;
use solidcraft_viewport::vec::Bounds;
use solidcraft_viewport::{Camera, Rgb, Scene, StandardView, MAX_SECTION_PLANES, SECTION_CAP};
use solidcraft_viewport_gpu::view::{Frame, GpuTarget, Renderer, SceneSlot};

use crate::scale_figure::ScaleFigure;
use crate::stage;
use crate::theme::Palette;

/// How the view is drawn.
enum Drawing {
    /// Not decided yet (the first frame decides).
    Undecided,
    /// SolidCraft's viewport pipelines in the window's render pass.
    Window,
    /// SolidCraft's CPU rasterizer, as a picture (no GPU render state).
    Cpu,
}

/// The scene as last built for the GPU: its key and geometry (uploaded once per key), what it
/// spans, its section, and the camera its ground grid was laid out for.
struct Built {
    key: u64,
    slot: SceneSlot,
    bounds: Bounds,
    clip: [[f32; 4]; MAX_SECTION_PLANES],
    clip_hatch: [usize; MAX_SECTION_PLANES],
    cap: [f32; 4],
    hatch: [[f32; 3]; HATCH_COLOURS],
    camera: Camera,
    grid: bool,
}

/// An open design.
pub struct Design {
    /// Its name, for the top bar.
    pub name: String,
    /// One line on what it is, for an example ("Parametric plate with fillets, …").
    pub about: Option<String>,
    session: Session,
    camera: Camera,
    drawing: Drawing,
    built: Option<Built>,
    next_key: u64,
    /// The design or its scenery changed: the scene is built again before the next frame.
    scene_dirty: bool,
    /// CPU drawing only: the picture and whether it needs drawing again.
    texture: Option<egui::TextureHandle>,
    picture_dirty: bool,
    drawn_px: [u32; 2],
    /// How long building the design took, in ms (running its commands through the kernel).
    pub build_ms: f64,
    /// How long the last scene took to build, in ms (design, stage and Sancho into the viewport's
    /// geometry); on the CPU, the whole picture.
    pub last_draw_ms: Option<f64>,
    /// What draws the view.
    pub renderer: String,
    /// The last error, shown over the view.
    pub error: Option<String>,
    /// Sancho, the scale figure; `None` if his figure could not be read (said once on stderr).
    sancho: Option<ScaleFigure>,
    show_sancho: bool,
    /// [`Design::turn`] moved the camera since the last frame.
    turning: bool,
    /// When the camera last moved: the view is drawn plainly until it has been still for
    /// [`STILL_AFTER`], then once supersampled ([`still_ssaa`]).
    last_move: Option<Instant>,
}

/// How long the view must stay still before it is drawn with smooth edges.
const STILL_AFTER: std::time::Duration = std::time::Duration::from_millis(250);

impl Design {
    /// A design built by running `script` (`{"commands": [{"command", "params"}, ...]}`), the way
    /// SolidCraft builds its samples.
    pub fn from_script(name: &str, script: &Value) -> Result<Self, String> {
        let started = Instant::now();
        let mut session = Session::default();
        let steps = script
            .get("commands")
            .and_then(Value::as_array)
            .ok_or("the script has no commands")?;
        for step in steps {
            let id = step
                .get("command")
                .and_then(Value::as_str)
                .ok_or("a step has no command")?;
            let params = step
                .get("params")
                .cloned()
                .unwrap_or_else(|| Value::Object(Default::default()));
            session
                .execute(id, &params)
                .map_err(|e| format!("{id}: {e}"))?;
        }
        let camera = solidcraft_engine::view::home_camera(&session).map_err(|e| e.to_string())?;
        let build_ms = started.elapsed().as_secs_f64() * 1000.0;
        let sancho = match ScaleFigure::placeholder(f64::from(mk_sancho::DEFAULT_HEIGHT_M) * 1000.0)
        {
            Ok(figure) => Some(figure),
            Err(e) => {
                eprintln!("makercad: Sancho is not shown: {e}");
                None
            }
        };
        let mut design = Self {
            name: name.into(),
            about: None,
            session,
            camera,
            drawing: Drawing::Undecided,
            built: None,
            next_key: 1,
            scene_dirty: true,
            texture: None,
            picture_dirty: true,
            drawn_px: [0, 0],
            build_ms,
            last_draw_ms: None,
            renderer: String::new(),
            error: None,
            sancho,
            show_sancho: true,
            turning: false,
            last_move: None,
        };
        design.home_view();
        Ok(design)
    }

    /// SolidCraft's first sample, the plate with fillets, two holes and a bored boss.
    pub fn sample() -> Result<Self, String> {
        Self::example(0)
    }

    /// SolidCraft's sample number `index` ([`examples`]), built by its own script.
    pub fn example(index: usize) -> Result<Self, String> {
        let sample = solidcraft_engine::sample::samples()
            .into_iter()
            .nth(index)
            .ok_or_else(|| format!("SolidCraft has no example {}", index + 1))?;
        let mut design = Self::from_script(sample.name, &(sample.script)())?;
        design.about = Some(sample.about.to_string());
        Ok(design)
    }

    /// Looks at the design from one side (or the corner), framed on the design alone.
    pub fn look_from(&mut self, side: StandardView) {
        let Ok(mut camera) = solidcraft_engine::view::home_camera(&self.session) else {
            return;
        };
        camera.set_view(side);
        if let Some((min, max)) = design_box(&self.session) {
            let mut b = Bounds::EMPTY;
            b.add(min);
            b.add(max);
            camera.fit(&b);
        }
        self.set_camera(camera);
    }

    /// What Undo would take back ("Bore"), if anything: the engine's own history.
    pub fn undo_label(&self) -> Option<&str> {
        step_label(&self.session.undo, self.session.history_part())
    }

    /// What Redo would put back, if anything.
    pub fn redo_label(&self) -> Option<&str> {
        step_label(&self.session.redo, self.session.history_part())
    }

    /// Takes the last step back (the engine's `edit.undo`).
    pub fn undo(&mut self) -> Result<(), String> {
        self.execute("edit.undo", &Value::Object(Default::default()))
            .map(|_| ())
    }

    /// Puts the last undone step back (`edit.redo`).
    pub fn redo(&mut self) -> Result<(), String> {
        self.execute("edit.redo", &Value::Object(Default::default()))
            .map(|_| ())
    }

    /// Runs one command on the design (the engine's command surface) and redraws.
    pub fn execute(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let result = self.session.execute(id, params).map_err(|e| e.to_string());
        self.scene_dirty = true;
        self.picture_dirty = true;
        result
    }

    /// The opening view: the design, its stage and Sancho, so its size reads at once.
    pub fn home_view(&mut self) {
        let Ok(mut camera) = solidcraft_engine::view::home_camera(&self.session) else {
            return;
        };
        let all = self.scenery_bounds();
        if !all.is_empty() {
            camera.fit(&all);
        }
        self.set_camera(camera);
    }

    /// The view that frames the design alone.
    pub fn fit_view(&mut self) {
        if let Ok(camera) = solidcraft_engine::view::home_camera(&self.session) {
            self.set_camera(camera);
        }
    }

    fn set_camera(&mut self, camera: Camera) {
        self.camera = camera;
        self.picture_dirty = true;
    }

    /// Turns the view as a drag of `dx` pixels would (`--spin` measures the frame rate with it).
    pub fn turn(&mut self, dx: f64) {
        self.camera.orbit(dx, 0.0);
        self.picture_dirty = true;
        self.turning = true;
    }

    /// Sancho's label for the top bar ("Sancho 1.68 m"); `None` when there is no figure.
    pub fn sancho_label(&self) -> Option<String> {
        self.sancho
            .as_ref()
            .map(|s| format!("Sancho {:.2} m", s.height_mm / 1000.0))
    }

    /// Sancho's height in mm while he stands beside the design.
    pub fn sancho_height_mm(&self) -> Option<f64> {
        self.sancho
            .as_ref()
            .filter(|_| self.show_sancho)
            .map(|s| s.height_mm)
    }

    /// Is Sancho standing beside the design?
    pub fn sancho_shown(&self) -> bool {
        self.show_sancho && self.sancho.is_some()
    }

    /// Shows or hides Sancho.
    pub fn show_sancho(&mut self, show: bool) {
        if show != self.show_sancho {
            self.show_sancho = show;
            self.scene_dirty = true;
            self.picture_dirty = true;
        }
    }

    /// The box around the design, its stage and (when shown) Sancho.
    fn scenery_bounds(&self) -> Bounds {
        let design = design_box(&self.session);
        let half = self
            .sancho
            .as_ref()
            .map_or(250.0, ScaleFigure::half_width_mm);
        let stage = stage::plan(design, half);
        let mut b = stage.bounds(half);
        if let Some((min, max)) = design {
            b.add(min);
            b.add(max);
        }
        if let Some(figure) = self.sancho.as_ref().filter(|_| self.show_sancho) {
            let s = figure.bounds_at(stage.sancho);
            b.add(s.min);
            b.add(s.max);
        }
        b
    }

    /// The view: takes the space left in `ui`, handles orbit, pan and zoom, and draws the model.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        render_state: Option<&egui_wgpu::RenderState>,
        palette: &Palette,
    ) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        let mut moving = std::mem::take(&mut self.turning);
        if response.dragged_by(PointerButton::Primary) {
            let d = response.drag_delta();
            self.camera.orbit(f64::from(d.x), f64::from(d.y));
            moving = true;
        }
        if response.dragged_by(PointerButton::Secondary)
            || response.dragged_by(PointerButton::Middle)
        {
            let d = response.drag_delta();
            self.camera
                .pan(f64::from(d.x), f64::from(d.y), f64::from(rect.height()));
            moving = true;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.camera
                    .zoom_at((-f64::from(scroll) * 0.002).exp(), None);
                moving = true;
            }
        }
        if moving {
            self.picture_dirty = true;
            self.last_move = Some(Instant::now());
        }
        if response.double_clicked() {
            self.fit_view();
        }
        // Plain frames while the view moves, then one smooth frame once it has settled (the
        // renderer keeps that picture, so a still view costs nothing more).
        let settling = self.last_move.is_some_and(|t| t.elapsed() < STILL_AFTER);
        if settling {
            ui.ctx().request_repaint_after(STILL_AFTER);
        }

        let ppp = ui.ctx().pixels_per_point();
        let px = [
            (rect.width() * ppp).round().max(16.0) as u32,
            (rect.height() * ppp).round().max(16.0) as u32,
        ];
        let painter = ui.painter_at(rect);
        solidcraft_viewport_egui::paint_background(
            &painter,
            rect,
            palette.bg_top,
            palette.bg_bottom,
        );

        if matches!(self.drawing, Drawing::Undecided) {
            self.drawing = match render_state {
                Some(rs) => {
                    install_viewport(rs);
                    self.renderer = format!("{:?}, in the window", rs.adapter.get_info().backend)
                        .to_lowercase();
                    Drawing::Window
                }
                None => {
                    self.renderer = "cpu".into();
                    Drawing::Cpu
                }
            };
        }
        match self.drawing {
            Drawing::Window => {
                // The floor stage's grid follows the camera: lay it out again once a move ends.
                let regrid = !moving
                    && self
                        .built
                        .as_ref()
                        .is_some_and(|b| b.grid && !same_view(&b.camera, &self.camera));
                if self.scene_dirty || self.built.is_none() || regrid {
                    self.rebuild();
                }
                if let Some(built) = &self.built {
                    let frame = self.frame(built, px, !settling);
                    painter.add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        solidcraft_viewport_egui::ViewportCallback(frame),
                    ));
                }
            }
            Drawing::Cpu | Drawing::Undecided => {
                if self.picture_dirty || px != self.drawn_px {
                    self.draw_picture(ui.ctx(), px);
                }
                if let Some(texture) = &self.texture {
                    painter.image(
                        texture.id(),
                        rect,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
        }
        if let Some(error) = &self.error {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                error,
                egui::FontId::proportional(15.0),
                palette.ink,
            );
        }
    }

    /// The design, its stage and Sancho as one scene, and the box around all of it.
    fn build_scene(&mut self) -> Result<(Scene, Bounds, bool), String> {
        let design = design_box(&self.session);
        let half = self
            .sancho
            .as_ref()
            .map_or(250.0, ScaleFigure::half_width_mm);
        let stage = stage::plan(design, half);
        let guides = solidcraft_engine::view::Guides {
            grid: stage.grid,
            axes: true,
        };
        let mut scene = solidcraft_engine::view::scene_with_guides(
            &self.session,
            &self.camera,
            &self.session.section,
            guides,
        )
        .map_err(|e| e.to_string())?;
        // Scenery, not design: the stage and Sancho are added to the picture only.
        let mut bounds = stage.bounds(half);
        if let Some((min, max)) = design {
            bounds.add(min);
            bounds.add(max);
        }
        scene.meshes.extend(stage.meshes(half));
        scene.lines.extend(stage.lines());
        if self.show_sancho {
            if let Some(figure) = self.sancho.as_mut() {
                scene.meshes.push(figure.mesh_at(stage.sancho));
                let s = figure.bounds_at(stage.sancho);
                bounds.add(s.min);
                bounds.add(s.max);
            }
        }
        Ok((scene, bounds, stage.grid))
    }

    /// Builds the scene and hands it to the GPU under a new key (uploaded on the next frame).
    fn rebuild(&mut self) {
        let started = Instant::now();
        match self.build_scene() {
            Ok((scene, bounds, grid)) => {
                let key = self.next_key;
                self.next_key = self.next_key.wrapping_add(1).max(1);
                let cap = scene.cap.unwrap_or(SECTION_CAP);
                let clip_hatch: Vec<usize> = (0..scene.clip.len())
                    .map(|k| scene.clip_hatch.get(k).copied().unwrap_or(k))
                    .collect();
                self.built = Some(Built {
                    key,
                    slot: Arc::new(Mutex::new(Some(
                        solidcraft_viewport_gpu::capture::scene_geometry(&scene),
                    ))),
                    bounds,
                    clip: solidcraft_viewport_gpu::view::clip_planes(&scene.clip),
                    clip_hatch: solidcraft_viewport_gpu::view::clip_hatches(&clip_hatch),
                    cap: [
                        f32::from(cap.0) / 255.0,
                        f32::from(cap.1) / 255.0,
                        f32::from(cap.2) / 255.0,
                        if scene.plain_cap { 0.0 } else { 1.0 },
                    ],
                    hatch: scene.part_hatch.map_or_else(Default::default, |c| {
                        c.map(|Rgb(r, g, b)| [r, g, b].map(|x| f32::from(x) / 255.0))
                    }),
                    camera: self.camera,
                    grid,
                });
                self.error = None;
            }
            Err(e) => self.error = Some(e),
        }
        self.scene_dirty = false;
        self.last_draw_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
    }

    /// This frame of the view: the current camera over the scene as built.
    fn frame(&self, built: &Built, [w, h]: [u32; 2], still: bool) -> Frame {
        let empty = || Arc::new(Mutex::new(None));
        let cam = &self.camera;
        let m = cam.view_proj(
            f64::from(w) / f64::from(h.max(1)),
            view_radius(&built.bounds, cam),
        );
        let (eye, back) = (cam.eye(), cam.back());
        Frame {
            product_lights: None,
            product: false,
            upload_error: None,
            key: built.key,
            slot: built.slot.clone(),
            sketch_key: 0,
            sketch_slot: empty(),
            hl_key: 0,
            hl_slot: empty(),
            pv_key: 0,
            pv_slot: empty(),
            pv_hide: false,
            preview_bounds: None,
            view_proj: m.to_f32(),
            turned_depth: Some(m.turned_depth_f32()),
            back: [back.x as f32, back.y as f32, back.z as f32],
            size_px: [w as f32, h as f32],
            clip: built.clip,
            clip_hatch: built.clip_hatch,
            cap: built.cap,
            images: Vec::new(),
            clouds: Vec::new(),
            analysis: [0.0; 8],
            analysis_colors: [0.0; 36],
            access: None,
            outline: true,
            eye: [
                eye.x as f32,
                eye.y as f32,
                eye.z as f32,
                if cam.fov > 0.0 { 1.0 } else { 0.0 },
            ],
            ao_radius: 0.0,
            ao_samples: solidcraft_viewport_gpu::view::AO_SAMPLES,
            ssaa: if still { still_ssaa() } else { 1.0 },
            hover_tint: [0.0; 3],
            selection_tint: [0.0; 3],
            hatch: built.hatch,
            sky: [0.0; 4],
            moving: None,
        }
    }

    /// Without a GPU render state: the scene drawn by SolidCraft's CPU rasterizer, as a picture.
    fn draw_picture(&mut self, ctx: &egui::Context, px: [u32; 2]) {
        let started = Instant::now();
        self.picture_dirty = false;
        self.drawn_px = px;
        match self.build_scene() {
            Ok((mut scene, bounds, _)) => {
                scene.radius = view_radius(&bounds, &self.camera);
                let [w, h] = px;
                let rgba =
                    solidcraft_viewport::render(&scene, &self.camera, w as usize, h as usize).rgba;
                if rgba.len() == (w as usize) * (h as usize) * 4 {
                    let image =
                        egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                    match &mut self.texture {
                        Some(texture) => texture.set(image, TextureOptions::LINEAR),
                        None => {
                            self.texture =
                                Some(ctx.load_texture("design-view", image, TextureOptions::LINEAR))
                        }
                    }
                    self.error = None;
                } else {
                    self.error = Some(format!(
                        "the picture came back {} bytes, not {}",
                        rgba.len(),
                        w * h * 4
                    ));
                }
            }
            Err(e) => self.error = Some(e),
        }
        self.last_draw_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
    }
}

/// Samples per pixel the view is drawn with (MSAA): 1 unless `--msaa` said otherwise.
static MSAA: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

/// Sets the view's samples per pixel (1, 2 or 4) for this run, before the first design opens
/// (`--msaa`). P0 condition 2, measured 2026-11-21 on the reference PC with the plate filling a
/// 1920 × 1017 window while turning: 4 samples 21 FPS, 2 samples 35 FPS, 1 sample 57 FPS (the
/// screen's 60 Hz). So the default is 1 sample while the view moves and a supersampled frame
/// once it is still ([`still_ssaa`]).
pub fn set_msaa(samples: u32) {
    let _ = MSAA.set(samples);
}

/// How much a still view is supersampled (each side): 2 with the default single sample, which
/// gives smooth edges once, when the view stops; 1 (none) when `--msaa` asked for more samples.
fn still_ssaa() -> f32 {
    if MSAA.get().copied().unwrap_or(1) > 1 {
        1.0
    } else {
        2.0
    }
}

/// SolidCraft's viewport pipelines in the window's renderer: installed once per window (a later
/// design reuses them). The scene is drawn offscreen with depth and MSAA ([`set_msaa`], 1 sample
/// by default) where the device has it, then into the window's pass, which has one sample and no
/// depth.
fn install_viewport(rs: &egui_wgpu::RenderState) {
    if rs
        .renderer
        .read()
        .callback_resources
        .get::<Renderer>()
        .is_some()
    {
        return;
    }
    let depth = solidcraft_viewport_gpu::depth_format(&rs.device, true);
    let flags = |f| rs.adapter.get_texture_format_features(f).flags;
    let format = rs.target_format;
    let wanted = MSAA.get().copied().unwrap_or(1);
    let samples = if wanted > 1
        && flags(format).sample_count_supported(wanted)
        && flags(depth).sample_count_supported(wanted)
    {
        wanted
    } else {
        1
    };
    let scene = GpuTarget {
        format,
        depth: Some(depth),
        samples,
        tess: false,
        oit: false,
        edges: false,
    };
    let window = GpuTarget {
        format,
        depth: None,
        samples: 1,
        tess: false,
        oit: false,
        edges: false,
    };
    // What the renderer can do, the draw statistics handle and the GPU meshing still compiling
    // are not needed: bodies are meshed on the CPU here (`gpu_meshing` false).
    let _ = solidcraft_viewport_egui::install(rs, scene, window, false, false, |e| {
        eprintln!("makercad: GPU error in the view: {e}");
    });
}

/// The radius the clipping planes keep in view: everything in `bounds` around the camera's
/// target, as SolidCraft's scenes work it out.
fn view_radius(bounds: &Bounds, cam: &Camera) -> f64 {
    if bounds.is_empty() {
        return cam.half_height().max(10.0) * 2.0;
    }
    (bounds.diagonal() * 0.5).max(cam.half_height()).max(10.0)
        + distance(bounds.center(), cam.target)
}

/// Whether two cameras show the same view.
fn same_view(a: &Camera, b: &Camera) -> bool {
    a.target == b.target
        && a.yaw == b.yaw
        && a.pitch == b.pitch
        && a.roll == b.roll
        && a.distance == b.distance
        && a.fov == b.fov
}

/// The design's box (min, max), or `None` while it is empty.
fn design_box(session: &Session) -> Option<(Vec3, Vec3)> {
    solidcraft_engine::view::bounds(session)
        .ok()
        .filter(|b| !b.is_empty())
        .map(|b| (b.min, b.max))
}

fn distance(a: Vec3, b: Vec3) -> f64 {
    ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
}

/// SolidCraft's examples, in its order: (name, one line on what it is).
pub fn examples() -> Vec<(&'static str, &'static str)> {
    solidcraft_engine::sample::samples()
        .into_iter()
        .map(|s| (s.name, s.about))
        .collect()
}

/// The label of the latest step of `part` (`None`: the model) in an undo or redo list, the way
/// `edit.undo` picks the step it takes back.
fn step_label(steps: &[solidcraft_engine::Snapshot], part: Option<u64>) -> Option<&str> {
    steps
        .iter()
        .rev()
        .find(|s| s.part == part)
        .map(|s| s.label.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_and_redo_name_the_engines_own_steps() {
        let mut d = Design::sample().expect("the sample builds");
        let last = d
            .undo_label()
            .map(str::to_string)
            .expect("the sample's steps are in its history");
        assert_eq!(d.redo_label(), None);
        d.undo().expect("undo");
        assert_eq!(
            d.redo_label(),
            Some(last.as_str()),
            "redo puts back what undo took"
        );
        d.redo().expect("redo");
        assert_eq!(d.undo_label(), Some(last.as_str()));
        assert_eq!(d.redo_label(), None);
    }

    #[test]
    fn every_example_in_the_project_menu_builds() {
        let all = examples();
        assert_eq!(all.len(), 6, "SolidCraft's six samples: {all:?}");
        for (i, (name, _)) in all.iter().enumerate() {
            let d = Design::example(i).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(d.name, *name);
            assert!(d.about.is_some(), "{name} says what it is");
        }
    }
}
