//! Writes Sancho as a `.glb`:  cargo run -p mk-sancho --example write_sancho -- sancho.glb [height_m]

fn main() {
    let mut args = std::env::args().skip(1);
    let out = args.next().unwrap_or_else(|| "sancho.glb".to_string());
    let height = args
        .next()
        .and_then(|h| h.parse::<f32>().ok())
        .unwrap_or(mk_sancho::DEFAULT_HEIGHT_M);
    let bytes = mk_sancho::write_glb(height);
    match std::fs::write(&out, &bytes) {
        Ok(()) => println!("wrote {out}: {} bytes, {height} m tall", bytes.len()),
        Err(e) => {
            eprintln!("{out}: {e}");
            std::process::exit(1);
        }
    }
}
