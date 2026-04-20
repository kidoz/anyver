#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Seg {
    Num(u64),
    Text(String),
}

pub(crate) fn tag_weight(s: &str) -> Option<i32> {
    match s {
        "~" => Some(-10),
        "dev" => Some(-5),
        "alpha" | "a" => Some(0),
        "beta" | "b" => Some(10),
        "milestone" | "m" => Some(15),
        "rc" | "cr" | "c" | "preview" | "pre" => Some(20),
        "snapshot" => Some(25),
        // Release-aliases: Maven `final`/`ga`/`release` and PEP 440 `r`/`rev`
        // mean "the release itself" — weight exactly 30 so they compare equal
        // to no qualifier. `normalized()` strips them trailing-wise.
        "final" | "ga" | "release" => Some(30),
        "^" => Some(32),
        "post" | "sp" | "patch" | "p" => Some(35),
        _ => None,
    }
}

pub(crate) fn is_prerelease_tag(s: &str) -> bool {
    tag_weight(s).is_some_and(|w| w < 30)
}

pub(crate) fn is_postrelease_tag(s: &str) -> bool {
    tag_weight(s).is_some_and(|w| w > 30)
}

pub(crate) fn strip_v(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() > 1 && (b[0] == b'v' || b[0] == b'V') && b[1].is_ascii_digit() { &s[1..] } else { s }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Ecosystem {
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
    pub(crate) fn from_str(input: &str) -> Result<Self, String> {
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
            other => Err(format!(
                "unsupported ecosystem '{other}'; expected one of: generic, semver, pep440, debian, rpm, ruby, maven, go, npm, nuget, composer, crates, hex, swift, calver, alpine, docker"
            )),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
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

pub(crate) fn autodetect_ecosystem(version: &str) -> Ecosystem {
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
    // Check .elN, .fcN, .amznN — require trailing digit to avoid false positives
    // (e.g. "1.0.elegant" should not match .el)
    for (marker, mlen) in [(".el", 3), (".fc", 3), (".amzn", 5)] {
        if let Some(pos) = lower.find(marker) {
            let after = pos + mlen;
            if after < lower.len() && lower.as_bytes()[after].is_ascii_digit() {
                return Ecosystem::Rpm;
            }
        }
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
pub(crate) struct ParsedVersion {
    pub(crate) raw: String,
    pub(crate) epoch: u64,
    pub(crate) segments: Vec<Seg>,
    pub(crate) build: String,
    pub(crate) is_prerelease: bool,
    pub(crate) is_postrelease: bool,
}

pub(crate) fn parse_generic(input: &str) -> ParsedVersion {
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

pub(crate) fn parse(input: &str) -> ParsedVersion {
    parse_generic(input)
}
