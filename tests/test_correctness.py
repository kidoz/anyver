"""Regressions for version, range, and security interoperability contracts."""

from itertools import permutations, product

import pytest

import anyver
from anyver import Version


def sign(value):
    return (value > 0) - (value < 0)


@pytest.mark.parametrize("package", ["PyPI", "npm", "unknown"])
def test_osv_semver_uses_range_type(package):
    affected = {
        "package": {"ecosystem": package},
        "ranges": [
            {
                "type": "SEMVER",
                "events": [
                    {"introduced": "1.0.0-alpha.1"},
                    {"fixed": "1.0.0-alpha.beta"},
                ],
            }
        ],
    }
    assert anyver.osv_affected("1.0.0-alpha.1", affected)
    assert not anyver.osv_affected("1.0.0-alpha.beta", affected)
    assert anyver.osv_affected("1.0.0-alpha.2", affected, ecosystem="pep440")


def test_osv_npm_ecosystem_windows_do_not_apply_npm_admission():
    affected = {
        "package": {"ecosystem": "npm"},
        "ranges": [
            {"type": "ECOSYSTEM", "events": [{"introduced": "0"}, {"fixed": "2.0.0"}]},
        ],
    }
    assert anyver.osv_affected("1.1.0-alpha", affected)


@pytest.mark.parametrize(
    "constraints,expected",
    [
        (["<2.0.0", ">=2.0.0"], True),
        (["2.0.0", "!=2.0.0"], False),
        (["<2.0.0", ">2.0.0"], False),
        (["<=2.0.0", ">2.0.0"], True),
        (["2.0.0", ">=2.0.0", "!=2.0.0"], False),
    ],
)
def test_vers_constraints_are_unordered(constraints, expected):
    for order in permutations(constraints):
        assert anyver.vers_contains("vers:npm/" + "|".join(order), "2.0.0") is expected


def test_vers_duplicate_bound_directions_apply_between_bounds():
    constraints = ["<1.0.0", ">=1.0.0", "<3.0.0", ">=3.0.0"]
    for order in permutations(constraints):
        assert anyver.vers_contains("vers:npm/" + "|".join(order), "2.0.0")


@pytest.mark.parametrize("spec", [">=1.0.0 <2.0.0", ">=1.0.0,<2.0.0", ">=1.0.0|<2.0.0"])
def test_build_vers_round_trip(spec):
    built = anyver.build_vers("npm", spec)
    assert built == "vers:npm/>=1.0.0|<2.0.0"
    assert anyver.vers_contains(built, "1.5.0")
    assert not anyver.vers_contains(built, "2.0.0")


@pytest.mark.parametrize("spec", ["^1.2.3", "~1.2", "1.2.x", ">=1.2.x", "1.2.*"])
def test_build_vers_rejects_shorthand(spec):
    with pytest.raises(ValueError):
        anyver.build_vers("npm", spec)


@pytest.mark.parametrize("value", ["1.2.3+", "1.2.3+foo..bar", "1.2.3+foo_bar", "1.2.3+foo+bar"])
def test_semver_build_grammar(value):
    assert Version.try_parse(value, ecosystem="semver") is None
    with pytest.raises(ValueError):
        anyver.compare_semver_strict(value, "1.2.3")


def test_semver_valid_build_ignored():
    assert anyver.compare_semver_strict("1.2.3+001.foo-bar", "1.2.3") == 0


@pytest.mark.parametrize(
    "value", ["1..0", "1.0+", "1!2!3", "1..a1", "1.0a..1", "1.0+abc..1", "1.0wat", "1.0.post1rc1"]
)
def test_pep440_rejects_malformed(value):
    assert Version.try_parse(value, ecosystem="pep440") is None


