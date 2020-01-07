use float_pretty_print::PrettyPrintFloat as ppf;
use std::f64;
use std::io::stdout;
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

struct App {
    data: Vec<(f64, f64)>,
    window: [f64; 2],
    width: usize,
    min: f64,
    max: f64,
    avg: f64,
    cur: f64,
    count: usize,
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
            count: 0,
        }
    }

    fn update(&mut self) -> Result<bool, failure::Error> {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        // let input = input.split_whitespace().next().unwrap();
        // let y:f64 = input.parse()?;

        let vec = input
            .split_whitespace()
            .filter_map(|s| s.parse::<f64>().ok())
            .collect::<Vec<f64>>();
        if vec.is_empty() {
            return Ok(false);
        }
        let y = vec[0];

        if self.data.len() >= self.width {
            self.data.remove(0);
            self.window[0] += 1.0;
            self.window[1] += 1.0;
        }
        self.data.push(((self.count as f64), y));
        self.count += 1;
        self.cur = y;
        if y > self.max {
            self.max = y;
        }
        if y < self.min {
            self.min = y;
        }
        self.avg = self.avg + y - (self.avg / (self.data.len() as f64));
        Ok(false)
    }
}

fn main() -> Result<(), failure::Error> {
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

        if app.update()? {
            break;
        }
    }

    Ok(())
}
