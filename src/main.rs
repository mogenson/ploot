use float_pretty_print::PrettyPrintFloat as ppf;
use std::f64;
use std::io::{stdin, stdout, Error, Read};
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

struct Series {
    data: Vec<(f64, f64)>,
    min: f64,
    max: f64,
    avg: f64,
    cur: f64,
}

impl Series {
    fn new() -> Series {
        Series {
            data: Vec::new(),
            min: 0.0,
            max: 0.0,
            avg: 0.0,
            cur: 0.0,
        }
    }
    fn update(&mut self, width: usize, point: (f64, f64)) {
        while self.data.len() > width {
            self.data.remove(0);
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

fn reader(stream: impl Read + Send + Sync + 'static) -> mpsc::Receiver<Result<u8, Error>> {
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

    let mut series = Series::new();
    let mut count: usize = 0;
    let pipe = reader(stdin());
    let tty = reader(get_tty()?);
    let mut input = String::new();

    loop {
        if let Ok(Ok(0x03)) = tty.try_recv() {
            break; // got ctrl-c
        }

        if let Ok(Ok(c)) = pipe.try_recv() {
            if c == '\n' as u8 {
                let vec = input
                    .split_whitespace()
                    .filter_map(|s| s.parse::<f64>().ok())
                    .collect::<Vec<f64>>();
                if !vec.is_empty() {
                    let width = terminal.size()?.width as usize;
                    series.update(width, (count as f64, vec[0]));
                    terminal.draw(|mut frame| {
                        let size = frame.size();
                        let window = if count - width > 0 {
                            [((count - width) as f64), (count as f64)]
                        } else {
                            [0.0, (width as f64)]
                        };

                        Chart::default()
                            .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
                            .x_axis(Axis::default().bounds(window).labels(&[
                                &format!("{:.0}", window[0]),
                                &format!("{:.0}", 0.25 * (window[1] - window[0]) + window[0]),
                                &format!("{:.0}", 0.50 * (window[1] - window[0]) + window[0]),
                                &format!("{:.0}", 0.75 * (window[1] - window[0]) + window[0]),
                                &format!("{:.0}", window[1]),
                            ]))
                            .y_axis(Axis::default().bounds([series.min, series.max]).labels(&[
                                &format!("{:1.5}", ppf(series.min)),
                                &format!(
                                    "{:1.5}",
                                    ppf(0.25 * (series.max - series.min) + series.min)
                                ),
                                &format!(
                                    "{:1.5}",
                                    ppf(0.50 * (series.max - series.min) + series.min)
                                ),
                                &format!(
                                    "{:1.5}",
                                    ppf(0.75 * (series.max - series.min) + series.min)
                                ),
                                &format!("{:1.5}", ppf(series.max)),
                            ]))
                            .datasets(&[Dataset::default()
                                .name(&format!(
                                    "Cur: {:1.5} Min: {:1.5} Max: {:1.5} Avg: {:1.5}",
                                    ppf(series.cur),
                                    ppf(series.min),
                                    ppf(series.max),
                                    ppf(series.avg)
                                ))
                                .marker(Marker::Braille)
                                .style(Style::default().fg(Color::Red))
                                .data(&series.data)])
                            .render(&mut frame, size);
                    })?;
                    count += 1;
                }
                input.clear();
            } else {
                input.push(c as char);
            }
        }
    }

    Ok(())
}
