use crate::parser::{Ecosystem, ParsedVersion, Seg, parse_generic, strip_v, tag_weight};
use std::cmp::Ordering;

pub(crate) fn normalized(segs: &[Seg]) -> &[Seg] {
    let mut end = segs.len();
    while end > 0 {
        if let Seg::Num(0) = segs[end - 1] {
            end -= 1;
        } else {
            break;
        }
    }
    &segs[..end]
}

pub(crate) fn cmp_two(a: &Seg, b: &Seg) -> Ordering {
    match (a, b) {
        (Seg::Num(x), Seg::Num(y)) => x.cmp(y),
        (Seg::Text(x), Seg::Text(y)) => {
            let wx = tag_weight(x);
            let wy = tag_weight(y);
            match (wx, wy) {
                (Some(wa), Some(wb)) => wa.cmp(&wb),
                (Some(wa), None) => wa.cmp(&29),
                (None, Some(wb)) => 29_i32.cmp(&wb),
                (None, None) => x.cmp(y),
            }
        }
        (Seg::Num(_), Seg::Text(_)) => Ordering::Greater,
        (Seg::Text(_), Seg::Num(_)) => Ordering::Less,
    }
}

pub(crate) fn text_effective_weight(s: &str) -> i32 {
    tag_weight(s).unwrap_or(29)
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn cmp_segments(left: &[Seg], right: &[Seg]) -> Ordering {
    // Normalize: strip trailing Num(0) to ensure transitivity
    // (e.g., 1.0 == 1.0.0 and 1.0 < 1.0.post1 implies 1.0.0 < 1.0.post1)
    let nl = normalized(left);
    let nr = normalized(right);
    let n = nl.len().max(nr.len());
    for i in 0..n {
        match (nl.get(i), nr.get(i)) {
            (None, None) => return Ordering::Equal,
            (None, Some(Seg::Num(v))) => {
                if *v == 0 {
                    continue;
                } // non-trailing zero before text
                return Ordering::Less;
            }
            (Some(Seg::Num(v)), None) => {
                if *v == 0 {
                    continue;
                }
                return Ordering::Greater;
            }
            (None, Some(Seg::Text(tag))) => {
                if text_effective_weight(tag) < 30 {
                    return Ordering::Greater;
                } // we are release, they pre
                return Ordering::Less; // they are post
            }
            (Some(Seg::Text(tag)), None) => {
                if text_effective_weight(tag) < 30 {
                    return Ordering::Less;
                } // we are pre
                return Ordering::Greater; // we are post
            }
            (Some(sa), Some(sb)) => {
                let ord = cmp_two(sa, sb);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
    Ordering::Equal
}

pub(crate) fn cmp_parsed(a: &ParsedVersion, b: &ParsedVersion) -> Ordering {
    a.epoch.cmp(&b.epoch).then_with(|| cmp_segments(&a.segments, &b.segments))
}

#[derive(Debug, Clone)]
pub(crate) enum ParsedRepr {
    Generic(ParsedVersion),
    Semver(SemVer),
    Debian(DebianVersion),
    Rpm(RpmVersion),
}

pub(crate) trait VersionStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String>;
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering;
}

pub(crate) struct GenericStrategy;
pub(crate) struct SemverStrategy;
pub(crate) struct Pep440Strategy;
pub(crate) struct DebianStrategy;
pub(crate) struct RpmStrategy;
pub(crate) struct RubyStrategy;
pub(crate) struct MavenStrategy;
pub(crate) struct GoStrategy;
pub(crate) struct NugetStrategy;
pub(crate) struct ComposerStrategy;
pub(crate) struct CalverStrategy;
pub(crate) struct AlpineStrategy;

impl VersionStrategy for GenericStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("generic strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for SemverStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        parse_semver_strict(input).map(ParsedRepr::Semver)
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Semver(a), ParsedRepr::Semver(b)) => cmp_semver_strict(a, b),
            _ => unreachable!("semver strategy received non-semver parsed values"),
        }
    }
}

