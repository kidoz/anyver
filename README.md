# anyver

[![CI](https://github.com/kidoz/anyver/actions/workflows/ci.yml/badge.svg)](https://github.com/kidoz/anyver/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/anyver)](https://pypi.org/project/anyver/)
[![Python](https://img.shields.io/pypi/pyversions/anyver)](https://pypi.org/project/anyver/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/kidoz/anyver/blob/main/LICENSE)

A high-performance Python library (written in Rust via PyO3) for parsing and comparing software version strings across all major package ecosystems.

Handles: **SemVer, PEP 440, npm, Go modules, Debian/dpkg, RPM, Ruby Gems, Maven, NuGet, Composer, Crates.io, Hex, Swift PM, CalVer, Alpine, Docker** — with a single generic parser and ecosystem-specific modes.

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
v.compare("1.2.3")      # 0
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
```

## Stable Version Filtering

Filter out pre-release versions or find the latest stable:

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

## Hashable (Sets and Dicts)

```python
s = {Version("1.0"), Version("1.0.0"), Version("2.0")}
len(s)  # 2 — "1.0" and "1.0.0" are equal, so deduplicated

d = {Version("1.0"): "stable"}
d[Version("1.0.0")]  # "stable"
```

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
5. `_alpha`, `_beta`, `_rc`, `_p`, `-rN` → Alpine
6. `+deb`, `+ubuntu` → Debian; `.el`, `.fc`, `.amzn` → RPM
7. Digit-letter patterns like `1a1`, `1b2`, `1rc1` → PEP 440
8. Dot-separated `.pre`, `.rc`, `.beta`, `.alpha` (no hyphens) → Ruby
9. Year-like first segment (1990-2100) → CalVer
10. Otherwise → Generic

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
# calver, alpine/apk, docker/oci
```

## Database Integration

### `to_dict()` — Structured Export

```python
Version("1.2.3-rc.1+build.42").to_dict()
# {
#   "raw": "1.2.3-rc.1+build.42",
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
# Guarantees perfect isomorphism with anyver.compare().
```

## Cross-Ecosystem Examples

```python
# PEP 440 (Python)
Version("1.0.dev1") < Version("1.0a1") < Version("1.0b1") < Version("1.0rc1") < Version("1.0") < Version("1.0.post1")

# Debian
Version("1.0~alpha") < Version("1.0~beta") < Version("1.0")

# RPM
Version("1.0~rc1") < Version("1.0") < Version("1.0^git1")

# Maven
Version("1.0-alpha-1") < Version("1.0-SNAPSHOT") < Version("1.0") < Version("1.0-sp-1")

# Go modules
Version("v2.0.0+incompatible") == Version("v2.0.0")

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

## Author

Aleksandr Pavlov <ckidoz@gmail.com>

## License

MIT — see [LICENSE](LICENSE) for details.
