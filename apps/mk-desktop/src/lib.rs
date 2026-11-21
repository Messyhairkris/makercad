//! MakerCAD's desktop shell: our own egui shell (decision D23) around SolidCraft's kernel and
//! viewport. The opening screen, and the Design workspace (SolidCraft's engine inside our shell,
//! P1); the Quest, the project file and the cut list follow.

pub mod design;
pub mod graphics;
pub mod home;
mod icon_data;
pub mod icons;
pub mod sancho;
pub mod scale_figure;
pub mod stage;
pub mod theme;

use std::time::{Duration, Instant};

use egui::{vec2, Align2, CornerRadius, Frame, Margin, RichText, Stroke};

use home::{ChipKind, HomeAction, HomeIcons, HomeModel, Recent};
use theme::Look;

/// The settings key the look is stored under. Never change it (compatibility rule C3).
const LOOK_KEY: &str = "look";

/// How the shell was started.
#[derive(Clone, Debug)]
pub struct Launch {
    /// When `main` started, for the first-frame time.
    pub started: Instant,
    /// The graphics backend choice; `None` when there is no window (headless snapshots).
    pub graphics: Option<graphics::Choice>,
    /// Show the design canvas's sample projects instead of this machine's (for screenshots).
    pub demo: bool,
    /// Maximize this long after the first frame (OpenGL misbehaves when it starts maximized).
    pub maximize_after: Option<Duration>,
    /// Write a PNG of the window here once it has settled, then close (`--screenshot`).
    pub screenshot: Option<std::path::PathBuf>,
    /// Start in this look instead of the saved one (`--look`).
    pub look: Option<Look>,
    /// Open SolidCraft's sample plate in the Design workspace at start (`--sample`).
    pub sample: bool,
    /// With `sample`: frame the plate alone, not the plate and Sancho (`--fit`, for screenshots).
    pub fit: bool,
    /// With `sample`: turn the view for this many seconds, print the frame times and close
    /// (`--spin`, for measuring).
    pub spin: Option<f64>,
}

/// What `--spin` measured: the time of each frame (egui's input time), once the view has settled.
#[derive(Default)]
struct Spin {
    times: Vec<f64>,
    reported: bool,
}

impl Spin {
    /// `"N frames in S s: mean X ms, median Y ms, p95 Z ms (F FPS)"` over the intervals.
    fn report(&self) -> String {
        let mut gaps: Vec<f64> = self
            .times
            .windows(2)
            .map(|w| (w[1] - w[0]) * 1000.0)
            .collect();
        if gaps.is_empty() {
            return "no frames measured".into();
        }
        gaps.sort_by(f64::total_cmp);
        let n = gaps.len();
        let mean = gaps.iter().sum::<f64>() / n as f64;
        let at = |q: f64| {
            gaps.get(((n as f64 - 1.0) * q).round() as usize)
                .copied()
                .unwrap_or(mean)
        };
        let span =
            self.times.last().copied().unwrap_or(0.0) - self.times.first().copied().unwrap_or(0.0);
        format!(
            "{n} frames in {span:.1} s: mean {mean:.1} ms, median {:.1} ms, p95 {:.1} ms ({:.0} FPS)",
            at(0.5),
            at(0.95),
            1000.0 / mean
        )
    }
}

/// Which screen the window shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Home,
    Design,
}

/// The app.
pub struct MakerApp {
    launch: Launch,
    look: Look,
    applied: Option<Look>,
    recents: Vec<Recent>,
    keep_going: Option<(String, String)>,
    sancho_says: String,
    frames: u32,
    first_frame: Option<Duration>,
    adapter: Option<String>,
    settings_open: bool,
    help_open: bool,
    notice: Option<(String, f64)>,
    maximize_at: Option<f64>,
    icons: icons::Icons,
    shot_done: std::sync::Arc<std::sync::atomic::AtomicBool>,
    render_state: Option<eframe::egui_wgpu::RenderState>,
    screen: Screen,
    design: Option<design::Design>,
    spin: Spin,
}

