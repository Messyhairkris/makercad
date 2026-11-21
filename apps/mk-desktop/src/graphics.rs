//! Which graphics backend to start on, and remembering the one that worked.
//!
//! MakerCAD v4 and v5 crashed on the reference machine's Intel HD 530 when the driver's
//! default path failed, and nothing remembered it. Idea Animator's answer (doc 16 §16.10) was
//! a launcher that falls back and remembers. This is the same idea inside the app: before the
//! window opens we write `trying <backend>`; after the first frame is drawn we write
//! `ok <backend>`. A start that finds `trying` left behind knows the last start never drew a
//! frame (a crash or a hang) and moves on to the next backend in [`ORDER`].
//!
//! The file is `%LOCALAPPDATA%\MakerCAD\graphics.txt` (or `$XDG_STATE_HOME`/`~/.local/state`
//! elsewhere). `--retest` forgets it. `--backend <name>` or `WGPU_BACKEND` always win and are
//! not remembered.
//!
//! [`wgpu_configuration`] also fixes D3D12's shader compiler for the window, as SolidCraft's
//! viewport expects it.

use std::path::PathBuf;

use eframe::egui_wgpu::{WgpuConfiguration, WgpuSetup};

/// The order backends are tried in on this platform, best first. Windows goes from D3D12
/// straight to OpenGL: Vulkan crashed the reference machine's Intel HD 530 driver (MakerCAD v5,
/// and SolidCraft in P0), so it is used only when asked for (`--backend vulkan`).
#[cfg(target_os = "windows")]
pub const ORDER: &[&str] = &["dx12", "gl"];
/// The order backends are tried in on this platform, best first.
#[cfg(target_os = "macos")]
pub const ORDER: &[&str] = &["metal"];
/// The order backends are tried in on this platform, best first.
#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub const ORDER: &[&str] = &["vulkan", "gl"];

/// The environment variable wgpu reads its backend from.
pub const BACKEND_ENV: &str = "WGPU_BACKEND";

/// Where a choice came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// `--backend` on the command line.
    Argument,
    /// The `WGPU_BACKEND` environment variable.
    Environment,
    /// The last start drew a frame on it.
    Remembered,
    /// The last start on the previous backend never drew a frame.
    FellBack,
    /// Nothing known yet: the first backend in [`ORDER`].
    FirstTry,
}

/// The backend to start on, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    /// A wgpu backend name: `dx12`, `vulkan`, `gl` or `metal`.
    pub backend: String,
    /// Where the choice came from.
    pub source: Source,
    /// The backend whose last start never drew a frame, if that is why we moved on.
    pub failed: Option<String>,
}

impl Choice {
    /// Whether the result of this start should be written to the memory file.
    pub fn remembered(&self) -> bool {
        !matches!(self.source, Source::Argument | Source::Environment)
    }

    /// One line for the status bar and the log.
    pub fn describe(&self) -> String {
        match (&self.source, &self.failed) {
            (Source::FellBack, Some(f)) => {
                format!(
                    "{} (the last start on {f} never drew a frame)",
                    self.backend
                )
            }
            (Source::Argument, _) => format!("{} (--backend)", self.backend),
            (Source::Environment, _) => format!("{} ({BACKEND_ENV})", self.backend),
            (Source::Remembered, _) => format!("{} (worked last time)", self.backend),
            _ => self.backend.clone(),
        }
    }
}

/// Decides the backend from the argument, the environment and the remembered `memory` text.
/// Pure, so it can be tested; [`choose`] does the file work around it.
pub fn decide(argument: Option<&str>, environment: Option<&str>, memory: Option<&str>) -> Choice {
    let clean = |s: &str| s.trim().to_ascii_lowercase();
    if let Some(a) = argument.map(clean).filter(|a| !a.is_empty()) {
        return Choice {
            backend: a,
            source: Source::Argument,
            failed: None,
        };
    }
    if let Some(e) = environment.map(clean).filter(|e| !e.is_empty()) {
        return Choice {
            backend: e,
            source: Source::Environment,
            failed: None,
        };
    }
    let first = ORDER.first().copied().unwrap_or("gl").to_string();
    let Some(memory) = memory else {
        return Choice {
            backend: first,
            source: Source::FirstTry,
            failed: None,
        };
    };
    let mut words = memory.split_whitespace();
    match (words.next(), words.next().map(clean)) {
        (Some("ok"), Some(b)) if ORDER.contains(&b.as_str()) => Choice {
            backend: b,
            source: Source::Remembered,
            failed: None,
        },
        (Some("trying"), Some(b)) => {
            let next = ORDER
                .iter()
                .position(|o| *o == b)
                .and_then(|i| ORDER.get(i + 1))
                .or(ORDER.last())
                .copied()
                .unwrap_or("gl");
            Choice {
                backend: next.to_string(),
                source: Source::FellBack,
                failed: Some(b),
            }
        }
        _ => Choice {
            backend: first,
            source: Source::FirstTry,
            failed: None,
        },
    }
}

/// The memory file, if the platform has a place for it.
pub fn memory_path() -> Option<PathBuf> {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))
    }?;
    Some(base.join("MakerCAD").join("graphics.txt"))
}

