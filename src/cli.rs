#[link (name = "beamwand", vers = "0.0")];
#[desc = "BEAM JIT Compiler"];
#[license = "BSD"];
#[author = "Fedor Indutny"];

extern mod extra;

use std::{io, os};

mod beam;

struct Options {
  display_help: bool,
  parser_only: bool,
  path: ~Path
}

fn start(path: &Path, options: &Options) -> bool {
  if path.to_str().len() == 0 || options.display_help {
    return display_usage();
  }

  if !path.exists() {
    io::stderr().write_line(fmt!("File %s doesn't exist!", path.to_str()));
    return false;
  }

  let content: ~[u8] = io::read_whole_file(path).get();

  if (options.parser_only) {
    return parse_and_print(content);
  } else {
    fail!(~"Compiler mode is not supported yet.");
  }
}

fn display_usage() -> bool {
  io::stdout().write_line("Usage: beamwand <scriptname>.beam");
  io::stdout().write_line("");
  return true;
}

fn parse_and_print(source: ~[u8]) -> bool {
  let ast = beam::parse(source);
  io::println(ast.to_str());
  return true;
}

fn main() {
  let args = os::args();

  // Gather options from command line
  let mut opt = Options {
    display_help: false,
    parser_only: false,
    path: ~Path("")
  };

  for args.eachi |i, arg| {
    // Skip app filename
    if i != 0 {
      match *arg {
        ~"-h" => opt.display_help = true,
        ~"--help" => opt.display_help = true,
        ~"--parser-only" => opt.parser_only = true,
        _ => opt.path = ~Path(*arg)
      }
    }
  }

  if !start(opt.path, &opt) {
    os::set_exit_status(1);
  }
}