/// Frames drawn before `--screenshot` takes its picture, so layout and icons have settled.
const SCREENSHOT_AFTER_FRAMES: u32 = 12;

impl MakerApp {
    /// Builds the app from eframe's creation context.
    pub fn new(cc: &eframe::CreationContext<'_>, launch: Launch) -> Self {
        let look = launch.look.unwrap_or_else(|| {
            cc.storage
                .and_then(|s| s.get_string(LOOK_KEY))
                .and_then(|k| Look::from_key(&k))
                .unwrap_or(Look::Daydream)
        });
        let adapter = cc.wgpu_render_state.as_ref().map(|rs| {
            let info = rs.adapter.get_info();
            format!("{} \u{b7} {:?}", info.name, info.backend)
        });
        let mut app = Self {
            look,
            applied: None,
            recents: Vec::new(),
            keep_going: None,
            sancho_says: "Hi, I'm Sancho! Tell me what you want to make, and we'll work out how \
                          to build it with what you have."
                .into(),
            frames: 0,
            first_frame: None,
            adapter,
            settings_open: false,
            help_open: false,
            notice: None,
            maximize_at: None,
            icons: icons::Icons::default(),
            shot_done: Default::default(),
            render_state: cc.wgpu_render_state.clone(),
            screen: Screen::Home,
            design: None,
            spin: Spin::default(),
            launch,
        };
        if app.launch.demo {
            app.load_demo();
        }
        if app.launch.sample {
            app.open_sample(0.0);
            if app.launch.fit {
                if let Some(design) = app.design.as_mut() {
                    design.fit_view();
                }
            }
        }
        app
    }

    /// Opens SolidCraft's sample plate in the Design workspace.
    fn open_sample(&mut self, now: f64) {
        match design::Design::sample() {
            Ok(d) => {
                self.design = Some(d);
                self.screen = Screen::Design;
            }
            Err(e) => self.notice = Some((format!("The sample did not build: {e}"), now + 8.0)),
        }
    }

    /// The look in use.
    pub fn look(&self) -> Look {
        self.look
    }

    /// Changes the look (as Settings does).
    pub fn set_look(&mut self, look: Look) {
        self.look = look;
    }

    /// The design canvas's sample content (docs/design/ui-v1/Main.dc.html).
    fn load_demo(&mut self) {
        let row = |name: &str, done, under_way, when: &str, chip: &str, chip_kind| Recent {
            name: name.into(),
            done,
            under_way,
            when: when.into(),
            chip: chip.into(),
            chip_kind,
        };
        self.recents = vec![
            row(
                "Alcove dresser",
                2,
                true,
                "due Dec 12",
                "2 open pages",
                ChipKind::Waiting,
            ),
            row(
                "Sharpener + box",
                4,
                false,
                "no deadline",
                "log complete",
                ChipKind::Done,
            ),
            row(
                "Hike pack v2",
                1,
                false,
                "due Mar 3",
                "just started",
                ChipKind::Neutral,
            ),
        ];
        self.keep_going = Some((
            "Alcove dresser".into(),
            "Last time: cut list done, two drawers glued. Next: drawer runners.".into(),
        ));
        self.sancho_says = "Welcome back! It's been 3 days. The drawer fronts are waiting.".into();
    }

    fn workshop_line(&self) -> String {
        let pc = match &self.adapter {
            Some(a) => format!("this PC: {a}"),
            None => "this PC: no window (snapshot)".into(),
        };
        if self.launch.demo {
            format!("41 tools \u{b7} 23 materials \u{b7} 6 salvage \u{b7} {pc}")
        } else {
            format!("No tools or materials yet \u{b7} {pc}")
        }
    }