impl VersionStrategy for Pep440Strategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_pep440(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("pep440 strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for DebianStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        parse_debian(input).map(ParsedRepr::Debian)
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Debian(a), ParsedRepr::Debian(b)) => cmp_debian(a, b),
            _ => unreachable!("debian strategy received non-debian parsed values"),
        }
    }
}

impl VersionStrategy for RpmStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        parse_rpm(input).map(ParsedRepr::Rpm)
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Rpm(a), ParsedRepr::Rpm(b)) => cmp_rpm(a, b),
            _ => unreachable!("rpm strategy received non-rpm parsed values"),
        }
    }
}

impl VersionStrategy for RubyStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_ruby(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("ruby strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for MavenStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_maven(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("maven strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for GoStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        // Go modules use SemVer with optional v-prefix (strip_v handled in parse_semver_strict)
        parse_semver_strict(input).map(ParsedRepr::Semver)
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Semver(a), ParsedRepr::Semver(b)) => cmp_semver_strict(a, b),
            _ => unreachable!("go strategy received non-semver parsed values"),
        }
    }
}

static GENERIC_STRATEGY: GenericStrategy = GenericStrategy;
static SEMVER_STRATEGY: SemverStrategy = SemverStrategy;
static PEP440_STRATEGY: Pep440Strategy = Pep440Strategy;
static DEBIAN_STRATEGY: DebianStrategy = DebianStrategy;
static RPM_STRATEGY: RpmStrategy = RpmStrategy;
static RUBY_STRATEGY: RubyStrategy = RubyStrategy;
static MAVEN_STRATEGY: MavenStrategy = MavenStrategy;
static GO_STRATEGY: GoStrategy = GoStrategy;
static NUGET_STRATEGY: NugetStrategy = NugetStrategy;
static COMPOSER_STRATEGY: ComposerStrategy = ComposerStrategy;
static CALVER_STRATEGY: CalverStrategy = CalverStrategy;
static ALPINE_STRATEGY: AlpineStrategy = AlpineStrategy;

impl VersionStrategy for NugetStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_nuget(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("nuget strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for ComposerStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_composer(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("composer strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for CalverStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_calver(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("calver strategy received non-generic parsed values"),
        }
    }
}

impl VersionStrategy for AlpineStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_alpine(input)?;
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }

    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!("alpine strategy received non-generic parsed values"),
        }
    }
}

pub(crate) fn strategy_for(ecosystem: Ecosystem) -> &'static dyn VersionStrategy {
    match ecosystem {
        Ecosystem::Generic | Ecosystem::Docker => &GENERIC_STRATEGY,
        Ecosystem::Semver
        | Ecosystem::Npm
        | Ecosystem::Crates
        | Ecosystem::Hex
        | Ecosystem::Swift => &SEMVER_STRATEGY,
        Ecosystem::Pep440 => &PEP440_STRATEGY,
        Ecosystem::Debian => &DEBIAN_STRATEGY,
        Ecosystem::Rpm => &RPM_STRATEGY,
        Ecosystem::Ruby => &RUBY_STRATEGY,
        Ecosystem::Maven => &MAVEN_STRATEGY,
        Ecosystem::Go => &GO_STRATEGY,
        Ecosystem::Nuget => &NUGET_STRATEGY,
        Ecosystem::Composer => &COMPOSER_STRATEGY,
        Ecosystem::Calver => &CALVER_STRATEGY,
        Ecosystem::Alpine => &ALPINE_STRATEGY,
    }
}

pub(crate) fn parse_for_ecosystem(ecosystem: Ecosystem, input: &str) -> Result<ParsedRepr, String> {
    strategy_for(ecosystem).parse(input)
}

pub(crate) fn compare_for_ecosystem(
    ecosystem: Ecosystem,
    a: &ParsedRepr,
    b: &ParsedRepr,
) -> Ordering {
    strategy_for(ecosystem).compare(a, b)
}

