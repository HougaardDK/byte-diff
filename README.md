# byte-diff

A small Rust CLI for comparing two binary files. It groups nearby changes and
shows the old and new bytes in hex. Offsets start at zero and `--` marks a byte
missing from one file.

## Try it on Linux

Run these from the repo folder with Rust 1.65 or newer:

```bash
cargo build --release --locked
demo=$(mktemp -d)
printf 'a\001\002z' > "$demo/old.bin"
printf 'a\003\004z!' > "$demo/new.bin"
./target/release/byte-diff "$demo/old.bin" "$demo/new.bin"
```

```text
0x00000001  01 02 -> 03 04
0x00000004  -- -> 21
2 changed, 1 added, 0 removed
```

It compares bytes at the same offset. An insert in the middle shifts the rest
of the comparison. The exit code is `0` for identical files, `1` for different
files and `2` for errors.
