#![no_main]

use libfuzzer_sys::fuzz_target;
use std::io::Write;

fn bounded_input(data: &[u8]) -> &[u8] {
    const MAX_FUZZ_INPUT: usize = 4 * 1024 * 1024;
    &data[..data.len().min(MAX_FUZZ_INPUT)]
}

fuzz_target!(|data: &[u8]| {
    let data = bounded_input(data);
    let Ok(mut file) = tempfile::NamedTempFile::new() else {
        return;
    };
    if file.write_all(data).is_err() {
        return;
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        ts_format::inspect(file.path())
    }));
    assert!(result.is_ok(), "format parser panicked on fuzz input");
});

#[cfg(test)]
mod tests {
    use super::bounded_input;

    #[test]
    fn fuzz_input_is_bounded() {
        let input = vec![0_u8; 4 * 1024 * 1024 + 1];
        assert_eq!(bounded_input(&input).len(), 4 * 1024 * 1024);
    }
}