#[derive(Debug, Clone)]
pub(crate) struct SemVer {
    pub(crate) major: u64,
    pub(crate) minor: u64,
    pub(crate) patch: u64,
    pub(crate) pre: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct DebianVersion {
    epoch: u64,
    upstream: String,
    revision: String,
}

#[derive(Debug, Clone)]
pub(crate) struct RpmVersion {
    epoch: u64,
    version: String,
    release: String,
}

pub(crate) fn parse_strict_u64(s: &str, label: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err(format!("empty {label}"));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(format!("leading zero in {label}: '{s}'"));
    }
    s.parse::<u64>().map_err(|_| format!("invalid {label}: '{s}'"))
}

pub(crate) fn validate_prerelease(pre: &str) -> Result<(), String> {
    if pre.is_empty() {
        return Err("empty prerelease".to_string());
    }
    for ident in pre.split('.') {
        if ident.is_empty() {
            return Err("empty identifier in prerelease".to_string());
        }
        if !ident.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(format!("invalid character in prerelease identifier: '{ident}'"));
        }
        let is_numeric = ident.bytes().all(|b| b.is_ascii_digit());
        if is_numeric && ident.len() > 1 && ident.starts_with('0') {
            return Err(format!("leading zero in numeric prerelease identifier: '{ident}'"));
        }
        if is_numeric && ident.parse::<u64>().is_err() {
            return Err(format!("numeric prerelease identifier out of range for u64: '{ident}'"));
        }
    }
    Ok(())
}

pub(crate) fn parse_semver_strict(input: &str) -> Result<SemVer, String> {
    let s = strip_v(input.trim());
    let s = match s.find('+') {
        Some(p) => &s[..p],
        None => s,
    };
    let (core, pre) = match s.find('-') {
        Some(p) => (&s[..p], Some(s[p + 1..].to_string())),
        None => (s, None),
    };
    let parts: Vec<&str> = core.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Expected 3 parts, got {}", parts.len()));
    }
    let major = parse_strict_u64(parts[0], "major")?;
    let minor = parse_strict_u64(parts[1], "minor")?;
    let patch = parse_strict_u64(parts[2], "patch")?;
    if let Some(ref p) = pre {
        validate_prerelease(p)?;
    }
    Ok(SemVer { major, minor, patch, pre })
}

#[allow(clippy::many_single_char_names)]
pub(crate) fn cmp_semver_strict(left: &SemVer, right: &SemVer) -> Ordering {
    let core = left
        .major
        .cmp(&right.major)
        .then(left.minor.cmp(&right.minor))
        .then(left.patch.cmp(&right.patch));
    if core != Ordering::Equal {
        return core;
    }
    match (&left.pre, &right.pre) {
        (None, None) => Ordering::Equal,
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (Some(lp), Some(rp)) => {
            let left_parts: Vec<&str> = lp.split('.').collect();
            let right_parts: Vec<&str> = rp.split('.').collect();
            let count = left_parts.len().max(right_parts.len());
            for i in 0..count {
                match (left_parts.get(i), right_parts.get(i)) {
                    (None, Some(_)) => return Ordering::Less,
                    (Some(_), None) => return Ordering::Greater,
                    (None, None) => return Ordering::Equal,
                    (Some(lv), Some(rv)) => {
                        let ord = match (lv.parse::<u64>(), rv.parse::<u64>()) {
                            (Ok(ln), Ok(rn)) => ln.cmp(&rn),
                            (Ok(_), Err(_)) => Ordering::Less,
                            (Err(_), Ok(_)) => Ordering::Greater,
                            (Err(_), Err(_)) => lv.cmp(rv),
                        };
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    }
                }
            }
            Ordering::Equal
        }
    }
}

