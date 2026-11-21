//! The Design workspace: SolidCraft's engine and kernel modelling inside MakerCAD's shell (P1).
//!
//! First light (2026-11-20). A design is an engine [`Session`]; every change goes through its
//! command surface (`Session::execute`), the same commands SolidCraft's app, its MCP server and
//! the Crew use. The model is drawn on the window's own GPU device with SolidCraft's viewport
//! pipelines (`solidcraft_viewport_gpu::capture::Capturer`), read back and shown as an image, the
//! way SolidCraft draws its thumbnails; where the device cannot draw pictures, its CPU rasterizer
//! does. Drag to orbit, right- or middle-drag to pan, scroll to zoom.
//!
//! Sancho stands beside the design at true size (decision D27, [`crate::scale_figure`]). The
//! opening view frames both, so the design's size reads at once; "Fit view" frames the design.
//!
//! The next step paints straight into the window's render pass
//! (`solidcraft_viewport_egui::ViewportCallback`), which saves the read-back on every frame.

use std::time::Instant;

use eframe::egui_wgpu;
use egui::{pos2, Color32, PointerButton, Rect, Sense, TextureOptions};
use serde_json::Value;
use solidcore_record::Vec3;
use solidcraft_engine::Session;
use solidcraft_viewport::{Camera, Rgb};

use crate::scale_figure::ScaleFigure;
use crate::theme::Palette;

/// What draws the model.
enum Painter {
    /// SolidCraft's viewport pipelines on the window's device.
    Gpu(solidcraft_viewport_gpu::capture::Capturer),
    /// SolidCraft's CPU rasterizer.
    Cpu,
}

