# Changelog 

## [Unreleased]

## [1.1.0] - 2026-06-27

Additive minor release: version range helpers, security-interoperability
utilities (PURL/VERS/OSV), 13 additional ecosystem modes, and correctness
fixes for Go and PEP 440 local versions. One behavior change is called out
under Fixed (Go now rejects input without a lowercase `v` prefix).

### Added
- Ecosystem-aware version range helpers:
  - `anyver.range_contains(version, range, ecosystem="generic")` — whether a
    version falls inside a range.
  - `anyver.ranges_intersect(left, right, ecosystem="generic")` — whether two
    ranges share at least one version.
  - Range syntax covers comma-AND, OR via `||`, npm/Cargo caret (`^1.2.3`) and
    tilde (`~1.2`) ranges, x-ranges (`1.2.x`), Maven/NuGet interval notation
    (`[1.0.0,2.0.0)`), and Terraform-style `~>`.
- Security-interoperability helpers:
  - `anyver.parse_purl(purl)`, `anyver.build_purl(type, name, version, ...)`,
    and `anyver.purl_ecosystem(purl)` for package-url (PURL) strings.
  - `anyver.build_vers(ecosystem, range)` and `anyver.vers_contains(vers, version)`
    for VERS-style version ranges.
  - `anyver.osv_affected(version, affected, ecosystem=None)` for OSV-shaped
    affected-range records.
- 13 additional ecosystem modes: Conda/mamba, Arch/pacman, Gentoo/Portage,
  R/CRAN, Spack, Dart/pub/Flutter, Julia, Terraform/OpenTofu, Helm, CPAN/Perl,
  Nix/nixpkgs, Homebrew/brew, and FreeBSD pkg (with their common aliases).

### Changed
- Upgraded PyO3 0.28 → 0.29 (still `abi3-py311`: one wheel per platform covers
  CPython 3.11, 3.12, 3.13, and 3.14+). No public API change.
- Upgraded the Criterion benchmark harness 0.7 → 0.8 (development only).
- New ecosystem modes start with input validation plus generic or
  SemVer-compatible ordering; the fully native comparators remain Debian/dpkg,
  RPM, the strict SemVer family, and PEP 440 local-version handling.

### Fixed
- **Go modules now require the mandatory lowercase `v` prefix.** `compare(...,
  ecosystem="go")` rejects inputs like `1.0.0` and `V1.0.0` that the strict
  SemVer path previously accepted by silently stripping the prefix. This is a
  behavior change for callers that passed non-`v` Go versions.
- `Version.sort_key()` now encodes the PEP 440 `+LOCAL` label, so it stays
  isomorphic with `compare()` for the version's ecosystem (`1.0+abc` sorts
  above `1.0`). Local-label segments use a neutral weight, so a local
  component such as `dev` is not misread as a pre-release tag.

## [1.0.0] - 2026-04-20

First stable release. The public API described in `README.md` and the
type stubs in `python/anyver/__init__.pyi` is now covered by SemVer
guarantees — see the "Stability & Versioning" section of the README for
the exact surface.

### Added
- `Version == str` and all rich comparisons now accept a plain string on the
  right-hand side (`Version("1.0") == "1.0.0"`). Comparing against unrelated
  types returns `NotImplemented` for `==`/`!=` instead of raising.
- Pickle and `copy.deepcopy` support via `__reduce__`, preserving both `raw`
  and the detected/assigned ecosystem.
- `Version.parse(s, ecosystem=...)` classmethod (raises) and
  `Version.try_parse(s, ecosystem=...)` staticmethod (returns `None`).
- `Version.from_dict(d)` classmethod reconstructs a `Version` from the dict
  produced by `to_dict()`. `to_dict()` now also emits an `"ecosystem"` key.
- `anyver.bump_prerelease(version, tag="alpha")` — increments the prerelease
  counter or starts a new `-{tag}.0`.
- `anyver.next_stable(version)` — strips prerelease/build metadata.
- `anyver.satisfies` now understands npm-style shorthand:
  - caret ranges (`^1.2.3`) with 0-version semantics
  - tilde ranges (`~1.2.3`, `~1.2`, `~1`)
  - x-ranges (`1.2.x`, `1.x`, `*`)
  - OR branches via `||`
- Type stubs (`anyver/__init__.pyi`) and `py.typed` marker are now shipped
  with every wheel, making the library fully typed for mypy/Pyright.

### Changed
- Switched to a mixed Python + Rust maturin layout. The compiled extension
  is now `anyver._anyver`; `anyver` itself is a Python package that
  re-exports the public API. Existing imports (`from anyver import Version`)
  continue to work unchanged.
- `Version.to_dict()` now includes the `"ecosystem"` key.
- The PyO3 extension is now built against the `abi3-py311` stable ABI, so a
  single wheel per platform covers every supported Python version (3.11+).

### Fixed
- PEP 440 local versions now compare correctly: `1.0+abc > 1.0` and
  `1.0+a < 1.0+b` per the PEP 440 spec. Previously the comparator routed
  PEP 440 through the generic strategy, which silently dropped the
  `+LOCAL` suffix and reported `1.0+abc == 1.0`.
- Maven release-alias qualifiers `final`, `ga`, and `release` now compare
  equal to the bare release (so `1.0-final == 1.0 < 1.0-sp-1`). These were
  previously treated as unknown pre-release text and sorted below the
  release. The alias set is also honored by the generic strategy, so
  `Version("1.0-final") == Version("1.0")` holds everywhere.

## [0.2.0] - 2026-04

### Added
- Per-ecosystem comparison strategies (semver strict, Debian `verrevcmp`,
  RPM `rpmvercmp`, PEP 440, Ruby, Maven, Go, NuGet, Composer, crates,
  Hex, Swift, CalVer, Alpine, Docker).
- `ecosystem="auto"` autodetection with explicit precedence rules.
- `satisfies`, `stable_versions`, `latest_stable`, `bump_major/minor/patch`.
- `Version.sort_key()` DB-friendly tuple encoding isomorphic to `compare`.
- Rust criterion benchmarks and Python pytest-benchmark suites.

## [0.1.0] - Initial release

- Core `Version` class with generic parser + comparator.
- Module-level `compare`, `sort_versions`, `batch_compare`,
  `min_version`/`max_version`, boolean helpers.
- `compare_semver_strict` for strict SemVer 2.0.0 ordering.
