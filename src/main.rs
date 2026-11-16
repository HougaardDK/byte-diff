use std::env;
use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "usage: byte-diff <old-file> <new-file>";

fn open(path: &Path) -> io::Result<BufReader<File>> {
    let with_path = |err: io::Error| io::Error::new(err.kind(), format!("{path:?}: {err}"));
    if !path.metadata().map_err(with_path)?.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{path:?}: not a file"),
        ));
    }
    let file = File::open(path).map_err(with_path)?;
    Ok(BufReader::new(file))
}

fn read_byte(file: &mut BufReader<File>, done: &mut bool) -> io::Result<Option<u8>> {
    if *done {
        return Ok(None);
    }
    let mut b = [0];
    match file.read_exact(&mut b) {
        Ok(()) => Ok(Some(b[0])),
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
            *done = true;
            Ok(None)
        }
        Err(err) => Err(err),
    }
}

fn print_row(out: &mut impl Write, start: u64, row: &[(Option<u8>, Option<u8>)]) -> io::Result<()> {
    write!(out, "0x{start:08x} ")?;
    for &(a, _) in row {
        match a {
            Some(byte) => write!(out, " {byte:02x}")?,
            None => write!(out, " --")?,
        }
    }

    write!(out, " ->")?;
    for &(_, b) in row {
        match b {
            Some(byte) => write!(out, " {byte:02x}")?,
            None => write!(out, " --")?,
        }
    }
    writeln!(out)
}

fn compare(a: &Path, b: &Path, out: &mut impl Write) -> io::Result<bool> {
    let mut old = open(a)?;
    let mut new = open(b)?;
    let mut old_done = false;
    let mut new_done = false;
    let width = 16;
    let mut row = Vec::with_capacity(width);
    let mut start = 0;
    let mut pos = 0;
    let mut changed = 0u64;
    let mut added = 0u64;
    let mut removed = 0u64;

    loop {
        let x = read_byte(&mut old, &mut old_done)?;
        let y = read_byte(&mut new, &mut new_done)?;

        // same byte, end the row
        if !row.is_empty() && (x == y || row.len() == width) {
            print_row(out, start, &row)?;
            row.clear();
        }
        if x.is_none() && y.is_none() {
            break;
        }
        if x != y {
            if row.is_empty() {
                start = pos;
            }
            row.push((x, y));
            if x.is_none() {
                added += 1;
            } else if y.is_none() {
                removed += 1;
            } else {
                changed += 1;
            }
        }
        pos += 1;
    }

    writeln!(out, "{changed} changed, {added} added, {removed} removed")?;
    out.flush()?;
    Ok(changed != 0 || added != 0 || removed != 0)
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() == 1 && (args[0] == "-h" || args[0] == "--help") {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    if args.len() != 2 {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }

    // todo: summary-only flag
    let old = Path::new(&args[0]);
    let new = Path::new(&args[1]);
    let mut out = BufWriter::new(io::stdout().lock());
    match compare(old, new, &mut out) {
        Ok(false) => ExitCode::SUCCESS,
        Ok(true) => ExitCode::from(1),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("byte-diff: {err}");
            ExitCode::from(2)
        }
    }
}
