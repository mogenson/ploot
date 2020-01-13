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

    let mut count: usize = 0;
    let pipe = reader(stdin());
    let tty = reader(get_tty()?);
    let mut input = String::new();

    let mut data: Vec<(f64, f64)> = Vec::new();

    terminal.draw(|mut frame| {
        let size = frame.size();
        Block::default()
            .title("Waiting for data...")
            .borders(Borders::ALL)
            .render(&mut frame, size);
    })?;

    loop {
        if let Ok(Ok(0x03)) = tty.try_recv() {
            break; // got ctrl-c
        }

        if let Ok(Ok(c)) = pipe.try_recv() {
            if c != '\n' as u8 {
                input.push(c as char);
                continue;
            }

            let v = input
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect::<Vec<f64>>();
            if v.is_empty() {
                continue;
            }
            input.clear();

            terminal.draw(|mut frame| {
                let size = frame.size();
                let width = size.width;
                let window = [count as f64 - width as f64, count as f64];

                data.push((count as f64, v[0])); // update
                data.retain(|&p| p.0 >= window[0]); // trim
                count += 1;
                                                    // TODO check if empty
                let mut sum: f64 = 0.0;
                let mut min: f64 = 0.0;
                let mut max: f64 = 0.0;
                for p in &data {
                    sum += p.1;
                    if p.1 < min {
                        min = p.1;
                    }
                    if p.1 > max {
                        max = p.1;
                    }
                }
                let avg: f64 = sum / data.len() as f64;

                Chart::default()
                    .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
                    .x_axis(Axis::default().bounds(window).labels(&[
                        &format!("{:.0}", window[0]),
                        &format!("{:.0}", 0.25 * (window[1] - window[0]) + window[0]),
                        &format!("{:.0}", 0.50 * (window[1] - window[0]) + window[0]),
                        &format!("{:.0}", 0.75 * (window[1] - window[0]) + window[0]),
                        &format!("{:.0}", window[1]),
                    ]))
                    .y_axis(Axis::default().bounds([min, max]).labels(&[
                        &format!("{:1.5}", ppf(min)),
                        &format!("{:1.5}", ppf(0.25 * (max - min) + min)),
                        &format!("{:1.5}", ppf(0.50 * (max - min) + min)),
                        &format!("{:1.5}", ppf(0.75 * (max - min) + min)),
                        &format!("{:1.5}", ppf(max)),
                    ]))
                    .datasets(&[Dataset::default()
                        .name(&format!(
                            "Cur: {:1.5} Min: {:1.5} Max: {:1.5} Avg: {:1.5}",
                            ppf(v[0]),
                            ppf(min),
                            ppf(max),
                            ppf(avg)
                        ))
                        .marker(Marker::Braille)
                        .style(Style::default().fg(Color::Red))
                        .data(&data)])
                    .render(&mut frame, size);
            })?;
        }
    }

    Ok(())
}
