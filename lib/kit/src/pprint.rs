use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use ptree::item::StringItem;
use ptree::output::print_tree_with;
use ptree::print_config::{StyleWhen, UTF_CHARS_DASHED};
use ptree::{Color, PrintConfig};
use std::env;
use std::io::Error;
use std::io::Write;
use tabled::{Style, Table, Tabled};

pub fn progress() -> ProgressBar {
    let bar = ProgressBar::new(100);
    bar.set_style(
        ProgressStyle::with_template("{prefix} {name} [{bar:30}] ({elapsed}) {msg}")
            .unwrap()
            .progress_chars("=> "),
    );
    bar
}

pub fn trace() -> bool {
    match env::var("TRACE") {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub fn init_trace() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .without_time()
        .init();
}

pub fn init_log() {
    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S%.3f"),
                record.args()
            )
        })
        .init();
}

/// Main struct that holds the state for one Write stream
pub struct LogUpdate<W: Write> {
    pub stream: W,
    pub previous_line_count: u16,
    pub cursor_is_hidden: bool,
}

impl<W: Write> LogUpdate<W> {
    /// Create a new LogUpdate instance.
    pub fn new(mut stream: W) -> Result<Self, Error> {
        let _ = write!(stream, "{}", ansi_escapes::CursorHide);
        let _ = stream.flush();

        Ok(LogUpdate {
            stream: stream,
            previous_line_count: 0,
            cursor_is_hidden: true,
        })
    }

    /// Update the log to the provided text.
    pub fn render(&mut self, text: &str) -> Result<(), Error> {
        let _ = write!(
            self.stream,
            "{}{}\n",
            ansi_escapes::EraseLines(self.previous_line_count),
            text
        );
        let _ = self.stream.flush();

        self.previous_line_count = text.chars().filter(|x| *x == '\n').count() as u16 + 2;

        Ok(())
    }

    /// Clear the logged output.
    pub fn clear(&mut self) -> Result<(), Error> {
        let _ = write!(
            self.stream,
            "{}",
            ansi_escapes::EraseLines(self.previous_line_count)
        );
        let _ = self.stream.flush();

        self.previous_line_count = 0;

        Ok(())
    }

    /// Persist the logged output.
    /// Useful if you want to start a new log session below the current one.
    pub fn done(&mut self) -> Result<(), Error> {
        if self.cursor_is_hidden {
            let _ = write!(self.stream, "{}", ansi_escapes::CursorShow);
            let _ = self.stream.flush();
        }

        self.previous_line_count = 0;
        self.cursor_is_hidden = false;

        Ok(())
    }
}

impl<W: Write> Drop for LogUpdate<W> {
    fn drop(&mut self) {
        if self.cursor_is_hidden {
            write!(self.stream, "{}", ansi_escapes::CursorShow).unwrap();
            self.stream.flush().unwrap();
        }
    }
}

pub fn print_table<T: Tabled>(x: Vec<T>) {
    let table = Table::new(x).with(Style::psql()).to_string();
    println!("{}", table);
}

pub fn print_tree(tree: StringItem) {
    let config = {
        let mut config = PrintConfig::from_env();
        config.branch = ptree::Style {
            foreground: Some(Color::White),
            bold: false,
            dimmed: true,
            ..ptree::Style::default()
        };
        config.leaf = ptree::Style {
            bold: false,
            ..ptree::Style::default()
        };
        config.characters = UTF_CHARS_DASHED.into();
        config.styled = StyleWhen::Never;
        config.indent = 4;
        config
    };

    print_tree_with(&tree, &config).unwrap();
}
