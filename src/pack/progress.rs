use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Instant, Duration};
use std::thread;

/// Progress tracker for long-running operations
pub struct Progress {
    total: usize,
    current: Arc<AtomicUsize>,
    start: Instant,
    label: String,
}

impl Progress {
    pub fn new(total: usize, label: &str) -> Self {
        Progress {
            total,
            current: Arc::new(AtomicUsize::new(0)),
            start: Instant::now(),
            label: label.to_string(),
        }
    }

    /// Increment progress
    pub fn inc(&self) {
        self.current.fetch_add(1, Ordering::SeqCst);
    }

    /// Set progress to exact value
    pub fn set(&self, value: usize) {
        self.current.store(value, Ordering::SeqCst);
    }

    /// Get current progress
    pub fn current(&self) -> usize {
        self.current.load(Ordering::SeqCst)
    }

    /// Display progress bar
    pub fn display(&self) {
        let current = self.current();
        let percent = if self.total > 0 {
            (current as f64 / self.total as f64) * 100.0
        } else {
            0.0
        };

        let elapsed = self.start.elapsed();
        let rate = if elapsed.as_secs_f64() > 0.0 {
            current as f64 / elapsed.as_secs_f64()
        } else {
            0.0
        };

        let remaining = if rate > 0.0 {
            Duration::from_secs_f64((self.total - current) as f64 / rate)
        } else {
            Duration::from_secs(0)
        };

        let bar_width = 30;
        let filled = ((percent / 100.0) * bar_width as f64) as usize;
        let empty = bar_width - filled;

        eprint!(
            "\r[{}{}] {:.1}% ({}/{}) {}/s ETA {}",
            "=".repeat(filled),
            " ".repeat(empty),
            percent,
            current,
            self.total,
            rate.ceil() as usize,
            format_duration(remaining)
        );
    }

    /// Start displaying progress continuously
    pub fn start_display(self) -> ProgressHandle {
        let current = self.current.clone();
        let total = self.total;
        let label = self.label.clone();

        let handle = thread::spawn(move || {
            loop {
                let current_val = current.load(Ordering::SeqCst);
                let percent = if total > 0 {
                    (current_val as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                let bar_width: usize = 40;
                let filled = ((percent / 100.0) * bar_width as f64) as usize;
                let empty = bar_width.saturating_sub(filled);

                eprint!(
                    "\r[{}{}] {:.1}% {}/{} - {}",
                    "=".repeat(filled),
                    " ".repeat(empty),
                    percent,
                    current_val,
                    total,
                    label
                );

                if current_val >= total {
                    eprintln!("\n✓ Complete!");
                    break;
                }

                thread::sleep(Duration::from_millis(100));
            }
        });

        ProgressHandle { handle }
    }
}

pub struct ProgressHandle {
    handle: std::thread::JoinHandle<()>,
}

impl ProgressHandle {
    pub fn wait(self) {
        let _ = self.handle.join();
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m{}s", secs / 60, secs % 60)
    } else {
        format!("{}h{}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Simple spinner for indeterminate progress
pub struct Spinner {
    label: String,
}

impl Spinner {
    pub fn new(label: &str) -> Self {
        Spinner {
            label: label.to_string(),
        }
    }

    pub fn spin(&self, tick: usize) {
        let frames = ["|", "/", "-", "\\"];
        let frame = frames[tick % frames.len()];
        eprint!("\r{} {} ", frame, self.label);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_creation() {
        let progress = Progress::new(100, "test");
        assert_eq!(progress.current(), 0);
        assert_eq!(progress.total, 100);
    }

    #[test]
    fn test_progress_increment() {
        let progress = Progress::new(100, "test");
        progress.inc();
        assert_eq!(progress.current(), 1);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h1m");
    }
}