@pytest.mark.parametrize(
    "a,b,expected",
    [
        ("1.0rc1", "1.0.0rc1", 0),
        ("1.0-1", "1.0.post1", 0),
        ("1.0+abc1", "1.0+abc.1", 1),
        ("1.0+abc.0", "1.0+abc", 1),
        ("1.0a", "1.0alpha0", 0),
        ("1.0r1", "1.0.post1", 0),
        ("1.0.dev1", "1.0a0", -1),
        ("1.0a1.dev1", "1.0a1", -1),
        ("1.0.post1.dev1", "1.0.post1", -1),
        ("1.0+1", "1.0+abc", 1),
    ],
)
def test_pep440_comparison(a, b, expected):
    assert anyver.compare(a, b, ecosystem="pep440") == expected


@pytest.mark.parametrize(
    "eco,a,b",
    [
        ("rpm", "1.0+a", "1.0.a"),
        ("rpm", "1.01", "1.1"),
        ("debian", "1.0", "1.0-0"),
        ("debian", "1.01", "1.1"),
        ("pep440", "1.0rc1", "1.0.0RC1"),
        ("pep440", "1.0-1", "1.0.post1"),
        ("maven", "1-a1", "1-alpha-1"),
        ("generic", "1.0-a", "1-alpha"),
        ("semver", "1.0.0+build", "1.0.0+other"),
    ],
)
def test_equal_versions_have_identical_hashes(eco, a, b):
    left, right = Version(a, ecosystem=eco), Version(b, ecosystem=eco)
    assert left == right
    assert hash(left) == hash(right)
    assert len({left, right}) == 1
    assert {left: "found"}[right] == "found"


def test_equality_domain():
    generic = Version("1.0", ecosystem="generic")
    a, b = (Version("1.0+" + label, ecosystem="pep440") for label in ["a", "b"])
    assert a != b and a != generic and b != generic
    assert generic != "1.0.0" and "1.0.0" != generic
    assert generic.compare("1.0.0") == 0
    with pytest.raises(TypeError):
        _ = a < generic


@pytest.mark.parametrize(
    "eco,values",
    [
        (
            "generic",
            [
                "1",
                "1.0",
                "1.0.0-rc1",
                "1.0-rc1",
                "1-post",
                "1-a",
                "1-alpha",
                "1-final7",
                "1.7",
                "1.18446744073709551615",
                "1.9223372036854775808",
                "1.9223372036854775807",
            ],
        ),
        (
            "semver",
            [
                "1.0.0-alpha",
                "1.0.0-alpha.0",
                "1.0.0-alpha.1",
                "1.0.0-alpha.beta",
                "1.0.0-Alpha",
                "1.0.0",
                "1.0.0+foo",
                "2.0.0",
            ],
        ),
        (
            "pep440",
            [
                "1.0",
                "1.0.0",
                "1.0rc1",
                "1.0.0rc1",
                "1.0.dev0",
                "1.0.post1.dev1",
                "1.0.post1",
                "1.0+abc",
                "1.0+abc1",
                "1.0+abc.1",
                "1.0+1",
                "1!0.1",
            ],
        ),
    ],
)
def test_sort_keys_match_all_pairwise_comparisons(eco, values):
    for a, b in product(values, repeat=2):
        left, right = Version(a, ecosystem=eco), Version(b, ecosystem=eco)
        ka, kb = left.sort_key(), right.sort_key()
        assert sign((ka > kb) - (ka < kb)) == anyver.compare(a, b, ecosystem=eco), (eco, a, b)
        assert (left == right) == (ka == kb)


