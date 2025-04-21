//! In memory logger for `tracing`
//!
//! ## Usage
//!
//! ```rust
//! logger::install();
//!
//! // Your application code here
//! info!("This is an info message");
//! debug!("This is a debug message");
//! warn!("This is a warning message");
//! }
//! ```

use std::{
    io::{self, Write},
    panic,
    sync::{Arc, Mutex},
};

use once_cell::sync::Lazy;
use tracing_subscriber::{
    fmt::{self, MakeWriter},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// A global buffer that will be used to store log messages.
static SHARED_BUFFER: Lazy<Arc<Mutex<Vec<u8>>>> = Lazy::new(|| Arc::new(Mutex::new(Vec::new())));

/// Installs a subscriber for capturing `tracing` logs in memory.
pub fn install() {
    let make_writer = InMemoryMakeWriter(SHARED_BUFFER.clone());

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_ansi(true).with_writer(make_writer));

    subscriber.init();

    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        dump();
        prev(info);
    }));
}

/// Dumps logged messages to the standard output.
pub fn dump() {
    let bytes = SHARED_BUFFER.lock().unwrap();
    let output = String::from_utf8_lossy(&bytes);
    print!("{}", output);
}

#[derive(Clone)]
struct InMemoryMakeWriter(Arc<Mutex<Vec<u8>>>);

impl<'a> MakeWriter<'a> for InMemoryMakeWriter {
    type Writer = InMemoryWriter;

    fn make_writer(&'a self) -> Self::Writer {
        InMemoryWriter(self.0.clone())
    }
}

struct InMemoryWriter(Arc<Mutex<Vec<u8>>>);

impl Write for InMemoryWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut locked = self.0.lock().unwrap();
        locked.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
