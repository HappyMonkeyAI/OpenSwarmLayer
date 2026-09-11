use std::io::Write;

fn main() {
    for seed in 0_u16..512 {
        let mut state = u32::from(seed).wrapping_add(1);
        let mut data = [0_u8; 96];
        for byte in &mut data {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            *byte = (state >> 24) as u8;
        }
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let Ok(mut file) = tempfile::NamedTempFile::new() else {
                return;
            };
            let _ = file.write_all(&data);
            let _ = ts_format::inspect(file.path());
        }))
        .expect("format parser panicked on fallback fuzz input");
    }
    println!("format_inspect_smoke: 512 bounded malformed inputs passed");
}
