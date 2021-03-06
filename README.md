# Ploot

Plot streaming data from stdin to a tty terminal. Useful for displaying data piped from a serial port or long running process.

![Example](./demo.svg)

Project used to be called ttyplot-rs.

## Install

```
$ git clone https://github.com/mogenson/ploot.git
$ cd ploot
$ cargo build # or cargo install --path .
```

## Usage

Pipe data from a process into `ploot`. Press <kbd>ctrl</kbd><kbd>c</kbd> to quit.

## Data format

Each line of data points can be positive or negative floating point numbers, separated by spaces. Ex: `1 2.3 -4.56`. A new frame of one to multiple series of data points is shown when a newline character is received. Statistics like minimum, maximum, average, and current value are shown for the visible data for each series.

## Options

```
-m, --min <min>        Lower bound of window (default: largest data point in window)
-M, --max <max>        Upper bound of window (default: smallest data point in window)
-w, --width <width>    Number of data points to display in window (default: terminal width)
-f, --file <file>      Log received data to file in CSV format
```

Copy `ploot.bash` to `/usr/share/bash-completion/completions` for command line tab-completion of options. A bash completion script can also be generated with the `--completions` flag.

## Examples

```
# build example program
$ rustc feed_input.rs
$ ./feed_input | ploot
```

```
# serial port
$ cat /dev/ttyUSB0 | ploot
```

```
# CPU percentage
$ sar 1 | awk '{ print 100.0-$NF; fflush(); }' | ploot 
```

```
# ping time
$ ping 8.8.8.8 | awk -F '[= ]' '{ print 0+$(NF-1); fflush(); }' | ploot
```

```
# bash saw wave
$ for ((i=0;; i++)); do echo `expr $i % 20`; sleep 0.1; done | ploot 
```

## Acknowledgment

Inspired by [ttyplot](https://github.com/tenox7/ttyplot). Some difference include: `ploot` can plot negative values and an arbitrary number of concurrent series, data window width can be specified, data points are plotted with points instead of bars, data rate is not calculated or plotted.
