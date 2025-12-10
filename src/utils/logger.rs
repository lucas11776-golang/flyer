use once_cell::sync::OnceCell;
use slog::{o, Drain, Logger};
use slog_async::Async;
use slog_term::{CompactFormat, TermDecorator};

static GLOBAL_LOGGER: OnceCell<Logger> = OnceCell::new();

pub(crate) fn init() -> Logger {
    let decorator = TermDecorator::new().stdout().build();
    let drain = CompactFormat::new(decorator).build().fuse();
    Logger::root(Async::new(drain).build().fuse(), o!())
}

pub(crate) fn logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| init())
}