@pytest.mark.parametrize(
    ("ecosystem", "lower", "upper"),
    [
        ("generic", "1.9", "1.10"),
        ("semver", "1.0.0-1", "1.0.0-alpha"),
        ("npm", "1.0.0-rc.2", "1.0.0-rc.10"),
        ("pep440", "1.0+linux.2", "1.0+linux.10"),
        ("debian", "1:1.0+deb1-2", "1:1.0+deb1-10"),
        ("rpm", "1.0^git1-1", "1.0.1-1"),
        ("ruby", "1.0.pre", "1.0"),
        ("maven", "1.0-SNAPSHOT", "1.0"),
        ("go", "v1.0.0-rc.1", "v1.0.0"),
        ("nuget", "1.0.0.2", "1.0.0.10"),
        ("composer", "v1.0-beta", "v1.0"),
        ("crates", "1.0.0-alpha", "1.0.0"),
        ("hex", "1.0.0-rc.2", "1.0.0-rc.10"),
        ("swift", "1.0.0-beta", "1.0.0"),
        ("calver", "2024.09", "2024.10"),
        ("alpine", "1.0-r2", "1.0-r10"),
        ("docker", "latest", "stable"),
        ("conda", "1.0", "1.1"),
        ("arch", "1:1.0-2", "2:1.0-1"),
        ("gentoo", "1.0_alpha", "1.0"),
        ("cran", "1.0-2", "1.0-10"),
        ("spack", "1.9", "1.10"),
        ("dart", "1.0.0-dev.2", "1.0.0"),
        ("julia", "1.0.0-rc1", "1.0.0"),
        ("terraform", "1.9.0", "1.10.0"),
        ("helm", "1.0.0-beta.2", "1.0.0"),
        ("cpan", "v1.2.9", "v1.2.10"),
        ("nix", "23.11", "24.05"),
        ("homebrew", "1.0_2", "1.0_10"),
        ("freebsd", "1.0_2", "1.0_10"),
        ("hackage", "1.2", "1.2.0"),
        ("jdk", "21.0.1+9", "21.0.1+10"),
    ],
)
def test_registered_ecosystem_entry_points(ecosystem, lower, upper):
    left, right = Version(lower, ecosystem=ecosystem), Version(upper, ecosystem=ecosystem)
    assert left.ecosystem == right.ecosystem == ecosystem
    assert left < right
    assert left.sort_key() < right.sort_key()
    assert anyver.compare(left, right, ecosystem=ecosystem) == -1
    assert anyver.sort_versions([upper, lower], ecosystem=ecosystem) == [lower, upper]


def test_hackage_dotted_versions_preserve_precision():
    assert Version("1.2.0", ecosystem="Cabal").release() == (1, 2, 0)
    assert len({Version(value, ecosystem="hackage") for value in ["1.2", "1.2.0"]}) == 2
    assert anyver.sort_versions(["1.10", "1.2.1", "1.2.0", "1.2"], ecosystem="hackage") == [
        "1.2",
        "1.2.0",
        "1.2.1",
        "1.10",
    ]
    assert anyver.satisfies("1.2.0", ">1.2,<1.2.1", ecosystem="hackage")
    for value in ["", "v1.2", "1..2", "1.02", "1.2-rc1", "1.2+3"]:
        assert Version.try_parse(value, ecosystem="hackage") is None


def test_jdk_runtime_precedence_and_stability():
    ordered = [
        "21-2",
        "21-10",
        "21-184467440737095516160",
        "21-EA",
        "21-ea",
        "21-ea+10",
        "21",
        "21+-vendor",
        "21+0",
        "21+2",
        "21+10",
        "21+10-LTS",
        "21+10-vendor",
        "21.0.1",
    ]
    versions = [Version(value, ecosystem="openjdk") for value in ordered]
    for left, right in zip(versions, versions[1:], strict=False):
        assert left < right
        assert left.sort_key() < right.sort_key()
    equal = Version("21-002", ecosystem="jdk")
    assert versions[0] == equal
    assert hash(versions[0]) == hash(equal)
    assert versions[0].sort_key() == equal.sort_key()
    assert equal.release() == (21,)
    assert equal.minor == 0
    assert equal.is_prerelease
    assert Version("21+-ea", ecosystem="jdk").is_stable
    assert Version("21+10-LTS", ecosystem="java-runtime").build == "10-LTS"
    assert anyver.latest_stable(["21+10", "22-ea"], ecosystem="jdk") == "21+10"
    assert anyver.satisfies("21+10", ">=21+9,<21.0.1", ecosystem="jdk")


@pytest.mark.parametrize(
    "value",
    ["0", "21.0", "21.01", "21+", "21-ea+-vendor", "21+01", "21--vendor", "21+2147483648"],
)
def test_jdk_rejects_invalid_runtime_versions(value):
    assert Version.try_parse(value, ecosystem="jdk") is None


