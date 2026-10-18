use crate::domain::{error::RuntimeError, resource::SystemMemory};

/// Parses raw `/proc/meminfo` contents into structured `SystemMemory`.
/// Supports kernels with and without `MemAvailable` (Linux 3.14+).
pub fn parse_meminfo(content: &str) -> Result<SystemMemory, RuntimeError> {
    let mut mem_total_kb: Option<u64> = None;
    let mut mem_free_kb: Option<u64> = None;
    let mut mem_available_kb: Option<u64> = None;
    let mut buffers_kb: Option<u64> = None;
    let mut cached_kb: Option<u64> = None;
    let mut swap_total_kb: Option<u64> = None;
    let mut swap_free_kb: Option<u64> = None;

    for line in content.lines() {
        let mut parts = line.splitn(2, ':');
        let key = match parts.next() {
            Some(k) => k.trim(),
            None => continue,
        };
        let val_str = match parts.next() {
            Some(v) => v.trim(),
            None => continue,
        };

        // Extract numerical value, ignoring trailing "kB" if present
        let num_str = val_str.split_whitespace().next().unwrap_or("0");
        let val_num: u64 = match num_str.parse() {
            Ok(n) => n,
            Err(_) => continue,
        };

        match key {
            "MemTotal" => mem_total_kb = Some(val_num),
            "MemFree" => mem_free_kb = Some(val_num),
            "MemAvailable" => mem_available_kb = Some(val_num),
            "Buffers" => buffers_kb = Some(val_num),
            "Cached" => cached_kb = Some(val_num),
            "SwapTotal" => swap_total_kb = Some(val_num),
            "SwapFree" => swap_free_kb = Some(val_num),
            _ => {}
        }
    }

    let total_kb = mem_total_kb.ok_or_else(|| {
        RuntimeError::InvalidMetadata("MemTotal missing from /proc/meminfo".to_string())
    })?;

    if total_kb == 0 {
        return Err(RuntimeError::InvalidMetadata(
            "MemTotal in /proc/meminfo cannot be zero".to_string(),
        ));
    }

    // MemAvailable fallback: MemFree + Buffers + Cached (pre-3.14 kernels)
    let available_kb = mem_available_kb.unwrap_or_else(|| {
        let free = mem_free_kb.unwrap_or(0);
        let buf = buffers_kb.unwrap_or(0);
        let cache = cached_kb.unwrap_or(0);
        free.saturating_add(buf).saturating_add(cache)
    });

    let total_bytes = total_kb.saturating_mul(1024);
    let available_bytes = available_kb.saturating_mul(1024).min(total_bytes);
    let used_bytes = total_bytes.saturating_sub(available_bytes);
    let used_percentage = if total_bytes > 0 {
        ((used_bytes as f64 / total_bytes as f64) * 100.0)
            .round()
            .min(100.0) as u8
    } else {
        0
    };

    let swap_total_bytes = swap_total_kb.unwrap_or(0).saturating_mul(1024);
    let swap_free_bytes = swap_free_kb
        .unwrap_or(0)
        .saturating_mul(1024)
        .min(swap_total_bytes);
    let swap_used_bytes = swap_total_bytes.saturating_sub(swap_free_bytes);

    Ok(SystemMemory {
        total_bytes,
        available_bytes,
        used_bytes,
        used_percentage,
        swap_total_bytes,
        swap_free_bytes,
        swap_used_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_meminfo_standard_linux() {
        let sample = "\
MemTotal:        4000000 kB
MemFree:          500000 kB
MemAvailable:    2500000 kB
Buffers:          100000 kB
Cached:          1900000 kB
SwapTotal:       2000000 kB
SwapFree:        1500000 kB
";
        let res = parse_meminfo(sample).expect("Should parse standard meminfo");
        assert_eq!(res.total_bytes, 4000000 * 1024);
        assert_eq!(res.available_bytes, 2500000 * 1024);
        assert_eq!(res.used_bytes, 1500000 * 1024);
        assert_eq!(res.used_percentage, 38);
        assert_eq!(res.swap_total_bytes, 2000000 * 1024);
        assert_eq!(res.swap_free_bytes, 1500000 * 1024);
        assert_eq!(res.swap_used_bytes, 500000 * 1024);
    }

    #[test]
    fn test_parse_meminfo_fallback_when_available_absent() {
        let sample = "\
MemTotal:        2000000 kB
MemFree:          200000 kB
Buffers:          100000 kB
Cached:           300000 kB
";
        let res = parse_meminfo(sample).expect("Should compute fallback available");
        assert_eq!(res.total_bytes, 2000000 * 1024);
        // Fallback: 200000 + 100000 + 300000 = 600000 kB
        assert_eq!(res.available_bytes, 600000 * 1024);
        assert_eq!(res.used_bytes, 1400000 * 1024);
        assert_eq!(res.used_percentage, 70);
    }

    #[test]
    fn test_parse_meminfo_missing_total_fails() {
        let sample = "\
MemFree:          500000 kB
MemAvailable:    2500000 kB
";
        let res = parse_meminfo(sample);
        assert!(matches!(res, Err(RuntimeError::InvalidMetadata(_))));
    }

    #[test]
    fn test_parse_meminfo_zero_total_fails() {
        let sample = "\
MemTotal:              0 kB
MemAvailable:          0 kB
";
        let res = parse_meminfo(sample);
        assert!(matches!(res, Err(RuntimeError::InvalidMetadata(_))));
    }

    #[test]
    fn test_parse_meminfo_malformed_lines_tolerated() {
        let sample = "\
Random text line here
MemTotal:        1000000 kB
InvalidKeyNoColon
MemAvailable:    bad_number kB
MemFree:          400000 kB
Buffers:          100000 kB
Cached:           100000 kB
";
        let res = parse_meminfo(sample).expect("Should tolerate malformed lines");
        assert_eq!(res.total_bytes, 1000000 * 1024);
        // Available was bad_number, fallback to free (400000) + buffers (100000) + cached (100000) = 600000
        assert_eq!(res.available_bytes, 600000 * 1024);
    }
}