    fn status_line(&self) -> String {
        let mut parts = vec![
            format!("MakerCAD {}", env!("CARGO_PKG_VERSION")),
            "foundation build".into(),
        ];
        if let Some(c) = &self.launch.graphics {
            parts.push(format!("graphics: {}", c.describe()));
        }
        if let Some(t) = self.first_frame {
            parts.push(format!("first frame {} ms", t.as_millis()));
        }
        if let (Screen::Design, Some(d)) = (self.screen, &self.design) {
            parts.push(format!("built in {:.0} ms", d.build_ms));
            if let Some(ms) = d.last_draw_ms {
                parts.push(format!("scene {:.0} ms ({})", ms, d.renderer));
            }
        }
        parts.join("  \u{b7}  ")
    }

    fn act(&mut self, action: HomeAction, now: f64) {
        let say = |text: &str| Some((text.to_string(), now + 5.0));
        self.notice = match action {
            HomeAction::Settings => {
                self.settings_open = true;
                None
            }
            HomeAction::Help => {
                self.help_open = true;
                None
            }
            HomeAction::StartNew => {
                say("The Quest (Start something new) is the first screen of P1. Not built yet.")
            }
            HomeAction::KeepGoing if self.keep_going.is_some() => {
                say("The Workspace is still a design (docs/design/ui-v1). It is built in P1.")
            }
            HomeAction::KeepGoing => say("Nothing under way yet. Start something new first."),
            HomeAction::OpenRecent(_) => {
                say("Opening a project needs the project file, which comes with P1.")
            }
            HomeAction::OpenWorkshop => {
                say("The workshop (your tools and materials) comes with P1.")
            }
            HomeAction::ImportProject => say("Import comes with the project file (P1)."),
            HomeAction::Examples => {
                self.open_sample(now);
                None
            }
        };
    }

    fn windows(&mut self, ctx: &egui::Context) {
        let mut settings_open = self.settings_open;
        egui::Window::new("Settings")
            .open(&mut settings_open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, vec2(-32.0, 80.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Look").strong());
                ui.horizontal(|ui| {
                    for look in Look::ALL {
                        if ui
                            .selectable_label(self.look == look, look.name())
                            .clicked()
                        {
                            self.look = look;
                        }
                    }
                });
                ui.add_space(8.0);
                ui.label(RichText::new("Graphics").strong());
                let graphics = self
                    .launch
                    .graphics
                    .as_ref()
                    .map(graphics::Choice::describe)
                    .unwrap_or_else(|| "none (headless)".into());
                ui.label(graphics);
                ui.label(
                    RichText::new(
                        "To change it, start with --backend dx12, vulkan or gl. --retest forgets \
                         what worked and tries the best backend again.",
                    )
                    .size(13.0),
                );
            });
        self.settings_open = settings_open;

        let mut help_open = self.help_open;
        let status = self.status_line();
        let adapter = self.adapter.clone().unwrap_or_else(|| "none".into());
        egui::Window::new("About MakerCAD")
            .open(&mut help_open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, vec2(-32.0, 80.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Design with what you have.").size(18.0).strong());
                ui.label("A project tool with a CAD core, for people who design and build the thing themselves.");
                ui.add_space(8.0);
                ui.label(format!("Graphics adapter: {adapter}"));
                ui.label(RichText::new(status).size(13.0));
                ui.label(RichText::new("MIT OR Apache-2.0. Built on SolidCraft's kernel (MIT OR Apache-2.0).").size(13.0));
            });
        self.help_open = help_open;
    }
}

