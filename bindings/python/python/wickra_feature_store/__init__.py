"""Wickra Feature Store — the data-driven feature-matrix core.

Build a :class:`FeatureStore` from a spec JSON, drive it with command JSONs, and
read back feature matrices. The same command protocol crosses every language
binding, so this Python front-end drives the exact same core as the native CLI.
"""

from ._wickra_feature_store import FeatureStore, __version__

__all__ = ["FeatureStore", "__version__"]
