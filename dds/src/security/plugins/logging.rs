use super::{logging_listener::LoggerListener, types::SecurityException};

/// Log levels as defined in Section 9.6.2.1.1 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Default)]
pub enum LogLevel {
    /// Security error causing a shutdown or failure of the `DomainParticipant`.
    FatalLevel,
    /// Major security error or fault.
    SevereLevel,
    /// Minor security error or fault.
    ErrorLevel,
    /// Undesirable or unexpected behavior.
    WarningLevel,
    /// Important security event.
    NoticeLevel,
    /// Interesting security event.
    #[default]
    InfoLevel,
    /// Detailed information on the flow of the security events.
    DebugLevel,
    /// Even more detailed information.
    TraceLevel,
}

/// Options for configuring the logger as defined in Section 9.6.2.1 of the DDS Security specification.
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct LogOptions {
    /// Specifies what level of log messages will be logged. Messages at or below the `log_level` are logged.
    pub log_level: LogLevel,
    /// Specifies the full path to a local file for logging events. If `None`, logger will not log messages to a file.
    pub log_file: Option<String>,
    /// Specifies whether the log events should be distributed over DDS.
    pub distribute: bool,
}

/// Logging plugin interface as defined in Section 9.6.2.2 of the DDS Security specification.
pub trait Logging: Send + 'static {
    /// Sets the options for the logger.
    ///
    /// This must be called before [`enable_logging`](Self::enable_logging); it is an error to set the options after logging has been enabled.
    ///
    /// # Arguments
    ///
    /// * `options` - The [`LogOptions`] object with the required options.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn set_log_options(&mut self, options: LogOptions) -> Result<(), SecurityException>;

    /// Log a message.
    ///
    /// The logger shall log the message if its `log_level` is at or above the level set in the [`LogOptions`].
    ///
    /// # Arguments
    ///
    /// * `log_level` - The level of the log message.
    /// * `message` - The log message.
    /// * `category` - A category for the log message.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case logging fails.
    fn log(
        &mut self,
        log_level: LogLevel,
        message: &str,
        category: &str,
    ) -> Result<(), SecurityException>;

    /// Enables logging.
    ///
    /// After this method is called, any call to [`log`](Self::log) shall log the messages according to the options.
    /// After this method is called, the options may not be modified.
    ///
    /// # Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn enable_logging(&mut self) -> Result<(), SecurityException>;

    /// Sets the [`LoggerListener`] that the `Logging` plugin will use to notify the application of log events.
    ///
    /// # Arguments
    ///
    /// * `listener` - An optional [`LoggerListener`] object to be attached to the logger object. If `None`, indicates no listener.
    ///
    /// #Errors
    ///
    /// Returns [`SecurityException`] providing details in case operation fails.
    fn set_listener<L>(&mut self, listener: Option<L>) -> Result<(), SecurityException>
    where
        L: LoggerListener;
}

impl Logging for () {
    fn set_log_options(&mut self, _options: LogOptions) -> Result<(), SecurityException> {
        unimplemented!()
    }

    fn log(
        &mut self,
        _log_level: LogLevel,
        _message: &str,
        _category: &str,
    ) -> Result<(), SecurityException> {
        unimplemented!()
    }

    fn enable_logging(&mut self) -> Result<(), SecurityException> {
        unimplemented!()
    }

    fn set_listener<L>(&mut self, _listener: Option<L>) -> Result<(), SecurityException>
    where
        L: LoggerListener,
    {
        unimplemented!()
    }
}
