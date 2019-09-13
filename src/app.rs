use std::env;
use std::path::PathBuf;

use clap::{App, AppSettings, Arg, ArgGroup, ArgMatches, SubCommand};
use dirs;

use crate::templates;

const NAME: &str = "cargo-eval";

#[inline(always)]
const fn name() -> &'static str {
    NAME
}

#[inline(always)]
fn subcommand_name() -> &'static str {
    &name()[6..]
}

pub fn data_dir() -> Option<PathBuf> {
    Some(dirs::data_local_dir()?.join(name()))
}

pub fn cache_dir() -> Option<PathBuf> {
    Some(dirs::cache_dir()?.join(name()))
}

fn app() -> App<'static, 'static> {
    let mut app = SubCommand::with_name(subcommand_name())
    .version(env!("CARGO_PKG_VERSION"))
    .about("Compiles and runs “Cargoified Rust scripts”.")
    .usage("cargo eval [FLAGS OPTIONS] [--] <script> <args>...")

    /*
    Major script modes.
    */
    .arg(Arg::with_name("script")
        .help("Script file (with or without extension) to execute.")
        .index(1)
        .required_unless("clear_cache")
    )
    .arg(Arg::with_name("args")
        .help("Additional arguments passed to the script.")
        .index(2)
        .multiple(true)
    )
    .arg(Arg::with_name("expr")
        .help("Execute <script> as a literal expression and display the result.")
        .long("expr")
        .short("e")
        .requires("script")
    )
    .arg(Arg::with_name("loop")
        .help("Execute <script> as a literal closure once for each line from stdin.")
        .long("loop")
        .short("l")
        .requires("script")
    )
    .group(ArgGroup::with_name("expr_or_loop")
        .args(&["expr", "loop"])
    )

    /*
    Options that impact the script being executed.
    */
    .arg(Arg::with_name("count")
        .help("Invoke the loop closure with two arguments: line, and line number.")
        .long("count")
        .requires("loop")
    )
    .arg(Arg::with_name("debug")
        .help("Build a debug executable, not an optimised one.")
        .long("debug")
        .requires("script")
    )
    .arg(Arg::with_name("dep")
        .help("Add an additional Cargo dependency.  Each SPEC can be either just the package name (which will assume the latest version) or a full `name=version` spec.")
        .long("dep")
        .short("d")
        .takes_value(true)
        .multiple(true)
        .number_of_values(1)
        .requires("script")
    )
    .arg(Arg::with_name("features")
         .help("Cargo features to pass when building and running.")
         .long("features")
         .takes_value(true)
    )
    .arg(Arg::with_name("unstable_features")
        .help("Add a #![feature] declaration to the crate.")
        .long("unstable-feature")
        .short("u")
        .takes_value(true)
        .multiple(true)
        .requires("expr_or_loop")
    )

    /*
    Options that change how cargo eval itself behaves, and don't alter what the script will do.
    */
    .arg(Arg::with_name("build_only")
        .help("Build the script, but don't run it.")
        .long("build-only")
        .requires("script")
        .conflicts_with_all(&["args"])
    )
    .arg(Arg::with_name("clear_cache")
        .help("Clears out the script cache.")
        .long("clear-cache")
    )
    .arg(Arg::with_name("force")
        .help("Force the script to be rebuilt.")
        .long("force")
        .requires("script")
    )
    .arg(Arg::with_name("gen_pkg_only")
        .help("Generate the Cargo package, but don't compile or run it.")
        .long("gen-pkg-only")
        .requires("script")
        .conflicts_with_all(&["args", "build_only", "debug", "force", "test", "bench"])
    )
    .arg(Arg::with_name("pkg_path")
        .help("Specify where to place the generated Cargo package.")
        .long("pkg-path")
        .takes_value(true)
        .requires("script")
        .conflicts_with_all(&["clear_cache", "force"])
    )
    .arg(Arg::with_name("use_bincache")
        .help("Override whether or not the shared binary cache will be used for compilation.")
        .long("use-shared-binary-cache")
        .takes_value(true)
        .possible_values(&["no", "yes"])
    )
    .arg(Arg::with_name("test")
        .help("Compile and run tests.")
        .long("test")
        .conflicts_with_all(&["bench", "debug", "args", "force"])
    )
    .arg(Arg::with_name("bench")
        .help("Compile and run benchmarks.  Requires a nightly toolchain.")
        .long("bench")
        .conflicts_with_all(&["test", "debug", "args", "force"])
    )
    .arg(Arg::with_name("template")
        .help("Specify a template to use for expression scripts.")
        .long("template")
        .short("t")
        .takes_value(true)
        .requires("expr")
    );

    #[cfg(windows)]
    {
        app = app.subcommand(crate::file_assoc::Args::subcommand())
    }

    app = app.subcommand(templates::Args::subcommand());

    app
}

pub fn get_matches() -> ArgMatches<'static> {
    let mut args = env::args().collect::<Vec<_>>();

    let subcommand = app();

    // Insert subcommand argument if called directly.
    if args.get(1).map(|s| *s == subcommand_name()) != Some(true) {
        args.insert(1, subcommand_name().into());
    }

    // We have to wrap our command for the output to look right.
    App::new("cargo")
        .bin_name("cargo")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(subcommand)
        .get_matches_from(args)
        .subcommand_matches(subcommand_name())
        .unwrap()
        .clone()
}
