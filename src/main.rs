use float_pretty_print::PrettyPrintFloat as ppf;
use std::cmp::max;
use std::f64;
use std::f64::{MAX, MIN};
use std::io::{stdin, stdout, Cursor, Read};
use std::result::Result;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use structopt::clap::Shell;
use structopt::StructOpt;
use termion::get_tty;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Axis, Block, Borders, Chart, Dataset, Marker, Paragraph, Text, Widget};
use tui::Terminal;

#[derive(StructOpt)]
#[structopt(about = "Plot streaming data from stdin to a tty terminal. \
                     Useful for displaying data piped from a serial port or long running process. \
                     To plot multiple data streams, separate data points with whitespace. \
                     Use CTRL-C to quit.")]
struct Opts {
    #[structopt(
        short,
        long,
        help = "Number of data points to display in window (default: terminal width)"
    )]
    width: Option<usize>,
    #[structopt(
        short = "M",
        long,
        help = "Upper bound of window (default: smallest data point in window)"
    )]
    max: Option<f64>,
    #[structopt(
        short = "m",
        long,
        help = "Lower bound of window (default: largest data point in window)"
    )]
    min: Option<f64>,
    #[structopt(long = "completions", help = "Generate Bash tab-completion script")]
    completions: bool,
}

fn reader(stream: impl Read + Send + Sync + 'static) -> Receiver<Result<u8, std::io::Error>> {
    let (tx, rx) = channel();
    thread::spawn(move || {
        for i in stream.bytes() {
            if tx.send(i).is_err() {
                return;
            }
        }
    });
    rx
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse command line arguments
    let opts: Opts = Opts::from_args();
    if opts.completions {
        println!(
            "Generating Bash tab-completion script: {}.bash",
            env!("CARGO_PKG_NAME")
        );
        Opts::clap().gen_completions(env!("CARGO_PKG_NAME"), Shell::Bash, ".");
        return Ok(());
    }

    // terminal initialization
    let stdout = stdout();
    let stdout = stdout.lock().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    // internal app variables
    let mut count: i32 = 0;
    let pipe = reader(stdin());
    let tty = reader(get_tty()?);
    let mut input = String::new();
    let mut data: Vec<Vec<(f64, f64)>> = Vec::new();

    // color constants
    let colors = vec![
        Color::Cyan,
        Color::Yellow,
        Color::Green,
        Color::Blue,
        Color::Red,
        Color::Magenta,
        Color::White,
    ];

    // write help message to string
    let mut cursor = Cursor::new(Vec::new());
    Opts::clap().write_help(&mut cursor)?;
    let usage = String::from_utf8(cursor.into_inner())?;

    // draw to center of terminal while waiting for data
    terminal.draw(|mut frame| {
        let size = frame.size();
        let text = vec![
            Text::styled(
                "Waiting for data...\n\n\n",
                Style::default().fg(Color::Yellow),
            ),
            Text::raw(usage),
        ];

        Block::default()
            .title(env!("CARGO_PKG_NAME"))
            .borders(Borders::ALL)
            .render(&mut frame, size);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(5)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(size);

        Paragraph::new(text.iter())
            .alignment(Alignment::Left)
            .wrap(false)
            .render(&mut frame, chunks[0]);
    })?;

    // main app loop
    loop {
        // exit loop if ctrl-c pressed
        if let Ok(Ok(0x03)) = tty.try_recv() {
            break;
        }

        // read char from piped stdin
        if let Ok(Ok(c)) = pipe.try_recv() {
            if c != b'\n' {
                input.push(c as char);
                continue; // add char to string and wait for next
            }

            // parse input string as vector of floats, silence format errors
            let new_data: Vec<f64> = input
                .split_whitespace()
                .filter_map(|s| s.parse().ok())
                .collect();

            // bail if no new data
            if new_data.is_empty() {
                continue;
            }

            input.clear(); // reset input string

            // plot new data
            terminal.draw(|mut frame| {
                // pick terminal width or use specified width
                let size = frame.size();
                let width = if let Some(width) = opts.width {
                    width as i32
                } else {
                    size.width as i32 - 8 // y_label + margins = 8
                };

                // display window width and height
                let x_win = if count - width > 0 {
                    [(count - width) as f64, count as f64] // [x_min, x_max]
                } else {
                    [0.0, width as f64]
                };
                let mut y_win = [MAX, MIN]; // [y_min, y_max]

                // use legends for data statistics
                let mut legends: Vec<String> = Vec::new();

                // format data for plotting
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

                    // bail if series is empty
                    if data[i].is_empty() {
                        continue;
                    }

                    // legend with min, max, avg
                    let mut sum: f64 = 0.0;
                    let mut min: f64 = MAX;
                    let mut max: f64 = MIN;

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
                        ppf(data[i].last().unwrap().1), // guaranteed to exist
                        ppf(min),
                        ppf(max),
                        ppf(sum / data[i].len() as f64)
                    ));
                }

                // override with user options
                if let Some(min) = opts.min {
                    y_win[0] = min;
                }
                if let Some(max) = opts.max {
                    y_win[1] = max;
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
                    .block(
                        Block::default()
                            .title(env!("CARGO_PKG_NAME"))
                            .borders(Borders::ALL),
                    )
                    .x_axis(Axis::default().bounds(x_win).labels(&x_labels))
                    .y_axis(Axis::default().bounds(y_win).labels(&y_labels))
                    .datasets(&datasets)
                    .render(&mut frame, size);
            })?;

            count += 1; // increment current data count
        }
    }

    Ok(())
}
