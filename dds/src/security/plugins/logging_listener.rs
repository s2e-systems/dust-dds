use super::{logging::LogLevel, types::SecurityException};

/// LoggerListener plugin interface as defined in Section 9.6.2.3 of the DDS Security specification.
pub trait LoggerListener: Send + 'static {
    /// Called when a log message is logged by the Logging plugin.
    ///
    /// # Arguments
    ///
    /// * `log_level` - The level of the log message.
    /// * `message` - The log message.
    /// * `category` - The category of the log message.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] if an error occurs.
    fn on_log(
        &mut self,
        log_level: LogLevel,
        message: &str,
        category: &str,
    ) -> Result<(), SecurityException>;
}

/// Alias for [`LoggerListener`].
pub use LoggerListener as LoggingListener;

impl LoggerListener for () {
    fn on_log(
        &mut self,
        _log_level: LogLevel,
        _message: &str,
        _category: &str,
    ) -> Result<(), SecurityException> {
        unimplemented!()
    }
}
