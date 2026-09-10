"""Cross-language golden: every binding must produce byte-identical matrix JSON.

The fixtures live in the repository-root ``golden/`` directory: the specs, two
datasets and the blessed responses. A spec named ``feeds_*`` is folded over
``data-feeds.json``, which carries one side feed per family; every other spec is
folded over the candle-only ``data.json``. Sending the candle-only dataset to a
fed spec is refused by name — that refusal is the point of the feed check, so
picking the right dataset is part of speaking the protocol correctly.
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
    source = "data-feeds.json" if spec_path.stem.startswith("feeds_") else "data.json"
    dataset = json.loads((GOLDEN / source).read_text(encoding="utf-8"))
    expected = (GOLDEN / "expected" / f"{spec_path.stem}.json").read_text(
        encoding="utf-8"
    )
    store = FeatureStore(spec_path.read_text(encoding="utf-8"))
    response = store.command(json.dumps({"cmd": "build_batch", "data": dataset}))
    assert response == expected.strip()
