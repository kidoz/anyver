use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::cmp::Ordering;

// ============================================================================
// Part 1: PARSER
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum Seg {
    Num(u64),
    Text(String),
}

impl Seg {
    fn to_py<'py>(&self, py: Python<'py>) -> Bound<'py, PyAny> {
        match self {
            Seg::Num(n) => (*n).into_pyobject(py).unwrap().into_any(),
            Seg::Text(s) => s.as_str().into_pyobject(py).unwrap().into_any(),
        }
    }
}

fn tag_weight(s: &str) -> Option<i32> {
    match s {
        "~" => Some(-10),
        "dev" => Some(-5),
        "alpha" | "a" => Some(0),
        "beta" | "b" => Some(10),
        "milestone" | "m" => Some(15),
        "rc" | "cr" | "c" | "preview" | "pre" => Some(20),
        "snapshot" => Some(25),
        "^" => Some(32),
        "post" | "sp" | "patch" | "p" => Some(35),
        _ => None,
    }
}

fn is_prerelease_tag(s: &str) -> bool {
    tag_weight(s).is_some_and(|w| w < 30)
}

fn is_postrelease_tag(s: &str) -> bool {
    tag_weight(s).is_some_and(|w| w > 30)
}

fn strip_v(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() > 1 && (b[0] == b'v' || b[0] == b'V') && b[1].is_ascii_digit() { &s[1..] } else { s }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Ecosystem {
    Generic,
    Semver,
    Pep440,
    Debian,
    Rpm,
    Ruby,
    Maven,
    Go,
    Npm,
    Nuget,
    Composer,
    Crates,
    Hex,
    Swift,
    Calver,
    Alpine,
    Docker,
}

impl Ecosystem {
    fn from_str(input: &str) -> PyResult<Self> {
        match input.to_ascii_lowercase().as_str() {
            "generic" => Ok(Self::Generic),
            "semver" | "semver_strict" | "semver-strict" => Ok(Self::Semver),
            "pep440" | "pep-440" | "python" => Ok(Self::Pep440),
            "debian" | "dpkg" | "deb" => Ok(Self::Debian),
            "rpm" | "redhat" => Ok(Self::Rpm),
            "ruby" | "gem" | "rubygems" => Ok(Self::Ruby),
            "maven" | "mvn" => Ok(Self::Maven),
            "go" | "golang" => Ok(Self::Go),
            "npm" | "node" | "nodejs" => Ok(Self::Npm),
            "nuget" | "dotnet" => Ok(Self::Nuget),
            "composer" | "packagist" | "php" => Ok(Self::Composer),
            "crates" | "cargo" | "crates.io" => Ok(Self::Crates),
            "hex" | "elixir" | "erlang" => Ok(Self::Hex),
            "swift" | "swiftpm" => Ok(Self::Swift),
            "calver" => Ok(Self::Calver),
            "alpine" | "apk" => Ok(Self::Alpine),
            "docker" | "oci" => Ok(Self::Docker),
            other => Err(PyValueError::new_err(format!(
                "unsupported ecosystem '{other}'; expected one of: generic, semver, pep440, debian, rpm, ruby, maven, go, npm, nuget, composer, crates, hex, swift, calver, alpine, docker"
            ))),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::Semver => "semver",
            Self::Pep440 => "pep440",
            Self::Debian => "debian",
            Self::Rpm => "rpm",
            Self::Ruby => "ruby",
            Self::Maven => "maven",
            Self::Go => "go",
            Self::Npm => "npm",
            Self::Nuget => "nuget",
            Self::Composer => "composer",
            Self::Crates => "crates",
            Self::Hex => "hex",
            Self::Swift => "swift",
            Self::Calver => "calver",
            Self::Alpine => "alpine",
            Self::Docker => "docker",
        }
    }
}

fn autodetect_ecosystem(version: &str) -> Ecosystem {
    let lower = version.to_ascii_lowercase();

    // 1. Unambiguous characters
    if lower.contains('!') {
        return Ecosystem::Pep440;
    }
    if lower.contains('~') {
        return Ecosystem::Debian;
    }
    if lower.contains('^') {
        return Ecosystem::Rpm;
    }

    // 2. Strong Substring Markers
    if lower.contains(".post") || lower.contains(".dev") {
        return Ecosystem::Pep440;
    }
    if lower.ends_with("+incompatible") {
        return Ecosystem::Go;
    }
    if lower.ends_with("-snapshot") {
        return Ecosystem::Maven;
    }
    if lower.contains("_alpha")
        || lower.contains("_beta")
        || lower.contains("_p")
        || lower.contains("_rc")
        || lower.contains("_pre")
    {
        return Ecosystem::Alpine;
    }
    if lower.contains("+deb") || lower.contains("+ubuntu") {
        return Ecosystem::Debian;
    }
    if lower.contains(".el") || lower.contains(".fc") || lower.contains(".amzn") {
        return Ecosystem::Rpm;
    }

    // 3. Regex / Pattern matching
    let bytes = lower.as_bytes();
    for i in 1..bytes.len().saturating_sub(1) {
        if (bytes[i] == b'a' || bytes[i] == b'b')
            && bytes[i - 1].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
        {
            return Ecosystem::Pep440;
        }
        if i >= 2
            && bytes[i - 1] == b'r'
            && bytes[i] == b'c'
            && bytes[i - 2].is_ascii_digit()
            && bytes[i + 1].is_ascii_digit()
        {
            return Ecosystem::Pep440;
        }
    }

    if let Some(pos) = lower.rfind("-r")
        && pos > 0
        && pos + 2 < lower.len()
        && lower[pos + 2..].bytes().all(|b| b.is_ascii_digit())
    {
        return Ecosystem::Alpine;
    }

    if !lower.contains('-') && !lower.contains('_') {
        for part in lower.split('.') {
            if part.bytes().any(|b| b.is_ascii_alphabetic())
                && (part == "pre"
                    || part == "rc"
                    || part.starts_with("beta")
                    || part.starts_with("alpha"))
            {
                return Ecosystem::Ruby;
            }
        }
    }

    // 4. Structural Fallbacks
    if let Some(first_part) = lower.split(|c: char| !c.is_ascii_digit()).next()
        && let Ok(year) = first_part.parse::<u32>()
        && (1990..=2100).contains(&year)
        && lower.contains('.')
    {
        return Ecosystem::Calver;
    }

    Ecosystem::Generic
}

#[derive(Debug, Clone)]
struct ParsedVersion {
    raw: String,
    epoch: u64,
    segments: Vec<Seg>,
    build: String,
    is_prerelease: bool,
    is_postrelease: bool,
}

fn parse_generic(input: &str) -> ParsedVersion {
    let raw = input.to_string();
    let trimmed = strip_v(input.trim());

    let (epoch, after_epoch) = {
        let mut found = None;
        for (i, ch) in trimmed.char_indices() {
            if ch == ':' || ch == '!' {
                if let Ok(e) = trimmed[..i].parse::<u64>() {
                    found = Some((e, &trimmed[i + 1..]));
                }
                break;
            }
            if !ch.is_ascii_digit() {
                break;
            }
        }
        found.unwrap_or((0, trimmed))
    };

    let (ver_part, build) = match after_epoch.find('+') {
        Some(pos) => (&after_epoch[..pos], after_epoch[pos + 1..].to_string()),
        None => (after_epoch, String::new()),
    };

    let lower = ver_part.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let len = bytes.len();
    let mut segs: Vec<Seg> = Vec::with_capacity(16);
    let mut is_pre = false;
    let mut is_post = false;
    let mut i = 0;

    while i < len {
        let ch = bytes[i];
        if ch == b'.' || ch == b'-' || ch == b'_' {
            i += 1;
            continue;
        }
        if ch == b'~' {
            segs.push(Seg::Text("~".into()));
            is_pre = true;
            i += 1;
            continue;
        }
        if ch == b'^' {
            segs.push(Seg::Text("^".into()));
            is_post = true;
            i += 1;
            continue;
        }
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && bytes[i].is_ascii_digit() {
                i += 1;
            }
            let n: u64 = lower[start..i].parse().unwrap_or(u64::MAX);
            segs.push(Seg::Num(n));
            continue;
        }
        if ch.is_ascii_alphabetic() {
            let start = i;
            while i < len && bytes[i].is_ascii_alphabetic() {
                i += 1;
            }
            let word = lower[start..i].to_string();
            if is_prerelease_tag(&word) {
                is_pre = true;
            }
            if is_postrelease_tag(&word) {
                is_post = true;
            }
            segs.push(Seg::Text(word));
            continue;
        }
        i += 1;
    }

    ParsedVersion {
        raw,
        epoch,
        segments: segs,
        build,
        is_prerelease: is_pre,
        is_postrelease: is_post,
    }
}

fn parse(input: &str) -> ParsedVersion {
    parse_generic(input)
}

// ============================================================================
// Part 2: COMPARISON
// ============================================================================

