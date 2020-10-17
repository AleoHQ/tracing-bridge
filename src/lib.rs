use serde::{Serialize, Deserialize};

use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TracingEvent {
    pub metadata: TracingMetadata,
    pub fields: HashMap<String, String>,
}

#[derive(Default)]
struct TracingMetadataFields {
    pub fields: HashMap<String, String>,
}

impl TracingMetadataFields {
    fn fields_from_event(event: &tracing_core::Event<'_>) -> HashMap<String, String> {
        let mut visitor = Self::default();
        event.record(&mut visitor);
        visitor.fields
    }
}

impl tracing_core::field::Visit for TracingMetadataFields {
    fn record_debug(&mut self, field: &tracing_core::Field, value: &dyn std::fmt::Debug) {
        self.fields.insert(field.name().to_owned(), format!("{:?}", value));
    }
}

impl From<&tracing_core::Event<'_>> for TracingEvent {
    fn from(event: &tracing_core::Event<'_>) -> Self {
        let fields = TracingMetadataFields::fields_from_event(event);
        
        Self {
            metadata: event.metadata().into(),
            fields,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct TracingMetadata {
    /// The name of the span described by this metadata.
    pub name: String,

    /// The part of the system that the span that this metadata describes
    /// occurred in.
    pub target: String,

    /// The level of verbosity of the described span.
    pub level: TracingLevel,

    /// The name of the Rust module where the span occurred, or `None` if this
    /// could not be determined.
    pub module_path: Option<String>,

    /// The name of the source code file where the span occurred, or `None` if
    /// this could not be determined.
    pub file: Option<PathBuf>,

    /// The line number in the source code file where the span occurred, or
    /// `None` if this could not be determined.
    pub line: Option<u32>,

    /// The kind of the callsite.
    pub kind: TracingCallsiteKind,
}

impl From<&tracing_core::Metadata<'_>> for TracingMetadata {
    fn from(metadata: &tracing_core::Metadata<'_>) -> Self {
        let kind = if metadata.is_event() {
            TracingCallsiteKind::Event
        } else if metadata.is_span() {
            TracingCallsiteKind::Span
        } else {
            panic!("Unknown callsite kind for metadata: {:?}", metadata);
        };

        Self {
            name: metadata.name().to_owned(),
            target: metadata.target().to_owned(),
            level: metadata.level().into(),
            module_path: metadata.module_path().map(|path| path.into()),
            file: metadata.file().map(|file| file.into()),
            line: metadata.line(),
            kind,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum TracingLevel {
    /// The "trace" level.
    ///
    /// Designates very low priority, often extremely verbose, information.
    Trace,
    /// The "debug" level.
    ///
    /// Designates lower priority information.
    Debug,
    /// The "info" level.
    ///
    /// Designates useful information.
    Info,
    /// The "warn" level.
    ///
    /// Designates hazardous situations.
    Warn,
    /// The "error" level.
    ///
    /// Designates very serious errors.
    Error,
}

impl From<&tracing_core::Level> for TracingLevel {
    fn from(level: &tracing_core::Level) -> Self {
        match level {
            &tracing_core::Level::TRACE => Self::Trace,
            &tracing_core::Level::DEBUG => Self::Debug,
            &tracing_core::Level::INFO => Self::Info,
            &tracing_core::Level::WARN => Self::Warn,
            &tracing_core::Level::ERROR => Self::Error,
        }
    }
}

impl Into<tracing_core::Level> for &TracingLevel {
    fn into(self) -> tracing_core::Level {
        match self {
            &TracingLevel::Trace => tracing_core::Level::TRACE,
            &TracingLevel::Debug => tracing_core::Level::DEBUG,
            &TracingLevel::Info => tracing_core::Level::INFO,
            &TracingLevel::Warn => tracing_core::Level::WARN,
            &TracingLevel::Error => tracing_core::Level::ERROR,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum TracingCallsiteKind {
    Event,
    Span,
}

impl Into<tracing_core::metadata::Kind> for &TracingCallsiteKind {
    fn into(self) -> tracing_core::metadata::Kind {
        match self {
            &TracingCallsiteKind::Event => tracing_core::metadata::Kind::EVENT,
            &TracingCallsiteKind::Span => tracing_core::metadata::Kind::SPAN,
        }
    }
}