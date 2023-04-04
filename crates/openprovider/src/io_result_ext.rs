
use std::io::ErrorKind;

use std::io::Error;

pub trait IOResultExt<T> : Sized {

    fn ok_kind<F: FnOnce(ErrorKind) -> bool>(self, pred: F) -> Result<Option<T>, Error>;

    fn ok_not_found(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::NotFound))
    }

    fn ok_permission_denied(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::PermissionDenied))
    }

    fn ok_connection_refused(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ConnectionRefused))
    }

    fn ok_connection_reset(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ConnectionRefused))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_connection_host_unreachable(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::HostUnreachable))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_network_unreachable(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::NetworkUnreachable))
    }

    fn ok_connection_aborted(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ConnectionAborted))
    }

    fn ok_not_connected(self) -> Result<Option<T>, Error> { 
        self.ok_kind(|kind| matches!(kind, ErrorKind::NotConnected))
    }

    fn ok_addr_in_use(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::AddrInUse))
    }

    fn ok_addr_not_available(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::AddrNotAvailable))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_network_down(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::NetworkDown))
    }

    fn ok_broken_pipe(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::BrokenPipe))
    }

    fn ok_already_exists(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::AlreadyExists))
    }

    fn ok_would_block(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::WouldBlock))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_not_a_directory(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::NotADirectory))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_is_a_directory(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::IsADirectory))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_read_only_filesystem(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ReadOnlyFilesystem))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_filesystem_loop(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::FilesystemLoop))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_stale_network_file_handle(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::StaleNetworkFileHandle))
    }

    fn ok_invalid_input(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::InvalidInput))
    }

    fn ok_invalid_data(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::InvalidData))
    }

    fn ok_timed_out(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::TimedOut))
    }

    fn ok_write_zero(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::WriteZero))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_storage_full(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::StorageFull))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_not_seekable(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::NotSeekable))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_filesystem_quota_exceeded(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::FilesystemQuotaExceeded))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_file_too_large(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::FileTooLarge))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_resource_busy(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ResourceBusy))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_executable_file_busy(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ExecutableFileBusy))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_deadlock(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::Deadlock))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_crosses_devices(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::CrossesDevices))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_too_many_links(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::TooManyLinks))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_invalid_filename(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::InvalidFilename))
    }

    #[cfg(feature = "io_error_more")]
    fn ok_argument_list_too_long(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::ArgumentListTooLong))
    }

    fn ok_interrupted(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::Interrupted))
    }

    fn ok_unsupported(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::Unsupported))
    }

    fn ok_unexpected_eof(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::UnexpectedEof))
    }

    fn ok_out_of_memory(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::OutOfMemory))
    }

    fn ok_other(self) -> Result<Option<T>, Error> {
        self.ok_kind(|kind| matches!(kind, ErrorKind::Other))
    }

}

impl <T> IOResultExt<T> for std::result::Result<T, Error> {

    fn ok_kind<F: FnOnce(ErrorKind) -> bool>(self, pred: F) -> Result<Option<T>, Error> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(err) => 
                if pred(err.kind()) {
                    Ok(None)
                } else {
                    Err(err)
                }
        }
    }

}

