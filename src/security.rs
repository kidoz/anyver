//! Security-interoperability helpers for PURL and VERS strings.

use crate::parser::Ecosystem;
use crate::strategies::{ParsedRepr, compare_for_ecosystem, parse_for_ecosystem};
use std::cmp::Ordering;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Purl {
    pub(crate) package_type: String,
    pub(crate) namespace: Option<String>,
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) qualifiers: Option<String>,
    pub(crate) subpath: Option<String>,
}

impl Purl {
    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let rest = input
            .strip_prefix("pkg:")
            .ok_or_else(|| format!("invalid purl '{input}'; expected 'pkg:' prefix"))?;
        let (without_subpath, subpath) = match rest.split_once('#') {
            Some((base, sub)) => (base, Some(sub.to_string())),
            None => (rest, None),
        };
        let (without_qualifiers, qualifiers) = match without_subpath.split_once('?') {
            Some((base, query)) => (base, Some(query.to_string())),
            None => (without_subpath, None),
        };
        let (package_type, path) = without_qualifiers
            .split_once('/')
            .ok_or_else(|| format!("invalid purl '{input}'; expected package type and name"))?;
        if package_type.is_empty() || path.is_empty() {
            return Err(format!("invalid purl '{input}'; package type and name are required"));
        }
        let (name_path, version) = match without_qualifiers.rsplit_once('@') {
            Some((left, ver)) if left.contains('/') => {
                let (_, path_without_type) = left.split_once('/').unwrap_or(("", left));
                (path_without_type, Some(ver.to_string()))
            }
            _ => (path, None),
        };
        let mut parts: Vec<&str> = name_path.split('/').filter(|p| !p.is_empty()).collect();
        let name = parts
            .pop()
            .ok_or_else(|| format!("invalid purl '{input}'; missing package name"))?
            .to_string();
        let namespace = if parts.is_empty() { None } else { Some(parts.join("/")) };
        Ok(Self {
            package_type: package_type.to_ascii_lowercase(),
            namespace,
            name,
            version,
            qualifiers,
            subpath,
        })
    }

    pub(crate) fn ecosystem(&self) -> Option<Ecosystem> {
        match self.package_type.as_str() {
            "generic" => Some(Ecosystem::Generic),
            "npm" => Some(Ecosystem::Npm),
            "pypi" | "python" => Some(Ecosystem::Pep440),
            "deb" | "debian" => Some(Ecosystem::Debian),
            "rpm" => Some(Ecosystem::Rpm),
            "gem" | "rubygems" => Some(Ecosystem::Ruby),
            "maven" => Some(Ecosystem::Maven),
            "golang" | "go" => Some(Ecosystem::Go),
            "nuget" => Some(Ecosystem::Nuget),
            "composer" => Some(Ecosystem::Composer),
            "cargo" | "crates" => Some(Ecosystem::Crates),
            "hex" => Some(Ecosystem::Hex),
            "swift" => Some(Ecosystem::Swift),
            "alpine" | "apk" => Some(Ecosystem::Alpine),
            "docker" | "oci" => Some(Ecosystem::Docker),
            "conda" => Some(Ecosystem::Conda),
            "arch" | "pacman" => Some(Ecosystem::Arch),
            "gentoo" | "portage" => Some(Ecosystem::Gentoo),
            "cran" | "r" => Some(Ecosystem::Cran),
            "spack" => Some(Ecosystem::Spack),
            "pub" | "dart" => Some(Ecosystem::Dart),
            "julia" => Some(Ecosystem::Julia),
            "terraform" | "opentofu" => Some(Ecosystem::Terraform),
            "helm" => Some(Ecosystem::Helm),
            "cpan" | "perl" => Some(Ecosystem::Cpan),
            "nix" | "nixpkgs" => Some(Ecosystem::Nix),
            "brew" | "homebrew" => Some(Ecosystem::Homebrew),
            "freebsd" | "pkg" => Some(Ecosystem::Freebsd),
            _ => None,
        }
    }

    pub(crate) fn to_purl(&self) -> String {
        let mut out = format!("pkg:{}/", self.package_type);
        if let Some(namespace) = &self.namespace {
            out.push_str(namespace);
            out.push('/');
        }
        out.push_str(&self.name);
        if let Some(version) = &self.version {
            out.push('@');
            out.push_str(version);
        }
        if let Some(qualifiers) = &self.qualifiers {
            out.push('?');
            out.push_str(qualifiers);
        }
        if let Some(subpath) = &self.subpath {
            out.push('#');
            out.push_str(subpath);
        }
        out
    }
}

