"""Pin the public surface of the FeatureStore class across bindings."""

from wickra_feature_store import FeatureStore

EXPECTED_METHODS = {"command", "version"}


def test_expected_methods_present() -> None:
    for name in EXPECTED_METHODS:
        assert hasattr(FeatureStore, name), f"missing method: {name}"


def test_no_unexpected_public_methods() -> None:
    public = {name for name in dir(FeatureStore) if not name.startswith("_")}
    assert public == EXPECTED_METHODS
