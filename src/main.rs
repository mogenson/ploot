use float_pretty_print::PrettyPrintFloat as ppf;
use std::f64;
use std::io::{stdin, stdout, Error, Read};
use std::result::Result;
use std::sync::mpsc;
use std::thread;
use termion::get_tty;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::style::{Color, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Widget};
use tui::Terminal;

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

    let mut data: Vec<Vec<(f64, f64)>> = Vec::new();
    let colors = vec![
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::Magenta,
        Color::White,
    ];

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

            let new_data = input
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect::<Vec<f64>>();
            if new_data.is_empty() {
                continue;
            }
            input.clear();

            terminal.draw(|mut frame| {
                let size = frame.size();
                let width = size.width;
                let window = [count as f64 - width as f64, count as f64];

                for (i, d) in new_data.iter().enumerate() {
                    if data.len() <= i {
                        data.push(Vec::new()); // add new series
                    }
                    data[i].push((count as f64, *d)); // update series
                }
                count += 1;

                for d in &mut data {
                    d.retain(|&p| p.0 >= window[0]); // trim old points
                }

                data.retain(|d| !d.is_empty()); // trim old series

                // calculate min, max, avg for each series
                let mut min: Vec<f64> = Vec::new();
                let mut max: Vec<f64> = Vec::new();
                let mut avg: Vec<f64> = Vec::new();
                let mut global_min: f64 = 0.0;
                let mut global_max: f64 = 0.0;

                for d in &data {
                    let mut sum: f64 = 0.0;
                    let mut local_min: f64 = 0.0;
                    let mut local_max: f64 = 0.0;

                    for p in d {
                        sum += p.1;
                        if p.1 < local_min {
                            local_min = p.1;
                        }
                        if p.1 > local_max {
                            local_max = p.1;
                        }
                    }

                    if local_min < global_min {
                        global_min = local_min;
                    }
                    if local_max > global_max {
                        global_max = local_max;
                    }

                    min.push(local_min);
                    max.push(local_max);
                    avg.push(sum / d.len() as f64);
                }

                // make labels
                let mut x_labels: Vec<String> = Vec::new();
                let mut y_labels: Vec<String> = Vec::new();
                for i in 0..5 {
                    x_labels.push(format!(
                        "{:.0}",
                        i as f64 * 0.25 * (window[1] - window[0]) + window[0]
                    ));
                    y_labels.push(format!(
                        "{:1.5}",
                        ppf(i as f64 * 0.25 * (global_max - global_min) + global_min)
                    ));
                }

                // make legend and dataset
                let mut legends: Vec<String> = Vec::new();
                for (i, d) in data.iter().enumerate() {
                    legends.push(format!(
                        "Cur: {:1.5} Min: {:1.5} Max: {:1.5} Avg: {:1.5}",
                        ppf(d.last().unwrap().1),
                        ppf(min[i]),
                        ppf(max[i]),
                        ppf(avg[i])
                    ));
                }

                // make datasets
                let mut datasets: Vec<Dataset> = Vec::new();
                for (i, d) in data.iter().enumerate() {
                    datasets.push(
                        Dataset::default()
                            .name(&legends[i])
                            .marker(Marker::Braille)
                            .style(Style::default().fg(colors[i % colors.len()]))
                            .data(d),
                    );
                }

                Chart::default()
                    .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
                    .x_axis(Axis::default().bounds(window).labels(&x_labels))
                    .y_axis(
                        Axis::default()
                            .bounds([global_min, global_max])
                            .labels(&y_labels),
                    )
                    .datasets(&datasets)
                    .render(&mut frame, size);
            })?;
        }
    }

    Ok(())
}
