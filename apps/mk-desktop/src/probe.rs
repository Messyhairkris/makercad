//! The capability probe's P1 part (doc 12 section 12.2): each graphics backend tried in a child
//! process, so a driver that crashes takes the child with it and never the app, and the result
//! written as the hardware profile (`mk_probe::profile`, Crew-written) beside `graphics.txt`.
//!
//! The app runs it on a background thread the first time it starts without a profile (it never
//! blocks startup; a failed probe means potato), and `graphics::decide` starts the window on the
//! profile's backend when nothing better is known. The viewport and Sancho benchmarks come later:
//! until they run, their times are unknown and every machine classifies as potato, which is the
//! honest answer for a machine nobody has measured.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use eframe::wgpu;
use mk_probe::profile::{self, Attempt, Backend, Facts, Profile};

/// `makercad --probe-backend dx12|gl`: the child's mode.
pub const CHILD_ARG: &str = "--probe-backend";
/// How long one backend may take before its child is stopped and counted as failed.
pub const TIMEOUT: Duration = Duration::from_secs(25);

/// The backend's name as wgpu and `graphics.txt` spell it.
pub fn backend_key(backend: Backend) -> &'static str {
    match backend {
        Backend::Dx12 => "dx12",
        Backend::Gl => "gl",
        Backend::Cpu => "cpu",
    }
}

/// The child: creates an adapter and a device on `backend` alone, prints one line (`ok NAME` or
/// `fail WHY`) and returns the exit code. A driver crash ends the process before it prints.
pub fn child(backend: &str) -> i32 {
    let backends = match backend {
        "dx12" => wgpu::Backends::DX12,
        "gl" => wgpu::Backends::GL,
        "vulkan" => wgpu::Backends::VULKAN,
        "metal" => wgpu::Backends::METAL,
        other => {
            println!("fail unknown backend {other}");
            return 2;
        }
    };
    let mut desc = wgpu::InstanceDescriptor::new_without_display_handle();
    desc.backends = backends;
    // The same D3D12 shader compiler as the window (graphics::wgpu_configuration).
    desc.backend_options.dx12.shader_compiler = solidcraft_viewport_gpu::dx12_compiler();
    let instance = wgpu::Instance::new(desc);
    let options = wgpu::RequestAdapterOptions::default();
    let adapter = match pollster::block_on(instance.request_adapter(&options)) {
        Ok(adapter) => adapter,
        Err(e) => {
            println!("fail no adapter: {e}");
            return 1;
        }
    };
    let info = adapter.get_info();
    match pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())) {
        Ok(_) => {
            println!("ok {} ({:?})", info.name, info.device_type);
            0
        }
        Err(e) => {
            println!("fail no device on {}: {e}", info.name);
            1
        }
    }
}

/// What one child said: the attempt, and its line (or why there was none).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tried {
    pub attempt: Attempt,
    pub said: String,
}

/// Reads a child's first line: `ok ...` is a backend that initialised; anything else is not.
pub fn read_line(backend: Backend, exit_ok: bool, output: &str) -> Tried {
    let line = output.lines().next().unwrap_or("").trim().to_string();
    let initialised = exit_ok && line.starts_with("ok ");
    let said = if line.is_empty() {
        "no answer (the driver may have crashed the probe)".to_string()
    } else {
        line
    };
    Tried {
        attempt: Attempt {
            backend,
            initialised,
        },
        said,
    }
}

/// Runs `exe --probe-backend NAME` for one backend, stopping it after [`TIMEOUT`].
fn try_backend(exe: &Path, backend: Backend) -> Tried {
    let mut command = Command::new(exe);
    command
        .arg(CHILD_ARG)
        .arg(backend_key(backend))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // CREATE_NO_WINDOW: no console flashes up for the child.
        command.creation_flags(0x0800_0000);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(e) => return read_line(backend, false, &format!("could not start the probe: {e}")),
    };
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() < TIMEOUT => {
                std::thread::sleep(Duration::from_millis(100))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
        }
    };
    let mut output = String::new();
    if let Some(mut out) = child.stdout.take() {
        let _ = out.read_to_string(&mut output);
    }
    match status {
        Some(status) => read_line(backend, status.success(), &output),
        None => read_line(backend, false, "fail timed out"),
    }
}

/// Logical cores and total RAM in MB, as the OS reports them (0 when it does not say).
fn machine() -> (u32, u64) {
    let cores = std::thread::available_parallelism()
        .map(|n| u32::try_from(n.get()).unwrap_or(u32::MAX))
        .unwrap_or(0);
    (cores, total_ram_mb())
}

#[cfg(windows)]
fn total_ram_mb() -> u64 {
    #[repr(C)]
    struct MemoryStatusEx {
        length: u32,
        memory_load: u32,
        total_phys: u64,
        avail_phys: u64,
        total_page_file: u64,
        avail_page_file: u64,
        total_virtual: u64,
        avail_virtual: u64,
        avail_extended_virtual: u64,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalMemoryStatusEx(buffer: *mut MemoryStatusEx) -> i32;
    }
    let mut status = MemoryStatusEx {
        length: std::mem::size_of::<MemoryStatusEx>() as u32,
        memory_load: 0,
        total_phys: 0,
        avail_phys: 0,
        total_page_file: 0,
        avail_page_file: 0,
        total_virtual: 0,
        avail_virtual: 0,
        avail_extended_virtual: 0,
    };
    // SAFETY: the buffer is a properly sized MEMORYSTATUSEX with its length set, as the API asks.
    let ok = unsafe { GlobalMemoryStatusEx(&mut status) };
    if ok != 0 {
        status.total_phys / (1024 * 1024)
    } else {
        0
    }
}

