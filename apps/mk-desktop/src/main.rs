//! `makercad`: starts the desktop app.
//!
//! ```text
//! makercad [--backend dx12|vulkan|gl] [--retest] [--demo] [--sample [--fit]] [--windowed]
//!          [--screenshot PATH] [--look daydream|pastel|night-workshop] [--size WxH]
//! ```
//!
//! `--backend` picks the graphics backend for this start only; `--retest` forgets the backend
//! that worked last time (see `graphics.rs`); `--demo` shows the design canvas's sample
//! projects; `--sample` opens SolidCraft's sample plate in the Design workspace (`--fit`: framed
//! on the plate alone, not the plate and Sancho); `--windowed` does not maximize. `--screenshot` draws the window, writes it as a PNG and closes (for agents
//! and reviews: AGENTS.md rule 10); it implies `--windowed`.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use mk_desktop::theme::Look;
use mk_desktop::{graphics, Launch, MakerApp};

struct Args {
    backend: Option<String>,
    retest: bool,
    demo: bool,
    sample: bool,
    fit: bool,
    windowed: bool,
    screenshot: Option<PathBuf>,
    look: Option<Look>,
    size: [f32; 2],
}

const USAGE: &str = "usage: makercad [--backend dx12|vulkan|gl] [--retest] [--demo] [--sample [--fit]] [--windowed] \
                     [--screenshot PATH] [--look daydream|pastel|night-workshop] [--size WxH]";

fn parse_args() -> Result<Args, String> {
    let mut a = Args {
        backend: None,
        retest: false,
        demo: false,
        sample: false,
        fit: false,
        windowed: false,
        screenshot: None,
        look: None,
        size: [1280.0, 800.0],
    };
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        let mut value = || {
            it.next()
                .ok_or_else(|| format!("{arg} needs a value; {USAGE}"))
        };
        match arg.as_str() {
            "--backend" => a.backend = Some(value()?),
            "--retest" => a.retest = true,
            "--demo" => a.demo = true,
            "--sample" => a.sample = true,
            "--fit" => a.fit = true,
            "--windowed" => a.windowed = true,
            "--screenshot" => a.screenshot = Some(PathBuf::from(value()?)),
            "--look" => {
                let v = value()?;
                a.look =
                    Some(Look::from_key(&v).ok_or_else(|| format!("unknown look {v}; {USAGE}"))?);
            }
            "--size" => {
                let v = value()?;
                let (w, h) = v
                    .split_once('x')
                    .ok_or_else(|| format!("--size WxH, not {v}"))?;
                let w: f32 = w.parse().map_err(|_| format!("bad width in {v}"))?;
                let h: f32 = h.parse().map_err(|_| format!("bad height in {v}"))?;
                if !(320.0..=8192.0).contains(&w) || !(240.0..=8192.0).contains(&h) {
                    return Err(format!("--size {v} is out of range"));
                }
                a.size = [w, h];
            }
            "-h" | "--help" => return Err(USAGE.into()),
            other => return Err(format!("unknown argument {other}; {USAGE}")),
        }
    }
    if a.screenshot.is_some() {
        a.windowed = true;
    }
    Ok(a)
}

fn main() {
    let started = Instant::now();
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let choice = graphics::choose(args.backend.as_deref(), args.retest);
    eprintln!("makercad: graphics backend {}", choice.describe());

    // OpenGL resizes badly while the window is being created (Idea Animator's finding), so on
    // OpenGL the window opens normal-sized and is maximized a moment after the first frame.
    let gl = choice.backend == "gl";
    let maximize_now = !args.windowed && !gl;
    let maximize_after = (!args.windowed && gl).then(|| Duration::from_millis(1000));

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MakerCAD")
            .with_app_id("makercad")
            .with_inner_size(args.size)
            .with_min_inner_size([900.0, 600.0])
            .with_maximized(maximize_now),
        // A screenshot run must not move the saved window size or look.
        persist_window: args.screenshot.is_none(),
        wgpu_options: graphics::wgpu_configuration(),
        ..Default::default()
    };
    let launch = Launch {
        started,
        graphics: Some(choice),
        demo: args.demo,
        maximize_after,
        screenshot: args.screenshot,
        look: args.look,
        sample: args.sample,
        fit: args.fit,
    };
    let result = eframe::run_native(
        "MakerCAD",
        options,
        Box::new(move |cc| Ok(Box::new(MakerApp::new(cc, launch)))),
    );
    if let Err(e) = result {
        eprintln!("makercad: the window could not start: {e}");
        eprintln!(
            "makercad: start again and the next graphics backend is tried, or pass --backend gl"
        );
        std::process::exit(1);
    }
}