/// Map an OSV `affected.package.ecosystem` value onto a comparison algorithm.
///
/// OSV qualifies distro ecosystems with a release (`Debian:11`, `Alpine:v3.18`,
/// `Ubuntu:22.04:LTS`); only the leading name selects the algorithm. Names with
/// no anyver equivalent (`GitHub Actions`, `Android`, `Linux`, `OSS-Fuzz`, …)
/// fall back to the generic comparator: an unrecognised ecosystem should not
/// fail the whole record, since the versions in it are usually still ordered
/// the way the generic parser reads them.
pub(crate) fn osv_ecosystem(name: &str) -> Ecosystem {
    let base = name.split(':').next().unwrap_or(name).trim().to_ascii_lowercase();
    match base.as_str() {
        "ubuntu" => Ecosystem::Debian,
        "rocky linux" | "almalinux" | "red hat" | "opensuse" | "suse" | "mageia" | "openeuler"
        | "photon os" => Ecosystem::Rpm,
        "wolfi" | "chainguard" | "alpaquita" | "minimos" => Ecosystem::Alpine,
        "bioconductor" => Ecosystem::Cran,
        "swifturl" => Ecosystem::Swift,
        other => Ecosystem::from_str(other).unwrap_or(Ecosystem::Generic),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VersOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl VersOp {
    fn as_str(self) -> &'static str {
        match self {
            Self::Eq => "",
            Self::Ne => "!=",
            Self::Lt => "<",
            Self::Le => "<=",
            Self::Gt => ">",
            Self::Ge => ">=",
        }
    }

    fn opens_upward(self) -> bool {
        matches!(self, Self::Gt | Self::Ge)
    }

    fn opens_downward(self) -> bool {
        matches!(self, Self::Lt | Self::Le)
    }

    fn is_inclusive(self) -> bool {
        matches!(self, Self::Ge | Self::Le)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VersConstraint {
    pub(crate) op: VersOp,
    pub(crate) version: String,
}

/// A parsed `vers:` range, per the PURL-spec VERS grammar:
/// `vers:<versioning-scheme>/<constraint>|<constraint>|…`.
///
/// Constraints are an *unordered set* evaluated by the spec's containment
/// algorithm — not a conjunction. `vers:npm/1.2.3|>=2.0.0|<5.0.0` matches
/// 1.2.3 as well as everything in [2.0.0, 5.0.0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Vers {
    pub(crate) ecosystem: Ecosystem,
    /// Empty when the range is the `*` wildcard, which matches every version.
    pub(crate) constraints: Vec<VersConstraint>,
    pub(crate) any: bool,
}

fn parse_vers_constraint(token: &str) -> Result<VersConstraint, String> {
    let (op, version) = if let Some(v) = token.strip_prefix(">=") {
        (VersOp::Ge, v)
    } else if let Some(v) = token.strip_prefix("<=") {
        (VersOp::Le, v)
    } else if let Some(v) = token.strip_prefix("!=") {
        (VersOp::Ne, v)
    } else if let Some(v) = token.strip_prefix('>') {
        (VersOp::Gt, v)
    } else if let Some(v) = token.strip_prefix('<') {
        (VersOp::Lt, v)
    } else if let Some(v) = token.strip_prefix('=') {
        (VersOp::Eq, v)
    } else {
        // VERS makes the `=` comparator optional: a bare version is equality.
        (VersOp::Eq, token)
    };
    let version = version.trim();
    if version.is_empty() {
        return Err(format!("missing version in VERS constraint '{token}'"));
    }
    if version.starts_with(['^', '~', '=', '>', '<', '!'])
        || version
            .split(['-', '+'])
            .next()
            .is_some_and(|core| core.split('.').any(|p| matches!(p, "x" | "X" | "*")))
    {
        return Err(format!("shorthand is not supported in VERS: '{token}'"));
    }
    Ok(VersConstraint { op, version: version.to_string() })
}

impl Vers {
    pub(crate) fn new(ecosystem: Ecosystem, spec: &str) -> Result<Self, String> {
        let spec = spec.trim();
        if spec.is_empty() {
            return Err("empty VERS range; use '*' to match every version".to_string());
        }
        if spec == "*" {
            return Ok(Self { ecosystem, constraints: Vec::new(), any: true });
        }
        // `|` is the spec separator. `,` and whitespace are accepted too so the
        // comma-joined strings earlier anyver releases emitted still parse.
        let constraints = spec
            .split(|c: char| c == '|' || c == ',' || c.is_whitespace())
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(parse_vers_constraint)
            .collect::<Result<Vec<_>, _>>()?;
        for constraint in &constraints {
            parse_for_ecosystem(ecosystem, &constraint.version)?;
        }
        if constraints.is_empty() {
            return Err(format!("no constraints in VERS range '{spec}'"));
        }
        Ok(Self { ecosystem, constraints, any: false })
    }

    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let rest = input
            .trim()
            .strip_prefix("vers:")
            .ok_or_else(|| format!("invalid VERS '{input}'; expected 'vers:' prefix"))?;
        let (scheme, spec) = rest.split_once('/').ok_or_else(|| {
            format!("invalid VERS '{input}'; expected 'vers:<versioning-scheme>/<constraints>'")
        })?;
        let ecosystem = Ecosystem::from_str(scheme.trim())?;
        Self::new(ecosystem, spec)
    }

    /// Render back to a canonical `vers:` string.
    pub(crate) fn to_vers(&self) -> String {
        let scheme = self.ecosystem.vers_scheme();
        if self.any {
            return format!("vers:{scheme}/*");
        }
        let joined = self
            .constraints
            .iter()
            .map(|c| format!("{}{}", c.op.as_str(), c.version))
            .collect::<Vec<_>>()
            .join("|");
        format!("vers:{scheme}/{joined}")
    }

    /// The VERS containment algorithm. Equality and exclusion constraints are
    /// resolved first, then the version is located within the *sorted* list of
    /// ordering constraints: it is contained when the constraint below it opens
    /// upward and the one above it opens downward.
    pub(crate) fn contains(&self, version: &str) -> Result<bool, String> {
        if self.any {
            return Ok(true);
        }
        let eco = self.ecosystem;
        let target = parse_for_ecosystem(eco, version)?;
        let cmp = |a: &ParsedRepr, b: &ParsedRepr| compare_for_ecosystem(eco, a, b);

        let mut ordered: Vec<(ParsedRepr, VersOp)> = Vec::with_capacity(self.constraints.len());
        let mut equal = false;
        let mut excluded = false;
        for constraint in &self.constraints {
            let parsed = parse_for_ecosystem(eco, &constraint.version)?;
            let matches = cmp(&target, &parsed) == Ordering::Equal;
            match constraint.op {
                VersOp::Ne => excluded |= matches,
                VersOp::Eq => equal |= matches,
                op => ordered.push((parsed, op)),
            }
        }
        if excluded {
            return Ok(false);
        }
        if equal {
            return Ok(true);
        }
        if ordered.is_empty() {
            return Ok(false);
        }
        ordered.sort_by(|a, b| cmp(&a.0, &b.0));

        // Combine equal bounds before evaluating the neighbouring intervals.
        let mut groups: Vec<(ParsedRepr, bool, bool, bool)> = Vec::new();
        for (parsed, op) in ordered {
            if let Some(last) = groups.last_mut()
                && cmp(&last.0, &parsed) == Ordering::Equal
            {
                last.1 |= op.opens_downward();
                last.2 |= op.opens_upward();
                last.3 |= op.is_inclusive();
            } else {
                groups.push((parsed, op.opens_downward(), op.opens_upward(), op.is_inclusive()));
            }
        }
        let mut below = None;
        for (parsed, down, up, inclusive) in groups {
            match cmp(&target, &parsed) {
                Ordering::Equal => return Ok(inclusive),
                Ordering::Less => return Ok(below.unwrap_or(true) && down),
                Ordering::Greater => below = Some(up),
            }
        }
        Ok(below.unwrap_or(false))
    }
}
