# A simple, very, very basic, SFO editor

Use this if using a hex editor is a bit annoying.
Can output all entries, you can (currently) only edit integer entries.

Very little sanity checking, you should probably check with an hex-editor afterwards

## Usage

```
$ cargo run -- --help
sfo_editor 0.1.0
David Sawatzke <david-sawatzke@users.noreply.github.com>

USAGE:
    sfo_editor [FLAGS] <file> <SUBCOMMAND>

FLAGS:
    -d, --debug      Activate debug mode
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <file>    Sfo file

SUBCOMMANDS:
    help     Prints this message or the help of the given subcommand(s)
    read     Simply output the fields
    write    Set a Integer parameter
```

## LICENSE
Licensed under GPLv3 or later
