use std::io::Write;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::SyncSender;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;

use crate::smart_enum;

smart_enum!(pub, LogSeverity, Debug, Info, Warning, Error, Critical);

struct LogMessage {
    severity: LogSeverity,
    datetime: DateTime<Utc>,
    message: String,
}

impl LogMessage {
    fn new(severity: LogSeverity, datetime: DateTime<Utc>, message: String) -> Self {
        Self {
            severity,
            datetime,
            message,
        }
    }
}

pub struct LoggerServer {
    rx: Receiver<LogMessage>,
    writer: Box<dyn Write + Send>,
}

impl LoggerServer {
    fn new(rx: Receiver<LogMessage>, writer: Box<dyn Write + Send>) -> Self {
        Self { rx, writer }
    }

    pub fn work(mut self) {
        loop {
            match self.rx.recv_timeout(Duration::from_secs(1)) {
                Ok(log_message) => {
                    let log_entry: Vec<u8> = format!(
                        "[{}] {}: {}\n",
                        log_message.datetime.format("%Y-%m-%d %H:%M:%S%.9f %Z"),
                        log_message.severity,
                        log_message.message
                    )
                    .into();

                    self.writer.write_all(&log_entry).unwrap();
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.writer.flush().unwrap();
                }
                _ => break,
            }
        }
    }
}

#[derive(Clone)]
pub struct LoggerClient {
    tx: SyncSender<LogMessage>,
}

impl LoggerClient {
    fn new(tx: SyncSender<LogMessage>) -> Self {
        Self { tx }
    }

    pub fn log<T: Into<String>>(&self, severity: LogSeverity, message: T) {
        let log_message = LogMessage::new(severity, Utc::now(), message.into());
        self.tx.send(log_message).unwrap();
    }
}

pub fn create_logger(
    buffer_size: usize,
    writer: Box<dyn Write + Send>,
) -> (LoggerServer, LoggerClient) {
    let (tx, rx) = sync_channel(buffer_size);

    (LoggerServer::new(rx, writer), LoggerClient::new(tx))
}

#[cfg(test)]
mod tests {
    use std::io::BufWriter;
    use std::io::{self};

    use super::create_logger;
    use super::LogSeverity;
    use crate::job::Scheduler;
    use crate::thread_pool;

    thread_pool!(TestThreadCategory, Logger: 1, Client: 10);

    #[test]
    fn test_logger() {
        let scheduler = Scheduler::new(ThreadPoolDescriptor {});

        let (server, client) = create_logger(32, Box::new(BufWriter::new(io::sink())));
        scheduler.schedule_job(TestThreadCategory::Logger, || server.work());

        for _ in 0..10 {
            let client = client.clone();
            scheduler.schedule_job(TestThreadCategory::Client, move || {
                for _ in 0..1000 {
                    client.log(LogSeverity::Critical, "Test log message");
                }
            });
        }
    }
}
