use float_pretty_print::PrettyPrintFloat as ppf;
use std::cmp::max;
use std::f64;
use std::io::{stdin, stdout, Error, Read};
use std::result::Result;
use std::sync::mpsc;
use std::thread;
use termion::get_tty;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::Alignment;
use tui::style::{Color, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Paragraph, Text, Widget};
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

    let mut count: i32 = 0;
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
        let text = vec![
            Text::raw("Waiting for data...\n"),
            Text::raw("CTRL-C to exit\n"),
        ];
        Paragraph::new(text.iter())
            .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
            .alignment(Alignment::Center)
            .render(&mut frame, size);
    })?;

    loop {
        if let Ok(Ok(0x03)) = tty.try_recv() {
            break; // got ctrl-c
        }

        if let Ok(Ok(c)) = pipe.try_recv() {
            if c != b'\n' {
                input.push(c as char);
                continue;
            }

            // parse input string as vector of floats
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
                let width = size.width as i32 - 8; // y_label + margins = 8
                let x_win = if count - width > 0 {
                    [(count - width) as f64, count as f64] // [x_min, x_max]
                } else {
                    [0.0, width as f64]
                };

                let mut y_win = [0.0f64; 2]; // [y_min, y_max]
                let mut legends: Vec<String> = Vec::new();

                for i in 0..max(data.len(), new_data.len()) {
                    // trim old data points
                    if i < data.len() {
                        while !data[i].is_empty() {
                            if data[i][0].0 < x_win[0] {
                                data[i].remove(0);
                            } else {
                                break;
                            }
                        }
                    } else {
                        data.push(Vec::new()); // add new data series
                    }

                    // add new data points
                    if i < new_data.len() {
                        data[i].push((count as f64, new_data[i]));
                    }

                    // bail if empty
                    if data[i].is_empty() {
                        continue;
                    }

                    // legend with min, max, avg
                    let mut sum: f64 = 0.0;
                    let mut min: f64 = 0.0;
                    let mut max: f64 = 0.0;
                    for p in &data[i] {
                        sum += p.1;
                        if p.1 < min {
                            min = p.1;
                        }
                        if p.1 > max {
                            max = p.1;
                        }
                    }

                    if min < y_win[0] {
                        y_win[0] = min;
                    }
                    if max > y_win[1] {
                        y_win[1] = max;
                    }

                    legends.push(format!(
                        "Cur: {:1.5} Min: {:1.5} Max: {:1.5} Avg: {:1.5}",
                        ppf(data[i].last().unwrap().1),
                        ppf(min),
                        ppf(max),
                        ppf(sum / data[i].len() as f64)
                    ));
                }

                // make labels
                let mut x_labels: Vec<String> = Vec::new();
                let mut y_labels: Vec<String> = Vec::new();
                for i in 0..5 {
                    let step = i as f64 * 0.25;
                    x_labels.push(format!("{:.0}", step * (x_win[1] - x_win[0]) + x_win[0]));
                    let y = format!("{:1.5}", ppf(step * (y_win[1] - y_win[0]) + y_win[0]));
                    y_labels.push(format!("{:>5}", y));
                }

                // make datasets
                let mut datasets: Vec<Dataset> = Vec::new();
                for (i, d) in data.iter().enumerate() {
                    if !d.is_empty() {
                        datasets.push(
                            Dataset::default()
                                .name(&legends[i])
                                .marker(Marker::Braille)
                                .style(Style::default().fg(colors[i % colors.len()]))
                                .data(d),
                        );
                    }
                }

                // plot
                Chart::default()
                    .block(Block::default().title("ttyplot-rs").borders(Borders::ALL))
                    .x_axis(Axis::default().bounds(x_win).labels(&x_labels))
                    .y_axis(Axis::default().bounds(y_win).labels(&y_labels))
                    .datasets(&datasets)
                    .render(&mut frame, size);
            })?;
            count += 1;
        }
    }

    Ok(())
}