pub(crate) fn validate_pep440(input: &str) -> Result<(), String> {
    let s = strip_v(input.trim());
    // Extract epoch (N!)
    let after_epoch = if let Some(pos) = s.find('!') {
        let epoch_str = &s[..pos];
        if epoch_str.is_empty() || !epoch_str.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("invalid PEP 440 epoch: '{epoch_str}'"));
        }
        &s[pos + 1..]
    } else {
        s
    };
    // Strip +local
    let ver_part = match after_epoch.find('+') {
        Some(pos) => &after_epoch[..pos],
        None => after_epoch,
    };
    if ver_part.is_empty() {
        return Err(format!("empty PEP 440 version: '{input}'"));
    }
    if !ver_part.as_bytes()[0].is_ascii_digit() {
        return Err(format!("PEP 440 version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn extract_epoch_colon(s: &str) -> (u64, &str) {
    for (i, ch) in s.char_indices() {
        if ch == ':' {
            if let Ok(e) = s[..i].parse::<u64>() {
                return (e, &s[i + 1..]);
            }
            break;
        }
        if !ch.is_ascii_digit() {
            break;
        }
    }
    (0, s)
}

pub(crate) fn parse_debian(input: &str) -> Result<DebianVersion, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Debian version".to_string());
    }
    let (epoch, rest) = extract_epoch_colon(s);
    // Split upstream and revision at last '-'
    let (upstream, revision) = match rest.rfind('-') {
        Some(pos) => (rest[..pos].to_string(), rest[pos + 1..].to_string()),
        None => (rest.to_string(), String::new()),
    };
    Ok(DebianVersion { epoch, upstream, revision })
}

pub(crate) fn dpkg_order(c: Option<u8>) -> i32 {
    match c {
        None => 0,
        Some(b'~') => -1,
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}

pub(crate) fn verrevcmp(a: &[u8], b: &[u8]) -> Ordering {
    let mut ia = 0usize;
    let mut ib = 0usize;
    while ia < a.len() || ib < b.len() {
        // Compare non-digit parts
        loop {
            let a_nondigit = ia < a.len() && !a[ia].is_ascii_digit();
            let b_nondigit = ib < b.len() && !b[ib].is_ascii_digit();
            if !a_nondigit && !b_nondigit {
                break;
            }
            let ac = dpkg_order(if ia < a.len() { Some(a[ia]) } else { None });
            let bc = dpkg_order(if ib < b.len() { Some(b[ib]) } else { None });
            if ac != bc {
                return ac.cmp(&bc);
            }
            if ia < a.len() {
                ia += 1;
            }
            if ib < b.len() {
                ib += 1;
            }
            if ia >= a.len() && ib >= b.len() {
                break;
            }
        }
        // Skip leading zeros
        while ia < a.len() && a[ia] == b'0' {
            ia += 1;
        }
        while ib < b.len() && b[ib] == b'0' {
            ib += 1;
        }
        // Compare digit runs
        let mut first_diff = 0i32;
        while ia < a.len() && a[ia].is_ascii_digit() && ib < b.len() && b[ib].is_ascii_digit() {
            if first_diff == 0 {
                first_diff = i32::from(a[ia]) - i32::from(b[ib]);
            }
            ia += 1;
            ib += 1;
        }
        if ia < a.len() && a[ia].is_ascii_digit() {
            return Ordering::Greater;
        }
        if ib < b.len() && b[ib].is_ascii_digit() {
            return Ordering::Less;
        }
        if first_diff != 0 {
            return if first_diff > 0 { Ordering::Greater } else { Ordering::Less };
        }
    }
    Ordering::Equal
}

pub(crate) fn cmp_debian(a: &DebianVersion, b: &DebianVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| verrevcmp(a.upstream.as_bytes(), b.upstream.as_bytes()))
        .then_with(|| verrevcmp(a.revision.as_bytes(), b.revision.as_bytes()))
}

pub(crate) fn parse_rpm(input: &str) -> Result<RpmVersion, String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty RPM version".to_string());
    }
    let (epoch, rest) = extract_epoch_colon(s);
    // Split version and release at last '-'
    let (version, release) = match rest.rfind('-') {
        Some(pos) => (rest[..pos].to_string(), rest[pos + 1..].to_string()),
        None => (rest.to_string(), String::new()),
    };
    Ok(RpmVersion { epoch, version, release })
}