def test_semver_large_numeric_prereleases():
    ordered = [
        "1.0.0-18446744073709551615",
        "1.0.0-18446744073709551616",
        "1.0.0-100000000000000000000",
        "1.0.0-alpha",
        "1.0.0",
    ]
    for lower, upper in zip(ordered, ordered[1:], strict=False):
        assert anyver.compare_semver_strict(lower, upper) == -1
        assert Version(lower, "semver").sort_key() < Version(upper, "semver").sort_key()
    assert Version("1.0.0-1", "semver").release() == (1, 0, 0)


def test_nuget_uses_case_insensitive_version_release_precedence():
    assert anyver.compare("1.2.3-1", "1.2.3-alpha", "nuget") == -1
    assert anyver.compare("1.2.3-rc", "1.2.3-rc.0", "nuget") == -1
    assert anyver.compare("1.2.3-10000000000", "1.2.3-2147483648", "nuget") == -1
    assert anyver.latest_stable(["1.2.3-1", "1.2"], "nuget") == "1.2"
    left, right = Version("1.02-RC+x", "nuget"), Version("1.2.0.0-rc+y", "nuget")
    assert left == right
    assert hash(left) == hash(right)
    assert left.sort_key() == right.sort_key()
    assert Version("1-2", "nuget").release() == (1,)
    for malformed in ["1.2.3.4.5", "1.2.3-01", "1.2.3-b+", "1..2"]:
        assert Version.try_parse(malformed, "nuget") is None


def test_rubygems_raw_comparison_and_portable_sorting():
    assert anyver.compare("1.0.a", "1.0.A", "ruby") == 1
    assert anyver.compare("1.0-1", "1.0", "ruby") == -1
    assert anyver.compare("1.0-rc1", "1.0.pre.rc1", "ruby") == 0
    assert anyver.latest_stable(["2.0.final", "1.0"], "ruby") == "1.0"
    assert Version.try_parse("1.2+foo", "ruby") is None
    a, equivalent, b = (Version(value, "ruby") for value in ("1.0.a", "1.a", "1.b"))
    assert a == equivalent
    assert a > b > equivalent
    assert hash(a) == hash(equivalent)
    assert a.sort_key() == equivalent.sort_key() < b.sort_key()
    assert anyver.sort_versions(["1.b", "1.0.a", "1.a"], "ruby") == ["1.0.a", "1.a", "1.b"]


def test_cpan_decimal_dotted_and_alpha_versions():
    a, b = Version("1.002003", "cpan"), Version("v1.2.3", "cpan")
    assert a == b
    assert hash(a) == hash(b)
    assert a.sort_key() == b.sort_key()
    assert anyver.compare("1.02", "1.2", "cpan") == -1
    assert anyver.compare("1.23_4", "1.234", "cpan") == 0
    assert Version("1.23_4", "cpan").is_prerelease
    assert anyver.latest_stable(["1.23_4", "1.23"], "cpan") == "1.23"
    assert Version.try_parse("1.2_3.4", "cpan") is None


def test_cran_requires_numeric_components():
    assert anyver.compare("1.2-3", "1.2.3", "cran") == 0
    for malformed in ["1", "1..2", "1.-2", "1.2-"]:
        assert Version.try_parse(malformed, "cran") is None


def test_alpine_package_suffixes_commits_and_revisions():
    versions = [
        "1.0_pre",
        "1.0_rc",
        "1.0_rc0",
        "1.0",
        "1.0-r0",
        "1.0-r1",
        "1.0~abc",
        "1.0_git",
        "1.0_p",
        "1.0a",
        "1.0.0",
        "1.01",
        "1.1",
    ]
    assert anyver.sort_versions(versions[::-1], "alpine") == versions
    for text in ["1.0a.1", "1.0-r", "1.0~ZZ", "1.0_p1a"]:
        assert Version.try_parse(text, "alpine") is None
    assert Version("1.0_rc1-r2", "apk").is_prerelease
    assert Version("1.0-r0", "apk").is_postrelease
    assert anyver.latest_stable(["1.0-r0", "1.1_rc1-r2"], "apk") == "1.0-r0"
    assert anyver.compare("01.0-r00", "1.0-r0", "apk") == 0


