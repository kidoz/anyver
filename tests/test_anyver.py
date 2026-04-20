"""Comprehensive test suite for anyver library."""

import pytest

import anyver
from anyver import Version

# ============================================================================
# Version object: construction, str, repr
# ============================================================================


class TestVersionConstruction:
    def test_str(self):
        v = Version("1.2.3-alpha.1+build.42")
        assert str(v) == "1.2.3-alpha.1+build.42"

    def test_repr(self):
        v = Version("1.2.3-alpha.1+build.42")
        assert repr(v) == "Version('1.2.3-alpha.1+build.42')"

    def test_version_shortcut(self):
        v = anyver.version("2.0.0")
        assert str(v) == "2.0.0"

    def test_empty_string(self):
        v = Version("")
        assert str(v) == ""
        assert not bool(v)

    def test_non_string_raises_typeerror(self):
        with pytest.raises(TypeError):
            Version(123)


# ============================================================================
# Properties
# ============================================================================


class TestProperties:
    def test_epoch_default(self):
        assert Version("1.2.3").epoch == 0

    def test_epoch_pep440(self):
        assert Version("1!2.3.4.post1+local").epoch == 1

    def test_epoch_debian(self):
        assert Version("1:2.3-1").epoch == 1

    def test_build(self):
        assert Version("1.2.3-alpha.1+build.42").build == "build.42"

    def test_build_local(self):
        assert Version("1!2.3.4.post1+local").build == "local"

    def test_build_empty(self):
        assert Version("1.0").build == ""

    def test_is_prerelease_true(self):
        assert Version("1.2.3-alpha.1+build.42").is_prerelease is True

    def test_is_prerelease_false(self):
        assert Version("1!2.3.4.post1+local").is_prerelease is False

    def test_is_postrelease_true(self):
        assert Version("1!2.3.4.post1+local").is_postrelease is True

    def test_is_postrelease_false(self):
        assert Version("1.2.3-alpha.1+build.42").is_postrelease is False

    def test_is_postrelease_caret(self):
        assert Version("1.0^git1").is_postrelease is True

    def test_major(self):
        assert Version("3.14.159").major == 3

    def test_minor(self):
        assert Version("3.14.159").minor == 14

    def test_patch(self):
        assert Version("3.14.159").patch == 159

    def test_count(self):
        assert Version("3.14.159").count == 3

    def test_major_missing(self):
        v = Version("1")
        assert v.major == 1
        assert v.minor == 0
        assert v.patch == 0


# ============================================================================
# Segments and release
# ============================================================================


class TestSegments:
    def test_segments_tuple(self):
        v = Version("1.2.3-alpha.1+build.42")
        assert v.segments() == (1, 2, 3, "alpha", 1)

    def test_segments_type(self):
        assert isinstance(Version("1.0").segments(), tuple)

    def test_release_tuple(self):
        v = Version("1.2.3-alpha.1+build.42")
        assert v.release() == (1, 2, 3)

    def test_pep440_segments(self):
        assert Version("5.0a1").segments() == (5, 0, "a", 1)
        assert Version("5.0a1").release() == (5, 0)

    def test_semver_rc_segments(self):
        assert Version("1.2.3-rc1").segments() == (1, 2, 3, "rc", 1)
        assert Version("1.2.3-rc1").release() == (1, 2, 3)


# ============================================================================
# Indexing
# ============================================================================


class TestIndexing:
    def test_numeric_index(self):
        v = Version("10.20.30")
        assert v[0] == 10
        assert v[1] == 20
        assert v[2] == 30

    def test_negative_index(self):
        v = Version("10.20.30")
        assert v[-1] == 30

    def test_text_index(self):
        v = Version("1.0.0-beta.3")
        assert v[3] == "beta"
        assert v[4] == 3

    def test_out_of_range(self):
        v = Version("1.0.0-beta.3")
        with pytest.raises(IndexError):
            _ = v[100]


# ============================================================================
# Sequence protocol: len, bool
# ============================================================================


class TestSequence:
    def test_len_3(self):
        assert len(Version("1.2.3")) == 3

    def test_len_5(self):
        assert len(Version("1.2.3-rc.1")) == 5

    def test_bool_true(self):
        assert bool(Version("1.0"))

    def test_bool_empty(self):
        assert not bool(Version(""))


# ============================================================================
# Hash, set, dict
# ============================================================================


class TestHash:
    def test_set_dedup(self):
        s = {Version("1.0"), Version("1.0.0"), Version("2.0")}
        assert len(s) == 2

    def test_dict_lookup(self):
        d = {Version("1.0"): "one"}
        assert d[Version("1.0.0")] == "one"


# ============================================================================
# Comparison operators
# ============================================================================


