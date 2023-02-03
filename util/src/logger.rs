use std::{
    io::Write,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
};

use chrono::{DateTime, Utc};

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
        while let Ok(log_message) = self.rx.recv() {
            let log_entry: Vec<u8> = format!(
                "[{}] {}: {}\n",
                log_message.datetime.format("%Y-%m-%d %H:%M:%S%.9f %Z"),
                log_message.severity,
                log_message.message
            )
            .into();

            self.writer.write_all(&log_entry).unwrap();
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
    use std::io::{self, BufWriter};

    use crate::{job::Scheduler, thread_pool};

    use super::{create_logger, LogSeverity};

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