/// An open design.
pub struct Design {
    /// Its name, for the top bar.
    pub name: String,
    session: Session,
    camera: Camera,
    painter: Option<Painter>,
    texture: Option<egui::TextureHandle>,
    dirty: bool,
    drawn_px: [u32; 2],
    /// How long building the design took, in ms (running its commands through the kernel).
    pub build_ms: f64,
    /// How long the last picture took, in ms (scene, draw, read-back, upload).
    pub last_draw_ms: Option<f64>,
    /// What drew the last picture (`gpu`, a named adapter, or `cpu`).
    pub renderer: String,
    /// The last error, shown over the view.
    pub error: Option<String>,
    /// Sancho, the scale figure; `None` if his figure could not be read (said once on stderr).
    sancho: Option<ScaleFigure>,
    show_sancho: bool,
}

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
            session,
            camera,
            painter: None,
            texture: None,
            dirty: true,
            drawn_px: [0, 0],
            build_ms,
            last_draw_ms: None,
            renderer: String::new(),
            error: None,
            sancho,
            show_sancho: true,
        };
        design.home_view();
        Ok(design)
    }

    /// SolidCraft's first sample, the plate with fillets, two holes and a bored boss.
    pub fn sample() -> Result<Self, String> {
        let sample = solidcraft_engine::sample::samples()
            .into_iter()
            .next()
            .ok_or("SolidCraft has no samples")?;
        Self::from_script(sample.name, &(sample.script)())
    }

    /// Runs one command on the design (the engine's command surface) and redraws.
    pub fn execute(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let result = self.session.execute(id, params).map_err(|e| e.to_string());
        self.dirty = true;
        result
    }

    /// The opening view: the design and Sancho beside it, so its size reads at once.
    pub fn home_view(&mut self) {
        let Ok(mut camera) = solidcraft_engine::view::home_camera(&self.session) else {
            return;
        };
        if let Some(figure) = self.sancho.as_ref().filter(|_| self.show_sancho) {
            let design = design_box(&self.session);
            let mut both = figure.bounds_at(figure.spot(design));
            if let Some((min, max)) = design {
                both.add(min);
                both.add(max);
            }
            camera.fit(&both);
        }
        self.camera = camera;
        self.dirty = true;
    }

    /// The view that frames the design alone.
    pub fn fit_view(&mut self) {
        if let Ok(camera) = solidcraft_engine::view::home_camera(&self.session) {
            self.camera = camera;
            self.dirty = true;
        }
    }

    /// Sancho's label for the top bar ("Sancho 1.68 m"); `None` when there is no figure.
    pub fn sancho_label(&self) -> Option<String> {
        self.sancho
            .as_ref()
            .map(|s| format!("Sancho {:.2} m", s.height_mm / 1000.0))
    }

    /// Is Sancho standing beside the design?
    pub fn sancho_shown(&self) -> bool {
        self.show_sancho && self.sancho.is_some()
    }

    /// Shows or hides Sancho.
    pub fn show_sancho(&mut self, show: bool) {
        if show != self.show_sancho {
            self.show_sancho = show;
            self.dirty = true;
        }
    }

    /// The view: takes the space left in `ui`, handles orbit, pan and zoom, and draws the model.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        render_state: Option<&egui_wgpu::RenderState>,
        palette: &Palette,
    ) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        if response.dragged_by(PointerButton::Primary) {
            let d = response.drag_delta();
            self.camera.orbit(f64::from(d.x), f64::from(d.y));
            self.dirty = true;
        }
        if response.dragged_by(PointerButton::Secondary)
            || response.dragged_by(PointerButton::Middle)
        {
            let d = response.drag_delta();
            self.camera
                .pan(f64::from(d.x), f64::from(d.y), f64::from(rect.height()));
            self.dirty = true;
        }
        if response.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                self.camera
                    .zoom_at((-f64::from(scroll) * 0.002).exp(), None);
                self.dirty = true;
            }
        }
        if response.double_clicked() {
            self.fit_view();
        }

        let ppp = ui.ctx().pixels_per_point();
        let px = [
            (rect.width() * ppp).round().max(16.0) as u32,
            (rect.height() * ppp).round().max(16.0) as u32,
        ];
        if px != self.drawn_px {
            self.dirty = true;
        }
        if self.dirty {
            self.draw(ui.ctx(), render_state, px, palette);
        }

        let painter = ui.painter_at(rect);
        match &self.texture {
            Some(texture) => {
                painter.image(
                    texture.id(),
                    rect,
                    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                painter.rect_filled(rect, 0.0, palette.bg_bottom);
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

    fn draw(
        &mut self,
        ctx: &egui::Context,
        render_state: Option<&egui_wgpu::RenderState>,
        px: [u32; 2],
        palette: &Palette,
    ) {
        self.dirty = false;
        self.drawn_px = px;
        let painter = self.painter.get_or_insert_with(|| match render_state {
            Some(rs) => match solidcraft_viewport_gpu::capture::Capturer::new(&rs.adapter, rs.device.clone(), rs.queue.clone()) {
                Ok(capturer) => Painter::Gpu(capturer),
                Err(e) => {
                    eprintln!("makercad: the model is drawn by the CPU: the GPU cannot draw pictures: {e}");
                    Painter::Cpu
                }
            },
            None => Painter::Cpu,
        });
        let started = Instant::now();
        let guides = solidcraft_engine::view::Guides {
            grid: true,
            axes: true,
        };
        let mut scene = match solidcraft_engine::view::scene_with_guides(
            &self.session,
            &self.camera,
            &self.session.section,
            guides,
        ) {
            Ok(scene) => scene,
            Err(e) => {
                self.error = Some(e.to_string());
                return;
            }
        };
        // Our look's background, not SolidCraft's.
        scene.background = Some((rgb(palette.bg_top), rgb(palette.bg_bottom)));
        // Sancho beside the design (not part of it: added to the picture only). The scene's
        // radius sets the clipping planes, so it has to reach him too.
        if self.show_sancho {
            if let Some(figure) = self.sancho.as_mut() {
                let spot = figure.spot(design_box(&self.session));
                let b = figure.bounds_at(spot);
                scene.meshes.push(figure.mesh_at(spot));
                scene.radius = scene
                    .radius
                    .max(b.diagonal() * 0.5 + distance(b.center(), self.camera.target));
            }
        }
        let [w, h] = px;
        let drawn = match painter {
            Painter::Gpu(capturer) => capturer
                .rgba(&scene, &self.camera, w, h)
                .map(|rgba| (rgba, capturer.renderer().to_string()))
                .map_err(|e| e.to_string()),
            Painter::Cpu => Ok((
                solidcraft_viewport::render(&scene, &self.camera, w as usize, h as usize).rgba,
                "cpu".to_string(),
            )),
        };
        match drawn {
            Ok((rgba, renderer)) if rgba.len() == (w as usize) * (h as usize) * 4 => {
                let image =
                    egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
                match &mut self.texture {
                    Some(texture) => texture.set(image, TextureOptions::LINEAR),
                    None => {
                        self.texture =
                            Some(ctx.load_texture("design-view", image, TextureOptions::LINEAR))
                    }
                }
                self.renderer = renderer;
                self.error = None;
            }
            Ok((rgba, _)) => {
                self.error = Some(format!(
                    "the picture came back {} bytes, not {}",
                    rgba.len(),
                    w * h * 4
                ))
            }
            Err(e) => self.error = Some(e),
        }
        self.last_draw_ms = Some(started.elapsed().as_secs_f64() * 1000.0);
    }
}

fn rgb(c: Color32) -> Rgb {
    Rgb(c.r(), c.g(), c.b())
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