impl MakerApp {
    /// The Design workspace: a top bar, and SolidCraft's model view filling the rest.
    fn design_screen(&mut self, ui: &mut egui::Ui, palette: &theme::Palette) {
        let mut go_home = false;
        let mut fit_view = false;
        let name = self
            .design
            .as_ref()
            .map(|d| d.name.clone())
            .unwrap_or_default();
        let sancho_label = self.design.as_ref().and_then(|d| d.sancho_label());
        let mut show_sancho = self.design.as_ref().is_some_and(|d| d.sancho_shown());
        egui::Panel::top("design-bar")
            .frame(Frame::new().fill(palette.bg2).inner_margin(Margin::symmetric(16, 8)).stroke(Stroke::new(1.0, palette.line)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Plain text: the window's font has no arrow glyph (it drew a box).
                    if ui.button("Home").clicked() {
                        go_home = true;
                    }
                    ui.add_space(12.0);
                    ui.label(RichText::new(name).size(17.0).strong().color(palette.ink));
                    ui.add_space(12.0);
                    ui.label(RichText::new("Design").size(14.0).color(palette.muted));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Fit view").on_hover_text("Frame the design").clicked() {
                            fit_view = true;
                        }
                        if let Some(label) = &sancho_label {
                            ui.toggle_value(&mut show_sancho, label.as_str())
                                .on_hover_text("Sancho stands beside the design at true size, so you can see how big it is");
                        }
                        ui.label(
                            RichText::new("drag: turn \u{b7} right-drag: move \u{b7} scroll: zoom \u{b7} double-click: fit")
                                .size(12.0)
                                .color(palette.muted),
                        );
                    });
                });
            });
        let render_state = self.render_state.clone();
        if let Some(design) = self.design.as_mut() {
            design.show_sancho(show_sancho);
            if fit_view {
                design.fit_view();
            }
            egui::CentralPanel::default()
                .frame(Frame::NONE)
                .show(ui, |ui| {
                    design.show(ui, render_state.as_ref(), palette);
                });
        }
        if go_home {
            self.screen = Screen::Home;
        }
    }

    /// The short message over the bottom of the window.
    fn show_notice(&mut self, ctx: &egui::Context, now: f64) {
        if let Some((text, until)) = self.notice.clone() {
            if now > until {
                self.notice = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(250));
                egui::Area::new(egui::Id::unique("notice"))
                    .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -48.0))
                    .show(ctx, |ui| {
                        Frame::new()
                            .fill(theme::SANCHO_INK)
                            .corner_radius(CornerRadius::same(14))
                            .inner_margin(Margin::symmetric(18, 12))
                            .show(ui, |ui| {
                                ui.label(RichText::new(text).size(15.0).color(theme::PAPER));
                            });
                    });
            }
        }
    }
}

/// Writes an egui image as a PNG.
fn save_png(image: &egui::ColorImage, path: &std::path::Path) -> Result<(), String> {
    let [w, h] = image.size;
    let bytes: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
    let width = u32::try_from(w).map_err(|e| e.to_string())?;
    let height = u32::try_from(h).map_err(|e| e.to_string())?;
    let buffer = image::RgbaImage::from_raw(width, height, bytes)
        .ok_or_else(|| "the picture's size and pixels disagree".to_string())?;
    buffer.save(path).map_err(|e| e.to_string())
}

