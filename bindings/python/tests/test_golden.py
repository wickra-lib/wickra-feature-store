"""Cross-language golden: every binding must produce byte-identical matrix JSON.

The fixtures live in the repository-root ``golden/`` directory (specs + a shared
dataset + expected responses). They are added in a later phase (P-FS-4); until
then this test skips cleanly.
"""

import json
import pathlib

import pytest

from wickra_feature_store import FeatureStore

ROOT = pathlib.Path(__file__).resolve().parents[3]
GOLDEN = ROOT / "golden"


def _spec_files() -> list[pathlib.Path]:
    specs = GOLDEN / "specs"
    if not specs.exists():
        return []
    return sorted(specs.glob("*.json"))


@pytest.mark.skipif(not GOLDEN.exists(), reason="golden fixtures not present yet")
@pytest.mark.parametrize("spec_path", _spec_files())
def test_golden_build_is_byte_identical(spec_path: pathlib.Path) -> None:
    dataset = json.loads((GOLDEN / "data.json").read_text(encoding="utf-8"))
    expected = (GOLDEN / "expected" / f"{spec_path.stem}.json").read_text(
        encoding="utf-8"
    )
    store = FeatureStore(spec_path.read_text(encoding="utf-8"))
    response = store.command(json.dumps({"cmd": "build_batch", "data": dataset}))
    assert response == expected.strip()
