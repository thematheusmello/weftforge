//! Editable garment construction in SI units, independent of the desktop UI.
//!
//! A [`Project`] is authoritative. [`Mesh`] is a rebuildable rest mesh carrying
//! its source revision; a solver's changing positions must be stored separately.

mod document;
mod geometry;
mod offset;
mod persistence;
mod sewing;
pub mod simulation;

pub use document::*;
pub use geometry::*;
pub use offset::*;
pub use persistence::{
    DXF_IMPORT_SCOPE, DxfPatternImport, FABRIC_SURFACE_FORMAT_VERSION, FORMAT_VERSION,
    MAX_DXF_BYTES, MAX_MTL_BYTES, MAX_OBJ_BYTES, MAX_SVG_BYTES, OBJ_IMPORT_SCOPE, ObjPatternImport,
    PatternSnapshotOptions, PrintPaper, SVG_IMPORT_SCOPE, StagedExport, SvgPatternImport,
    recovery_path,
};
pub use sewing::*;

use std::fmt;

pub type Id = u64;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error(pub String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

pub(crate) fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(Error(message.to_owned()))
    }
}
