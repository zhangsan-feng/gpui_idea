use std::fmt::Write as _;

use tracing::{Event, Metadata, Subscriber, field::Visit};
use tracing_subscriber::{Layer, layer::Context, prelude::*};

// Keep existing structured tracing events on the transplanted fern logger.
pub(super) fn initialize() {
    tracing::subscriber::set_global_default(tracing_subscriber::registry().with(TracingToLog))
        .expect("init tracing log bridge failed");
}

struct TracingToLog;

impl<S: Subscriber> Layer<S> for TracingToLog {
    fn enabled(&self, metadata: &Metadata<'_>, _: Context<'_, S>) -> bool {
        log::logger().enabled(
            &log::Metadata::builder()
                .level(level(metadata.level()))
                .target(metadata.target())
                .build(),
        )
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let metadata = event.metadata();
        let mut fields = Fields::default();
        event.record(&mut fields);
        let message = format!("{}{}", fields.message, fields.fields);
        log::logger().log(
            &log::Record::builder()
                .level(level(metadata.level()))
                .target(metadata.target())
                .file(metadata.file())
                .line(metadata.line())
                .args(format_args!("{message}"))
                .build(),
        );
    }
}

#[derive(Default)]
struct Fields {
    message: String,
    fields: String,
}

impl Visit for Fields {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            let _ = write!(self.message, "{value:?}");
        } else {
            let _ = write!(self.fields, " {}={value:?}", field.name());
        }
    }
}

fn level(level: &tracing::Level) -> log::Level {
    match *level {
        tracing::Level::ERROR => log::Level::Error,
        tracing::Level::WARN => log::Level::Warn,
        tracing::Level::INFO => log::Level::Info,
        tracing::Level::DEBUG => log::Level::Debug,
        tracing::Level::TRACE => log::Level::Trace,
    }
}
