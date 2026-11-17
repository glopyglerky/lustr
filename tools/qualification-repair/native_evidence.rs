//! Interpret the pinned Lust diagnostic records, not substrings in arbitrary output.
//! A retained no-body-work route still does collection; this is not a timing claim.
use std::collections::BTreeMap;

type Record<'a> = BTreeMap<&'a str, &'a str>;

fn record<'a>(text: &'a str, prefix: &str) -> Result<Record<'a>, &'static str> {
    let mut lines = text.lines().filter_map(|line| line.strip_prefix(prefix));
    let line = lines.next().ok_or("missing diagnostic record")?;
    if lines.next().is_some() {
        return Err("multiple diagnostic records for one request");
    }
    let mut fields = BTreeMap::new();
    for word in line.split_whitespace() {
        let (key, value) = word.split_once('=').ok_or("malformed diagnostic field")?;
        if key.is_empty() || value.is_empty() || fields.insert(key, value).is_some() {
            return Err("empty or duplicate diagnostic field");
        }
    }
    Ok(fields)
}

fn number(fields: &Record<'_>, name: &str) -> Result<u64, &'static str> {
    let value = fields.get(name).ok_or("missing work counter")?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return Err("invalid work counter");
    }
    value.parse().map_err(|_| "work counter overflow")
}

/// Require a known retained route and no repeated body/code-generation work.
/// Preserve `bodies` as a distinct route: its collection work remains measured.
pub fn retained_check(text: &str) -> Result<&'static str, &'static str> {
    let stats = record(text, "lust-stats: ")?;
    let visits = record(text, "lust-visits: ")?;
    if number(&stats, "fns")? == 0 {
        return Err("empty fixture cannot establish retained work");
    }
    for key in ["rechecked", "invalidated", "c_bytes", "dirty_units", "units_compiled", "reemitted", "discovered"] {
        if number(&stats, key)? != 0 {
            return Err("request repeated body or generation work");
        }
    }
    if number(&visits, "checked")? != 0 {
        return Err("request checked a body");
    }
    match (stats.get("mode").copied(), visits.get("path").copied(), visits.get("why").copied()) {
        (Some("reuse"), Some("reuse"), _) => Ok("reuse"),
        (Some("bodies"), Some("chunks" | "patch"), Some("bodies")) => Ok("bodies-without-body-work"),
        _ => Err("unrecognized or contradictory retained route"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const OBSERVED: &str = concat!(
        "lust-stats: mode=bodies fns=2 rechecked=0 reused=0 invalidated=0 c_bytes=0 dirty_units=0 units_compiled=0 unit_cache_hits=0 table_rebuilt=0 instances=2 reemitted=0 templates=0 discovered=0\n",
        "lust-visits: path=chunks why=bodies files=1 nodes=0 spans=0 scanned=0 slot_fns=0 collect=4 discovery=0 checked=0 kept=none prog_spans=0 reads=0 keyed=0 generic=on pool=308/308\n",
        "lust check fixture.rs: ok (2 functions, bodies: 0 rechecked, 0 reused by reads)\n  collect 1.31 ms\n"
    );
    #[test]
    fn observed_pristine_zero_body_route_is_retained_not_zero_total_work() {
        assert_eq!(retained_check(OBSERVED), Ok("bodies-without-body-work"));
        assert!(OBSERVED.contains("collect=4"));
    }
    #[test]
    fn explicit_reuse_remains_distinct() {
        let text = OBSERVED.replace("mode=bodies", "mode=reuse").replace("path=chunks", "path=reuse");
        assert_eq!(retained_check(&text), Ok("reuse"));
    }
    #[test]
    fn all_nonzero_work_counters_reject() {
        for key in ["rechecked", "invalidated", "c_bytes", "dirty_units", "units_compiled", "reemitted", "discovered", "checked"] {
            let old = format!(" {key}=0 ");
            assert!(OBSERVED.contains(&old), "fixture lacks {key}");
            assert!(retained_check(&OBSERVED.replace(&old, &format!(" {key}=1 "))).is_err(), "{key}");
        }
    }
    #[test]
    fn missing_work_counter_rejects() {
        for key in ["fns=2", "rechecked=0", "units_compiled=0", "checked=0"] {
            assert!(retained_check(&OBSERVED.replace(&format!(" {key} "), " ")).is_err(), "{key}");
        }
    }
    #[test]
    fn full_route_is_not_accepted_with_zero_counters() {
        assert!(retained_check(&OBSERVED.replace("mode=bodies", "mode=full")).is_err());
        assert!(retained_check(&OBSERVED.replace("why=bodies", "why=full")).is_err());
    }
    #[test]
    fn contradictory_reuse_route_rejects() {
        assert!(retained_check(&OBSERVED.replace("path=chunks", "path=reuse")).is_err());
        assert!(retained_check(&OBSERVED.replace("mode=bodies", "mode=reuse")).is_err());
    }
    #[test]
    fn duplicate_record_rejects() {
        assert!(retained_check(&format!("{OBSERVED}{OBSERVED}")).is_err());
    }
    #[test]
    fn duplicate_counter_rejects_even_when_equal() {
        assert!(retained_check(&OBSERVED.replace("rechecked=0", "rechecked=0 rechecked=0")).is_err());
    }
    #[test]
    fn malformed_counter_rejects() {
        for bad in ["-0", "0x0", "0oops", "18446744073709551616"] {
            assert!(retained_check(&OBSERVED.replace("rechecked=0", &format!("rechecked={bad}"))).is_err());
        }
    }
    #[test]
    fn unrelated_text_cannot_supply_evidence() {
        assert!(retained_check("log says mode=reuse path=reuse rechecked=0 checked=0").is_err());
        assert!(retained_check(&OBSERVED.replace("lust-stats:", "fake-lust-stats:")).is_err());
    }
    #[test]
    fn empty_fixture_rejects() {
        assert!(retained_check(&OBSERVED.replace("fns=2", "fns=0")).is_err());
    }
    #[test]
    fn nonzero_checked_is_not_confused_with_rechecked() {
        assert!(retained_check(&OBSERVED.replace(" checked=0 ", " checked=7 ")).is_err());
    }
}