class TestOperators:
    def test_lt(self):
        assert Version("1.0") < Version("2.0")

    def test_le(self):
        assert Version("1.0") <= Version("1.0.0")

    def test_eq(self):
        assert Version("1.0") == Version("1.0.0")

    def test_ne(self):
        assert Version("1.0") != Version("2.0")

    def test_gt(self):
        assert Version("2.0") > Version("1.0")

    def test_ge(self):
        assert Version("2.0") >= Version("1.0")

    def test_eq_vprefix(self):
        assert Version("v1.0") == Version("1.0")

    def test_eq_build(self):
        assert Version("1.0+a") == Version("1.0+b")

    def test_eq_epoch_zero(self):
        assert Version("0:1.0") == Version("1.0")


# ============================================================================
# Version.compare() method
# ============================================================================


class TestVersionCompare:
    def test_compare_str_lt(self):
        assert Version("1.5.0").compare("2.0.0") == -1

    def test_compare_str_gt(self):
        assert Version("1.5.0").compare("1.0.0") == 1

    def test_compare_str_eq(self):
        assert Version("1.5.0").compare("1.5.0") == 0

    def test_compare_version_obj(self):
        assert Version("1.5.0").compare(Version("2.0")) == -1


# ============================================================================
# Module-level compare() with mixed types
# ============================================================================


class TestModuleCompare:
    def test_str_str(self):
        assert anyver.compare("1.0", "2.0") == -1

    def test_ver_ver(self):
        assert anyver.compare(Version("1.0"), Version("2.0")) == -1

    def test_ver_str(self):
        assert anyver.compare(Version("1.0"), "2.0") == -1

    def test_str_ver(self):
        assert anyver.compare("1.0", Version("2.0")) == -1


# ============================================================================
# Basic numeric comparisons
# ============================================================================


class TestBasicNumeric:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1", "2", -1),
            ("2", "1", 1),
            ("1", "1", 0),
            ("1.0", "1.0.0", 0),
            ("1.0.0", "1.0.1", -1),
            ("1.0.1", "1.1.0", -1),
            ("1.1.0", "2.0.0", -1),
            ("10.0.0", "9.0.0", 1),
            ("1.2.3", "1.2.3", 0),
            ("0.0.1", "0.0.2", -1),
            ("1.2", "1.10", -1),
            ("1.0.0.0.0", "1.0.0", 0),
            ("0", "0.0.0", 0),
        ],
    )
    def test_basic(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# v-prefix
# ============================================================================


class TestVPrefix:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("v1.0.0", "1.0.0", 0),
            ("V1.0.0", "1.0.0", 0),
            ("v2.0.0", "v1.0.0", 1),
            ("v0.1.0", "v0.2.0", -1),
        ],
    )
    def test_vprefix(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# SemVer pre-release
# ============================================================================


class TestSemVerPrerelease:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0.0-alpha", "1.0.0-alpha.1", -1),
            ("1.0.0-alpha", "1.0.0-beta", -1),
            ("1.0.0-beta", "1.0.0-rc", -1),
            ("1.0.0-rc", "1.0.0", -1),
            ("1.0.0-alpha", "1.0.0", -1),
            ("1.0.0+build", "1.0.0", 0),
            ("1.0.0+build.1", "1.0.0+build.2", 0),
            ("1.0.0+20130313144700", "1.0.0", 0),
        ],
    )
    def test_semver(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# SemVer strict
# ============================================================================


class TestSemVerStrict:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0.0-alpha", "1.0.0-alpha.1", -1),
            ("1.0.0-alpha.1", "1.0.0-alpha.beta", -1),
            ("1.0.0-beta.2", "1.0.0-beta.11", -1),
            ("1.0.0-rc.1", "1.0.0", -1),
            ("1.0.0+build", "1.0.0", 0),
            ("0.0.0", "0.0.1", -1),
        ],
    )
    def test_strict(self, a, b, expected):
        assert anyver.compare_semver_strict(a, b) == expected

    def test_invalid_raises(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.0", "2.0")


# ============================================================================
# Ecosystem dispatch
# ============================================================================


class TestEcosystemDispatch:
    def test_compare_default_generic(self):
        # Generic parser keeps Num > Text rule in prerelease segments
        assert anyver.compare("1.0.0-alpha.1", "1.0.0-alpha.beta") == 1

    def test_compare_semver_ecosystem(self):
        # Strict SemVer: numeric prerelease identifiers sort before alpha
        assert (
            anyver.compare(
                "1.0.0-alpha.1",
                "1.0.0-alpha.beta",
                ecosystem="semver",
            )
            == -1
        )

    def test_version_compare_semver_ecosystem(self):
        v = Version("1.0.0-alpha.1")
        assert v.compare("1.0.0-alpha.beta", ecosystem="semver") == -1

    def test_module_compare_version_inputs_semver_ecosystem(self):
        assert (
            anyver.compare(
                Version("1.0.0-alpha.1"),
                Version("1.0.0-alpha.beta"),
                ecosystem="semver",
            )
            == -1
        )

    def test_sort_semver_ecosystem(self):
        values = ["1.0.0-alpha.beta", "1.0.0-alpha.1", "1.0.0"]
        assert anyver.sort_versions(values, ecosystem="semver") == [
            "1.0.0-alpha.1",
            "1.0.0-alpha.beta",
            "1.0.0",
        ]

    def test_batch_compare_semver_ecosystem(self):
        pairs = [("1.0.0-alpha.1", "1.0.0-alpha.beta")]
        assert anyver.batch_compare(pairs, ecosystem="semver") == [-1]

    def test_bool_helpers_semver_ecosystem(self):
        assert anyver.lt("1.0.0-alpha.1", "1.0.0-alpha.beta", ecosystem="semver") is True
        assert anyver.gte("1.0.0", "1.0.0", ecosystem="semver") is True
        assert anyver.lte("1.0.0-alpha.1", "1.0.0", ecosystem="semver") is True

    def test_invalid_ecosystem_raises(self):
        with pytest.raises(ValueError):
            anyver.compare("1.0", "2.0", ecosystem="unknown")

    def test_invalid_semver_in_semver_ecosystem_raises(self):
        with pytest.raises(ValueError):
            anyver.compare("1.0", "2.0.0", ecosystem="semver")


# ============================================================================
# PEP 440
# ============================================================================


class TestPEP440:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0a1", "1.0a2", -1),
            ("1.0a2", "1.0b1", -1),
            ("1.0b1", "1.0rc1", -1),
            ("1.0rc1", "1.0", -1),
            ("1.0.dev1", "1.0a1", -1),
            ("1.0.dev1", "1.0", -1),
            ("1.0", "1.0.post1", -1),
            ("1.0.post1", "1.0.post2", -1),
            ("1!0.1", "2.0", 1),
        ],
    )
    def test_pep440(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# Debian/dpkg
# ============================================================================


class TestDebian:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0~alpha", "1.0~beta", -1),
            ("1.0~rc1", "1.0", -1),
            ("1.0~alpha", "1.0", -1),
            ("1:0.1", "2.0", 1),
            ("2:1.0", "1:2.0", 1),
            ("0:1.0", "1.0", 0),
            ("1.0+deb9u1", "1.0+deb9u2", 0),
        ],
    )
    def test_debian(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# RPM
# ============================================================================


class TestRPM:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0~rc1", "1.0", -1),
            ("1.0", "1.0^git1", -1),
            ("1.0-1^git1", "1.0-1.fc33", -1),
            # Native RPM tests for caret
            ("1.0^", "1.0", 1),
            ("1.0", "1.0^", -1),
            ("1.0^git1", "1.0", 1),
            ("1.0^git1", "1.0^git2", -1),
            ("1.0^git1", "1.01", -1),
            ("1.0^20160101", "1.0.1", -1),
            ("1.0~rc1^git1", "1.0~rc1", 1),
            ("1.0^git1~pre", "1.0^git1", -1),
        ],
    )
    def test_rpm(self, a, b, expected):
        assert anyver.compare(a, b, ecosystem="rpm") == expected


