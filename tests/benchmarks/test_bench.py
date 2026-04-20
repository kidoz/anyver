"""Python-level benchmarks for anyver.

Run: `pytest tests/benchmarks/ --benchmark-only`
     `just bench-py`

Measures the end-to-end user-facing API, including PyO3 marshaling overhead.
For pure algorithmic perf (no PyO3), see `benches/bench_versions.rs`.
"""

import random

import pytest

import anyver
from anyver import Version

# Representative version strings across ecosystems.
SEMVER_SIMPLE = "1.2.3"
SEMVER_PRERELEASE = "1.0.0-alpha.1+build.42"
PEP440_COMPLEX = "1!2.3.4.post1.dev2+local.build"
DEBIAN_EPOCH = "2:1.0.2l-2+deb9u1"
RPM_KERNEL = "5.14.0-362.13.1.el9_3"
MAVEN_LIFECYCLE = "4.0.0-alpha-13-SNAPSHOT"


# ============================================================================
# Version construction
# ============================================================================


class TestConstructBench:
    @pytest.mark.parametrize(
        "version",
        [
            SEMVER_SIMPLE,
            SEMVER_PRERELEASE,
            PEP440_COMPLEX,
            DEBIAN_EPOCH,
            RPM_KERNEL,
            MAVEN_LIFECYCLE,
        ],
    )
    def test_version_autodetect(self, benchmark, version):
        """Version() with autodetect — pays the full parse+detect cost."""
        benchmark(Version, version)

    @pytest.mark.parametrize(
        "version,eco",
        [
            (SEMVER_PRERELEASE, "semver"),
            (PEP440_COMPLEX, "pep440"),
            (DEBIAN_EPOCH, "debian"),
            (RPM_KERNEL, "rpm"),
            (MAVEN_LIFECYCLE, "maven"),
        ],
    )
    def test_version_explicit_ecosystem(self, benchmark, version, eco):
        """Version() with explicit ecosystem — skips autodetect."""
        benchmark(Version, version, eco)


# ============================================================================
# Comparison
# ============================================================================


class TestCompareBench:
    def test_compare_strings(self, benchmark):
        benchmark(anyver.compare, "1.2.3", "1.2.4")

    def test_compare_version_objects(self, benchmark):
        a = Version("1.2.3")
        b = Version("1.2.4")
        benchmark(anyver.compare, a, b)

    def test_compare_semver_strict(self, benchmark):
        benchmark(anyver.compare_semver_strict, "1.0.0-alpha.1", "1.0.0-alpha.beta")

    def test_richcmp_operator(self, benchmark):
        a = Version("1.2.3")
        b = Version("1.2.4")
        benchmark(lambda: a < b)


# ============================================================================
# Bulk operations
# ============================================================================


def _random_versions(n: int, seed: int = 42) -> list[str]:
    rng = random.Random(seed)
    return [f"{rng.randint(0, 20)}.{rng.randint(0, 99)}.{rng.randint(0, 99)}" for _ in range(n)]


class TestBulkBench:
    def test_sort_1000_strings(self, benchmark):
        versions = _random_versions(1000)
        benchmark(anyver.sort_versions, versions)

    def test_sort_1000_version_objects(self, benchmark):
        versions = [Version(v) for v in _random_versions(1000)]
        benchmark(anyver.sort_versions, versions)

    def test_batch_compare_1000_pairs(self, benchmark):
        vs = _random_versions(2000)
        pairs = list(zip(vs[::2], vs[1::2], strict=True))
        benchmark(anyver.batch_compare, pairs)

    def test_max_of_1000(self, benchmark):
        versions = _random_versions(1000)
        benchmark(anyver.max_version, versions)

    def test_min_of_1000(self, benchmark):
        versions = _random_versions(1000)
        benchmark(anyver.min_version, versions)


# ============================================================================
# Constraint & stability helpers (v0.2.0 API)
# ============================================================================


class TestConstraintBench:
    def test_satisfies_single(self, benchmark):
        benchmark(anyver.satisfies, "1.5.0", ">=1.0.0")

    def test_satisfies_compound(self, benchmark):
        benchmark(anyver.satisfies, "1.5.0", ">=1.0.0,<2.0.0,!=1.5.1")

    def test_latest_stable_1000(self, benchmark):
        # Mix ~20% prereleases into a 1000-version list.
        rng = random.Random(42)
        versions = []
        for v in _random_versions(1000):
            if rng.random() < 0.2:
                versions.append(f"{v}-rc{rng.randint(1, 9)}")
            else:
                versions.append(v)
        benchmark(anyver.latest_stable, versions)

    def test_stable_versions_1000(self, benchmark):
        rng = random.Random(42)
        versions = []
        for v in _random_versions(1000):
            if rng.random() < 0.2:
                versions.append(f"{v}-rc{rng.randint(1, 9)}")
            else:
                versions.append(v)
        benchmark(anyver.stable_versions, versions)


# ============================================================================
# Sort key generation
# ============================================================================


class TestSortKeyBench:
    def test_sort_key_simple(self, benchmark):
        v = Version(SEMVER_SIMPLE)
        benchmark(v.sort_key)

    def test_sort_key_complex(self, benchmark):
        v = Version(PEP440_COMPLEX)
        benchmark(v.sort_key)

    def test_sort_1000_by_sort_key(self, benchmark):
        versions = [Version(v) for v in _random_versions(1000)]
        benchmark(sorted, versions, key=Version.sort_key)
