//! MakerCAD's desktop shell: our own egui shell (decision D23) around SolidCraft's kernel and
//! viewport. The opening screen, and the Design workspace (SolidCraft's engine inside our shell,
//! P1) with its project bar, environment tabs, toolbox, status bar and command search
//! (`shell.rs`); the Quest, the project file and the cut list follow.
//!
//! Every button, menu, search result and shortcut becomes an [`Act`], carried out in one place
//! ([`MakerApp::run`]).

pub mod commands;
pub mod design;
pub mod environments;
pub mod graphics;
pub mod home;
mod icon_data;
pub mod icons;
pub mod probe;
pub mod project_file;
pub mod sancho;
pub mod scale_figure;
pub mod shell;
pub mod stage;
pub mod theme;

use std::time::{Duration, Instant};

use egui::{vec2, Align2, CornerRadius, Frame, Key, Margin, Modifiers, RichText, Stroke};

use commands::{Act, Command, Units};
use environments::Env;
use home::{ChipKind, HomeAction, HomeIcons, HomeModel, Recent};
use theme::Look;

/// The settings key the look is stored under. Never change it (compatibility rule C3).
const LOOK_KEY: &str = "look";
/// The settings key the units are stored under. Never change it (C3).
const UNITS_KEY: &str = "units";

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
    /// Open command search at start with this typed (`--search`, for screenshots and checks).
    pub search: Option<String>,
    /// With `sample`: start on this environment's tab (`--tab`).
    pub tab: Option<Env>,
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
    /// The open design's project file, once it has one (saved or opened).
    project: Option<project_file::Project>,
    /// The folder projects are saved in and listed from.
    projects_dir: std::path::PathBuf,
    /// The project files in that folder, newest first: the opening screen's rows.
    listed: Vec<project_file::Listed>,
    spin: Spin,
    /// The environment tab in front.
    env: Env,
    /// Lengths in millimetres or inches.
    units: Units,
    /// Command search (Ctrl+K).
    search: shell::Search,
    /// SolidCraft's examples (name, one line).
    examples: Vec<(&'static str, &'static str)>,
    /// Every command, for search.
    commands: Vec<Command>,
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
        let units = cc
            .storage
            .and_then(|s| s.get_string(UNITS_KEY))
            .and_then(|k| Units::from_key(&k))
            .unwrap_or(Units::Mm);
        let examples = design::examples();
        let commands = commands::all(&examples);
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
            project: None,
            projects_dir: project_file::folder(),
            listed: Vec::new(),
            spin: Spin::default(),
            env: Env::Design,
            units,
            search: shell::Search::default(),
            examples,
            commands,
            launch,
        };
        if app.launch.demo {
            app.load_demo();
        } else {
            app.refresh_projects();
        }
        if app.launch.sample {
            app.open_example(0, 0.0);
            if app.launch.fit {
                if let Some(design) = app.design.as_mut() {
                    design.fit_view();
                }
            }
            if let Some(tab) = app.launch.tab {
                app.env = tab;
            }
        }
        if let Some(text) = app.launch.search.clone() {
            app.search.open_with(&text);
        }
        app
    }

    /// Opens SolidCraft's example number `index` in the Design workspace.
    fn open_example(&mut self, index: usize, now: f64) {
        let name = self.examples.get(index).map_or("the example", |e| e.0);
        match design::Design::example(index) {
            Ok(d) => {
                self.design = Some(d);
                // An example is not a project until it is saved.
                self.project = None;
                self.screen = Screen::Design;
                self.env = Env::Design;
            }
            Err(e) => self.notice = Some((format!("{name} did not build: {e}"), now + 8.0)),
        }
    }

    /// Reads the projects folder again and makes the opening screen's rows from it.
    fn refresh_projects(&mut self) {
        self.listed = project_file::list(&self.projects_dir);
        let now = std::time::SystemTime::now();
        self.recents = self
            .listed
            .iter()
            .map(|l| Recent {
                name: project_file::stem(&l.file).to_string(),
                done: 0,
                under_way: false,
                when: saved_when(l.modified, now),
                chip: "saved".into(),
                chip_kind: ChipKind::Neutral,
            })
            .collect();
        self.keep_going = self.recents.first().map(|r| {
            (
                r.name.clone(),
                format!("Last {}. Open it to carry on where you left off.", r.when),
            )
        });
    }

    /// Saves the open design as a project file (Ctrl+S); its first save makes the project.
    /// Returns what to tell the person.
    fn save_project(&mut self) -> String {
        let Some(design) = self.design.as_ref() else {
            return "Open a design first (Examples), then save it as a project.".into();
        };
        let bytes = match design.design_bytes() {
            Ok(bytes) => bytes,
            Err(e) => return format!("Not saved: the design could not be written ({e})."),
        };
        if self.project.is_none() {
            match project_file::Project::new(&design.name, design.about.as_deref().unwrap_or("")) {
                Ok(project) => self.project = Some(project),
                Err(e) => return format!("Not saved: {e}"),
            }
        }
        let Some(project) = self.project.as_ref() else {
            return "Not saved.".into();
        };
        let result = project.save(&self.projects_dir, &bytes);
        let file = project.file.clone();
        self.refresh_projects();
        match result {
            Ok(path) => format!(
                "Saved {file} in {}",
                path.parent().unwrap_or(&path).display()
            ),
            Err(e) => format!("Not saved: {e}"),
        }
    }

    /// Opens the project file listed at `index` in the Design workspace. Returns what to tell
    /// the person.
    fn open_project(&mut self, index: usize) -> String {
        let Some(listed) = self.listed.get(index).cloned() else {
            return "That project is no longer in the folder.".into();
        };
        let loaded = match project_file::open(&self.projects_dir, &listed.file) {
            Ok(loaded) => loaded,
            Err(e) => return format!("{} did not open: {e}", listed.file),
        };
        let name = loaded.project.intent.name.clone();
        match design::Design::from_design_bytes(&name, &loaded.design) {
            Ok(mut d) => {
                d.about = Some(loaded.project.intent.goal.clone());
                self.design = Some(d);
                self.project = Some(loaded.project);
                self.screen = Screen::Design;
                self.env = Env::Design;
                let mut text = if loaded.recovered {
                    format!(
                        "{} was damaged, so its recovery copy (the save before) opened.",
                        listed.file
                    )
                } else {
                    format!("Opened {}.", listed.file)
                };
                if loaded.changed_outside > 0 {
                    text.push_str(&format!(
                        " {} of its parts changed outside MakerCAD since it was saved.",
                        loaded.changed_outside
                    ));
                }
                text
            }
            Err(e) => format!("{}: its design did not open ({e})", listed.file),
        }
    }

    /// Carries out what the person asked for: a button, a menu, a search result or a shortcut.
    fn run(&mut self, act: Act, now: f64) {
        let mut say = |text: String, seconds: f64| self.notice = Some((text, now + seconds));
        match act {
            Act::Home => self.screen = Screen::Home,
            Act::OpenExample(i) => self.open_example(i, now),
            // Ctrl+K again closes it.
            Act::Search if self.search.open => self.search.open = false,
            Act::Search => self.search.open(),
            Act::Settings => self.settings_open = true,
            Act::About => self.help_open = true,
            Act::Save => {
                let text = self.save_project();
                self.notice = Some((text, now + 6.0));
            }
            Act::SetLook(look) => self.look = look,
            Act::SetUnits(units) => self.units = units,
            Act::Later(l) if l.phase == "now" => say(format!("{}: {}", l.what, l.note), 6.0),
            Act::Later(l) => say(format!("{} comes in {}. {}", l.what, l.phase, l.note), 7.0),
            Act::Tab(env) => {
                if self.design.is_some() {
                    self.env = env;
                    self.screen = Screen::Design;
                } else {
                    say(
                        "The environments are tabs of a project: open one first (Examples).".into(),
                        6.0,
                    );
                }
            }
            Act::FitView | Act::WholeView | Act::LookFrom(_) | Act::Sancho => {
                if let Some(d) = self.design.as_mut() {
                    match act {
                        Act::FitView => d.fit_view(),
                        Act::WholeView => d.home_view(),
                        Act::LookFrom(side) => d.look_from(side),
                        _ => d.show_sancho(!d.sancho_shown()),
                    }
                    self.env = Env::Design;
                }
            }
            Act::Undo | Act::Redo => {
                if let Some(d) = self.design.as_mut() {
                    let (label, result, done) = if act == Act::Undo {
                        (d.undo_label().map(str::to_string), d.undo(), "Undone")
                    } else {
                        (d.redo_label().map(str::to_string), d.redo(), "Redone")
                    };
                    self.env = Env::Design;
                    match (result, label) {
                        (Ok(()), Some(label)) => {
                            self.notice = Some((format!("{done}: {label}"), now + 3.0))
                        }
                        (Ok(()), None) => {}
                        (Err(e), _) => self.notice = Some((e, now + 5.0)),
                    }
                }
            }
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
            HomeAction::KeepGoing if !self.listed.is_empty() => {
                Some((self.open_project(0), now + 6.0))
            }
            HomeAction::KeepGoing if self.keep_going.is_some() => {
                say("These are the design canvas's sample projects (--demo); they do not open.")
            }
            HomeAction::KeepGoing => {
                say("Nothing saved yet. Open an example and save it (Ctrl+S) to make a project.")
            }
            HomeAction::OpenRecent(i) if i < self.listed.len() => {
                Some((self.open_project(i), now + 6.0))
            }
            HomeAction::OpenRecent(_) => {
                say("These are the design canvas's sample projects (--demo); they do not open.")
            }
            HomeAction::OpenWorkshop => {
                say("The workshop (your tools and materials) comes with P1.")
            }
            HomeAction::ImportProject => say("Import comes with the project file (P1)."),
            HomeAction::Examples => {
                self.open_example(0, now);
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
                ui.label(RichText::new("Sizes in").strong());
                ui.horizontal(|ui| {
                    for (units, name) in [(Units::Mm, "millimetres"), (Units::In, "inches")] {
                        if ui.selectable_label(self.units == units, name).clicked() {
                            self.units = units;
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
    /// The Design workspace (doc 11 §11.4): the project bar and environment tabs along the top,
    /// the toolbox down the left, and SolidCraft's model view filling the rest. A tab whose
    /// environment comes later shows what it will do instead of the view.
    fn design_screen(&mut self, ui: &mut egui::Ui, now: f64) {
        let p = self.look.palette();
        let env = self.env;
        let mut acts: Vec<Act> = Vec::new();
        {
            let design = self.design.as_ref();
            let model = shell::BarModel {
                name: design.map_or("", |d| d.name.as_str()),
                about: design.and_then(|d| d.about.as_deref()),
                undo: design.and_then(|d| d.undo_label()),
                redo: design.and_then(|d| d.redo_label()),
                sancho: design.and_then(|d| d.sancho_label().map(|l| (d.sancho_shown(), l))),
                examples: &self.examples,
            };
            let mut kit = shell::Kit::new(self.look, &mut self.icons);
            let bar = |fill| {
                Frame::new()
                    .fill(fill)
                    .inner_margin(Margin::symmetric(12, 0))
                    .stroke(Stroke::new(1.0, p.line))
            };
            egui::Panel::top("project-bar")
                .exact_size(shell::BAR_H)
                .frame(bar(p.panel))
                .show(ui, |ui| {
                    acts.extend(shell::project_bar(ui, &mut kit, &model))
                });
            egui::Panel::top("env-tabs")
                .exact_size(shell::TABS_H)
                .frame(bar(p.bg2))
                .show(ui, |ui| acts.extend(shell::env_tabs(ui, &mut kit, env)));
            if env == Env::Design {
                egui::Panel::left("toolbox")
                    .exact_size(shell::TOOL_W)
                    .resizable(false)
                    .frame(Frame::new().fill(p.panel).stroke(Stroke::new(1.0, p.line)))
                    .show(ui, |ui| acts.extend(shell::toolbox(ui, &mut kit)));
            }
        }
        let render_state = self.render_state.clone();
        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ui, |ui| {
                if env == Env::Design {
                    let rect = ui.max_rect();
                    if let Some(design) = self.design.as_mut() {
                        design.show(ui, render_state.as_ref(), &p);
                    }
                    ui.painter().text(
                        rect.left_bottom() + vec2(12.0, -10.0),
                        Align2::LEFT_BOTTOM,
                        "drag: turn \u{b7} right-drag: move \u{b7} scroll: zoom \u{b7} double-click: fit",
                        egui::FontId::proportional(12.0),
                        p.muted,
                    );
                } else {
                    egui::Frame::new().fill(p.bg2).show(ui, |ui| {
                        ui.set_min_size(ui.available_size());
                        let mut kit = shell::Kit::new(self.look, &mut self.icons);
                        acts.extend(shell::env_page(ui, &mut kit, env));
                    });
                }
            });
        for act in acts {
            self.run(act, now);
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
/// When a project file was last saved, for the opening screen ("saved today").
fn saved_when(modified: Option<std::time::SystemTime>, now: std::time::SystemTime) -> String {
    match modified.and_then(|m| now.duration_since(m).ok()) {
        None => "saved".into(),
        Some(age) => match age.as_secs() / 86_400 {
            0 => "saved today".into(),
            1 => "saved yesterday".into(),
            days => format!("saved {days} days ago"),
        },
    }
}

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

        // Shortcuts: Ctrl+K anywhere; undo and redo when no text field has the keyboard.
        let mut acts: Vec<Act> = Vec::new();
        let typing = ctx.egui_wants_keyboard_input();
        ctx.input_mut(|i| {
            if i.consume_key(Modifiers::COMMAND, Key::K) {
                acts.push(Act::Search);
            }
            if self.screen == Screen::Design && i.consume_key(Modifiers::COMMAND, Key::S) {
                acts.push(Act::Save);
            }
            if !typing && !self.search.open && self.screen == Screen::Design {
                if i.consume_key(Modifiers::COMMAND | Modifiers::SHIFT, Key::Z)
                    || i.consume_key(Modifiers::COMMAND, Key::Y)
                {
                    acts.push(Act::Redo);
                } else if i.consume_key(Modifiers::COMMAND, Key::Z) {
                    acts.push(Act::Undo);
                }
            }
        });
        for act in acts.drain(..) {
            self.run(act, now);
        }

        let palette = self.look.palette();
        let design_open = self.screen == Screen::Design && self.design.is_some();
        let details = self.status_line();
        let backend = self
            .launch
            .graphics
            .as_ref()
            .map_or_else(|| "no window".to_string(), |c| c.backend.clone());
        let sancho_mm = self
            .design
            .as_ref()
            .filter(|_| design_open)
            .and_then(design::Design::sancho_height_mm);
        {
            let status = shell::StatusModel {
                units: self.units,
                sancho_mm,
                backend: &backend,
                details: &details,
                design: design_open,
            };
            let mut kit = shell::Kit::new(self.look, &mut self.icons);
            egui::Panel::bottom("status")
                .frame(
                    Frame::new()
                        .fill(palette.bg2)
                        .inner_margin(Margin::symmetric(16, 4))
                        .stroke(Stroke::new(1.0, palette.line)),
                )
                .show(ui, |ui| {
                    acts.extend(shell::status_bar(ui, &mut kit, &status))
                });
        }
        for act in acts.drain(..) {
            self.run(act, now);
        }

        if design_open {
            self.design_screen(ui, now);
        } else {
            self.home_screen(ui, &ctx, now);
        }
        self.windows(&ctx);
        let mut kit = shell::Kit::new(self.look, &mut self.icons);
        let found = shell::search_window(
            &ctx,
            &mut kit,
            &mut self.search,
            &self.commands,
            design_open,
        );
        if let Some(act) = found {
            self.run(act, now);
        }
        self.show_notice(&ctx, now);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // A screenshot run leaves the settings alone; so does a `--look` start the person didn't
        // change (2026-11-19: a Night Workshop screenshot turned the next start dark).
        if self.launch.screenshot.is_some() {
            return;
        }
        storage.set_string(UNITS_KEY, self.units.key().to_string());
        if self.launch.look != Some(self.look) {
            storage.set_string(LOOK_KEY, self.look.key().to_string());
        }
    }
}

impl MakerApp {
    /// The opening screen.
    fn home_screen(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, now: f64) {
        let palette = self.look.palette();

        let ppp = ctx.pixels_per_point();
        let mut home_icons = HomeIcons::default();
        for (id, points) in HomeIcons::WANTED {
            let px = (points * ppp).round().clamp(1.0, 512.0) as u32;
            let texture = self.icons.get(ctx, id, self.look, px);
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
