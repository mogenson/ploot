use float_pretty_print::PrettyPrintFloat as ppf;
use std::f64;
use std::io::Error;
use std::io::{stdout, Read};
use std::result::Result;
use std::sync::mpsc;
use std::thread;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

use termion::get_tty;

/* TODO:
 * read terminal size each loop, resize vectors
 * vector of vectors for each data point
 * clap arguments
 * show min / max/ avg for current window
 */

struct App {
    data: Vec<(f64, f64)>,
    window: [f64; 2],
    width: usize,
    min: f64,
    max: f64,
    avg: f64,
    cur: f64,
}

impl App {
    fn new(width: usize) -> App {
        let data = Vec::new();
        App {
            data,
            window: [0.0, (width as f64)],
            width: width,
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            cur: 0.0,
        }
    }

    fn update(&mut self, point: (f64, f64)) {
        if self.data.len() >= self.width {
            self.data.remove(0);
            self.window[0] += 1.0;
            self.window[1] += 1.0;
        }
        self.data.push(point);
        self.cur = point.1;
        if self.cur > self.max {
            self.max = self.cur;
        }
        if self.cur < self.min {
            self.min = self.cur;
        }
        self.avg = self.avg + self.cur - (self.avg / (self.data.len() as f64));
    }
}

fn input_reader(stream: impl Read + Send + Sync + 'static) -> mpsc::Receiver<Result<u8, Error>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for i in stream.bytes() {
            if tx.send(i).is_err() {
                return;
            }
        }
    });

    rx
}

fn main() -> Result<(), Error> {
    // Terminal initialization
    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // App
    let size = terminal.size()?;
    let mut app = App::new(size.width as usize);
    let mut count: f64 = 0.0;

    let tty_rx = input_reader(get_tty()?);

    loop {
        terminal.draw(|mut f| {
            let size = f.size();
            Chart::default()
                .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
                .x_axis(Axis::default().bounds(app.window).labels(&[
                    &format!("{:.0}", app.window[0]),
                    &format!(
                        "{:.0}",
                        0.25 * (app.window[1] - app.window[0]) + app.window[0]
                    ),
                    &format!(
                        "{:.0}",
                        0.50 * (app.window[1] - app.window[0]) + app.window[0]
                    ),
                    &format!(
                        "{:.0}",
                        0.75 * (app.window[1] - app.window[0]) + app.window[0]
                    ),
                    &format!("{:.0}", app.window[1]),
                ]))
                .y_axis(Axis::default().bounds([app.min, app.max]).labels(&[
                    &format!("{:1.5}", ppf(app.min)),
                    &format!("{:1.5}", ppf(0.25 * (app.max - app.min) + app.min)),
                    &format!("{:1.5}", ppf(0.50 * (app.max - app.min) + app.min)),
                    &format!("{:1.5}", ppf(0.75 * (app.max - app.min) + app.min)),
                    &format!("{:1.5}", ppf(app.max)),
                ]))
                .datasets(&[Dataset::default()
                    .name(&format!(
                        "Cur: {:1.5} Min: {:1.5} Max: {:1.5} Avg: {:1.5}",
                        ppf(app.cur),
                        ppf(app.min),
                        ppf(app.max),
                        ppf(app.avg)
                    ))
                    .marker(Marker::Braille)
                    .style(Style::default().fg(Color::Red))
                    .data(&app.data)])
                .render(&mut f, size);
        })?;

        if let Ok(Ok(0x03)) = tty_rx.try_recv() {
            break;
        }

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let vec = input
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<f64>>();
        if !vec.is_empty() {
            app.update((count, vec[0]));
            count += 1.0;
        }
    }

    Ok(())
}