fn normalized(segs: &[Seg]) -> &[Seg] {
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

fn cmp_two(a: &Seg, b: &Seg) -> Ordering {
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

fn text_effective_weight(s: &str) -> i32 {
    tag_weight(s).unwrap_or(29)
}

#[allow(clippy::many_single_char_names)]
fn cmp_segments(left: &[Seg], right: &[Seg]) -> Ordering {
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

fn cmp_parsed(a: &ParsedVersion, b: &ParsedVersion) -> Ordering {
    a.epoch.cmp(&b.epoch).then_with(|| cmp_segments(&a.segments, &b.segments))
}

#[derive(Debug, Clone)]
enum ParsedRepr {
    Generic(ParsedVersion),
    Semver(SemVer),
    Debian(DebianVersion),
    Rpm(RpmVersion),
}

trait VersionStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String>;
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering;
}

struct GenericStrategy;
struct SemverStrategy;
struct Pep440Strategy;
struct DebianStrategy;
struct RpmStrategy;
struct RubyStrategy;
struct MavenStrategy;
struct GoStrategy;
struct NugetStrategy;
struct ComposerStrategy;
struct CalverStrategy;
struct AlpineStrategy;

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

fn strategy_for(ecosystem: Ecosystem) -> &'static dyn VersionStrategy {
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

fn parse_for_ecosystem(ecosystem: Ecosystem, input: &str) -> Result<ParsedRepr, String> {
    strategy_for(ecosystem).parse(input)
}

fn compare_for_ecosystem(ecosystem: Ecosystem, a: &ParsedRepr, b: &ParsedRepr) -> Ordering {
    strategy_for(ecosystem).compare(a, b)
}

// ============================================================================
// Part 3: Python class
// ============================================================================

#[pyclass(name = "Version", from_py_object)]
#[derive(Debug, Clone)]
struct PyVersion {
    inner: ParsedVersion,
    eco: Ecosystem,
}

#[pymethods]
impl PyVersion {
    #[new]
    #[pyo3(signature = (version, ecosystem = "auto"))]
    fn new(version: &str, ecosystem: &str) -> PyResult<Self> {
        let eco = if ecosystem.eq_ignore_ascii_case("auto") {
            autodetect_ecosystem(version)
        } else {
            Ecosystem::from_str(ecosystem)?
        };
        Ok(PyVersion { inner: parse(version), eco })
    }

    fn __richcmp__(&self, other: &PyVersion, op: CompareOp) -> bool {
        let ord = cmp_parsed(&self.inner, &other.inner);
        match op {
            CompareOp::Lt => ord == Ordering::Less,
            CompareOp::Le => ord != Ordering::Greater,
            CompareOp::Eq => ord == Ordering::Equal,
            CompareOp::Ne => ord != Ordering::Equal,
            CompareOp::Gt => ord == Ordering::Greater,
            CompareOp::Ge => ord != Ordering::Less,
        }
    }

    fn __str__(&self) -> &str {
        &self.inner.raw
    }

    fn __repr__(&self) -> String {
        format!("Version('{}')", self.inner.raw)
    }

    fn __len__(&self) -> usize {
        self.inner.segments.len()
    }

    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    fn __getitem__<'py>(&self, py: Python<'py>, idx: isize) -> PyResult<Bound<'py, PyAny>> {
        let len = self.inner.segments.len() as isize;
        let real_idx = if idx < 0 { len + idx } else { idx };
        if real_idx < 0 || real_idx >= len {
            return Err(PyIndexError::new_err(format!(
                "segment index {idx} out of range (version has {len} segments)"
            )));
        }
        Ok(self.inner.segments[real_idx as usize].to_py(py))
    }

    fn __bool__(&self) -> bool {
        !self.inner.segments.is_empty()
    }

    fn __hash__(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.inner.epoch.hash(&mut hasher);
        let segs = normalized(&self.inner.segments);
        for s in segs {
            match s {
                Seg::Num(n) => {
                    0u8.hash(&mut hasher);
                    n.hash(&mut hasher);
                }
                Seg::Text(t) => {
                    1u8.hash(&mut hasher);
                    t.hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    #[getter]
    fn raw(&self) -> &str {
        &self.inner.raw
    }

    #[getter]
    fn ecosystem(&self) -> &'static str {
        self.eco.as_str()
    }

    #[getter]
    fn epoch(&self) -> u64 {
        self.inner.epoch
    }

    #[getter]
    fn build(&self) -> &str {
        &self.inner.build
    }

    #[getter]
    fn is_prerelease(&self) -> bool {
        self.inner.is_prerelease
    }

    #[getter]
    fn is_postrelease(&self) -> bool {
        self.inner.is_postrelease
    }

    #[getter]
    fn count(&self) -> usize {
        self.inner.segments.len()
    }

    #[getter]
    fn major(&self) -> u64 {
        match self.inner.segments.first() {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    #[getter]
    fn minor(&self) -> u64 {
        match self.inner.segments.get(1) {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    #[getter]
    fn patch(&self) -> u64 {
        match self.inner.segments.get(2) {
            Some(Seg::Num(n)) => *n,
            _ => 0,
        }
    }

    fn segments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let items: Vec<Bound<'py, PyAny>> =
            self.inner.segments.iter().map(|s| s.to_py(py)).collect();
        PyTuple::new(py, items)
    }

    fn release<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let nums: Vec<Bound<'py, PyAny>> = self
            .inner
            .segments
            .iter()
            .take_while(|s| matches!(s, Seg::Num(_)))
            .map(|s| s.to_py(py))
            .collect();
        PyTuple::new(py, nums)
    }

    #[pyo3(signature = (other, ecosystem = "generic"))]
    fn compare(&self, other: &Bound<'_, PyAny>, ecosystem: &str) -> PyResult<i32> {
        let eco = if ecosystem.eq_ignore_ascii_case("auto") {
            self.eco
        } else {
            Ecosystem::from_str(ecosystem)?
        };
        let left = parse_for_ecosystem(eco, &self.inner.raw).map_err(PyValueError::new_err)?;
        let right = extract_parsed_for_ecosystem(other, eco)?;
        Ok(match compare_for_ecosystem(eco, &left, &right) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        })
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("raw", &self.inner.raw)?;
        dict.set_item("epoch", self.inner.epoch)?;
        dict.set_item("major", self.major())?;
        dict.set_item("minor", self.minor())?;
        dict.set_item("patch", self.patch())?;
        dict.set_item("build", &self.inner.build)?;
        dict.set_item("is_prerelease", self.inner.is_prerelease)?;
        dict.set_item("is_postrelease", self.inner.is_postrelease)?;
        Ok(dict)
    }

    fn sort_key<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        // Build a normalized segment list for sort_key encoding:
        // 1. Strip trailing Num(0) from the whole list (standard normalization)
        // 2. Find the first Text segment; strip trailing Num(0) from the
        //    numeric prefix before it. This ensures that Num(0) segments
        //    between the release part and the first tag are transparent,
        //    matching the comparison algorithm's Num(0)-vs-Missing skip.
        let segs = normalized(&self.inner.segments);
        let first_text = segs.iter().position(|s| matches!(s, Seg::Text(_)));
        let (prefix, suffix) = match first_text {
            Some(pos) => {
                let mut end = pos;
                while end > 0 {
                    if let Seg::Num(0) = segs[end - 1] {
                        end -= 1;
                    } else {
                        break;
                    }
                }
                (&segs[..end], &segs[pos..])
            }
            None => (segs, &[][..]),
        };

        let cap = prefix.len() + suffix.len() + 2;
        let mut tuples: Vec<Bound<'py, PyAny>> = Vec::with_capacity(cap);

        let mk_tuple = |py: Python<'py>, a: i64, b: i64, c: &str| -> PyResult<Bound<'py, PyAny>> {
            Ok(PyTuple::new(
                py,
                vec![
                    a.into_pyobject(py).unwrap().into_any(),
                    b.into_pyobject(py).unwrap().into_any(),
                    c.into_pyobject(py).unwrap().into_any(),
                ],
            )?
            .into_any())
        };

        #[allow(clippy::cast_possible_wrap)]
        tuples.push(mk_tuple(py, 2, self.inner.epoch as i64, "")?);

        for seg in prefix.iter().chain(suffix.iter()) {
            #[allow(clippy::cast_possible_wrap)]
            let t = match seg {
                Seg::Num(n) => mk_tuple(py, 2, *n as i64, "")?,
                Seg::Text(s) => {
                    let w = i64::from(tag_weight(s).unwrap_or(29));
                    mk_tuple(py, 1, w, s)?
                }
            };
            tuples.push(t);
        }

        tuples.push(mk_tuple(py, 1, 30, "")?);

        PyTuple::new(py, tuples)
    }
}

fn resolve_eco_for_obj(ecosystem: &str, obj: &Bound<'_, PyAny>) -> PyResult<Ecosystem> {
    if ecosystem.eq_ignore_ascii_case("auto") {
        if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
            Ok(v.eco)
        } else if let Ok(s) = obj.extract::<String>() {
            Ok(autodetect_ecosystem(&s))
        } else {
            Err(PyTypeError::new_err("expected Version or str"))
        }
    } else {
        Ecosystem::from_str(ecosystem)
    }
}

fn extract_parsed_for_ecosystem(
    obj: &Bound<'_, PyAny>,
    ecosystem: Ecosystem,
) -> PyResult<ParsedRepr> {
    if let Ok(v) = obj.extract::<PyRef<PyVersion>>() {
        match ecosystem {
            Ecosystem::Generic => Ok(ParsedRepr::Generic(v.inner.clone())),
            _ => parse_for_ecosystem(ecosystem, &v.inner.raw).map_err(PyValueError::new_err),
        }
    } else if let Ok(s) = obj.extract::<String>() {
        parse_for_ecosystem(ecosystem, &s).map_err(PyValueError::new_err)
    } else {
        Err(PyTypeError::new_err("expected Version or str"))
    }
}

fn compare_str_with_ecosystem(a: &str, b: &str, ecosystem: &str) -> PyResult<Ordering> {
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        autodetect_ecosystem(a)
    } else {
        Ecosystem::from_str(ecosystem)?
    };
    let left = parse_for_ecosystem(eco, a).map_err(PyValueError::new_err)?;
    let right = parse_for_ecosystem(eco, b).map_err(PyValueError::new_err)?;
    Ok(compare_for_ecosystem(eco, &left, &right))
}

// ============================================================================
// Part 4: Module-level functions
// ============================================================================

#[pyfunction]
#[pyo3(signature = (s, ecosystem = "auto"))]
fn version(s: &str, ecosystem: &str) -> PyResult<PyVersion> {
    PyVersion::new(s, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn compare(a: &Bound<'_, PyAny>, b: &Bound<'_, PyAny>, ecosystem: &str) -> PyResult<i32> {
    let eco = resolve_eco_for_obj(ecosystem, a)?;
    let pa = extract_parsed_for_ecosystem(a, eco)?;
    let pb = extract_parsed_for_ecosystem(b, eco)?;
    Ok(match compare_for_ecosystem(eco, &pa, &pb) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
fn sort_versions<'py>(
    _py: Python<'py>,
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Vec<Bound<'py, PyAny>>> {
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        Ecosystem::from_str(ecosystem)?
    };
    let mut pairs: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    pairs.sort_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0));
    Ok(pairs.into_iter().map(|(_, obj)| obj).collect())
}

#[pyfunction(signature = (pairs, ecosystem = "generic"))]
fn batch_compare(
    pairs: Vec<(Bound<'_, PyAny>, Bound<'_, PyAny>)>,
    ecosystem: &str,
) -> PyResult<Vec<i32>> {
    let is_auto = ecosystem.eq_ignore_ascii_case("auto");
    let base_eco = if is_auto { Ecosystem::Generic } else { Ecosystem::from_str(ecosystem)? };

    pairs
        .iter()
        .map(|(a, b)| {
            let eco = if is_auto { resolve_eco_for_obj("auto", a)? } else { base_eco };
            let pa = extract_parsed_for_ecosystem(a, eco)?;
            let pb = extract_parsed_for_ecosystem(b, eco)?;
            Ok(match compare_for_ecosystem(eco, &pa, &pb) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            })
        })
        .collect()
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn gt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn ge(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn lt(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Less)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn le(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Greater)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn eq(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? == Ordering::Equal)
}
#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn ne(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    Ok(compare_str_with_ecosystem(a, b, ecosystem)? != Ordering::Equal)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn gte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    ge(a, b, ecosystem)
}