pub(crate) fn rpmverscmp(a: &str, b: &str) -> Ordering {
    if a == b {
        return Ordering::Equal;
    }
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    let mut ia = 0usize;
    let mut ib = 0usize;

    while ia < ab.len() || ib < bb.len() {
        // Skip leading non-alphanumeric (except ~ and ^)
        while ia < ab.len() && !ab[ia].is_ascii_alphanumeric() && ab[ia] != b'~' && ab[ia] != b'^' {
            ia += 1;
        }
        while ib < bb.len() && !bb[ib].is_ascii_alphanumeric() && bb[ib] != b'~' && bb[ib] != b'^' {
            ib += 1;
        }

        // Handle ~ (sorts before everything including end)
        if (ia < ab.len() && ab[ia] == b'~') || (ib < bb.len() && bb[ib] == b'~') {
            if ia >= ab.len() || ab[ia] != b'~' {
                return Ordering::Greater;
            }
            if ib >= bb.len() || bb[ib] != b'~' {
                return Ordering::Less;
            }
            ia += 1;
            ib += 1;
            continue;
        }

        // Handle ^ (sorts after everything else, opposite of ~)
        if (ia < ab.len() && ab[ia] == b'^') || (ib < bb.len() && bb[ib] == b'^') {
            if ia >= ab.len() {
                return Ordering::Less; // a at end, b has ^ → a < b
            }
            if ib >= bb.len() {
                return Ordering::Greater; // b at end, a has ^ → a > b
            }
            if ab[ia] != b'^' {
                return Ordering::Greater; // a not ^, b is ^ → a > b
            }
            if bb[ib] != b'^' {
                return Ordering::Less; // a is ^, b not ^ → a < b
            }
            ia += 1;
            ib += 1;
            continue;
        }

        // End of one string
        if ia >= ab.len() || ib >= bb.len() {
            break;
        }

        let is_num = ab[ia].is_ascii_digit();

        let sa = ia;
        let sb = ib;

        if is_num {
            while ia < ab.len() && ab[ia].is_ascii_digit() {
                ia += 1;
            }
            while ib < bb.len() && bb[ib].is_ascii_digit() {
                ib += 1;
            }
        } else {
            while ia < ab.len() && ab[ia].is_ascii_alphabetic() {
                ia += 1;
            }
            while ib < bb.len() && bb[ib].is_ascii_alphabetic() {
                ib += 1;
            }
        }

        // If b run is empty, type mismatch — digits always win
        if sb == ib {
            return if is_num { Ordering::Greater } else { Ordering::Less };
        }

        if is_num {
            // Skip leading zeros
            let mut za = sa;
            let mut zb = sb;
            while za < ia && ab[za] == b'0' {
                za += 1;
            }
            while zb < ib && bb[zb] == b'0' {
                zb += 1;
            }
            // Longer number is greater
            let len_a = ia - za;
            let len_b = ib - zb;
            if len_a != len_b {
                return len_a.cmp(&len_b);
            }
            // Same length — lexical comparison
            let cmp = ab[za..ia].cmp(&bb[zb..ib]);
            if cmp != Ordering::Equal {
                return cmp;
            }
        } else {
            // Alpha run — lexical comparison
            let cmp = ab[sa..ia].cmp(&bb[sb..ib]);
            if cmp != Ordering::Equal {
                return cmp;
            }
        }
    }

    // Whichever still has characters remaining is newer
    if ia >= ab.len() && ib >= bb.len() {
        Ordering::Equal
    } else if ia < ab.len() {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

pub(crate) fn cmp_rpm(a: &RpmVersion, b: &RpmVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| rpmverscmp(&a.version, &b.version))
        .then_with(|| rpmverscmp(&a.release, &b.release))
}

pub(crate) fn validate_ruby(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Ruby gem version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Ruby gem version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_maven(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Maven version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Maven version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_nuget(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty NuGet version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("NuGet version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_composer(input: &str) -> Result<(), String> {
    let s = strip_v(input.trim());
    if s.is_empty() {
        return Err("empty Composer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Composer version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_calver(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty CalVer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("CalVer version must start with a digit: '{input}'"));
    }
    Ok(())
}

pub(crate) fn validate_alpine(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Alpine version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Alpine version must start with a digit: '{input}'"));
    }
    Ok(())
}
