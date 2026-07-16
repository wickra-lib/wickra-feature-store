"""Type stubs for the wickra_feature_store package."""

__version__: str

class FeatureStore:
    """A feature store driven by JSON commands."""

    def __init__(self, spec_json: str) -> None:
        """Build a feature store from a spec JSON string.

        Raises ``ValueError`` if the spec is malformed or invalid.
        """
        ...

    def command(self, cmd_json: str) -> str:
        """Apply a command JSON and return the resulting response JSON.

        Raises ``ValueError`` if the command envelope cannot be handled.
        """
        ...

    @staticmethod
    def version() -> str:
        """The library version."""
        ...
