from __future__ import annotations

from typing import Any


class RemoteJobError(Exception):
    """Error reported by the backend (craftpath.v1.Error shape)."""

    def __init__(self, code: str, message: str, details: dict | None = None) -> None:
        super().__init__(f"{code}: {message}")
        self.code = code
        self.message = message
        self.details = details or {}

    @classmethod
    def from_dict(cls, body: Any) -> "RemoteJobError":
        if isinstance(body, dict):
            return cls(
                code=body.get("code", "INTERNAL"),
                message=body.get("message", "unknown error"),
                details=body.get("details"),
            )
        return cls(code="INTERNAL", message=str(body))

    def raise_as_native(self) -> None:
        """Re-raise as the matching native exception type so error handling
        is identical for both engines."""
        from .. import _native

        mapping = {
            "TARGET_UNREACHABLE": getattr(_native, "TargetUnreachableError", None),
            "AFFIX_UNREACHABLE": getattr(_native, "ItemUnreachableError", None),
            "RAM_LIMIT_REACHED": getattr(_native, "RamLimitError", None),
            "PROVIDER_DATA_ERROR": getattr(_native, "ProviderDataError", None),
            "ESSENCE_INTERMEDIARY_REQUIRED": getattr(_native, "EssenceIntermediaryError", None),
        }
        native_cls = mapping.get(self.code)
        if native_cls is not None:
            raise native_cls(self.message) from self
        raise self
