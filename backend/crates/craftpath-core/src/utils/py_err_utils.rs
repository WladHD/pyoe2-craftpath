//! Mapping of [`CraftPathError`](crate::api::errors::CraftPathError) onto a typed
//! Python exception hierarchy. Only compiled with the `python` feature.

use pyo3::PyErr;
use pyo3::create_exception;
use pyo3::exceptions::PyException;

use crate::api::errors::CraftPathError;

create_exception!(
    pyoe2_craftpath,
    CraftPathException,
    PyException,
    "Base exception for all pyoe2-craftpath errors."
);
create_exception!(
    pyoe2_craftpath,
    TargetUnreachableError,
    CraftPathException,
    "The target item could not be reached from the given starting item."
);
create_exception!(
    pyoe2_craftpath,
    ItemUnreachableError,
    CraftPathException,
    "A required affix is unreachable with the given item configuration or level constraints."
);
create_exception!(
    pyoe2_craftpath,
    RamLimitError,
    CraftPathException,
    "The configured RAM limit was reached and the calculation was aborted."
);
create_exception!(
    pyoe2_craftpath,
    ProviderDataError,
    CraftPathException,
    "A definition lookup (affix, base group, essence, ...) failed; the item info provider data is likely incomplete."
);
create_exception!(
    pyoe2_craftpath,
    EssenceIntermediaryError,
    CraftPathException,
    "A perfect essence requires an intermediary step to be applied."
);

/// Convert any error bubbling out of the calculation core into the matching
/// typed Python exception. Falls back to the base `CraftPathException`.
pub fn to_py_err(err: anyhow::Error) -> PyErr {
    let msg = format!("{err}");

    match err.downcast_ref::<CraftPathError>() {
        Some(CraftPathError::ItemMatrixCouldNotReachTarget()) => {
            TargetUnreachableError::new_err(msg)
        }
        Some(
            CraftPathError::ItemUnreachable(..)
            | CraftPathError::ItemUnreachableMinLevelConstraint(..),
        ) => ItemUnreachableError::new_err(msg),
        Some(CraftPathError::RamLimitReached(..)) => RamLimitError::new_err(msg),
        Some(
            CraftPathError::ItemWithoutAffixInformation(..)
            | CraftPathError::AffixWithoutDefinition(..)
            | CraftPathError::AffixWithoutEssence(..)
            | CraftPathError::BaseGroupWithoutDefinition(..)
            | CraftPathError::EssenceWithoutDefinition(..)
            | CraftPathError::BaseItemWithoutBaseGroup(..),
        ) => ProviderDataError::new_err(msg),
        Some(CraftPathError::EssenceIntermediaryStepRequired(..)) => {
            EssenceIntermediaryError::new_err(msg)
        }
        Some(CraftPathError::Cancelled()) => {
            pyo3::exceptions::PyInterruptedError::new_err(msg)
        }
        None => CraftPathException::new_err(msg),
    }
}