#[pyfunction(signature = (a, b, ecosystem = "generic"))]
fn lte(a: &str, b: &str, ecosystem: &str) -> PyResult<bool> {
    le(a, b, ecosystem)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
fn max_version<'py>(
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
    if versions.is_empty() {
        return Err(PyValueError::new_err("empty sequence"));
    }
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        Ecosystem::from_str(ecosystem)?
    };
    let mut parsed: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    parsed.sort_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0));
    Ok(parsed.pop().unwrap().1)
}

#[pyfunction(signature = (versions, ecosystem = "generic"))]
fn min_version<'py>(
    versions: Vec<Bound<'py, PyAny>>,
    ecosystem: &str,
) -> PyResult<Bound<'py, PyAny>> {
    if versions.is_empty() {
        return Err(PyValueError::new_err("empty sequence"));
    }
    let eco = if ecosystem.eq_ignore_ascii_case("auto") {
        if let Some(first) = versions.first() {
            resolve_eco_for_obj("auto", first)?
        } else {
            Ecosystem::Generic
        }
    } else {
        Ecosystem::from_str(ecosystem)?
    };
    let parsed: Vec<(ParsedRepr, Bound<'py, PyAny>)> = versions
        .into_iter()
        .map(|obj| {
            let p = extract_parsed_for_ecosystem(&obj, eco)?;
            Ok((p, obj))
        })
        .collect::<PyResult<Vec<_>>>()?;
    let (_, min_obj) =
        parsed.into_iter().min_by(|a, b| compare_for_ecosystem(eco, &a.0, &b.0)).unwrap();
    Ok(min_obj)
}

// ============================================================================
// Strict SemVer 2.0.0
// ============================================================================

#[pyfunction]
fn compare_semver_strict(a: &str, b: &str) -> PyResult<i32> {
    let pa = parse_for_ecosystem(Ecosystem::Semver, a).map_err(PyValueError::new_err)?;
    let pb = parse_for_ecosystem(Ecosystem::Semver, b).map_err(PyValueError::new_err)?;
    Ok(match compare_for_ecosystem(Ecosystem::Semver, &pa, &pb) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    })
}

#[derive(Debug, Clone)]
struct SemVer {
    major: u64,
    minor: u64,
    patch: u64,
    pre: Option<String>,
}

#[derive(Debug, Clone)]
struct DebianVersion {
    epoch: u64,
    upstream: String,
    revision: String,
}

#[derive(Debug, Clone)]
struct RpmVersion {
    epoch: u64,
    version: String,
    release: String,
}

fn parse_strict_u64(s: &str, label: &str) -> Result<u64, String> {
    if s.is_empty() {
        return Err(format!("empty {label}"));
    }
    if s.len() > 1 && s.starts_with('0') {
        return Err(format!("leading zero in {label}: '{s}'"));
    }
    s.parse::<u64>().map_err(|_| format!("invalid {label}: '{s}'"))
}

fn validate_prerelease(pre: &str) -> Result<(), String> {
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

fn parse_semver_strict(input: &str) -> Result<SemVer, String> {
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
fn cmp_semver_strict(left: &SemVer, right: &SemVer) -> Ordering {
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

// ============================================================================
// PEP 440 validator
// ============================================================================

fn validate_pep440(input: &str) -> Result<(), String> {
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

// ============================================================================
// Debian parser + comparator (dpkg verrevcmp)
// ============================================================================

fn extract_epoch_colon(s: &str) -> (u64, &str) {
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

fn parse_debian(input: &str) -> Result<DebianVersion, String> {
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

fn dpkg_order(c: Option<u8>) -> i32 {
    match c {
        None => 0,
        Some(b'~') => -1,
        Some(c) if c.is_ascii_digit() => 0,
        Some(c) if c.is_ascii_alphabetic() => i32::from(c),
        Some(c) => i32::from(c) + 256,
    }
}

fn verrevcmp(a: &[u8], b: &[u8]) -> Ordering {
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

fn cmp_debian(a: &DebianVersion, b: &DebianVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| verrevcmp(a.upstream.as_bytes(), b.upstream.as_bytes()))
        .then_with(|| verrevcmp(a.revision.as_bytes(), b.revision.as_bytes()))
}

// ============================================================================
// RPM parser + comparator (rpmverscmp)
// ============================================================================

fn parse_rpm(input: &str) -> Result<RpmVersion, String> {
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

fn rpmverscmp(a: &str, b: &str) -> Ordering {
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

fn cmp_rpm(a: &RpmVersion, b: &RpmVersion) -> Ordering {
    a.epoch
        .cmp(&b.epoch)
        .then_with(|| rpmverscmp(&a.version, &b.version))
        .then_with(|| rpmverscmp(&a.release, &b.release))
}

// ============================================================================
// Ruby / Maven validators
// ============================================================================

fn validate_ruby(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Ruby gem version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Ruby gem version must start with a digit: '{input}'"));
    }
    Ok(())
}

fn validate_maven(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Maven version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Maven version must start with a digit: '{input}'"));
    }
    Ok(())
}

fn validate_nuget(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty NuGet version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("NuGet version must start with a digit: '{input}'"));
    }
    Ok(())
}

fn validate_composer(input: &str) -> Result<(), String> {
    let s = strip_v(input.trim());
    if s.is_empty() {
        return Err("empty Composer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Composer version must start with a digit: '{input}'"));
    }
    Ok(())
}

fn validate_calver(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty CalVer version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("CalVer version must start with a digit: '{input}'"));
    }
    Ok(())
}

fn validate_alpine(input: &str) -> Result<(), String> {
    let s = input.trim();
    if s.is_empty() {
        return Err("empty Alpine version".to_string());
    }
    if !s.as_bytes()[0].is_ascii_digit() {
        return Err(format!("Alpine version must start with a digit: '{input}'"));
    }
    Ok(())
}

// ============================================================================
// Python module
// ============================================================================

#[pymodule]
fn anyver(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVersion>()?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(compare_semver_strict, m)?)?;
    m.add_function(wrap_pyfunction!(sort_versions, m)?)?;
    m.add_function(wrap_pyfunction!(batch_compare, m)?)?;
    m.add_function(wrap_pyfunction!(gt, m)?)?;
    m.add_function(wrap_pyfunction!(ge, m)?)?;
    m.add_function(wrap_pyfunction!(gte, m)?)?;
    m.add_function(wrap_pyfunction!(lt, m)?)?;
    m.add_function(wrap_pyfunction!(le, m)?)?;
    m.add_function(wrap_pyfunction!(lte, m)?)?;
    m.add_function(wrap_pyfunction!(eq, m)?)?;
    m.add_function(wrap_pyfunction!(ne, m)?)?;
    m.add_function(wrap_pyfunction!(max_version, m)?)?;
    m.add_function(wrap_pyfunction!(min_version, m)?)?;
    Ok(())
}