def test_arch_package_release_and_separator_order():
    assert anyver.compare("1.0", "1.0.0", "arch") == -1
    assert anyver.compare("1a", "1.a", "arch") == -1
    assert anyver.compare("1...a", "1.1", "arch") == 1
    assert anyver.compare("1.0+1", "1.0.1", "arch") == 0
    bare, r0, r1 = (Version(value, "pacman") for value in ("1.0", "1.0-0", "1.0-1"))
    assert bare == r0
    assert bare == r1
    assert r0 < r1
    assert hash(bare) == hash(r0) == hash(r1)
    assert bare.sort_key() < r0.sort_key() < r1.sort_key()
    assert anyver.sort_versions(["1.0-1", "1.0", "1.0-0"], "arch") == [
        "1.0",
        "1.0-0",
        "1.0-1",
    ]


def test_debian_rejects_invalid_epoch_and_empty_revision():
    assert Version("0001:1.0-1", "debian").epoch == 1
    signed = Version("+1:1.0", "debian")
    assert signed.epoch == 1
    assert signed.raw == "+1:1.0"
    assert signed == Version("1:1.0", "debian")
    assert anyver.compare("2147483647:1:2", "2147483646:10", "debian") == 1
    for malformed in ["-0:1.0", ":1.0", "1:", "2147483648:1", "1.0-", "1.0 2"]:
        assert Version.try_parse(malformed, "debian") is None


def test_docker_validates_tag_syntax():
    assert Version.try_parse("_Release.1-rc", "docker") is not None
    assert Version.try_parse("x" * 128, "docker") is not None
    for malformed in ["a:", ".initial", "-initial", "has+plus", "x" * 129, "有効"]:
        assert Version.try_parse(malformed, "docker") is None


def test_homebrew_orders_formula_revisions_after_the_upstream_version():
    assert anyver.compare("1.0_1", "1.0.1", "homebrew") == -1
    assert anyver.compare("1.0", "1.0_0", "brew") == 0
    assert anyver.compare("HEAD-abc", "HEAD-def", "homebrew") == 0
    assert anyver.sort_versions(["HEAD", "1.0_10", "1.0.1", "1.0_1"], "homebrew") == [
        "1.0_1",
        "1.0_10",
        "1.0.1",
        "HEAD",
    ]


def test_maven_sort_key_breaks_only_the_cycle_edge():
    alpha = Version("1-foo-alpha", ecosystem="maven")
    release = Version("1-foo", ecosystem="maven")
    sp = Version("1-foo.sp-0", ecosystem="maven")
    assert sp < alpha
    assert alpha < release
    assert release < sp
    assert alpha.sort_key() < release.sort_key() < sp.sort_key()


@pytest.mark.parametrize("spec", ["^1.2.3-beta.1", "~1.2.3-beta.1"])
def test_shorthand_preserves_prerelease(spec):
    assert anyver.satisfies("1.2.3-beta.1", spec, ecosystem="npm")
    assert anyver.satisfies("1.2.3-beta.2", spec, ecosystem="npm")
    assert not anyver.satisfies("1.2.4-beta.1", spec, ecosystem="npm")


def test_npm_prerelease_admission_is_per_branch():
    assert not anyver.satisfies("1.1.0-alpha", ">=1.0.0", ecosystem="npm")
    assert not anyver.satisfies("1.1.0-alpha", "*", ecosystem="npm")
    assert anyver.satisfies("1.1.0-alpha", ">=1.1.0-alpha", ecosystem="npm")
    assert not anyver.satisfies("1.1.0-alpha", ">=1.1.0 || <1.0.0-alpha", ecosystem="npm")


