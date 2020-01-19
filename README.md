# ttyplot-rs

<img src="./demo.svg" alt="Demo cast pipeing output from feed_input.rs into ttyplot-rs">

```
ttyplot-rs 0.1.0
Plot streaming data from stdin to a tty terminal. Useful for displaying data piped from a serial port or long running
process. To plot multiple data streams, separate data points with whitespace. Use CTRL-C to quit.

USAGE:
    ttyplot-rs [FLAGS] [OPTIONS]

FLAGS:
        --completions    Generate Bash tab-completion script
    -h, --help           Prints help information
    -V, --version        Prints version information

OPTIONS:
    -M, --max <max>        Upper bound of window (default: smallest data point in window)
    -m, --min <min>        Lower bound of window (default: largest data point in window)
    -w, --width <width>    Number of data points to display in window (default: terminal width)
```