impl eframe::App for MakerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if self.applied != Some(self.look) {
            theme::apply(&ctx, self.look);
            self.applied = Some(self.look);
        }
        let now = ctx.input(|i| i.time);
        self.frames = self.frames.saturating_add(1);
        if self.first_frame.is_none() {
            self.first_frame = Some(self.launch.started.elapsed());
            if self.adapter.is_none() {
                self.adapter = frame.wgpu_render_state().map(|rs| {
                    let info = rs.adapter.get_info();
                    format!("{} \u{b7} {:?}", info.name, info.backend)
                });
            }
            self.maximize_at = self.launch.maximize_after.map(|d| now + d.as_secs_f64());
        }
        // A backend counts as working once a few frames have gone to the screen.
        if self.frames < 4 {
            ctx.request_repaint();
        } else if self.frames == 4 {
            if let Some(c) = &self.launch.graphics {
                graphics::mark_ok(c);
            }
        }
        if let Some(at) = self.maximize_at {
            if now >= at {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                self.maximize_at = None;
            } else {
                ctx.request_repaint_after(Duration::from_millis(100));
            }
        }
        // `--screenshot PATH`: the real window, drawn by the real renderer, saved as a PNG.
        if let Some(path) = self.launch.screenshot.clone() {
            use std::sync::atomic::Ordering;
            if self.shot_done.load(Ordering::SeqCst) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else if self.frames == SCREENSHOT_AFTER_FRAMES {
                let done = self.shot_done.clone();
                let repaint = ctx.clone();
                let callback = egui::ScreenshotCallback::new(move |image| {
                    match save_png(&image, &path) {
                        Ok(()) => eprintln!("makercad: wrote {}", path.display()),
                        Err(e) => eprintln!("makercad: {}: {e}", path.display()),
                    }
                    done.store(true, Ordering::SeqCst);
                    repaint.request_repaint();
                });
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
            }
            ctx.request_repaint();
        }
        // `--spin SECONDS`: turn the view a little every frame, then report the frame times.
        if let (Some(seconds), Screen::Design) = (self.launch.spin, self.screen) {
            if self.frames > SCREENSHOT_AFTER_FRAMES && !self.spin.reported {
                if let Some(design) = self.design.as_mut() {
                    design.turn(4.0);
                }
                self.spin.times.push(now);
                let first = self.spin.times.first().copied().unwrap_or(now);
                if now - first >= seconds {
                    eprintln!("makercad: spin: {}", self.spin.report());
                    self.spin.reported = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            ctx.request_repaint();
        }

        let palette = self.look.palette();
        egui::Panel::bottom("status")
            .frame(
                Frame::new()
                    .fill(palette.bg2)
                    .inner_margin(Margin::symmetric(16, 6))
                    .stroke(Stroke::new(1.0, palette.line)),
            )
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.status_line())
                        .size(12.0)
                        .color(palette.muted),
                );
            });

        if self.screen == Screen::Design && self.design.is_some() {
            self.design_screen(ui, &palette);
            self.windows(&ctx);
            self.show_notice(&ctx, now);
            return;
        }

        let ppp = ctx.pixels_per_point();
        let mut home_icons = HomeIcons::default();
        for (id, points) in HomeIcons::WANTED {
            let px = (points * ppp).round().clamp(1.0, 512.0) as u32;
            let texture = self.icons.get(&ctx, id, self.look, px);
            match id {
                "app.settings" => home_icons.settings = texture,
                "app.help" => home_icons.help = texture,
                "app.new" => home_icons.new = texture,
                "app.open" => home_icons.open = texture,
                _ => home_icons.workshop = texture,
            }
        }
        let keep_going = self
            .keep_going
            .as_ref()
            .map(|(a, b)| (a.as_str(), b.as_str()));
        let workshop_line = self.workshop_line();
        let model = HomeModel {
            palette,
            icons: home_icons,
            recents: &self.recents,
            keep_going,
            workshop_line,
            sancho_says: &self.sancho_says,
            sancho_height_mm: 1680.0,
        };
        let action = egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ui, |ui| home::show(ui, &model))
            .inner;
        if let Some(action) = action {
            self.act(action, now);
        }

        self.windows(&ctx);
        self.show_notice(&ctx, now);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // A screenshot run, or a `--look` start the person didn't change, leaves the saved look
        // alone (2026-11-19: a Night Workshop screenshot turned the next start dark).
        if self.launch.screenshot.is_some() || self.launch.look == Some(self.look) {
            return;
        }
        storage.set_string(LOOK_KEY, self.look.key().to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::Spin;

    #[test]
    fn the_spin_report_reads_the_frame_intervals() {
        // 11 frames 20 ms apart, then one slow frame of 100 ms.
        let mut times: Vec<f64> = (0..11).map(|i| f64::from(i) * 0.020).collect();
        times.push(0.300);
        let report = Spin {
            times,
            reported: false,
        }
        .report();
        assert!(report.starts_with("11 frames in 0.3 s"), "{report}");
        assert!(report.contains("median 20.0 ms"), "{report}");
        assert!(report.contains("p95 100.0 ms"), "{report}");
        assert!(report.contains("(37 FPS)"), "{report}");
        assert_eq!(Spin::default().report(), "no frames measured");
    }
}