@pytest.mark.parametrize("eco", ["terraform", "hex"])
def test_pessimistic_upper_bound(eco):
    assert anyver.satisfies("1.9.0", "~>1.2", ecosystem=eco)
    assert not anyver.satisfies("2.0.0", "~>1.2", ecosystem=eco)
    assert not anyver.satisfies("1.3.0", "~>1.2.3", ecosystem=eco)


@pytest.mark.parametrize(
    "spec",
    [
        "18446744073709551615.x",
        "1.18446744073709551615.x",
        "<=18446744073709551615.x",
        "^18446744073709551615.0.0",
        "~1.18446744073709551615.0",
        "^0.0.18446744073709551615",
    ],
)
def test_range_overflow_raises(spec):
    with pytest.raises(ValueError, match="exceeds u64"):
        anyver.satisfies("1.2.3", spec)


@pytest.mark.parametrize(
    "function,expected",
    [
        (anyver.sort_versions, ["1.0", "2.0"]),
        (anyver.min_version, "1.0"),
        (anyver.max_version, "2.0"),
        (anyver.stable_versions, ["2.0", "1.0"]),
        (anyver.latest_stable, "2.0"),
    ],
)
def test_iterable_inputs(function, expected):
    assert function(v for v in ["2.0", "1.0"]) == expected


def test_batch_compare_iterator_and_runtime_exports():
    assert anyver.batch_compare(pair for pair in [("1", "2"), ("2", "1")]) == [-1, 1]
    assert anyver.VersionLike == Version | str
    v = Version("1.2.3-rc1")
    assert tuple(v.__iter__()) == v.segments()


@pytest.mark.parametrize(
    "spec,inside,outside", [("^0", "0.9.0", "1.0.0"), ("^0.0", "0.0.9", "0.1.0")]
)
def test_caret_partial_zero_precision(spec, inside, outside):
    assert anyver.satisfies(inside, spec, ecosystem="npm")
    assert not anyver.satisfies(outside, spec, ecosystem="npm")


def test_vers_accepts_x_as_prerelease_identifier():
    built = anyver.build_vers("npm", ">=1.2.3-alpha.x")
    assert anyver.vers_contains(built, "1.2.3-alpha.x")


@pytest.mark.parametrize(
    "function",
    [
        anyver.sort_versions,
        anyver.batch_compare,
        anyver.min_version,
        anyver.max_version,
        anyver.stable_versions,
        anyver.latest_stable,
    ],
)
def test_iterator_errors_are_preserved(function):
    def broken():
        raise RuntimeError("iterator failed")
        yield  # pragma: no cover

    with pytest.raises(RuntimeError, match="iterator failed"):
        function(broken())


@pytest.mark.parametrize("bound", ["1.2.3-rc.x", "1.2.3+linux.x86"])
def test_exact_semver_suffix_is_not_a_wildcard(bound):
    assert anyver.satisfies(bound, f"=={bound}", "semver")
    assert not anyver.satisfies("1.2.9", f"=={bound}", "semver")
    assert anyver.satisfies("1.2.9", "1.2.x", "semver")


def test_osv_limits_apply_to_the_whole_range():
    events = [
        {"introduced": "0"},
        {"fixed": "1.0.0"},
        {"introduced": "2.0.0"},
        {"limit": "3.0.0"},
        {"limit": "2.1.0"},
    ]
    record = {"ranges": [{"type": "SEMVER", "events": events}]}
    for version, expected in [
        ("0.9.0", True),
        ("1.5.0", False),
        ("2.9.9", True),
        ("3.0.0", False),
        ("3.1.0", False),
    ]:
        assert anyver.osv_affected(version, record) is expected
    record["versions"] = ["3.1.0"]
    assert anyver.osv_affected("3.1.0", record)
    record["ranges"].append({"type": "SEMVER", "events": [{"introduced": "4.0.0"}]})
    assert anyver.osv_affected("4.0.0", record)
    events.insert(0, {"limit": "*"})
    assert anyver.osv_affected("3.0.0", record)


