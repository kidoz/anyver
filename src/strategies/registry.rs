//! Strategy trait, the per-ecosystem parsed representation, and ecosystem
//! dispatch. This is the wiring layer: it picks a [`VersionStrategy`] for an
//! [`Ecosystem`] and routes parse/compare calls to the responsibility modules
//! (`generic`, `semver`, `pep440`, `debian`, `rpm`, `validators`).

use crate::parser::{Ecosystem, ParsedVersion, parse_generic};
use std::cmp::Ordering;

use super::debian::{DebianVersion, cmp_debian, parse_debian};
use super::generic::cmp_parsed;
use super::pep440::{Pep440Version, cmp_pep440, parse_pep440};
use super::rpm::{RpmVersion, cmp_rpm, parse_rpm};
use super::semver::{SemVer, cmp_semver_strict, parse_semver_strict};
use super::validators::{
    validate_alpine, validate_calver, validate_composer, validate_docker, validate_go,
    validate_maven, validate_nuget, validate_ruby,
};

#[derive(Debug, Clone)]
pub(crate) enum ParsedRepr {
    Generic(ParsedVersion),
    Semver(SemVer),
    Debian(DebianVersion),
    Rpm(RpmVersion),
    Pep440(Pep440Version),
}

pub(crate) trait VersionStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String>;
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering;
}

// Macros to reduce strategy boilerplate.
// validate→generic: validate input, then parse/compare via generic parser.
// semver: parse/compare via strict SemVer (validation included in parse_semver_strict).
macro_rules! impl_generic_strategy {
    ($name:ident, $validate:path) => {
        impl VersionStrategy for $name {
            fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
                $validate(input)?;
                Ok(ParsedRepr::Generic(parse_generic(input)))
            }
            fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
                match (left, right) {
                    (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
                    _ => unreachable!(),
                }
            }
        }
    };
}
macro_rules! impl_semver_strategy {
    ($name:ident) => {
        impl VersionStrategy for $name {
            fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
                parse_semver_strict(input).map(ParsedRepr::Semver)
            }
            fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
                match (left, right) {
                    (ParsedRepr::Semver(a), ParsedRepr::Semver(b)) => cmp_semver_strict(a, b),
                    _ => unreachable!(),
                }
            }
        }
    };
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
pub(crate) struct CratesStrategy;
pub(crate) struct HexStrategy;
pub(crate) struct SwiftStrategy;
pub(crate) struct DockerStrategy;

// Generic strategy — no validation
impl VersionStrategy for GenericStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        Ok(ParsedRepr::Generic(parse_generic(input)))
    }
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Generic(a), ParsedRepr::Generic(b)) => cmp_parsed(a, b),
            _ => unreachable!(),
        }
    }
}

// Debian and RPM have custom parsers/comparators
impl VersionStrategy for DebianStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        parse_debian(input).map(ParsedRepr::Debian)
    }
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Debian(a), ParsedRepr::Debian(b)) => cmp_debian(a, b),
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }
}

// SemVer-based ecosystems (validation included in parse_semver_strict)
impl_semver_strategy!(SemverStrategy);
impl_semver_strategy!(CratesStrategy);
impl_semver_strategy!(HexStrategy);
impl_semver_strategy!(SwiftStrategy);

// Go is strict SemVer but additionally requires the mandatory lowercase `v`
// prefix that `strip_v` would otherwise tolerate dropping or upper-casing.
impl VersionStrategy for GoStrategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        validate_go(input)?;
        parse_semver_strict(input).map(ParsedRepr::Semver)
    }
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Semver(a), ParsedRepr::Semver(b)) => cmp_semver_strict(a, b),
            _ => unreachable!(),
        }
    }
}

// PEP 440 preserves and compares the local-version label (`+LOCAL`) so that
// `1.0+abc` sorts strictly greater than `1.0` per PEP 440 §"Local version
// identifiers". The generic strategy silently drops that suffix.
impl VersionStrategy for Pep440Strategy {
    fn parse(&self, input: &str) -> Result<ParsedRepr, String> {
        parse_pep440(input).map(ParsedRepr::Pep440)
    }
    fn compare(&self, left: &ParsedRepr, right: &ParsedRepr) -> Ordering {
        match (left, right) {
            (ParsedRepr::Pep440(a), ParsedRepr::Pep440(b)) => cmp_pep440(a, b),
            _ => unreachable!(),
        }
    }
}

// Validate→generic ecosystems
impl_generic_strategy!(RubyStrategy, validate_ruby);
impl_generic_strategy!(MavenStrategy, validate_maven);
impl_generic_strategy!(NugetStrategy, validate_nuget);
impl_generic_strategy!(ComposerStrategy, validate_composer);
impl_generic_strategy!(CalverStrategy, validate_calver);
impl_generic_strategy!(AlpineStrategy, validate_alpine);
impl_generic_strategy!(DockerStrategy, validate_docker);

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
static CRATES_STRATEGY: CratesStrategy = CratesStrategy;
static HEX_STRATEGY: HexStrategy = HexStrategy;
static SWIFT_STRATEGY: SwiftStrategy = SwiftStrategy;
static DOCKER_STRATEGY: DockerStrategy = DockerStrategy;

pub(crate) fn strategy_for(ecosystem: Ecosystem) -> &'static dyn VersionStrategy {
    match ecosystem {
        Ecosystem::Generic => &GENERIC_STRATEGY,
        Ecosystem::Semver | Ecosystem::Npm => &SEMVER_STRATEGY,
        Ecosystem::Crates => &CRATES_STRATEGY,
        Ecosystem::Hex => &HEX_STRATEGY,
        Ecosystem::Swift => &SWIFT_STRATEGY,
        Ecosystem::Docker => &DOCKER_STRATEGY,
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