#[cfg(not(windows))]
fn total_ram_mb() -> u64 {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|kb| kb.parse::<u64>().ok())
        })
        .map_or(0, |kb| kb / 1024)
}

/// Tries every backend of `mk_probe`'s order and gathers this machine's facts. The viewport and
/// Sancho benchmarks are not built yet, so their times are unknown (NaN).
pub fn measure(exe: &Path) -> (Facts, Vec<Tried>) {
    let tried: Vec<Tried> = profile::BACKEND_ORDER
        .iter()
        .map(|&b| try_backend(exe, b))
        .collect();
    let (cores, ram_total_mb) = machine();
    let facts = Facts {
        attempts: tried.iter().map(|t| t.attempt).collect(),
        frame_p95_ms: f64::NAN,
        sancho_vat_ms: f64::NAN,
        cores,
        ram_total_mb,
        vram_mb: None,
    };
    (facts, tried)
}

/// `%LOCALAPPDATA%\MakerCAD\profile.toml` (beside `graphics.txt`).
pub fn profile_path() -> Option<PathBuf> {
    crate::graphics::memory_path().map(|p| p.with_file_name("profile.toml"))
}

/// The saved profile, if there is one (a damaged file still gives a whole profile).
pub fn load() -> Option<Profile> {
    let text = std::fs::read_to_string(profile_path()?).ok()?;
    Some(profile::parse(&text).profile)
}

/// Runs the probe and writes the profile, with what each backend said as comments.
pub fn run_and_save(exe: &Path) -> Result<Profile, String> {
    let (facts, tried) = measure(exe);
    let profile = profile::from_facts(Some(&facts));
    let path = profile_path().ok_or("no folder for the profile")?;
    let mut text = String::from(
        "# MakerCAD's hardware profile (doc 12 section 12.2), written by the probe.\n",
    );
    for t in &tried {
        text.push_str(&format!(
            "# {}: {}\n",
            backend_key(t.attempt.backend),
            t.said
        ));
    }
    text.push_str(&format!(
        "# {} cores, {} MB RAM; the viewport and Sancho benchmarks are not run yet.\n",
        facts.cores, facts.ram_total_mb
    ));
    text.push_str(&profile::to_toml(&profile));
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(profile)
}

/// Starts the probe on a background thread when there is no profile yet (first start). Returns
/// whether it started.
pub fn start_if_needed() -> bool {
    if profile_path().is_none_or(|p| p.exists()) {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    std::thread::Builder::new()
        .name("probe".into())
        .spawn(move || match run_and_save(&exe) {
            Ok(p) => eprintln!(
                "makercad: probe: {:?}, backend {}",
                p.class,
                backend_key(p.backend)
            ),
            Err(e) => eprintln!("makercad: probe: {e}"),
        })
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_child_that_says_ok_and_exits_cleanly_initialised() {
        let t = read_line(
            Backend::Dx12,
            true,
            "ok Intel(R) HD Graphics 530 (IntegratedGpu)\n",
        );
        assert!(t.attempt.initialised);
        assert_eq!(t.said, "ok Intel(R) HD Graphics 530 (IntegratedGpu)");
    }

    #[test]
    fn a_crash_a_failure_or_silence_did_not() {
        assert!(
            !read_line(Backend::Dx12, false, "ok X\n")
                .attempt
                .initialised
        );
        assert!(
            !read_line(Backend::Gl, true, "fail no adapter: none\n")
                .attempt
                .initialised
        );
        let silent = read_line(Backend::Gl, false, "");
        assert!(!silent.attempt.initialised);
        assert!(silent.said.contains("crashed"));
    }

    #[test]
    fn the_backends_are_spelled_as_wgpu_and_graphics_txt_spell_them() {
        assert_eq!(backend_key(Backend::Dx12), "dx12");
        assert_eq!(backend_key(Backend::Gl), "gl");
        assert!(crate::graphics::ORDER.contains(&backend_key(profile::BACKEND_ORDER[0])));
    }

    #[test]
    fn an_unknown_backend_is_refused_without_touching_the_gpu() {
        assert_eq!(child("vulkan-please"), 2);
    }

    #[test]
    fn this_machine_reports_cores_and_memory() {
        let (cores, ram) = machine();
        assert!(cores >= 1);
        assert!(ram >= 256, "{ram} MB");
    }

    #[test]
    fn unmeasured_benchmarks_classify_as_potato_on_a_working_backend() {
        let facts = Facts {
            attempts: vec![Attempt {
                backend: Backend::Dx12,
                initialised: true,
            }],
            frame_p95_ms: f64::NAN,
            sancho_vat_ms: f64::NAN,
            cores: 16,
            ram_total_mb: 65_536,
            vram_mb: None,
        };
        let p = profile::from_facts(Some(&facts));
        assert_eq!(p.backend, Backend::Dx12);
        assert_eq!(p.class, profile::Class::Potato);
    }
}
