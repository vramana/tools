use rome_console::{Console, EnvConsole};
use rome_formatter::FormatError;
use rome_fs::{FileSystem, OsFileSystem, RomePath};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

pub mod configuration;
mod file_handlers;
pub mod settings;
pub mod workspace;

pub use crate::configuration::{
    create_config, load_config, Configuration, ConfigurationError, ConfigurationType,
};
pub use crate::file_handlers::JsFormatSettings;
pub use crate::workspace::Workspace;

pub struct App<'app> {
    pub fs: DynRef<'app, dyn FileSystem>,
    pub workspace: DynRef<'app, dyn Workspace>,
    pub console: DynRef<'app, dyn Console>,
    pub config_path: Option<RomePath>,
}

/// Generic errors thrown during rome operations
pub enum RomeError {
    /// The project contains uncommitted changes
    DirtyWorkspace,
    /// The file does not exist in the [Workspace]
    NotFound,
    /// A file is not supported. It contains the extension of the file
    /// Use this error if Rome is trying to process a file that Rome can't understand
    SourceFileNotSupported(RomePath),
    /// The formatter encountered an error while formatting the file
    FormatError(FormatError),
    /// The file could not be formatted since it has syntax errors and `format_with_errors` is disabled
    FormatWithErrorsDisabled,
    /// Thrown when a rome can't read a generic directory
    CantReadDirectory(PathBuf),
    /// Thrown when a rome can't read a generic file
    CantReadFile(PathBuf),

    /// Error thrown when validating the configuration. Once deserialized, further checks have to be done.
    Configuration(ConfigurationError),
}

impl Debug for RomeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RomeError::NotFound => std::fmt::Display::fmt(self, f),
            RomeError::SourceFileNotSupported(_) => std::fmt::Display::fmt(self, f),
            RomeError::FormatError(_) => std::fmt::Display::fmt(self, f),
            RomeError::FormatWithErrorsDisabled => std::fmt::Display::fmt(self, f),
            RomeError::CantReadDirectory(_) => std::fmt::Display::fmt(self, f),
            RomeError::CantReadFile(_) => std::fmt::Display::fmt(self, f),
            RomeError::Configuration(_) => std::fmt::Display::fmt(self, f),
            RomeError::DirtyWorkspace => std::fmt::Display::fmt(self, f),
        }
    }
}

impl Display for RomeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RomeError::SourceFileNotSupported(path) => {
                let ext = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("<unknown>");

                write!(f, "Rome doesn't support the file extension {ext:?} yet")
            }
            RomeError::NotFound => {
                write!(f, "the file does not exist in the workspace")
            }
            RomeError::FormatError(cause) => {
                write!(
                    f,
                    "the formatter encountered an error while formatting the file: {}",
                    cause
                )
            }
            RomeError::FormatWithErrorsDisabled => {
                write!(f, "the file could not be formatted since it has syntax errors and `format_with_errors` is disabled")
            }
            RomeError::CantReadDirectory(path) => {
                write!(
                    f,
                    "Rome couldn't read the following directory, maybe for permissions reasons or it doesn't exists: {}",
                    path.display()
                )
            }
            RomeError::CantReadFile(path) => {
                write!(
                    f,
                    "Rome couldn't read the following file, maybe for permissions reasons or it doesn't exists: {}",
                    path.display()
                )
            }

            RomeError::Configuration(error) => std::fmt::Display::fmt(error, f),
            RomeError::DirtyWorkspace => {
                write!(f, "Uncommitted changes in repository")
            }
        }
    }
}

impl Error for RomeError {}

impl From<FormatError> for RomeError {
    fn from(err: FormatError) -> Self {
        Self::FormatError(err)
    }
}

impl<'app> App<'app> {
    /// Create a new instance of the app using the [OsFileSystem] and [EnvConsole]
    pub fn from_env(no_colors: bool) -> Self {
        Self::with_filesystem_and_console(
            DynRef::Owned(Box::new(OsFileSystem)),
            DynRef::Owned(Box::new(EnvConsole::new(no_colors))),
        )
    }
}

impl<'app> App<'app> {
    /// Create a new instance of the app using the specified [FileSystem] and [Console] implementation
    pub fn with_filesystem_and_console(
        fs: DynRef<'app, dyn FileSystem>,
        console: DynRef<'app, dyn Console>,
    ) -> Self {
        Self {
            fs,
            console,
            workspace: DynRef::Owned(workspace::server()),
            config_path: None,
        }
    }
}

/// Clone of [std::borrow::Cow] specialized for storing a trait object and
/// holding a mutable reference in the `Borrowed` variant instead of requiring
/// the inner type to implement [std::borrow::ToOwned]
pub enum DynRef<'app, T: ?Sized + 'app> {
    Owned(Box<T>),
    Borrowed(&'app mut T),
}

impl<'app, T: ?Sized + 'app> Deref for DynRef<'app, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            DynRef::Owned(inner) => inner,
            DynRef::Borrowed(inner) => inner,
        }
    }
}

impl<'app, T: ?Sized + 'app> DerefMut for DynRef<'app, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            DynRef::Owned(inner) => inner,
            DynRef::Borrowed(inner) => inner,
        }
    }
}
