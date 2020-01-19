# ttyplot-rs

<img src="./demo.svg" alt="Demo cast pipeing output from feed_input.rs into ttyplot-rs">

```
ttyplot-rs 0.1.0
Plot streaming data from stdin to a tty terminal. Useful for displaying data piped from a serial port
or long running process. To plot multiple data streams, separate data points with whitespace.
Use CTRL-C to quit.

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

## Install

```
$ git clone https://github.com/mogenson/ttyplot-rs.git
$ cd ttyplot-rs
$ cargo build # or cargo install --path .
```

## Examples

```
# build example program
$ rustc feed_input.rs
$ ./feed_input | ttyplot-rs
```

```
# serial port
$ cat /dev/ttyUSB0 | ttyplot-rs
```

```
# CPU percentage
$ sar 1 | awk '{ print 100.0-$NF; fflush(); } | ttyplot-rs 
```

```
# ping time
$ ping 8.8.8.8 | awk -F '[= ]' '{ print 0+$(NF-1); fflush(); }' | ttyplot-rs
```

```
# bash saw wave
$ for ((i=0;; i++)); do echo `expr $i % 20`; sleep 0.1; done | ttyplot-rs 
```