@pytest.mark.parametrize(
    "eco,value,post",
    [
        ("semver", "2.0.0-1", False),
        ("npm", "2.0.0-nightly.42", False),
        ("pep440", "2.0.post2.dev1", True),
    ],
)
def test_prerelease_flags_agree_with_selectors(eco, value, post):
    obj = Version(value, eco)
    assert obj.is_prerelease and not obj.is_stable
    assert obj.is_postrelease is post
    assert obj.to_dict()["is_prerelease"]
    assert obj.to_dict()["is_postrelease"] is post
    for candidate in [value, obj, Version(value, "generic")]:
        assert anyver.stable_versions(["1.9.0", candidate], eco) == ["1.9.0"]
        assert anyver.latest_stable(["1.9.0", candidate], eco) == "1.9.0"
    assert Version("2.0-1", "pep440").is_postrelease


@pytest.mark.parametrize(
    "spec,target,expected",
    [
        ("~=1.4", "1.9", True),
        ("~=V01.04", "1.9", True),
        ("~=1.4", "2.0.dev0", False),
        ("~=1.4.0", "1.5", False),
        ("~=1.4.0", "1.4.7", True),
        ("~=1!1.4rc1", "1!1.4", True),
        ("~=1!1.4rc1", "1!1.4rc0", False),
        ("~=1.2.3.4", "1.2.4.dev0", False),
        ("~=1.2.3.4", "1.2.3.9", True),
    ],
)
def test_pep440_compatible_release(spec, target, expected):
    assert anyver.range_contains(target, spec, "pep440") is expected
    assert anyver.ranges_intersect(spec, "==" + target, "pep440") is expected


@pytest.mark.parametrize("spec", ["~=1", "~=1.4+linux"])
def test_invalid_pep440_compatible_release(spec):
    with pytest.raises(ValueError):
        anyver.range_contains("1.4", spec, "pep440")


@pytest.mark.parametrize(
    "left,right,overlap",
    [
        (">1.0.0", "<1.0.1", False),
        (">1.0.0", "<=1.0.1", True),
        (">1.0.0", "<1.0.1-beta", False),
        (">=1.0.1-alpha", "<1.0.1-beta", True),
        (">1.0.1-alpha", "<1.0.1-alpha.0", False),
        (">1.0.1-alpha", "<=1.0.1-alpha.0", True),
        ("<=1.0.1-alpha", "<1.0.1-beta", True),
        (">=1.0.0,!=1.0.0,!=1.0.1", "<1.0.2", False),
        (">=1.0.0,!=1.0.0,!=1.0.1", "<=1.0.2", True),
        ("<0.0.0", "*", False),
        (">=0.0.0-0", "<0.0.0-0.0", True),
        (">1.0.18446744073709551615", "<=1.1.0", True),
        (">18446744073709551615.18446744073709551615.18446744073709551615", "*", False),
    ],
)
def test_npm_intersection_has_an_admitted_version(left, right, overlap):
    assert anyver.ranges_intersect(left, right, "npm") is overlap
    assert anyver.ranges_intersect(right, left, "npm") is overlap


def test_maven_collection_order_is_independent_of_input_order():
    values = ["1-foo-alpha", "1-foo", "1-foo.sp-0"]
    for order in permutations(values):
        assert anyver.sort_versions(order, "maven") == values
        assert anyver.min_version(order, "maven") == values[0]
        assert anyver.max_version(order, "maven") == values[-1]
        assert anyver.latest_stable(order, "maven") == values[-1]
    assert anyver.compare(values[-1], values[0], "maven") < 0


@pytest.mark.parametrize(
    "bump,template",
    [
        (anyver.bump_major, "{}.0.0"),
        (anyver.bump_minor, "1.{}.0"),
        (anyver.bump_patch, "1.0.{}"),
        (anyver.bump_prerelease, "1.0.0-alpha.{}"),
    ],
)
def test_bump_overflow_is_an_error(bump, template):
    limit = 2**64 - 1
    assert bump(template.format(limit - 1)) == template.format(limit)
    with pytest.raises(ValueError, match="exceeds u64"):
        bump(template.format(limit))