# ============================================================================
# Go modules
# ============================================================================


class TestGoModules:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("v1.0.0", "v1.0.1", -1),
            ("v1.0.0-alpha", "v1.0.0", -1),
            ("v2.0.0+incompatible", "v2.0.0", 0),
            ("v2.0.0+incompatible", "v2.0.1+incompatible", -1),
        ],
    )
    def test_go(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# Ruby Gems
# ============================================================================


class TestRubyGems:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0.0.pre", "1.0.0", -1),
            ("1.0.0.alpha", "1.0.0.beta", -1),
            ("1.0.0.beta", "1.0.0.rc1", -1),
            ("1.0.0.rc1", "1.0.0", -1),
            ("3.2", "3.10", -1),
        ],
    )
    def test_ruby(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# Maven
# ============================================================================


class TestMaven:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("1.0-alpha-1", "1.0-beta-1", -1),
            ("1.0-beta-1", "1.0-rc-1", -1),
            ("1.0-SNAPSHOT", "1.0", -1),
            ("1.0", "1.0-sp-1", -1),
        ],
    )
    def test_maven(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# Real-world packages
# ============================================================================


class TestRealWorld:
    @pytest.mark.parametrize(
        "a,b,expected",
        [
            ("4.2", "5.0a1", -1),
            ("5.0a1", "5.0b1", -1),
            ("5.0b1", "5.0rc1", -1),
            ("5.0rc1", "5.0", -1),
            ("7.1.0.beta1", "7.1.0.rc1", -1),
            ("7.1.0.rc1", "7.1.0", -1),
            ("18.17.0", "20.0.0", -1),
            ("5.15.0", "6.1.0", -1),
            ("5.3.30", "6.0.0", -1),
            ("2.31.0", "2.32.0", -1),
        ],
    )
    def test_real_world(self, a, b, expected):
        assert anyver.compare(a, b) == expected


# ============================================================================
# sort_versions
# ============================================================================


class TestSortVersions:
    def test_basic_sort(self):
        assert anyver.sort_versions(["2.0", "1.0-alpha", "1.0", "0.1"]) == [
            "0.1",
            "1.0-alpha",
            "1.0",
            "2.0",
        ]

    def test_full_sort(self):
        versions = [
            "2.0.0",
            "1.0.0-alpha",
            "1.0.0",
            "0.1.0",
            "1.0.0-rc.1",
            "1.0.0-beta",
            "3.0.0",
            "0.0.1",
        ]
        expected = [
            "0.0.1",
            "0.1.0",
            "1.0.0-alpha",
            "1.0.0-beta",
            "1.0.0-rc.1",
            "1.0.0",
            "2.0.0",
            "3.0.0",
        ]
        assert anyver.sort_versions(versions) == expected


# ============================================================================
# batch_compare
# ============================================================================


class TestBatchCompare:
    def test_batch(self):
        assert anyver.batch_compare([("1.0", "2.0"), ("2.0", "1.0"), ("1.0", "1.0")]) == [-1, 1, 0]


# ============================================================================
# min_version / max_version
# ============================================================================


class TestMinMax:
    def test_max(self):
        assert anyver.max_version(["1.0", "3.0", "2.0"]) == "3.0"

    def test_min(self):
        assert anyver.min_version(["1.0", "3.0", "2.0"]) == "1.0"

    def test_max_empty(self):
        with pytest.raises(ValueError):
            anyver.max_version([])

    def test_min_empty(self):
        with pytest.raises(ValueError):
            anyver.min_version([])


# ============================================================================
# Boolean helpers
# ============================================================================


class TestBooleanHelpers:
    def test_gt(self):
        assert anyver.gt("2.0", "1.0") is True

    def test_ge(self):
        assert anyver.ge("1.0", "1.0.0") is True

    def test_lt(self):
        assert anyver.lt("1.0", "2.0") is True

    def test_le(self):
        assert anyver.le("1.0", "1.0.0") is True

    def test_eq(self):
        assert anyver.eq("1.0", "1.0.0") is True

    def test_ne(self):
        assert anyver.ne("1.0", "2.0") is True


# ============================================================================
# to_dict
# ============================================================================


class TestToDict:
    def test_basic(self):
        v = Version("1.2.3-rc.1+build.42")
        d = v.to_dict()
        assert d == {
            "raw": "1.2.3-rc.1+build.42",
            "ecosystem": v.ecosystem,
            "epoch": 0,
            "major": 1,
            "minor": 2,
            "patch": 3,
            "build": "build.42",
            "is_prerelease": True,
            "is_postrelease": False,
        }

    def test_from_dict_roundtrip(self):
        for raw, eco in [
            ("1.2.3", "auto"),
            ("1!2.3.4.post1+local", "pep440"),
            ("v2.0.0+incompatible", "go"),
            ("1.0~rc1", "debian"),
        ]:
            v = Version(raw, ecosystem=eco)
            v2 = Version.from_dict(v.to_dict())
            assert v == v2
            assert v.ecosystem == v2.ecosystem
            assert v.raw == v2.raw


# ============================================================================
# sort_key
# ============================================================================


class TestSortKey:
    def test_basic(self):
        v = Version("1.2.3-rc.1+build.42")
        sk = v.sort_key()
        assert sk == (
            (2, 0, ""),  # epoch
            (2, 1, ""),  # major
            (2, 2, ""),  # minor
            (2, 3, ""),  # patch
            (1, 20, "rc"),  # rc (weight 20)
            (2, 1, ""),  # 1
            (1, 30, ""),  # release sentinel
        )

    def test_isomorphism_simple(self):
        """sort_key ordering matches compare for versions without trailing-zero ambiguity."""
        versions = ["0.0.1", "0.1.0", "1.0.0", "1.0.1", "2.0.0", "3.0.0", "10.0.0"]
        sorted_by_key = sorted(versions, key=lambda v: Version(v).sort_key())
        for i in range(len(sorted_by_key) - 1):
            a, b = sorted_by_key[i], sorted_by_key[i + 1]
            assert anyver.compare(a, b) <= 0, f"{a} should be <= {b}"

    def test_isomorphism_pep440(self):
        """sort_key ordering matches compare for PEP 440 dev/post versions."""
        versions = ["1.0.dev1", "1.0a1", "1.0b1", "1.0rc1", "1.0", "1.0.post1"]
        sorted_by_key = sorted(versions, key=lambda v: Version(v).sort_key())
        for i in range(len(sorted_by_key) - 1):
            a, b = sorted_by_key[i], sorted_by_key[i + 1]
            assert anyver.compare(a, b) < 0, f"{a} should be < {b}"

    def test_isomorphism_prerelease(self):
        """sort_key ordering matches compare for pre-release vs release."""
        versions = ["1.0.0-alpha", "1.0.0-beta", "1.0.0-rc", "1.0.0", "1.0.0-post"]
        sorted_by_key = sorted(versions, key=lambda v: Version(v).sort_key())
        for i in range(len(sorted_by_key) - 1):
            a, b = sorted_by_key[i], sorted_by_key[i + 1]
            assert anyver.compare(a, b) < 0, f"{a} should be < {b}"

    def test_sort_key_structure(self):
        """sort_key returns well-formed tuple of 3-element tuples."""
        sk = Version("1.0.0-alpha").sort_key()
        assert isinstance(sk, tuple)
        for item in sk:
            assert isinstance(item, tuple)
            assert len(item) == 3

    def test_equal_versions_same_key(self):
        """Equal versions must produce equal sort keys."""
        assert Version("1.0").sort_key() == Version("1.0.0").sort_key()
        assert Version("v1.0.0").sort_key() == Version("1.0.0").sort_key()


# ============================================================================
# Transitivity
# ============================================================================


class TestTransitivity:
    def test_trailing_zero_post_release(self):
        """1.0 == 1.0.0 and 1.0 < 1.0.post1 implies 1.0.0 < 1.0.post1"""
        assert anyver.compare("1.0", "1.0.0") == 0
        assert anyver.compare("1.0", "1.0.post1") == -1
        assert anyver.compare("1.0.0", "1.0.post1") == -1

    def test_trailing_zero_prerelease(self):
        """1.0 == 1.0.0 and 1.0.alpha < 1.0 implies 1.0.alpha < 1.0.0"""
        assert anyver.compare("1.0", "1.0.0") == 0
        assert anyver.compare("1.0.alpha", "1.0") == -1
        assert anyver.compare("1.0.alpha", "1.0.0") == -1


# ============================================================================
# Mixed types in sort_versions / batch_compare / min / max (Fix #5)
# ============================================================================


class TestMixedTypes:
    def test_sort_versions_mixed(self):
        result = anyver.sort_versions([Version("2.0"), "1.0", Version("0.5")])
        assert [str(v) for v in result] == ["0.5", "1.0", "2.0"]

    def test_sort_versions_all_version_objects(self):
        result = anyver.sort_versions([Version("3.0"), Version("1.0"), Version("2.0")])
        assert [str(v) for v in result] == ["1.0", "2.0", "3.0"]

    def test_batch_compare_mixed(self):
        result = anyver.batch_compare(
            [
                (Version("1.0"), "2.0"),
                ("2.0", Version("1.0")),
                (Version("1.0"), Version("1.0")),
            ]
        )
        assert result == [-1, 1, 0]

    def test_max_version_mixed(self):
        result = anyver.max_version([Version("1.0"), "3.0", Version("2.0")])
        assert str(result) == "3.0"

    def test_min_version_mixed(self):
        result = anyver.min_version([Version("1.0"), "3.0", Version("2.0")])
        assert str(result) == "1.0"

    def test_max_version_returns_original_type(self):
        v = Version("3.0")
        result = anyver.max_version(["1.0", v, "2.0"])
        assert isinstance(result, Version)

    def test_min_version_returns_original_type(self):
        result = anyver.min_version(["1.0", "3.0", "2.0"])
        assert isinstance(result, str)


# ============================================================================
# Strict SemVer validation (Fix #2)
# ============================================================================


class TestSemVerStrictValidation:
    def test_leading_zero_major(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("01.0.0", "1.0.0")

    def test_leading_zero_minor(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.01.0", "1.0.0")

    def test_leading_zero_patch(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.0.01", "1.0.0")

    def test_leading_zero_prerelease(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.0.0-01", "1.0.0")

    def test_empty_prerelease(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.0.0-", "1.0.0")

    def test_empty_prerelease_ident(self):
        with pytest.raises(ValueError):
            anyver.compare_semver_strict("1.0.0-alpha..1", "1.0.0")

    def test_valid_prerelease_with_hyphen(self):
        """SemVer allows hyphens in prerelease identifiers."""
        assert anyver.compare_semver_strict("1.0.0-alpha-1", "1.0.0") == -1

    def test_zero_parts_ok(self):
        """Single zero in version parts is valid."""
        assert anyver.compare_semver_strict("0.0.0", "0.0.1") == -1

    def test_zero_prerelease_ok(self):
        """Single zero as prerelease identifier is valid."""
        assert anyver.compare_semver_strict("1.0.0-0", "1.0.0") == -1


# ============================================================================
# Numeric overflow (Fix #1)
# ============================================================================


class TestOverflow:
    def test_overflow_does_not_become_zero(self):
        """Oversized numeric segments saturate at max, not 0."""
        v = Version("99999999999999999999999999999999")
        assert v.major > 0

    def test_overflow_greater_than_normal(self):
        assert anyver.compare("99999999999999999999999999999999", "999") == 1


# ============================================================================
# satisfies() — constraint matching (single + compound)
# ============================================================================


class TestSatisfies:
    @pytest.mark.parametrize(
        "version,constraint,expected",
        [
            # single-op constraints
            ("1.5.0", ">=1.0.0", True),
            ("1.5.0", ">=2.0.0", False),
            ("1.5.0", "<=2.0.0", True),
            ("1.5.0", "<=1.0.0", False),
            ("1.5.0", ">1.0.0", True),
            ("1.5.0", ">1.5.0", False),
            ("1.5.0", "<2.0.0", True),
            ("1.5.0", "<1.5.0", False),
            ("1.5.0", "==1.5.0", True),
            ("1.5.0", "==1.5.1", False),
            ("1.5.0", "!=1.0.0", True),
            ("1.5.0", "!=1.5.0", False),
            # trailing-zero equivalence flows through
            ("1.0.0", "==1.0", True),
            ("1.0", "==1.0.0", True),
        ],
    )
    def test_single_constraint(self, version, constraint, expected):
        assert anyver.satisfies(version, constraint) is expected

    @pytest.mark.parametrize(
        "version,constraint,expected",
        [
            # ranges
            ("1.5.0", ">=1.0.0,<2.0.0", True),
            ("2.0.0", ">=1.0.0,<2.0.0", False),  # upper bound exclusive
            ("0.9.0", ">=1.0.0,<2.0.0", False),  # below lower
            # ranges with spaces (parse_constraint trims each part)
            ("1.5.0", ">=1.0.0, <2.0.0", True),
            ("1.5.0", " >= 1.0.0 , < 2.0.0 ", True),
            # three-part constraint — all must match
            ("1.5.0", ">=1.0.0,<2.0.0,!=1.5.1", True),
            ("1.5.1", ">=1.0.0,<2.0.0,!=1.5.1", False),
        ],
    )
    def test_compound_constraint(self, version, constraint, expected):
        assert anyver.satisfies(version, constraint) is expected

    def test_ecosystem_semver_strict(self):
        # In strict SemVer, numeric prerelease identifiers sort before alpha.
        assert anyver.satisfies("1.0.0-alpha.1", "<1.0.0-alpha.beta", ecosystem="semver") is True

    def test_invalid_constraint_raises(self):
        with pytest.raises(ValueError):
            anyver.satisfies("1.0.0", "1.0.0")  # no operator

    def test_invalid_version_for_ecosystem_raises(self):
        with pytest.raises(ValueError):
            anyver.satisfies("1.0", ">=1.0.0", ecosystem="semver")


# ============================================================================
# stable_versions()
# ============================================================================


class TestStableVersions:
    def test_filters_prereleases(self):
        versions = ["1.0.0", "2.0.0-alpha", "2.0.0-rc1", "2.0.0", "1.5.0-beta"]
        result = anyver.stable_versions(versions)
        assert result == ["1.0.0", "2.0.0"]

    def test_keeps_postreleases(self):
        # post-releases are stable
        versions = ["1.0.0", "1.0.post1", "1.0.0-alpha"]
        result = anyver.stable_versions(versions)
        assert result == ["1.0.0", "1.0.post1"]

    def test_empty_input(self):
        assert anyver.stable_versions([]) == []

    def test_all_prerelease(self):
        assert anyver.stable_versions(["1.0-alpha", "2.0-rc1"]) == []

    def test_accepts_version_objects(self):
        versions = [Version("1.0.0"), Version("2.0.0-rc1"), Version("2.0.0")]
        result = anyver.stable_versions(versions)
        assert [str(v) for v in result] == ["1.0.0", "2.0.0"]

    def test_mixed_types(self):
        versions = ["1.0.0", Version("2.0.0-rc1"), Version("2.0.0"), "1.5.0-beta"]
        result = anyver.stable_versions(versions)
        assert [str(v) for v in result] == ["1.0.0", "2.0.0"]

    def test_preserves_order(self):
        # Filtering must not reorder
        versions = ["2.0.0", "1.0.0", "3.0.0-rc1", "1.5.0"]
        assert anyver.stable_versions(versions) == ["2.0.0", "1.0.0", "1.5.0"]

    def test_ecosystem_validation_rejects_invalid(self):
        with pytest.raises(ValueError):
            anyver.stable_versions(["1.0"], ecosystem="semver")


# ============================================================================
# latest_stable()
# ============================================================================


class TestLatestStable:
    def test_picks_latest_stable(self):
        versions = ["1.0.0", "2.0.0-rc1", "2.0.0", "1.5.0-beta"]
        assert anyver.latest_stable(versions) == "2.0.0"

    def test_ignores_prereleases_higher_than_stable(self):
        # A prerelease of a higher version must not beat a lower stable
        versions = ["1.0.0", "2.0.0-rc1"]
        assert anyver.latest_stable(versions) == "1.0.0"

    def test_includes_postreleases(self):
        versions = ["1.0.0", "1.0.post1"]
        assert anyver.latest_stable(versions) == "1.0.post1"

    def test_accepts_version_objects(self):
        versions = [Version("1.0.0"), Version("2.0.0-rc1"), Version("2.0.0")]
        result = anyver.latest_stable(versions)
        assert str(result) == "2.0.0"

    def test_empty_raises(self):
        with pytest.raises(ValueError):
            anyver.latest_stable([])

    def test_all_prerelease_raises(self):
        with pytest.raises(ValueError):
            anyver.latest_stable(["1.0-alpha", "2.0-rc1"])

    def test_semver_ecosystem(self):
        versions = ["1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0"]
        assert anyver.latest_stable(versions, ecosystem="semver") == "1.0.0"


# ============================================================================
# bump_major / bump_minor / bump_patch
# ============================================================================


class TestBumpHelpers:
    @pytest.mark.parametrize(
        "version,expected",
        [
            ("1.2.3", "2.0.0"),
            ("0.0.1", "1.0.0"),
            ("5", "6.0.0"),  # single-segment input
            ("1.2.3-alpha", "2.0.0"),  # prerelease stripped
            ("v1.2.3", "2.0.0"),  # v-prefix handled
        ],
    )
    def test_bump_major(self, version, expected):
        assert anyver.bump_major(version) == expected

    @pytest.mark.parametrize(
        "version,expected",
        [
            ("1.2.3", "1.3.0"),
            ("0.0.1", "0.1.0"),
            ("1.2.3-alpha", "1.3.0"),
            ("v1.2.3", "1.3.0"),
        ],
    )
    def test_bump_minor(self, version, expected):
        assert anyver.bump_minor(version) == expected

    @pytest.mark.parametrize(
        "version,expected",
        [
            ("1.2.3", "1.2.4"),
            ("0.0.0", "0.0.1"),
            ("1.2.3-alpha", "1.2.4"),
            ("v1.2.3", "1.2.4"),
        ],
    )
    def test_bump_patch(self, version, expected):
        assert anyver.bump_patch(version) == expected

    def test_bump_major_no_numeric_first_segment(self):
        # Non-numeric first segment → defaults to 1.0.0
        assert anyver.bump_major("alpha") == "1.0.0"


# ============================================================================
# bump_prerelease / next_stable
# ============================================================================


class TestBumpPrerelease:
    def test_fresh_from_release(self):
        assert anyver.bump_prerelease("1.2.3") == "1.2.3-alpha.0"

    def test_increment_same_tag(self):
        assert anyver.bump_prerelease("1.2.3-alpha.1") == "1.2.3-alpha.2"

    def test_tag_switch_resets_counter(self):
        assert anyver.bump_prerelease("1.2.3-alpha.5", "beta") == "1.2.3-beta.0"

    def test_custom_tag(self):
        assert anyver.bump_prerelease("1.2.3", "rc") == "1.2.3-rc.0"
        assert anyver.bump_prerelease("1.2.3-rc.2", "rc") == "1.2.3-rc.3"

    def test_missing_parts_fill_zero(self):
        assert anyver.bump_prerelease("1") == "1.0.0-alpha.0"


class TestNextStable:
    @pytest.mark.parametrize(
        "version,expected",
        [
            ("1.2.3", "1.2.3"),
            ("1.2.3-alpha.1", "1.2.3"),
            ("1.2.3-rc.2+build", "1.2.3"),
            ("v1.0", "1.0.0"),
            ("1.2", "1.2.0"),
        ],
    )
    def test_strip_prerelease(self, version, expected):
        assert anyver.next_stable(version) == expected


# ============================================================================
# Pickle / copy / str comparison / classmethods
# ============================================================================


class TestPickle:
    def test_roundtrip(self):
        import pickle

        v = Version("1!2.3.4.post1+local", ecosystem="pep440")
        p = pickle.loads(pickle.dumps(v))
        assert p == v
        assert p.ecosystem == v.ecosystem
        assert p.raw == v.raw

    def test_roundtrip_auto(self):
        import pickle

        v = Version("1.2.3")
        p = pickle.loads(pickle.dumps(v))
        assert p == v
        assert p.ecosystem == v.ecosystem

    def test_deepcopy(self):
        import copy

        v = Version("1.0~rc1", ecosystem="debian")
        d = copy.deepcopy(v)
        assert d == v
        assert d.ecosystem == v.ecosystem


class TestStrComparison:
    def test_eq_str(self):
        assert Version("1.0") == "1.0.0"

    def test_lt_str(self):
        assert Version("1.0") < "2.0"

    def test_ge_str(self):
        assert Version("2.0") >= "1.5"

    def test_ne_str(self):
        assert Version("1.0") != "2.0"

    def test_notimplemented_other_type(self):
        # Non-Version, non-str falls back to Python's rules → != returns True,
        # == returns False; no TypeError.
        assert (Version("1.0") == 123) is False
        assert (Version("1.0") != 123) is True

    def test_ordering_with_other_type_raises(self):
        import pytest

        with pytest.raises(TypeError):
            _ = Version("1.0") < 123


class TestParseClassmethods:
    def test_parse(self):
        v = Version.parse("1.2.3")
        assert v == Version("1.2.3")

    def test_parse_with_ecosystem(self):
        v = Version.parse("1!2.0", ecosystem="pep440")
        assert v.epoch == 1

    def test_parse_raises_on_invalid(self):
        import pytest

        with pytest.raises(ValueError):
            Version.parse("not-a-version", ecosystem="semver")

    def test_try_parse_valid(self):
        assert Version.try_parse("1.2.3") == Version("1.2.3")

    def test_try_parse_invalid(self):
        assert Version.try_parse("not-a-version", ecosystem="semver") is None


# ============================================================================
# Extended satisfies ranges
# ============================================================================


class TestSatisfiesShorthand:
    @pytest.mark.parametrize(
        "version,constraint,expected",
        [
            # caret
            ("1.2.3", "^1.0.0", True),
            ("1.9.9", "^1.0.0", True),
            ("2.0.0", "^1.0.0", False),
            ("0.2.5", "^0.2.3", True),
            ("0.3.0", "^0.2.3", False),
            ("0.0.3", "^0.0.3", True),
            ("0.0.4", "^0.0.3", False),
            # tilde
            ("1.2.5", "~1.2.3", True),
            ("1.3.0", "~1.2.3", False),
            ("1.2.0", "~1.2", True),
            ("1.3.0", "~1.2", False),
            ("1.5.0", "~1", True),
            ("2.0.0", "~1", False),
            # x-range
            ("1.2.5", "1.2.x", True),
            ("1.3.0", "1.2.x", False),
            ("1.0.0", "1.x", True),
            ("2.0.0", "1.x", False),
            ("99.0.0", "*", True),
            # OR
            ("1.5.0", "^1.0.0 || ^2.0.0", True),
            ("2.3.0", "^1.0.0 || ^2.0.0", True),
            ("3.0.0", "^1.0.0 || ^2.0.0", False),
            # mixed
            ("1.5.0", ">=1.0.0,<2.0.0", True),
            ("1.5.0", ">=1.0.0 || >=3.0.0", True),
        ],
    )
    def test_shorthand(self, version, constraint, expected):
        assert anyver.satisfies(version, constraint) is expected


class TestModuleSurface:
    def test_version_attribute(self):
        # should be a string, not empty
        assert isinstance(anyver.__version__, str)
        assert anyver.__version__

    def test_py_typed_marker_present(self):
        import os

        import anyver as pkg

        pkg_dir = os.path.dirname(pkg.__file__)
        assert os.path.exists(os.path.join(pkg_dir, "py.typed"))


# ============================================================================
# PEP 440 local-version ordering (public/public > public/no-local) — the
# generic parser used to drop `+LOCAL` entirely.
# ============================================================================


class TestPep440LocalVersion:
    def test_local_sorts_after_release(self):
        assert anyver.compare("1.0+abc", "1.0", ecosystem="pep440") == 1
        assert anyver.compare("1.0", "1.0+abc", ecosystem="pep440") == -1

    def test_local_equal_to_self(self):
        assert anyver.compare("1.0+abc", "1.0+abc", ecosystem="pep440") == 0

    def test_local_lex_ordering(self):
        assert anyver.compare("1.0+abc", "1.0+abd", ecosystem="pep440") == -1
        assert anyver.compare("1.0+abd", "1.0+abc", ecosystem="pep440") == 1

    def test_local_longer_wins_when_prefix_matches(self):
        assert anyver.compare("1.0+a.1", "1.0+a", ecosystem="pep440") == 1

    def test_local_numeric_beats_alpha(self):
        # Per PEP 440: numeric segments sort greater than alpha.
        assert anyver.compare("1.0+1", "1.0+a", ecosystem="pep440") == 1

    def test_local_separator_normalization(self):
        # PEP 440 treats `.`, `-`, `_` as equivalent separators in local versions.
        assert anyver.compare("1.0+a.b", "1.0+a-b", ecosystem="pep440") == 0
        assert anyver.compare("1.0+a.b", "1.0+a_b", ecosystem="pep440") == 0

    def test_public_ordering_still_wins(self):
        # Public part takes precedence over local.
        assert anyver.compare("1.0+zzz", "2.0", ecosystem="pep440") == -1
        assert anyver.compare("2.0", "1.0+zzz", ecosystem="pep440") == 1


# ============================================================================
# Maven release-alias qualifiers (`final`, `ga`, `release`) compare equal
# to the bare release — the generic parser used to treat them as unknown text
# and sort `1.0-final` below `1.0`.
# ============================================================================


class TestMavenReleaseAliases:
    @pytest.mark.parametrize("alias", ["final", "ga", "release"])
    def test_alias_equal_to_release(self, alias):
        assert anyver.compare(f"1.0-{alias}", "1.0", ecosystem="maven") == 0
        assert anyver.compare(f"1.0-{alias}", "1.0.0", ecosystem="maven") == 0

    def test_alias_below_sp(self):
        # Maven qualifier ordering: release < sp.
        assert anyver.compare("1.0-final", "1.0-sp-1", ecosystem="maven") == -1

    def test_alias_above_snapshot(self):
        assert anyver.compare("1.0-SNAPSHOT", "1.0-final", ecosystem="maven") == -1
        assert anyver.compare("1.0-SNAPSHOT", "1.0-ga", ecosystem="maven") == -1

    def test_alias_in_hash(self):
        # Hash contract: equal versions must hash equal.
        assert hash(Version("1.0-final", ecosystem="maven")) == hash(
            Version("1.0", ecosystem="maven")
        )

    def test_alias_works_in_generic_mode_too(self):
        # Also covered by the generic comparator since normalized() is shared.
        assert anyver.compare("1.0-final", "1.0") == 0