// ============================================================================
// Rust unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering::*;

    fn cmpg(a: &str, b: &str) -> Ordering {
        cmp_parsed(&parse(a), &parse(b))
    }

    // --- Basic numeric ---

    #[test]
    fn test_basic_1_lt_2() {
        assert_eq!(cmpg("1", "2"), Less);
    }
    #[test]
    fn test_basic_2_gt_1() {
        assert_eq!(cmpg("2", "1"), Greater);
    }
    #[test]
    fn test_basic_equal() {
        assert_eq!(cmpg("1", "1"), Equal);
    }
    #[test]
    fn test_trailing_zero_eq() {
        assert_eq!(cmpg("1.0", "1.0.0"), Equal);
    }
    #[test]
    fn test_patch_lt() {
        assert_eq!(cmpg("1.0.0", "1.0.1"), Less);
    }
    #[test]
    fn test_minor_lt() {
        assert_eq!(cmpg("1.0.1", "1.1.0"), Less);
    }
    #[test]
    fn test_major_lt() {
        assert_eq!(cmpg("1.1.0", "2.0.0"), Less);
    }
    #[test]
    fn test_double_digit_major() {
        assert_eq!(cmpg("10.0.0", "9.0.0"), Greater);
    }
    #[test]
    fn test_same_version() {
        assert_eq!(cmpg("1.2.3", "1.2.3"), Equal);
    }
    #[test]
    fn test_tiny_patch() {
        assert_eq!(cmpg("0.0.1", "0.0.2"), Less);
    }
    #[test]
    fn test_numeric_sort() {
        assert_eq!(cmpg("1.2", "1.10"), Less);
    }
    #[test]
    fn test_many_trailing_zeros() {
        assert_eq!(cmpg("1.0.0.0.0", "1.0.0"), Equal);
    }
    #[test]
    fn test_zero_eq() {
        assert_eq!(cmpg("0", "0.0.0"), Equal);
    }

    // --- v-prefix ---

    #[test]
    fn test_vprefix_lower() {
        assert_eq!(cmpg("v1.0.0", "1.0.0"), Equal);
    }
    #[test]
    fn test_vprefix_upper() {
        assert_eq!(cmpg("V1.0.0", "1.0.0"), Equal);
    }
    #[test]
    fn test_vprefix_compare() {
        assert_eq!(cmpg("v2.0.0", "v1.0.0"), Greater);
    }
    #[test]
    fn test_vprefix_lt() {
        assert_eq!(cmpg("v0.1.0", "v0.2.0"), Less);
    }

    // --- SemVer pre-release ---

    #[test]
    fn test_sv_alpha_lt_alpha1() {
        assert_eq!(cmpg("1.0.0-alpha", "1.0.0-alpha.1"), Less);
    }
    #[test]
    fn test_sv_alpha_lt_beta() {
        assert_eq!(cmpg("1.0.0-alpha", "1.0.0-beta"), Less);
    }
    #[test]
    fn test_sv_beta_lt_rc() {
        assert_eq!(cmpg("1.0.0-beta", "1.0.0-rc"), Less);
    }
    #[test]
    fn test_sv_rc_lt_release() {
        assert_eq!(cmpg("1.0.0-rc", "1.0.0"), Less);
    }
    #[test]
    fn test_sv_alpha_lt_release() {
        assert_eq!(cmpg("1.0.0-alpha", "1.0.0"), Less);
    }
    #[test]
    fn test_sv_build_ignored() {
        assert_eq!(cmpg("1.0.0+build", "1.0.0"), Equal);
    }
    #[test]
    fn test_sv_build_both_ignored() {
        assert_eq!(cmpg("1.0.0+build.1", "1.0.0+build.2"), Equal);
    }
    #[test]
    fn test_sv_build_timestamp() {
        assert_eq!(cmpg("1.0.0+20130313144700", "1.0.0"), Equal);
    }

    // --- PEP 440 ---

    #[test]
    fn test_pep_a1_lt_a2() {
        assert_eq!(cmpg("1.0a1", "1.0a2"), Less);
    }
    #[test]
    fn test_pep_a_lt_b() {
        assert_eq!(cmpg("1.0a2", "1.0b1"), Less);
    }
    #[test]
    fn test_pep_b_lt_rc() {
        assert_eq!(cmpg("1.0b1", "1.0rc1"), Less);
    }
    #[test]
    fn test_pep_rc_lt_rel() {
        assert_eq!(cmpg("1.0rc1", "1.0"), Less);
    }
    #[test]
    fn test_pep_dev_lt_a() {
        assert_eq!(cmpg("1.0.dev1", "1.0a1"), Less);
    }
    #[test]
    fn test_pep_dev_lt_rel() {
        assert_eq!(cmpg("1.0.dev1", "1.0"), Less);
    }
    #[test]
    fn test_pep_rel_lt_post() {
        assert_eq!(cmpg("1.0", "1.0.post1"), Less);
    }
    #[test]
    fn test_pep_post1_lt_post2() {
        assert_eq!(cmpg("1.0.post1", "1.0.post2"), Less);
    }
    #[test]
    fn test_pep_epoch() {
        assert_eq!(cmpg("1!0.1", "2.0"), Greater);
    }

    // --- Debian/dpkg ---

    #[test]
    fn test_deb_tilde_alpha_lt_beta() {
        assert_eq!(cmpg("1.0~alpha", "1.0~beta"), Less);
    }
    #[test]
    fn test_deb_tilde_rc_lt_rel() {
        assert_eq!(cmpg("1.0~rc1", "1.0"), Less);
    }
    #[test]
    fn test_deb_tilde_alpha_lt_rel() {
        assert_eq!(cmpg("1.0~alpha", "1.0"), Less);
    }
    #[test]
    fn test_deb_epoch_colon() {
        assert_eq!(cmpg("1:0.1", "2.0"), Greater);
    }
    #[test]
    fn test_deb_epoch_compare() {
        assert_eq!(cmpg("2:1.0", "1:2.0"), Greater);
    }
    #[test]
    fn test_deb_epoch_zero() {
        assert_eq!(cmpg("0:1.0", "1.0"), Equal);
    }
    #[test]
    fn test_deb_build_stripped() {
        assert_eq!(cmpg("1.0+deb9u1", "1.0+deb9u2"), Equal);
    }

    // --- RPM ---

    #[test]
    fn test_rpm_tilde_rc_lt_rel() {
        assert_eq!(cmpg("1.0~rc1", "1.0"), Less);
    }
    #[test]
    fn test_rpm_rel_lt_caret() {
        assert_eq!(cmpg("1.0", "1.0^git1"), Less);
    }
    #[test]
    fn test_rpm_caret_lt_revision() {
        assert_eq!(cmpg("1.0^git1", "1.0-1.fc33"), Less);
    }

    // --- Go modules ---

    #[test]
    fn test_go_basic() {
        assert_eq!(cmpg("v1.0.0", "v1.0.1"), Less);
    }
    #[test]
    fn test_go_alpha() {
        assert_eq!(cmpg("v1.0.0-alpha", "v1.0.0"), Less);
    }
    #[test]
    fn test_go_incompatible() {
        assert_eq!(cmpg("v2.0.0+incompatible", "v2.0.0"), Equal);
    }
    #[test]
    fn test_go_incompatible_lt() {
        assert_eq!(cmpg("v2.0.0+incompatible", "v2.0.1+incompatible"), Less);
    }

    // --- Ruby Gems ---

    #[test]
    fn test_ruby_pre_lt_rel() {
        assert_eq!(cmpg("1.0.0.pre", "1.0.0"), Less);
    }
    #[test]
    fn test_ruby_alpha_lt_beta() {
        assert_eq!(cmpg("1.0.0.alpha", "1.0.0.beta"), Less);
    }
    #[test]
    fn test_ruby_beta_lt_rc() {
        assert_eq!(cmpg("1.0.0.beta", "1.0.0.rc1"), Less);
    }
    #[test]
    fn test_ruby_rc_lt_rel() {
        assert_eq!(cmpg("1.0.0.rc1", "1.0.0"), Less);
    }
    #[test]
    fn test_ruby_numeric() {
        assert_eq!(cmpg("3.2", "3.10"), Less);
    }

    // --- Maven ---

    // ---- Qualifier ordering (spec §8) ----
    #[test]
    fn test_maven_alpha_lt_beta() {
        assert_eq!(cmpg("1.0-alpha-1", "1.0-beta-1"), Less);
    }
    #[test]
    fn test_maven_beta_lt_rc() {
        assert_eq!(cmpg("1.0-beta-1", "1.0-rc-1"), Less);
    }
    #[test]
    fn test_maven_snapshot_lt_rel() {
        assert_eq!(cmpg("1.0-SNAPSHOT", "1.0"), Less);
    }
    #[test]
    fn test_maven_rel_lt_sp() {
        assert_eq!(cmpg("1.0", "1.0-sp-1"), Less);
    }
    #[test]
    fn test_maven_alpha_lt_milestone() {
        assert_eq!(cmpg("1.0-alpha", "1.0-milestone"), Less);
    }
    #[test]
    fn test_maven_milestone_lt_rc() {
        assert_eq!(cmpg("1.0-milestone-1", "1.0-rc-1"), Less);
    }
    #[test]
    fn test_maven_rc_lt_snapshot() {
        assert_eq!(cmpg("1.0-rc-1", "1.0-SNAPSHOT"), Less);
    }
    #[test]
    fn test_maven_cr_eq_rc() {
        assert_eq!(cmpg("1.0-cr-1", "1.0-rc-1"), Equal);
    }
    #[test]
    fn test_maven_full_qualifier_chain() {
        // alpha < beta < milestone < rc < snapshot < release < sp
        let chain = [
            "1.0-alpha-1",
            "1.0-beta-1",
            "1.0-milestone-1",
            "1.0-rc-1",
            "1.0-SNAPSHOT",
            "1.0",
            "1.0-sp-1",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "{} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Qualifier aliases (a/b/m/cr) ----
    #[test]
    fn test_maven_a_alias_alpha() {
        assert_eq!(cmpg("1.0-a", "1.0-alpha"), Equal);
    }
    #[test]
    fn test_maven_b_alias_beta() {
        assert_eq!(cmpg("1.0-b", "1.0-beta"), Equal);
    }
    #[test]
    fn test_maven_m_alias_milestone() {
        assert_eq!(cmpg("1.0-m", "1.0-milestone"), Equal);
    }
    #[test]
    fn test_maven_cr_alias_rc() {
        assert_eq!(cmpg("1.0-cr", "1.0-rc"), Equal);
    }
    #[test]
    fn test_maven_a1_alias_alpha1() {
        assert_eq!(cmpg("1-a1", "1-alpha-1"), Equal);
    }
    #[test]
    fn test_maven_b2_alias_beta2() {
        assert_eq!(cmpg("1-b2", "1-beta-2"), Equal);
    }
    #[test]
    fn test_maven_m3_alias_milestone3() {
        assert_eq!(cmpg("1-m3", "1-milestone-3"), Equal);
    }

    // ---- Case insensitivity ----
    #[test]
    fn test_maven_case_insensitive_snapshot() {
        assert_eq!(cmpg("1.0-SNAPSHOT", "1.0-snapshot"), Equal);
    }
    #[test]
    fn test_maven_case_insensitive_alpha() {
        assert_eq!(cmpg("1.0-ALPHA", "1.0-alpha"), Equal);
    }
    #[test]
    fn test_maven_case_insensitive_rc() {
        assert_eq!(cmpg("1.0-RC-1", "1.0-rc-1"), Equal);
    }
    #[test]
    fn test_maven_case_insensitive_cr() {
        assert_eq!(cmpg("1.0-CR-1", "1.0-cr-1"), Equal);
    }
    #[test]
    fn test_maven_mixed_case_alphabet() {
        assert_eq!(cmpg("1-abcdefghijklmnopqrstuvwxyz", "1-ABCDEFGHIJKLMNOPQRSTUVWXYZ"), Equal);
    }

    // ---- Trailing zero equivalence ----
    #[test]
    fn test_maven_trailing_zeros_2() {
        assert_eq!(cmpg("1.0", "1.0.0"), Equal);
    }
    #[test]
    fn test_maven_trailing_zeros_3() {
        assert_eq!(cmpg("1", "1.0.0"), Equal);
    }
    #[test]
    fn test_maven_trailing_zeros_many() {
        assert_eq!(cmpg("1.0.0.0.0.0.0", "1"), Equal);
    }

    // ---- Basic numeric ordering ----
    #[test]
    fn test_maven_numeric_1_lt_2() {
        assert_eq!(cmpg("1", "2"), Less);
    }
    #[test]
    fn test_maven_numeric_1_5_lt_2() {
        assert_eq!(cmpg("1.5", "2"), Less);
    }
    #[test]
    fn test_maven_numeric_minor_order() {
        assert_eq!(cmpg("1.0", "1.1"), Less);
    }
    #[test]
    fn test_maven_numeric_patch_order() {
        assert_eq!(cmpg("1.0.0", "1.0.1"), Less);
    }
    #[test]
    fn test_maven_numeric_1_0_1_lt_1_1() {
        assert_eq!(cmpg("1.0.1", "1.1"), Less);
    }
    #[test]
    fn test_maven_numeric_1_1_lt_1_2_0() {
        assert_eq!(cmpg("1.1", "1.2.0"), Less);
    }

    // ---- Pre-release before release ----
    #[test]
    fn test_maven_alpha_before_release() {
        assert_eq!(cmpg("1.0-alpha-1", "1.0"), Less);
    }
    #[test]
    fn test_maven_alpha_snapshot_lt_alpha() {
        assert_eq!(cmpg("1.0-alpha-1-SNAPSHOT", "1.0-alpha-1"), Less);
    }
    #[test]
    fn test_maven_alpha1_lt_alpha2() {
        assert_eq!(cmpg("1.0-alpha-1", "1.0-alpha-2"), Less);
    }
    #[test]
    fn test_maven_beta1_lt_snapshot() {
        assert_eq!(cmpg("1.0-beta-1", "1.0-SNAPSHOT"), Less);
    }

    // ---- Post-release / sp ----
    #[test]
    fn test_maven_release_lt_post_numeric() {
        assert_eq!(cmpg("1.0", "1.0.1"), Less);
    }
    #[test]
    fn test_maven_sp1_lt_sp2() {
        assert_eq!(cmpg("1.0-sp-1", "1.0-sp-2"), Less);
    }

    // ---- Unknown qualifiers ----
    #[test]
    fn test_maven_unknown_qualifier_lexical_order() {
        assert_eq!(cmpg("2.0.1-klm", "2.0.1-lmn"), Less);
    }

    // ---- Real-world: Apache Maven Core lifecycle ----
    #[test]
    fn test_maven_rw_maven_core_alpha_chain() {
        let chain = [
            "2.0-alpha-1",
            "2.0-alpha-2",
            "2.0-alpha-3",
            "2.0-beta-1",
            "2.0-beta-2",
            "2.0-beta-3",
            "2.0",
            "2.0.1",
            "2.0.2",
            "2.0.11",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Maven Core: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Log4j lifecycle ----
    #[test]
    fn test_maven_rw_log4j_lifecycle() {
        let chain = [
            "2.0-alpha1",
            "2.0-alpha2",
            "2.0-beta1",
            "2.0-beta9",
            "2.0-rc1",
            "2.0-rc2",
            "2.0",
            "2.0.1",
            "2.0.2",
            "2.1",
            "2.17.1",
            "2.24.3",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Log4j: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: JUnit 4 lifecycle ----
    #[test]
    fn test_maven_rw_junit4_lifecycle() {
        let chain = [
            "4.12-beta-1",
            "4.12-beta-2",
            "4.12-beta-3",
            "4.12",
            "4.13-beta-1",
            "4.13-rc-1",
            "4.13-rc-2",
            "4.13",
            "4.13.1",
            "4.13.2",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "JUnit4: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: JUnit Jupiter lifecycle ----
    #[test]
    fn test_maven_rw_junit_jupiter_lifecycle() {
        let chain = [
            "5.9.0-M1",
            "5.9.0-RC1",
            "5.9.0",
            "5.9.1",
            "5.9.2",
            "5.9.3",
            "5.10.0-M1",
            "5.10.0-RC1",
            "5.10.0-RC2",
            "5.10.0",
            "5.10.5",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "JUnit Jupiter: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Spring Core milestone lifecycle ----
    #[test]
    fn test_maven_rw_spring_core_milestones() {
        let chain = ["2.0-m3", "2.0-m5", "2.0-rc1", "2.0-rc2", "2.0"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Spring Core: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Guava lifecycle ----
    #[test]
    fn test_maven_rw_guava_rc_chain() {
        let chain = ["14.0-rc1", "14.0-rc2", "14.0-rc3", "14.0", "14.0.1", "15.0-rc1", "15.0"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Guava: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: SLF4J lifecycle ----
    #[test]
    fn test_maven_rw_slf4j_lifecycle() {
        let chain = [
            "2.0.0-alpha0",
            "2.0.0-alpha7",
            "2.0.0-beta0",
            "2.0.0-beta1",
            "2.0.0",
            "2.0.1",
            "2.0.17",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "SLF4J: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Apache Commons Lang3 ----
    #[test]
    fn test_maven_rw_commons_lang3() {
        let chain = ["3.0", "3.0.1", "3.1", "3.4", "3.9", "3.10", "3.11", "3.12.0", "3.20.0"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Commons Lang3: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Jackson Databind ----
    #[test]
    fn test_maven_rw_jackson_four_component() {
        // Jackson uses four-component versions for security patches
        let chain = ["2.6.7", "2.6.7.1", "2.6.7.2", "2.6.7.3", "2.6.7.4", "2.6.7.5"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Jackson: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    #[test]
    fn test_maven_rw_jackson_rc_lifecycle() {
        // Jackson switched from .rc (dot) to -rc (hyphen) across versions
        let chain = ["2.12.0-rc1", "2.12.0-rc2", "2.12.0", "2.12.7", "2.12.7.1", "2.12.7.2"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Jackson: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Kafka Clients ----
    #[test]
    fn test_maven_rw_kafka_four_component() {
        // Kafka used four-component versions in early releases
        let chain = [
            "0.8.2-beta",
            "0.9.0.0",
            "0.9.0.1",
            "0.10.0.0",
            "0.10.2.2",
            "0.11.0.3",
            "1.0.0",
            "2.0.0",
            "3.0.0",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Kafka: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Log4j 3.0.0 alpha/beta lifecycle ----
    #[test]
    fn test_maven_rw_log4j3_prerelease() {
        let chain = ["3.0.0-alpha1", "3.0.0-beta1", "3.0.0-beta2", "3.0.0-beta3"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Log4j 3: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Maven Core 4.x prerelease ----
    #[test]
    fn test_maven_rw_maven4_prerelease() {
        let chain = [
            "4.0.0-alpha-2",
            "4.0.0-alpha-13",
            "4.0.0-beta-3",
            "4.0.0-beta-5",
            "4.0.0-rc-1",
            "4.0.0-rc-5",
        ];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Maven 4: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Commons IO with mixed component counts ----
    #[test]
    fn test_maven_rw_commons_io() {
        let chain =
            ["0.1", "1.0", "1.3.2", "1.4", "2.0", "2.0.1", "2.7", "2.8.0", "2.16.1", "2.21.0"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Commons IO: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Hibernate dot-separated qualifiers ----
    #[test]
    fn test_maven_rw_hibernate_lifecycle() {
        // Hibernate uses .Alpha, .Beta, .CR, .Final with dot separators
        let chain = ["6.6.0.Alpha1", "6.6.0.CR1", "6.6.0.CR2"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Hibernate: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: Netty dot-separated qualifiers ----
    #[test]
    fn test_maven_rw_netty_lifecycle() {
        // Netty uses .Alpha, .Beta, .CR with dot separators
        let chain = ["4.2.0.Alpha1", "4.2.0.Alpha5", "4.2.0.Beta1", "4.2.0.RC1", "4.2.0.RC4"];
        for i in 0..chain.len() - 1 {
            assert_eq!(
                cmpg(chain[i], chain[i + 1]),
                Less,
                "Netty: {} should be < {}",
                chain[i],
                chain[i + 1]
            );
        }
    }

    // ---- Real-world: timestamp version (Commons IO) ----
    #[test]
    fn test_maven_rw_timestamp_version() {
        // Commons IO had a timestamp version; it's just a huge numeric value
        assert_eq!(cmpg("2.21.0", "20030203.000550"), Less);
    }

    // ---- Maven ecosystem dispatch: real-world ----
    #[test]
    fn test_maven_eco_log4j_chain() {
        assert_eq!(compare_str_with_ecosystem("2.0-alpha1", "2.0-rc1", "maven").unwrap(), Less);
    }
    #[test]
    fn test_maven_eco_junit_milestone() {
        assert_eq!(compare_str_with_ecosystem("5.10.0-M1", "5.10.0-RC1", "maven").unwrap(), Less);
    }
    #[test]
    fn test_maven_eco_rc_lt_release() {
        assert_eq!(compare_str_with_ecosystem("5.10.0-RC1", "5.10.0", "maven").unwrap(), Less);
    }
    #[test]
    fn test_maven_eco_jackson_four_component() {
        assert_eq!(compare_str_with_ecosystem("2.6.7", "2.6.7.1", "maven").unwrap(), Less);
    }
    #[test]
    fn test_maven_eco_mvn_alias() {
        assert_eq!(compare_str_with_ecosystem("1.0-alpha", "1.0", "mvn").unwrap(), Less);
    }

    // ---- Maven validation ----
    #[test]
    fn test_maven_eco_reject_empty() {
        assert!(compare_str_with_ecosystem("", "1.0", "maven").is_err());
    }
    #[test]
    fn test_maven_eco_reject_non_digit_start() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "maven").is_err());
    }
    #[test]
    fn test_maven_eco_reject_v_prefix() {
        // Maven requires digit start; 'v1.0' starts with 'v'
        assert!(compare_str_with_ecosystem("v1.0", "1.0", "maven").is_err());
    }

    // ---- Maven properties via parser ----
    #[test]
    fn test_maven_parser_prerelease_alpha() {
        let v = parse("1.0-alpha-1");
        assert!(v.is_prerelease);
        assert!(!v.is_postrelease);
    }
    #[test]
    fn test_maven_parser_prerelease_snapshot() {
        let v = parse("1.0-SNAPSHOT");
        assert!(v.is_prerelease);
    }
    #[test]
    fn test_maven_parser_postrelease_sp() {
        let v = parse("1.0-sp-1");
        assert!(v.is_postrelease);
    }
    #[test]
    fn test_maven_parser_release_no_qualifier() {
        let v = parse("2.0.1");
        assert!(!v.is_prerelease);
        assert!(!v.is_postrelease);
    }
    #[test]
    fn test_maven_parser_segments_alpha() {
        let v = parse("4.0.0-alpha-13");
        assert_eq!(v.segments.len(), 5);
        assert_eq!(v.segments[0], Seg::Num(4));
        assert_eq!(v.segments[1], Seg::Num(0));
        assert_eq!(v.segments[2], Seg::Num(0));
        assert_eq!(v.segments[3], Seg::Text("alpha".into()));
        assert_eq!(v.segments[4], Seg::Num(13));
    }
    #[test]
    fn test_maven_parser_segments_snapshot() {
        let v = parse("1.0-SNAPSHOT");
        assert_eq!(v.segments.len(), 3);
        assert_eq!(v.segments[0], Seg::Num(1));
        assert_eq!(v.segments[1], Seg::Num(0));
        assert_eq!(v.segments[2], Seg::Text("snapshot".into()));
    }
    #[test]
    fn test_maven_parser_four_component() {
        let v = parse("2.6.7.5");
        assert_eq!(v.segments.len(), 4);
        assert_eq!(v.segments[3], Seg::Num(5));
    }

    // --- Real-world packages ---

    #[test]
    fn test_rw_django_major() {
        assert_eq!(cmpg("4.2", "5.0a1"), Less);
    }
    #[test]
    fn test_rw_django_alpha_lt_beta() {
        assert_eq!(cmpg("5.0a1", "5.0b1"), Less);
    }
    #[test]
    fn test_rw_django_rc_lt_rel() {
        assert_eq!(cmpg("5.0rc1", "5.0"), Less);
    }
    #[test]
    fn test_rw_rails_beta_lt_rc() {
        assert_eq!(cmpg("7.1.0.beta1", "7.1.0.rc1"), Less);
    }
    #[test]
    fn test_rw_rails_rc_lt_rel() {
        assert_eq!(cmpg("7.1.0.rc1", "7.1.0"), Less);
    }
    #[test]
    fn test_rw_node() {
        assert_eq!(cmpg("18.17.0", "20.0.0"), Less);
    }
    #[test]
    fn test_rw_kernel() {
        assert_eq!(cmpg("5.15.0", "6.1.0"), Less);
    }
    #[test]
    fn test_rw_spring() {
        assert_eq!(cmpg("5.3.30", "6.0.0"), Less);
    }
    #[test]
    fn test_rw_requests() {
        assert_eq!(cmpg("2.31.0", "2.32.0"), Less);
    }

    // --- Parser properties ---

    #[test]
    fn test_parser_segments() {
        let v = parse("1.2.3-alpha.1+build.42");
        assert_eq!(v.epoch, 0);
        assert_eq!(v.build, "build.42");
        assert!(v.is_prerelease);
        assert!(!v.is_postrelease);
        assert_eq!(v.segments.len(), 5);
        assert_eq!(v.segments[0], Seg::Num(1));
        assert_eq!(v.segments[3], Seg::Text("alpha".into()));
        assert_eq!(v.segments[4], Seg::Num(1));
    }

    #[test]
    fn test_parser_epoch_bang() {
        let v = parse("1!2.3");
        assert_eq!(v.epoch, 1);
        assert_eq!(v.segments, vec![Seg::Num(2), Seg::Num(3)]);
    }

    #[test]
    fn test_parser_epoch_colon() {
        let v = parse("1:2.3");
        assert_eq!(v.epoch, 1);
        assert_eq!(v.segments, vec![Seg::Num(2), Seg::Num(3)]);
    }

    #[test]
    fn test_parser_postrelease() {
        let v = parse("1.0.post1");
        assert!(v.is_postrelease);
        assert!(!v.is_prerelease);
    }

    #[test]
    fn test_parser_caret_postrelease() {
        let v = parse("1.0^git1");
        assert!(v.is_postrelease);
    }

    #[test]
    fn test_parser_tilde_prerelease() {
        let v = parse("1.0~rc1");
        assert!(v.is_prerelease);
    }

    #[test]
    fn test_parser_empty() {
        let v = parse("");
        assert_eq!(v.segments.len(), 0);
        assert_eq!(v.epoch, 0);
        assert_eq!(v.build, "");
    }

    #[test]
    fn test_parser_vprefix() {
        let v = parse("v1.0.0");
        assert_eq!(v.segments, vec![Seg::Num(1), Seg::Num(0), Seg::Num(0)]);
    }

    #[test]
    fn test_normalization_consistency() {
        let pa = parse("1.0");
        let pb = parse("1.0.0");
        let a = normalized(&pa.segments);
        let b = normalized(&pb.segments);
        assert_eq!(a, b);
    }

    // --- SemVer strict ---

    #[test]
    fn test_semver_strict_alpha_lt_alpha1() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("1.0.0-alpha").unwrap(),
                &parse_semver_strict("1.0.0-alpha.1").unwrap()
            ),
            Less
        );
    }

    #[test]
    fn test_semver_strict_alpha1_lt_alpha_beta() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("1.0.0-alpha.1").unwrap(),
                &parse_semver_strict("1.0.0-alpha.beta").unwrap()
            ),
            Less
        );
    }

    #[test]
    fn test_semver_strict_beta2_lt_beta11() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("1.0.0-beta.2").unwrap(),
                &parse_semver_strict("1.0.0-beta.11").unwrap()
            ),
            Less
        );
    }

    #[test]
    fn test_semver_strict_rc_lt_release() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("1.0.0-rc.1").unwrap(),
                &parse_semver_strict("1.0.0").unwrap()
            ),
            Less
        );
    }

    #[test]
    fn test_semver_strict_build_ignored() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("1.0.0+build").unwrap(),
                &parse_semver_strict("1.0.0").unwrap()
            ),
            Equal
        );
    }

    #[test]
    fn test_semver_strict_basic() {
        assert_eq!(
            cmp_semver_strict(
                &parse_semver_strict("0.0.0").unwrap(),
                &parse_semver_strict("0.0.1").unwrap()
            ),
            Less
        );
    }

    #[test]
    fn test_semver_strict_invalid() {
        assert!(parse_semver_strict("1.0").is_err());
    }

    // --- Unknown text ordering ---

    #[test]
    fn test_unknown_text_lt_release() {
        assert_eq!(cmpg("1.0.0.foo", "1.0.0"), Less);
    }
    #[test]
    fn test_unknown_text_gt_known_pre() {
        assert_eq!(cmpg("1.0.0-foo", "1.0.0-rc"), Greater);
    }
    #[test]
    fn test_unknown_text_lt_known_post() {
        assert_eq!(cmpg("1.0.0-foo", "1.0.0-post"), Less);
    }

    // --- Numeric overflow (Fix #1) ---

    #[test]
    fn test_overflow_saturates() {
        // A number that overflows u64 should saturate at u64::MAX, not become 0
        let v = parse("99999999999999999999999999999999");
        assert_eq!(v.segments, vec![Seg::Num(u64::MAX)]);
    }

    #[test]
    fn test_overflow_greater_than_normal() {
        // Saturated version should be greater than any normal version
        assert_eq!(cmpg("99999999999999999999999999999999", "999"), Greater);
    }

    // --- Strict SemVer validation (Fix #2) ---

    #[test]
    fn test_semver_strict_leading_zero_major() {
        assert!(parse_semver_strict("01.0.0").is_err());
    }

    #[test]
    fn test_semver_strict_leading_zero_minor() {
        assert!(parse_semver_strict("1.01.0").is_err());
    }

    #[test]
    fn test_semver_strict_leading_zero_patch() {
        assert!(parse_semver_strict("1.0.01").is_err());
    }

    #[test]
    fn test_semver_strict_leading_zero_prerelease() {
        assert!(parse_semver_strict("1.0.0-01").is_err());
    }

    #[test]
    fn test_semver_strict_empty_prerelease() {
        assert!(parse_semver_strict("1.0.0-").is_err());
    }

    #[test]
    fn test_semver_strict_empty_prerelease_ident() {
        assert!(parse_semver_strict("1.0.0-alpha..1").is_err());
    }

    #[test]
    fn test_semver_strict_valid_prerelease() {
        let sv = parse_semver_strict("1.0.0-alpha.1.x-y").unwrap();
        assert_eq!(sv.major, 1);
        assert_eq!(sv.pre, Some("alpha.1.x-y".to_string()));
    }

    #[test]
    fn test_semver_strict_zero_major_ok() {
        let sv = parse_semver_strict("0.0.0").unwrap();
        assert_eq!(sv.major, 0);
    }

    #[test]
    fn test_semver_strict_zero_prerelease_ok() {
        let sv = parse_semver_strict("1.0.0-0").unwrap();
        assert_eq!(sv.pre, Some("0".to_string()));
    }

    #[test]
    fn test_ecosystem_dispatch_generic_vs_semver() {
        assert_eq!(cmpg("1.0.0-alpha.1", "1.0.0-alpha.beta"), Greater);
        assert_eq!(
            compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "semver").unwrap(),
            Less
        );
    }

    #[test]
    fn test_ecosystem_dispatch_invalid_name() {
        assert!(compare_str_with_ecosystem("1.0.0", "2.0.0", "invalid").is_err());
    }

    // --- Ecosystem enum parsing ---

    #[test]
    fn test_ecosystem_pep440_aliases() {
        assert_eq!(Ecosystem::from_str("pep440").unwrap(), Ecosystem::Pep440);
        assert_eq!(Ecosystem::from_str("pep-440").unwrap(), Ecosystem::Pep440);
        assert_eq!(Ecosystem::from_str("python").unwrap(), Ecosystem::Pep440);
    }

    #[test]
    fn test_ecosystem_debian_aliases() {
        assert_eq!(Ecosystem::from_str("debian").unwrap(), Ecosystem::Debian);
        assert_eq!(Ecosystem::from_str("dpkg").unwrap(), Ecosystem::Debian);
        assert_eq!(Ecosystem::from_str("deb").unwrap(), Ecosystem::Debian);
    }

    #[test]
    fn test_ecosystem_rpm_aliases() {
        assert_eq!(Ecosystem::from_str("rpm").unwrap(), Ecosystem::Rpm);
        assert_eq!(Ecosystem::from_str("redhat").unwrap(), Ecosystem::Rpm);
    }

    #[test]
    fn test_ecosystem_ruby_aliases() {
        assert_eq!(Ecosystem::from_str("ruby").unwrap(), Ecosystem::Ruby);
        assert_eq!(Ecosystem::from_str("gem").unwrap(), Ecosystem::Ruby);
        assert_eq!(Ecosystem::from_str("rubygems").unwrap(), Ecosystem::Ruby);
    }

    #[test]
    fn test_ecosystem_maven_aliases() {
        assert_eq!(Ecosystem::from_str("maven").unwrap(), Ecosystem::Maven);
        assert_eq!(Ecosystem::from_str("mvn").unwrap(), Ecosystem::Maven);
    }

    #[test]
    fn test_ecosystem_go_aliases() {
        assert_eq!(Ecosystem::from_str("go").unwrap(), Ecosystem::Go);
        assert_eq!(Ecosystem::from_str("golang").unwrap(), Ecosystem::Go);
    }

    // --- PEP 440 ecosystem dispatch ---

    #[test]
    fn test_pep440_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0a1", "1.0a2", "pep440").unwrap(), Less);
    }

    #[test]
    fn test_pep440_eco_dev_lt_alpha() {
        assert_eq!(compare_str_with_ecosystem("1.0.dev1", "1.0a1", "pep440").unwrap(), Less);
    }

    #[test]
    fn test_pep440_eco_rc_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0rc1", "1.0", "pep440").unwrap(), Less);
    }

    #[test]
    fn test_pep440_eco_rel_lt_post() {
        assert_eq!(compare_str_with_ecosystem("1.0", "1.0.post1", "pep440").unwrap(), Less);
    }

    #[test]
    fn test_pep440_eco_epoch() {
        assert_eq!(compare_str_with_ecosystem("1!0.1", "2.0", "pep440").unwrap(), Greater);
    }

    #[test]
    fn test_pep440_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "pep440").is_err());
    }

    // --- Debian ecosystem dispatch ---

    #[test]
    fn test_debian_eco_tilde_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0~alpha", "1.0", "debian").unwrap(), Less);
    }

    #[test]
    fn test_debian_eco_tilde_ordering() {
        assert_eq!(compare_str_with_ecosystem("1.0~alpha", "1.0~beta", "debian").unwrap(), Less);
    }

    #[test]
    fn test_debian_eco_epoch() {
        assert_eq!(compare_str_with_ecosystem("1:0.1", "2.0", "debian").unwrap(), Greater);
    }

    #[test]
    fn test_debian_eco_epoch_compare() {
        assert_eq!(compare_str_with_ecosystem("2:1.0", "1:2.0", "debian").unwrap(), Greater);
    }

    #[test]
    fn test_debian_eco_plus_not_stripped() {
        // In Debian, + is NOT build metadata — it's part of the version
        assert_eq!(compare_str_with_ecosystem("1.0+deb9u1", "1.0+deb9u2", "debian").unwrap(), Less);
    }

    #[test]
    fn test_debian_eco_revision() {
        // revision is everything after last '-'
        assert_eq!(compare_str_with_ecosystem("1.0-1", "1.0-2", "debian").unwrap(), Less);
    }

    // --- RPM ecosystem dispatch ---

    #[test]
    fn test_rpm_eco_tilde_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0~rc1", "1.0", "rpm").unwrap(), Less);
    }

    #[test]
    fn test_rpm_eco_release_lt_caret() {
        assert_eq!(compare_str_with_ecosystem("1.0", "1.0^git1", "rpm").unwrap(), Less);
    }

    #[test]
    fn test_rpm_eco_basic_numeric() {
        assert_eq!(compare_str_with_ecosystem("1.0", "2.0", "rpm").unwrap(), Less);
    }

    #[test]
    fn test_rpm_eco_equal() {
        assert_eq!(compare_str_with_ecosystem("1.0", "1.0", "rpm").unwrap(), Equal);
    }

    #[test]
    fn test_rpm_eco_epoch() {
        assert_eq!(compare_str_with_ecosystem("1:1.0", "2.0", "rpm").unwrap(), Greater);
    }

    // --- Ruby ecosystem dispatch ---

    #[test]
    fn test_ruby_eco_pre_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0.0.pre", "1.0.0", "ruby").unwrap(), Less);
    }

    #[test]
    fn test_ruby_eco_alpha_lt_beta() {
        assert_eq!(compare_str_with_ecosystem("1.0.0.alpha", "1.0.0.beta", "ruby").unwrap(), Less);
    }

    #[test]
    fn test_ruby_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "ruby").is_err());
    }

    // --- Maven ecosystem dispatch ---

    #[test]
    fn test_maven_eco_alpha_lt_beta() {
        assert_eq!(compare_str_with_ecosystem("1.0-alpha-1", "1.0-beta-1", "maven").unwrap(), Less);
    }

    #[test]
    fn test_maven_eco_snapshot_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0-SNAPSHOT", "1.0", "maven").unwrap(), Less);
    }

    #[test]
    fn test_maven_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "maven").is_err());
    }

    // --- Go ecosystem dispatch ---

    #[test]
    fn test_go_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("v1.0.0", "v1.0.1", "go").unwrap(), Less);
    }

    #[test]
    fn test_go_eco_alpha_lt_release() {
        assert_eq!(compare_str_with_ecosystem("v1.0.0-alpha", "v1.0.0", "go").unwrap(), Less);
    }

    #[test]
    fn test_go_eco_invalid() {
        // Go requires valid SemVer
        assert!(compare_str_with_ecosystem("v1.0", "v2.0", "go").is_err());
    }

    // --- dpkg verrevcmp unit tests ---

    #[test]
    fn test_verrevcmp_equal() {
        assert_eq!(verrevcmp(b"1.0", b"1.0"), Equal);
    }

    #[test]
    fn test_verrevcmp_numeric() {
        assert_eq!(verrevcmp(b"1.1", b"1.2"), Less);
    }

    #[test]
    fn test_verrevcmp_tilde_lt_empty() {
        assert_eq!(verrevcmp(b"1.0~alpha", b"1.0"), Less);
    }

    #[test]
    fn test_verrevcmp_tilde_ordering() {
        assert_eq!(verrevcmp(b"1.0~a", b"1.0~b"), Less);
    }

    #[test]
    fn test_verrevcmp_plus_significant() {
        // + is NOT stripped in dpkg — it's a regular character
        assert_eq!(verrevcmp(b"1.0+deb9u1", b"1.0+deb9u2"), Less);
    }

    // --- rpmverscmp unit tests ---

    #[test]
    fn test_rpmverscmp_equal() {
        assert_eq!(rpmverscmp("1.0", "1.0"), Equal);
    }

    #[test]
    fn test_rpmverscmp_numeric() {
        assert_eq!(rpmverscmp("1.1", "1.2"), Less);
    }

    #[test]
    fn test_rpmverscmp_tilde() {
        assert_eq!(rpmverscmp("1.0~rc1", "1.0"), Less);
    }

    #[test]
    fn test_rpmverscmp_caret() {
        assert_eq!(rpmverscmp("1.0", "1.0^git1"), Less);
    }

    #[test]
    fn test_rpmverscmp_digits_gt_alpha() {
        assert_eq!(rpmverscmp("1.0.1", "1.0.a"), Greater);
    }

    // --- npm ecosystem dispatch (delegates to SemVer) ---

    #[test]
    fn test_npm_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "npm").unwrap(), Less);
    }

    #[test]
    fn test_npm_eco_prerelease() {
        assert_eq!(
            compare_str_with_ecosystem("1.0.0-alpha.1", "1.0.0-alpha.beta", "npm").unwrap(),
            Less
        );
    }

    #[test]
    fn test_npm_eco_invalid() {
        assert!(compare_str_with_ecosystem("1.0", "2.0", "npm").is_err());
    }

    #[test]
    fn test_npm_aliases() {
        assert_eq!(Ecosystem::from_str("npm").unwrap(), Ecosystem::Npm);
        assert_eq!(Ecosystem::from_str("node").unwrap(), Ecosystem::Npm);
        assert_eq!(Ecosystem::from_str("nodejs").unwrap(), Ecosystem::Npm);
    }

    // --- NuGet ecosystem dispatch ---

    #[test]
    fn test_nuget_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "nuget").unwrap(), Less);
    }

    #[test]
    fn test_nuget_eco_four_segments() {
        // NuGet allows 4 numeric segments
        assert_eq!(compare_str_with_ecosystem("1.0.0.0", "1.0.0.1", "nuget").unwrap(), Less);
    }

    #[test]
    fn test_nuget_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "nuget").is_err());
    }

    #[test]
    fn test_nuget_aliases() {
        assert_eq!(Ecosystem::from_str("nuget").unwrap(), Ecosystem::Nuget);
        assert_eq!(Ecosystem::from_str("dotnet").unwrap(), Ecosystem::Nuget);
    }

    // --- Composer ecosystem dispatch ---

    #[test]
    fn test_composer_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "2.0.0", "composer").unwrap(), Less);
    }

    #[test]
    fn test_composer_eco_stability() {
        assert_eq!(
            compare_str_with_ecosystem("1.0.0-alpha", "1.0.0-beta", "composer").unwrap(),
            Less
        );
    }

    #[test]
    fn test_composer_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "composer").is_err());
    }

    #[test]
    fn test_composer_aliases() {
        assert_eq!(Ecosystem::from_str("composer").unwrap(), Ecosystem::Composer);
        assert_eq!(Ecosystem::from_str("packagist").unwrap(), Ecosystem::Composer);
        assert_eq!(Ecosystem::from_str("php").unwrap(), Ecosystem::Composer);
    }

    // --- Crates.io ecosystem dispatch (delegates to SemVer) ---

    #[test]
    fn test_crates_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "crates").unwrap(), Less);
    }

    #[test]
    fn test_crates_eco_prerelease() {
        assert_eq!(compare_str_with_ecosystem("1.0.0-alpha", "1.0.0", "crates").unwrap(), Less);
    }

    #[test]
    fn test_crates_eco_invalid() {
        assert!(compare_str_with_ecosystem("1.0", "2.0", "crates").is_err());
    }

    #[test]
    fn test_crates_aliases() {
        assert_eq!(Ecosystem::from_str("crates").unwrap(), Ecosystem::Crates);
        assert_eq!(Ecosystem::from_str("cargo").unwrap(), Ecosystem::Crates);
        assert_eq!(Ecosystem::from_str("crates.io").unwrap(), Ecosystem::Crates);
    }

    // --- Hex ecosystem dispatch (delegates to SemVer) ---

    #[test]
    fn test_hex_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "hex").unwrap(), Less);
    }

    #[test]
    fn test_hex_eco_prerelease() {
        assert_eq!(compare_str_with_ecosystem("1.0.0-rc.1", "1.0.0", "hex").unwrap(), Less);
    }

    #[test]
    fn test_hex_aliases() {
        assert_eq!(Ecosystem::from_str("hex").unwrap(), Ecosystem::Hex);
        assert_eq!(Ecosystem::from_str("elixir").unwrap(), Ecosystem::Hex);
        assert_eq!(Ecosystem::from_str("erlang").unwrap(), Ecosystem::Hex);
    }

    // --- Swift ecosystem dispatch (delegates to SemVer) ---

    #[test]
    fn test_swift_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "2.0.0", "swift").unwrap(), Less);
    }

    #[test]
    fn test_swift_eco_invalid() {
        assert!(compare_str_with_ecosystem("1.0", "2.0", "swift").is_err());
    }

    #[test]
    fn test_swift_aliases() {
        assert_eq!(Ecosystem::from_str("swift").unwrap(), Ecosystem::Swift);
        assert_eq!(Ecosystem::from_str("swiftpm").unwrap(), Ecosystem::Swift);
    }

    // --- CalVer ecosystem dispatch ---

    #[test]
    fn test_calver_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("2024.01", "2024.02", "calver").unwrap(), Less);
    }

    #[test]
    fn test_calver_eco_yyyymmdd() {
        assert_eq!(compare_str_with_ecosystem("20240115", "20240201", "calver").unwrap(), Less);
    }

    #[test]
    fn test_calver_eco_with_micro() {
        assert_eq!(compare_str_with_ecosystem("2024.1.0", "2024.1.1", "calver").unwrap(), Less);
    }

    #[test]
    fn test_calver_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "2024.1", "calver").is_err());
    }

    // --- Alpine ecosystem dispatch ---

    #[test]
    fn test_alpine_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.0.0", "1.0.1", "alpine").unwrap(), Less);
    }

    #[test]
    fn test_alpine_eco_suffix_alpha() {
        assert_eq!(compare_str_with_ecosystem("1.0_alpha1", "1.0_beta1", "alpine").unwrap(), Less);
    }

    #[test]
    fn test_alpine_eco_suffix_pre_lt_release() {
        assert_eq!(compare_str_with_ecosystem("1.0_rc1", "1.0", "alpine").unwrap(), Less);
    }

    #[test]
    fn test_alpine_eco_invalid() {
        assert!(compare_str_with_ecosystem("abc", "1.0", "alpine").is_err());
    }

    #[test]
    fn test_alpine_aliases() {
        assert_eq!(Ecosystem::from_str("alpine").unwrap(), Ecosystem::Alpine);
        assert_eq!(Ecosystem::from_str("apk").unwrap(), Ecosystem::Alpine);
    }

    // --- Docker ecosystem dispatch (delegates to Generic) ---

    #[test]
    fn test_docker_eco_basic() {
        assert_eq!(compare_str_with_ecosystem("1.25.3", "1.25.4", "docker").unwrap(), Less);
    }

    #[test]
    fn test_docker_eco_calver() {
        assert_eq!(compare_str_with_ecosystem("24.04", "24.10", "docker").unwrap(), Less);
    }

    #[test]
    fn test_docker_eco_permissive() {
        // Docker accepts anything — no validation
        assert!(compare_str_with_ecosystem("latest", "stable", "docker").is_ok());
    }

    #[test]
    fn test_docker_aliases() {
        assert_eq!(Ecosystem::from_str("docker").unwrap(), Ecosystem::Docker);
        assert_eq!(Ecosystem::from_str("oci").unwrap(), Ecosystem::Docker);
    }
}
