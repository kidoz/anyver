# anyver

[![CI](https://github.com/kidoz/anyver/actions/workflows/ci.yml/badge.svg)](https://github.com/kidoz/anyver/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/anyver)](https://pypi.org/project/anyver/)
[![Python](https://img.shields.io/pypi/pyversions/anyver)](https://pypi.org/project/anyver/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kidoz/anyver/blob/main/LICENSE)

A high-performance Python library (written in Rust via PyO3) for parsing and comparing software version strings across all major package ecosystems.

Handles: **SemVer, PEP 440, npm, Go modules, Debian/dpkg, RPM, Ruby Gems, Maven, NuGet, Composer, Crates.io, Hex, Swift PM, CalVer, Alpine, Docker, Conda, Arch/pacman, Gentoo/Portage, R/CRAN, Spack, Dart pub, Julia Pkg, Terraform/OpenTofu, Helm, CPAN, Nix, Homebrew, FreeBSD pkg, Hackage/Cabal, JDK runtime versions** — with a single generic parser, ecosystem-specific modes, range helpers, and security-interoperability utilities.

## Installation

```bash
pip install anyver
```

**Build from source** (requires Rust toolchain):

```bash
pip install maturin
maturin develop --release
```

## Quick Start

```python
from anyver import Version
import anyver

# Parse and compare versions
Version("1.2.3") < Version("2.0.0")          # True
Version("1.0.0-alpha") < Version("1.0.0")    # True
Version("1.0") == Version("1.0.0")           # True (trailing zero equivalence)
Version("v1.0") == Version("1.0")            # True (v-prefix stripped)
Version("1.0+build") == Version("1.0+other") # True (build metadata ignored)

# Module-level compare
anyver.compare("1.0", "2.0")                 # -1
anyver.compare("2.0", "1.0")                 #  1
anyver.compare("1.0", "1.0.0")               #  0
```

## Version Object

```python
v = Version("1.2.3-rc.1+build.42")

# Properties
v.raw              # "1.2.3-rc.1+build.42"
v.ecosystem        # "generic" (detected ecosystem name)
v.epoch            # 0
v.build            # "build.42"
v.major            # 1
v.minor            # 2
v.patch            # 3
v.is_prerelease    # True
v.is_postrelease   # False
v.is_stable        # False (opposite of is_prerelease)

# Segments
v.segments()       # (1, 2, 3, "rc", 1)
v.release()        # (1, 2, 3)
len(v)             # 5
v[0]               # 1
v[3]               # "rc"
v[-1]              # 1

# String representations
str(v)             # "1.2.3-rc.1+build.42"
repr(v)            # "Version('1.2.3-rc.1+build.42')"

# Compare using the Version's own detected ecosystem
v.compare("1.2.3")      # -1
v.compare("2.0.0")      # -1
v.compare("0.1.0")      # 1
```

## Sorting and Batch Operations

```python
anyver.sort_versions(["2.0", "1.0-alpha", "1.0", "0.1"])
# ["0.1", "1.0-alpha", "1.0", "2.0"]

anyver.batch_compare([("1.0", "2.0"), ("2.0", "1.0"), ("1.0", "1.0")])
# [-1, 1, 0]

anyver.max_version(["1.0", "3.0", "2.0"])  # "3.0"
anyver.min_version(["1.0", "3.0", "2.0"])  # "1.0"
```

## Boolean Helpers

```python
anyver.gt("2.0", "1.0")    # True
anyver.ge("1.0", "1.0.0")  # True
anyver.lt("1.0", "2.0")    # True
anyver.le("1.0", "1.0.0")  # True
anyver.eq("1.0", "1.0.0")  # True
anyver.ne("1.0", "2.0")    # True
```

## Version Constraints

Check whether a version satisfies a constraint expression. Supports `>=`, `<=`, `>`, `<`, `==`, `!=` operators, combined with commas for AND logic:

```python
anyver.satisfies("1.5.0", ">=1.0.0,<2.0.0")   # True
anyver.satisfies("2.0.0", ">=1.0.0,<2.0.0")   # False
anyver.satisfies("1.0.0", ">=1.0.0")           # True
anyver.satisfies("1.0.0", "!=1.0.0")           # False
anyver.satisfies("1.0.0-alpha", ">1.0.0")      # False

# Works with any ecosystem
anyver.satisfies("5.14.0-503.19.1.el9_5", ">=5.14.0-427.0.0.el9_4", ecosystem="rpm")

# A mistyped operator is an error, not a version compared as text
anyver.satisfies("1.5.0", ">>1.0")   # ValueError
```

Range helpers also support OR with `||`, npm/Cargo-style caret and tilde ranges, x-ranges, Maven/NuGet interval notation, and Terraform-style `~>`:

```python
anyver.range_contains("1.5.0", ">=1.0.0,<2.0.0")    # True
anyver.range_contains("1.2.7", "1.2.x")              # True
anyver.range_contains("1.5.0", "[1.0.0,2.0.0)")      # True
anyver.ranges_intersect(">=1.0,<2.0", ">=1.5,<3.0")  # True
```

## Stable Version Filtering

The selected ecosystem determines stability. In SemVer mode, any prerelease suffix, including a numeric suffix such as `-1`, excludes the version from stable results.

```python
anyver.stable_versions(["2.0.0-rc1", "1.0.0", "2.0.0", "1.5.0-beta"])
# ["1.0.0", "2.0.0"]

anyver.latest_stable(["2.0.0-rc1", "1.0.0", "2.0.0", "1.5.0-beta"])
# "2.0.0"
```

## Version Bumping

```python
anyver.bump_major("1.2.3")          # "2.0.0"
anyver.bump_minor("1.2.3")          # "1.3.0"
anyver.bump_patch("1.2.3")          # "1.2.4"

# Pre-release tags are stripped
anyver.bump_major("1.2.3-alpha")    # "2.0.0"
anyver.bump_patch("1.0.0-rc1")     # "1.0.1"
```

The bump helpers raise `ValueError` when incrementing a component would exceed `u64::MAX`.

## Hashable (Sets and Dicts)

```python
s = {Version("1.0"), Version("1.0.0"), Version("2.0")}
len(s)  # 2 — "1.0" and "1.0.0" are equal, so deduplicated

d = {Version("1.0"): "stable"}
d[Version("1.0.0")]  # "stable"
```

Equality is scoped to the ecosystem: two `Version` objects are equal only
when their ecosystems match and that ecosystem's comparator returns equality.
Strings and versions from other ecosystems are unequal. Use `v.compare(other)`
or `anyver.compare(a, b, ecosystem=...)` for explicit comparison. Ordering
objects from different ecosystems raises `TypeError`.

Generic comparison normalizes zeros before suffix tags as well as at the end:
`1.0-rc1` equals `1.0.0-rc1`. Known tag aliases such as `a` and `alpha` also
compare equal. PEP 440 compares public release components and local labels
according to its own rules; local labels affect equality in that ecosystem.
For strict validation, pass an explicit ecosystem to `Version.try_parse()`.
Generic parsing and the auto mode's generic fallback remain permissive.

Hash values are randomized per process, like Python's own string hashes, so
they must not be persisted; store `sort_key()` instead.

A numeric component wider than `u64` keeps its exact value in the generic modes
(`1.18446744073709551616` is greater than `1.18446744073709551615`), and
`major`, `minor`, `patch`, `segments()` and `release()` return it as a Python
`int`. The bump helpers raise `ValueError` for such a component.

## Ecosystem Auto-Detection

By default, `Version()` and `anyver.version()` use `ecosystem="auto"` — the library inspects the version string and picks the best ecosystem automatically:

```python
# Auto-detected as PEP 440 (contains "!")
Version("1!2.0.0")                           # ecosystem=pep440

# Auto-detected as Debian (contains "~")
Version("1.0~rc1")                           # ecosystem=debian

# Auto-detected as RPM (contains ".fc" / ".el")
Version("5.14.0-362.24.1.el9_4")             # ecosystem=rpm

# Auto-detected as Go (ends with "+incompatible")
Version("v2.0.0+incompatible")               # ecosystem=go

# Auto-detected as Alpine (contains "_alpha", "_rc", "-rN")
Version("3.1.4-r5")                          # ecosystem=alpine

# Auto-detected as Maven (ends with "-SNAPSHOT")
Version("1.0-SNAPSHOT")                      # ecosystem=maven

# Auto-detected as CalVer (starts with year-like number)
Version("2024.1.15")                         # ecosystem=calver

# Falls back to generic when ambiguous
Version("1.2.3")                             # ecosystem=generic
```

Detection rules (in priority order):
1. `!` in string → PEP 440
2. `~` → Debian, `^` → RPM
3. `.post`, `.dev` → PEP 440
4. `+incompatible` → Go, `-SNAPSHOT` → Maven
5. `py` together with `_` → Conda (`1.0-py38_0`)
6. `_alpha`, `_beta`, `_rc`, `_p` → Alpine
7. `+deb`, `+ubuntu` → Debian; `.el`, `.fc`, `.amzn` → RPM
8. Digit-letter patterns like `1a1`, `1b2`, `1rc1` → PEP 440; a trailing `-rN` → Alpine
9. Dot-separated component exactly `pre`/`rc`, or starting with `beta`/`alpha` (no hyphens) → Ruby
10. Year-like first segment (1990-2100) → CalVer
11. Otherwise → Generic

A detected ecosystem whose parser rejects the string falls back to `generic`.

## Ecosystem-Specific Comparison

You can also set the ecosystem explicitly:

```python
# Strict SemVer (numeric < alpha in pre-release)
anyver.compare_semver_strict("1.0.0-alpha", "1.0.0")  # -1

# Debian/dpkg (tilde sorts before everything)
anyver.compare("1.0~rc1", "1.0", ecosystem="debian")   # -1

# RPM (caret for post-release snapshots)
anyver.compare("1.0", "1.0^git1", ecosystem="rpm")     # -1

# PEP 440 (epoch with !)
anyver.compare("1!0.1", "2.0", ecosystem="pep440")     # 1

# All supported ecosystems:
# auto (default for Version), generic (default for compare),
# semver, npm, pep440, debian, rpm, go, ruby/gem/rubygems,
# maven/mvn, nuget/dotnet, composer/php/packagist,
# crates/cargo, hex/elixir/erlang, swift/swiftpm,
# calver, alpine/apk, docker/oci,
# conda/mamba, arch/pacman, gentoo/portage, cran/r,
# spack, dart/pub/flutter, julia, terraform/opentofu, helm,
# cpan/perl, nix/nixpkgs, homebrew/brew, freebsd/freebsd-pkg,
# hackage/cabal, jdk/openjdk/java-runtime
```

| Comparison used | Ecosystem modes |
| --- | --- |
| Dedicated ordering | `pep440`, `debian`, `rpm`, `maven`, `ruby`, `nuget`, `alpine`, `arch`, `cpan`, `hackage`, `jdk` |
| Strict SemVer precedence | `semver`, `npm`, `go`, `crates`, `hex`, `swift`, `dart`, `julia`, `helm` |
| Numeric package versions | `cran` |
| Generic upstream ordering with formula revisions | `homebrew` |
| Generic ordering | `generic`, `composer`, `calver`, `docker`, `conda`, `gentoo`, `spack`, `terraform`, `nix`, `freebsd` |

The SemVer modes compare concrete versions with unsigned 64-bit release
components and arbitrary-length numeric prerelease identifiers. Each package
manager can impose its own input limits and range rules. The `dart` and
`julia` modes ignore build metadata under this comparator.

The generic modes use anyver's segment ordering and can differ from the
package managers' comparators. Their numeric components keep full precision,
including those wider than `u64`. CalVer schemes and Docker tags have
project-specific precedence. Docker mode validates tag syntax and uses the
generic comparator for ordering.

NuGet compares prerelease labels case-insensitively and ignores build
metadata. Alpine follows apk-tools 3 ordering, including commit suffixes and
package revisions, and orders numeric components as digit strings of any
width. For example, `1.0_pre < 1.0_rc < 1.0 < 1.0-r0`. CPAN
normalizes decimal and dotted forms, such as `1.002003 == v1.2.3`, and marks
developer releases with underscores as prereleases.

Select `hackage` or `jdk` explicitly when the source is known:

```python
anyver.compare("1.2", "1.2.0", ecosystem="hackage")       # -1
anyver.compare("21.0.1+9", "21.0.1+10", ecosystem="jdk") # -1
```

Hackage compares dotted unsigned 64-bit components left to right and keeps
trailing zeros significant. JDK mode supports Java 9+ runtime strings and follows
`Runtime.Version.compareTo`, including numeric build numbers and
case-sensitive optional build information. JDK mode accepts `21-ea+10` and
`21.0.1+12-LTS` and rejects legacy Java 8 strings such as `1.8.0_402`.

## Database Integration

### `to_dict()` — Structured Export

```python
Version("1.2.3-rc.1+build.42").to_dict()
# {
#   "raw": "1.2.3-rc.1+build.42",
#   "ecosystem": "generic",
#   "epoch": 0,
#   "major": 1,
#   "minor": 2,
#   "patch": 3,
#   "build": "build.42",
#   "is_prerelease": True,
#   "is_postrelease": False
# }
```

### `sort_key()` — Database-Friendly Sort Key

```python
Version("1.2.3-rc.1").sort_key()
# Tuple of tuples that preserves comparison order when sorted lexically.
# Isomorphic with comparing two versions of the same ecosystem, including
# PEP 440 local labels (e.g. `1.0+abc` sorts above `1.0`).
```

> **Scope:** keys order versions within a single ecosystem. The keys
> reproduce comparison except where native rules prevent a total order.
> Compare keys only from the same ecosystem. Numeric values retain their full
> precision, including digit runs wider than `u64` in the generic modes,
> Debian, RPM and Alpine, and Maven's arbitrary-precision numbers. The tuple
> shape is experimental; regenerate stored keys after upgrading.
>
> Maven `sort_key()` is a total order for database sorting. It matches
> `compare()` for ordinary releases and qualifiers, and versions Maven treats
> as equal share a key. A qualifier compared with a nested list can disagree
> with `compare()`, because that is the rule behind Maven's cycle. Verified
> against maven-artifact 3.10.0, `compare()` gives
> `1-foo.sp-0 < 1-foo-alpha < 1-foo < 1-foo.sp-0`, while the key sorts
> `1-foo-alpha < 1-foo < 1-foo.sp-0`. `compare()` keeps the native results.
> RubyGems can consider `1.0.a` equal to `1.a` while ordering them on
> opposite sides of `1.b`. Its key gives equal versions a shared position.
> Arch's `vercmp` treats `1.0` as equal to both `1.0-0` and `1.0-1`,
> while the key orders all three by package release.
> In Maven, RubyGems, and Arch modes, collection sorting and selection use
> the same total order as `sort_key()`. Hashes follow each ecosystem's
> equality rules.

The Maven reference check is optional and runs offline against a local JAR
with a JDK installed. It checks all pairs in a deterministic corpus, including
the cycle; ordinary tests retain the verified cycle as a regression:

```bash
ANYVER_MAVEN_ARTIFACT_JAR=/path/to/maven-artifact-3.10.0.jar \
  pytest tests/test_anyver.py -k maven_against_reference_artifact -v -s
```

## Security Interop

Helpers are provided for package-url (PURL), VERS version ranges, and OSV-shaped affected records:

```python
anyver.parse_purl("pkg:npm/%40scope/name@1.2.3")
# {"type": "npm", "namespace": "%40scope", "name": "name",
#  "version": "1.2.3", "ecosystem": "npm", ...}

anyver.build_purl("npm", "name", "1.2.3", "%40scope")
# "pkg:npm/%40scope/name@1.2.3"

anyver.build_vers("pypi", ">=1.0.0|<2.0.0")   # "vers:pypi/>=1.0.0|<2.0.0"
anyver.vers_contains("vers:npm/1.2.3|>=2.0.0|<5.0.0", "1.2.3")  # True
anyver.vers_contains("vers:npm/1.2.3|>=2.0.0|<5.0.0", "1.5.0")  # False
anyver.vers_contains("vers:npm/1.2.3|>=2.0.0|<5.0.0", "3.0.0")  # True

affected = {
    "package": {"ecosystem": "PyPI", "name": "demo"},
    "ranges": [{"type": "ECOSYSTEM", "events": [
        {"introduced": "0"}, {"fixed": "1.2.0"},
        {"introduced": "2.0.0"}, {"fixed": "2.3.0"},
    ]}],
}
anyver.osv_affected("1.1.0", affected)  # True  — first window
anyver.osv_affected("1.5.0", affected)  # False — between windows
anyver.osv_affected("2.1.0", affected)  # True  — second window
```

**VERS.** `vers:` strings follow the [PURL-spec VERS grammar][vers]: constraints
are separated by `|` and the versioning scheme is the PURL type (`pypi`, not
`pep440`; `apk` for Alpine). Constraints form a *set*, not a conjunction — `1.2.3|>=2.0.0|<5.0.0`
matches 1.2.3 as well as everything in `[2.0.0, 5.0.0)`. `,` and whitespace are
still accepted as separators on input for compatibility, but `build_vers()`
always emits the canonical `|` form. Use `satisfies()` for npm/cargo-style
shorthand (`^1.2.3`, `~1.2`, `1.2.x`); VERS has no shorthand syntax.

**OSV.** Each `introduced` → `fixed`/`last_affected` pair in a range is its own
affected window, and a range with several pairs matches versions in any of them.
Events are ordered by version before evaluation, so a `fixed` listed before its
`introduced` still reads as one window. A `limit` gates the whole range: a
version is affected only when it sorts before at least one listed limit,
whatever window it falls in, and a limit of `*` imposes no gate.
`type: "SEMVER"` always uses strict SemVer ordering, including for packages
whose ecosystem normally uses another comparator. `type: "ECOSYSTEM"` uses
the package ecosystem (or explicit override). OSV windows compare version
bounds directly and do not apply npm range prerelease admission.
`type: "GIT"` ranges are skipped, since their events are commit hashes rather
than versions. Ecosystem names are read as OSV writes them, including the
release suffix (`Debian:11`, `Alpine:v3.18`). Go records compare as SemVer,
because OSV writes module versions without the `v` prefix. An entry in
`versions` matches its exact spelling or any spelling the ecosystem compares
equal (`1.2.3.0` for NuGet's `1.2.3`).

An ecosystem anyver doesn't recognise raises `ValueError` instead of silently
comparing with the generic order. `SEMVER` and `GIT` ranges and exact `versions`
matches never need the ecosystem; only an `ECOSYSTEM` range does. Pass
`ecosystem="generic"` to opt into that comparison, or resolve names up front:

```python
anyver.osv_ecosystem("Debian:11")   # "debian"
anyver.osv_ecosystem("Go")          # "semver"

affected = {
    "package": {"ecosystem": "Android", "name": "demo"},
    "ranges": [{"events": [{"introduced": "1.0.0"}, {"fixed": "2.0.0"}]}],
}
anyver.osv_affected("1.5.0", affected)                        # ValueError
anyver.osv_affected("1.5.0", affected, ecosystem="generic")   # True
```

**Ranges.** A blank range is an error, not a universal match — pass `*` to match
every version. npm ranges admit prereleases only when that same AND branch
explicitly references a prerelease on the same major/minor/patch tuple.
An npm intersection requires a version admitted by both branches.
Wildcards apply to complete release-number components at any depth (`1.2.3.*`,
`1.2.3.4.x`) and keep an epoch (`==1!1.2.*`). In PEP 440, prefix matching
(`==1.2.*`) also covers the pre- and development releases of `1.2` itself.
In npm mode a bare complete version is an exact match, a bare partial one is an
x-range, `=` is plain equality, and `1.2.3 - 2.3.4` is a hyphen range. A
comparison operator inside a version (`>>1.0`, `>=1.0 <<2.0`) raises `ValueError`.
PEP 440 ranges compare by ordering rather than by pip's exclusive-comparison
rules, so `>0.1.1` matches `0.1.1.post3` and `<1.2` matches `1.2rc1`.
PEP 440 compatible-release bounds accept the usual optional `v` prefix.
`~=1.4` accepts later 1.x releases, and `~=1.4.0` accepts later 1.4.x releases.
Caret and tilde ranges preserve prerelease lower bounds. Pessimistic `~>1.2`
means `>=1.2.0,<2.0.0`, while `~>1.2.3` means `>=1.2.3,<1.3.0`.
An upper-bound increment beyond `u64::MAX` raises `ValueError`.

[vers]: https://github.com/package-url/purl-spec/blob/main/VERSION-RANGE-SPEC.rst

> **Note:** PURL components are returned exactly as they appear in the string;
> percent-encoded values (`%40scope`) are not decoded. `build_purl()` encodes
> the characters that would change how a purl is split (`/`, `@`, `?`, `#` and
> whitespace), so a name cannot inject a namespace or a version, and raises
> `ValueError` for a type that is not a purl type. `+` and `:` stay literal and
> `%` is left alone, so already-encoded input is not encoded twice.

## Cross-Ecosystem Examples

```python
# PEP 440 (Python)
anyver.sort_versions(
    ["1.0.post1", "1.0", "1.0rc1", "1.0b1", "1.0a1", "1.0.dev1"],
    ecosystem="pep440",
)
# ["1.0.dev1", "1.0a1", "1.0b1", "1.0rc1", "1.0", "1.0.post1"]

# Debian
Version("1.0~alpha", ecosystem="debian") < Version("1.0~beta", ecosystem="debian") < Version("1.0", ecosystem="debian")

# RPM
anyver.compare("1.0~rc1", "1.0", ecosystem="rpm") < 0
anyver.compare("1.0", "1.0^git1", ecosystem="rpm") < 0

# Maven — ecosystem="maven" implements Apache Maven's ComparableVersion
anyver.sort_versions(["1.0-sp-1", "1.0", "1.0-SNAPSHOT", "1.0-alpha-1"], ecosystem="maven")
# ["1.0-alpha-1", "1.0-SNAPSHOT", "1.0", "1.0-sp-1"]
anyver.compare("1.0-jre", "1.0", ecosystem="maven")   #  1 — unknown qualifiers rank above the release
anyver.compare("1.0-1", "1.0.1", ecosystem="maven")   # -1 — `-` nests, `.` does not

# Go modules
Version("v2.0.0+incompatible", ecosystem="go") == Version("v2.0.0", ecosystem="go")

# Ruby Gems — numeric, not lexical
Version("3.2") < Version("3.10")

# Epoch overrides everything
Version("1:0.1") > Version("999.0")
```

## Performance

Built in Rust for speed. Typical benchmarks (Apple M-series):

| Operation | Time |
|---|---|
| `Version("1.2.3")` construction | ~330 ns |
| `anyver.compare("1.2.3", "1.2.4")` | ~300 ns |
| `Version < Version` (pre-parsed) | ~66 ns |
| `sort_versions(1000)` | ~460 us |
| vs Python `packaging.version` | **~14x faster** |
| vs Python `semver` | **~23x faster** |

## Stability & Versioning

`anyver` follows [Semantic Versioning](https://semver.org/).

Version **2.2.1** is a maintenance patch: Rust and Python development
dependencies are refreshed, and the `to_dict()` example and Ruby detection
rule in this README are corrected. There are no API or behavior changes; see
the [2.2.1 changelog](CHANGELOG.md#221---2026-11-25).

The public API surface covered by SemVer guarantees:

**Stable** (breaking changes require a major version bump):

- The `Version` class constructor, operators, properties, and methods
  (`raw`, `ecosystem`, `epoch`, `build`, `major/minor/patch`, `is_prerelease`,
  `is_postrelease`, `is_stable`, `count`, `segments()`, `release()`,
  `compare()`, `to_dict()`, `from_dict()`, `sort_key()`, `parse()`, `try_parse()`).
- Module-level functions: `version`, `compare`, `compare_semver_strict`,
  `sort_versions`, `batch_compare`, `gt`/`ge`/`gte`/`lt`/`le`/`lte`/`eq`/`ne`,
  `max_version`, `min_version`, `stable_versions`, `latest_stable`,
  `satisfies`, `bump_major`/`bump_minor`/`bump_patch`/`bump_prerelease`,
  `next_stable`.
- The set of accepted ecosystem names (aliases may grow, but existing names
  keep their meaning).
- The comparison result (`-1`/`0`/`1`) for any pair of versions within a
  given ecosystem, within a major version.
- Type stubs (`.pyi`) and the `py.typed` marker.
- Pickle format and `__reduce__` output (stable across patch versions).

**Experimental** (may change within minor versions until documented stable):

- `Version.sort_key()` tuple shape (the encoding scheme can evolve if new
  ecosystems require additional distinguishing fields). This exception does
  not cover removing support for an ecosystem or starting to raise errors.
- The `ecosystem="auto"` detection heuristic. The *rules* may be refined to
  reduce false positives; the *existing correct* detections will not flip.
- Error messages and exception attribute text.

**Not covered by SemVer:**

- Rust internals (`src/`), benchmarks, CI configuration.
- Performance numbers (we aim to improve them monotonically, but there are
  no guarantees tied to the public version).

See [CHANGELOG.md](CHANGELOG.md) for release history.

## Author

Aleksandr Pavlov <ckidoz@gmail.com>

## License

MIT — see [LICENSE](LICENSE) for details.