/// Decides the backend for this start, records `trying`, and sets `WGPU_BACKEND` for wgpu.
pub fn choose(argument: Option<&str>, retest: bool) -> Choice {
    let path = memory_path();
    if retest {
        if let Some(p) = &path {
            let _ = std::fs::remove_file(p);
        }
    }
    let memory = path.as_ref().and_then(|p| std::fs::read_to_string(p).ok());
    let environment = std::env::var(BACKEND_ENV).ok();
    let choice = decide(argument, environment.as_deref(), memory.as_deref());
    if choice.remembered() {
        write_memory(&format!("trying {}", choice.backend));
    }
    // Single-threaded here: nothing else is running yet.
    std::env::set_var(BACKEND_ENV, &choice.backend);
    choice
}

/// Records that `choice` drew a frame.
pub fn mark_ok(choice: &Choice) {
    if choice.remembered() {
        write_memory(&format!("ok {}", choice.backend));
    }
}

/// eframe's wgpu settings for the window: its defaults, with D3D12's shader compiler chosen the
/// way SolidCraft's viewport chooses it (`solidcraft_viewport_gpu::dx12_compiler`): FXC, or DXC
/// only from beside the program, never a `dxcompiler.dll` that another program put on `PATH`.
/// wgpu's own default loads one by name. On the reference machine that was Obsidian's DXC
/// 1.8.2505, which crashed (an illegal instruction inside `dxcompiler.dll`) compiling the
/// viewport's shaders the moment a design opened, 2026-11-20. The viewport also builds its
/// pipelines on several threads only when the compiler is FXC, so the two must agree.
pub fn wgpu_configuration() -> WgpuConfiguration {
    let mut config = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup {
        setup
            .instance_descriptor
            .backend_options
            .dx12
            .shader_compiler = solidcraft_viewport_gpu::dx12_compiler();
    }
    config
}

fn write_memory(text: &str) {
    if let Some(p) = memory_path() {
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(p, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_argument_beats_everything() {
        let c = decide(Some(" GL "), Some("vulkan"), Some("ok dx12"));
        assert_eq!((c.backend.as_str(), c.source), ("gl", Source::Argument));
        assert!(!c.remembered());
    }

    #[test]
    fn the_environment_beats_the_memory() {
        let c = decide(None, Some("vulkan"), Some("ok dx12"));
        assert_eq!(
            (c.backend.as_str(), c.source),
            ("vulkan", Source::Environment)
        );
    }

    #[test]
    fn nothing_known_starts_at_the_top() {
        let c = decide(None, None, None);
        assert_eq!(c.backend, ORDER[0]);
        assert_eq!(c.source, Source::FirstTry);
        assert!(c.remembered());
    }

    #[test]
    fn a_backend_that_drew_a_frame_is_used_again() {
        let last = ORDER[ORDER.len() - 1];
        let c = decide(None, None, Some(&format!("ok {last}")));
        assert_eq!((c.backend.as_str(), c.source), (last, Source::Remembered));
    }

    #[test]
    fn a_start_that_never_drew_moves_to_the_next_backend() {
        let c = decide(None, None, Some(&format!("trying {}", ORDER[0])));
        let want = ORDER.get(1).copied().unwrap_or(ORDER[0]);
        assert_eq!(c.backend, want);
        assert_eq!(c.source, Source::FellBack);
        assert_eq!(c.failed.as_deref(), Some(ORDER[0]));
    }

    #[test]
    fn the_last_backend_failing_stays_on_the_last() {
        let last = ORDER[ORDER.len() - 1];
        let c = decide(None, None, Some(&format!("trying {last}")));
        assert_eq!(c.backend, last);
    }

    #[test]
    fn the_window_uses_the_viewports_d3d12_compiler_never_one_on_path() {
        // Regression (2026-11-20): wgpu's default loaded Obsidian's dxcompiler.dll from PATH,
        // which crashed compiling the viewport's shaders.
        let config = wgpu_configuration();
        let WgpuSetup::CreateNew(setup) = &config.wgpu_setup else {
            panic!("the window creates its own wgpu instance");
        };
        let ours = format!(
            "{:?}",
            setup
                .instance_descriptor
                .backend_options
                .dx12
                .shader_compiler
        );
        assert_eq!(
            ours,
            format!("{:?}", solidcraft_viewport_gpu::dx12_compiler())
        );
        assert!(
            !ours.contains("\"dxcompiler.dll\""),
            "a DXC loaded by name from PATH: {ours}"
        );
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn windows_never_falls_back_to_vulkan() {
        assert!(!ORDER.contains(&"vulkan"));
        let c = decide(None, None, Some("trying dx12"));
        assert_eq!(c.backend, "gl");
        // Asked for by name, it is still used.
        assert_eq!(decide(Some("vulkan"), None, None).backend, "vulkan");
    }

    #[test]
    fn junk_in_the_memory_file_starts_over() {
        for junk in ["", "ok", "ok teapot", "hello world"] {
            let c = decide(None, None, Some(junk));
            assert_eq!(
                (c.backend.as_str(), c.source),
                (ORDER[0], Source::FirstTry),
                "{junk:?}"
            );
        }
    }
}
