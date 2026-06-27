//! Security-interoperability helpers for PURL and VERS strings.

use crate::parser::Ecosystem;
use crate::ranges::RangeSet;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Vers {
    pub(crate) ecosystem: Ecosystem,
    pub(crate) range: RangeSet,
}

impl Vers {
    pub(crate) fn parse(input: &str) -> Result<Self, String> {
        let rest = input
            .strip_prefix("vers:")
            .ok_or_else(|| format!("invalid VERS '{input}'; expected 'vers:' prefix"))?;
        let (scheme, range) = rest.split_once('/').ok_or_else(|| {
            format!("invalid VERS '{input}'; expected 'vers:<ecosystem>/<range>'")
        })?;
        let ecosystem = Ecosystem::from_str(scheme)?;
        Ok(Self { ecosystem, range: RangeSet::parse(range)? })
    }

    pub(crate) fn contains(&self, version: &str) -> Result<bool, String> {
        self.range.contains(self.ecosystem, version)
    }
}
