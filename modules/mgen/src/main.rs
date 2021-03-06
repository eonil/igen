// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(rustc_private)]
#![feature(box_patterns)]
#![feature(box_syntax)]

extern crate arena;
extern crate getopts;
extern crate env_logger;
extern crate rustc;
extern crate rustc_data_structures;
extern crate rustc_codegen_utils;
extern crate rustc_driver;
extern crate rustc_resolve;
extern crate rustc_lint;
extern crate rustc_metadata;
extern crate rustc_target;
extern crate rustc_typeck;
extern crate serialize;
extern crate syntax;
extern crate syntax_pos;
#[macro_use] extern crate log;
extern crate rustc_errors as errors;
extern crate rustdoc;
extern crate serialize as rustc_serialize; // used by deriving
extern crate serde;
#[macro_use] extern crate serde_derive;

mod schema;

use errors::ColorConfig;
use std::collections::{BTreeMap, BTreeSet};
use std::default::Default;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use syntax::edition::Edition;
use rustc::session::{early_warn, early_error};
use rustc::session::search_paths::SearchPaths;
use rustc::session::config::{ErrorOutputType, RustcOptGroup, Externs, CodegenOptions};
use rustc::session::config::{nightly_options, build_codegen_options};
use rustc_target::spec::TargetTriple;
use rustc::session::config::get_cmd_lint_options;
pub use rustdoc::core;

pub fn main() {
    rustc_driver::set_sigpipe_handler();
    env_logger::init();
    syntax::with_globals(move || {
        get_args().map(|args| main_args(&args)).unwrap_or(1)
    });
}

pub fn main_args(args: &[String]) -> isize {
    let mut options = getopts::Options::new();
    for option in opts() {
        (option.apply)(&mut options);
    }
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(err) => {
            early_error(ErrorOutputType::default(), &err.to_string());
        }
    };
    // Check for unstable options.
    nightly_options::check_nightly_options(&matches, &opts());

    if matches.opt_present("h") || matches.opt_present("help") {
        usage("rustdoc");
        return 0;
    } else if matches.opt_present("version") {
        rustc_driver::version("rustdoc", &matches);
        return 0;
    }

    let color = match matches.opt_str("color").as_ref().map(|s| &s[..]) {
        Some("auto") => ColorConfig::Auto,
        Some("always") => ColorConfig::Always,
        Some("never") => ColorConfig::Never,
        None => ColorConfig::Auto,
        Some(arg) => {
            early_error(ErrorOutputType::default(),
                        &format!("argument for --color must be `auto`, `always` or `never` \
                                  (instead was `{}`)", arg));
        }
    };
    let error_format = match matches.opt_str("error-format").as_ref().map(|s| &s[..]) {
        Some("human") => ErrorOutputType::HumanReadable(color),
        Some("json") => ErrorOutputType::Json(false),
        Some("pretty-json") => ErrorOutputType::Json(true),
        Some("short") => ErrorOutputType::Short(color),
        None => ErrorOutputType::HumanReadable(color),
        Some(arg) => {
            early_error(ErrorOutputType::default(),
                        &format!("argument for --error-format must be `human`, `json` or \
                                  `short` (instead was `{}`)", arg));
        }
    };

    let diag = core::new_handler(error_format, None);

    if matches.free.is_empty() {
        diag.struct_err("missing file operand").emit();
        return 1;
    }
    if matches.free.len() > 1 {
        diag.struct_err("too many file operands").emit();
        return 1;
    }
    let input = &matches.free[0];

    let mut libs = SearchPaths::new();
    for s in &matches.opt_strs("L") {
        libs.add_path(s, error_format);
    }
    let externs = match parse_externs(&matches) {
        Ok(ex) => ex,
        Err(err) => {
            diag.struct_err(&err.to_string()).emit();
            return 1;
        }
    };

    /*
    let output = matches.opt_str("o").map(|s| PathBuf::from(&s));
    let cfgs = matches.opt_strs("cfg");

    let crate_name = matches.opt_str("crate-name");
    let maybe_sysroot = matches.opt_str("sysroot").map(PathBuf::from);
    let display_warnings = matches.opt_present("display-warnings");
    let linker = matches.opt_str("linker").map(PathBuf::from);
    let sort_modules_alphabetically = !matches.opt_present("sort-modules-by-appearance");
    let resource_suffix = matches.opt_str("resource-suffix");
    let enable_minification = !matches.opt_present("disable-minification");
    */
    let edition = matches.opt_str("edition").unwrap_or("2015".to_string());
    let edition = match edition.parse() {
        Ok(e) => e,
        Err(_) => {
            diag.struct_err("could not parse edition").emit();
            return 1;
        }
    };
    

    let cg = build_codegen_options(&matches, ErrorOutputType::default());
    rust_input(PathBuf::from(input), externs, edition, cg, &matches, error_format);
    return 0;
}
fn get_args() -> Option<Vec<String>> {
    env::args_os().enumerate()
        .map(|(i, arg)| arg.into_string().map_err(|arg| {
             early_warn(ErrorOutputType::default(),
                        &format!("Argument {} is not valid Unicode: {:?}", i, arg));
        }).ok())
        .collect()
}

fn stable<F>(name: &'static str, f: F) -> RustcOptGroup
    where F: Fn(&mut getopts::Options) -> &mut getopts::Options + 'static
{
    RustcOptGroup::stable(name, f)
}

fn unstable<F>(name: &'static str, f: F) -> RustcOptGroup
    where F: Fn(&mut getopts::Options) -> &mut getopts::Options + 'static
{
    RustcOptGroup::unstable(name, f)
}

pub fn opts() -> Vec<RustcOptGroup> {
    vec![
        stable("h", |o| o.optflag("h", "help", "show this help message")),
        stable("V", |o| o.optflag("V", "version", "print rustdoc's version")),
        stable("v", |o| o.optflag("v", "verbose", "use verbose output")),
        stable("o", |o| o.optopt("o", "output", "where to place the output", "PATH")),
        stable("crate-name", |o| {
            o.optopt("", "crate-name", "specify the name of this crate", "NAME")
        }),
        stable("L", |o| {
            o.optmulti("L", "library-path", "directory to add to crate search path",
                       "DIR")
        }),
        stable("cfg", |o| o.optmulti("", "cfg", "pass a --cfg to rustc", "")),
        stable("extern", |o| {
            o.optmulti("", "extern", "pass an --extern to rustc", "NAME=PATH")
        }),
        stable("C", |o| {
            o.optmulti("C", "codegen", "pass a codegen option to rustc", "OPT[=VALUE]")
        }),
        stable("test", |o| o.optflag("", "test", "run code examples as tests")),
        stable("test-args", |o| {
            o.optmulti("", "test-args", "arguments to pass to the test runner",
                       "ARGS")
        }),
        stable("target", |o| o.optopt("", "target", "target triple to document", "TRIPLE")),
        stable("e", |o| {
            o.optopt("e", "extend-css",
                     "To add some CSS rules with a given file to generate doc with your \
                      own theme. However, your theme might break if the rustdoc's generated HTML \
                      changes, so be careful!", "PATH")
        }),
        unstable("Z", |o| {
            o.optmulti("Z", "",
                       "internal and debugging options (only on nightly build)", "FLAG")
        }),
        stable("sysroot", |o| {
            o.optopt("", "sysroot", "Override the system root", "PATH")
        }),
        unstable("display-warnings", |o| {
            o.optflag("", "display-warnings", "to print code warnings when testing doc")
        }),
        unstable("crate-version", |o| {
            o.optopt("", "crate-version", "crate version to print into documentation", "VERSION")
        }),
        unstable("linker", |o| {
            o.optopt("", "linker", "linker used for building executable test code", "PATH")
        }),
        unstable("sort-modules-by-appearance", |o| {
            o.optflag("", "sort-modules-by-appearance", "sort modules by where they appear in the \
                                                         program, rather than alphabetically")
        }),
        unstable("themes", |o| {
            o.optmulti("", "themes",
                       "additional themes which will be added to the generated docs",
                       "FILES")
        }),
        unstable("theme-checker", |o| {
            o.optmulti("", "theme-checker",
                       "check if given theme is valid",
                       "FILES")
        }),
        unstable("resource-suffix", |o| {
            o.optopt("",
                     "resource-suffix",
                     "suffix to add to CSS and JavaScript files, e.g. \"light.css\" will become \
                      \"light-suffix.css\"",
                     "PATH")
        }),
        unstable("edition", |o| {
            o.optopt("", "edition",
                     "edition to use when compiling rust code (default: 2015)",
                     "EDITION")
        }),
        unstable("color", |o| {
            o.optopt("",
                     "color",
                     "Configure coloring of output:
                                          auto   = colorize, if output goes to a tty (default);
                                          always = always colorize output;
                                          never  = never colorize output",
                     "auto|always|never")
        }),
        unstable("error-format", |o| {
            o.optopt("",
                     "error-format",
                     "How errors and other messages are produced",
                     "human|json|short")
        }),
        unstable("disable-minification", |o| {
             o.optflag("",
                       "disable-minification",
                       "Disable minification applied on JS files")
        }),
        unstable("warn", |o| {
            o.optmulti("W", "warn", "Set lint warnings", "OPT")
        }),
        unstable("allow", |o| {
            o.optmulti("A", "allow", "Set lint allowed", "OPT")
        }),
        unstable("deny", |o| {
            o.optmulti("D", "deny", "Set lint denied", "OPT")
        }),
        unstable("forbid", |o| {
            o.optmulti("F", "forbid", "Set lint forbidden", "OPT")
        }),
        unstable("cap-lints", |o| {
            o.optmulti(
                "",
                "cap-lints",
                "Set the most restrictive lint level. \
                 More restrictive lints are capped at this \
                 level. By default, it is at `forbid` level.",
                "LEVEL",
            )
        }),
    ]
}

pub fn usage(argv0: &str) {
    let mut options = getopts::Options::new();
    for option in opts() {
        (option.apply)(&mut options);
    }
    println!("{}", options.usage(&format!("{} [options] <input>", argv0)));
}


/// Extracts `--extern CRATE=PATH` arguments from `matches` and
/// returns a map mapping crate names to their paths or else an
/// error message.
fn parse_externs(matches: &getopts::Matches) -> Result<Externs, String> {
    let mut externs = BTreeMap::new();
    for arg in &matches.opt_strs("extern") {
        let mut parts = arg.splitn(2, '=');
        let name = parts.next().ok_or("--extern value must not be empty".to_string())?;
        let location = parts.next()
                                 .ok_or("--extern value must be of the format `foo=bar`"
                                    .to_string())?;
        let name = name.to_string();
        externs.entry(name).or_insert_with(BTreeSet::new).insert(location.to_string());
    }
    Ok(Externs::new(externs))
}

/// Interprets the input file as a rust source file, passing it through the
/// compiler all the way through the analysis passes. The rustdoc output is then
/// generated from the cleaned AST of the crate.
///
/// This form of input will run all of the plug/cleaning passes
fn rust_input(cratefile: PathBuf,
                    externs: Externs,
                    edition: Edition,
                    cg: CodegenOptions,
                    matches: &getopts::Matches,
                    error_format: ErrorOutputType) 
{
    // First, parse the crate and extract all relevant information.
    let mut paths = SearchPaths::new();
    for s in &matches.opt_strs("L") {
        paths.add_path(s, ErrorOutputType::default());
    }
    let cfgs = matches.opt_strs("cfg");
    let triple = matches.opt_str("target").map(|target| {
        if target.ends_with(".json") {
            TargetTriple::TargetPath(PathBuf::from(target))
        } else {
            TargetTriple::TargetTriple(target)
        }
    });
    let maybe_sysroot = matches.opt_str("sysroot").map(PathBuf::from);
    let crate_name = matches.opt_str("crate-name");
    let crate_version = matches.opt_str("crate-version");

    info!("starting to run rustc");
    let display_warnings = matches.opt_present("display-warnings");

    let force_unstable_if_unmarked = matches.opt_strs("Z").iter().any(|x| {
        *x == "force-unstable-if-unmarked"
    });

    let (lint_opts, describe_lints, lint_cap) = get_cmd_lint_options(matches, error_format);

    let (tx, rx) = channel();

    rustc_driver::monitor(move || syntax::with_globals(move || {
        use rustc::session::config::Input;

        let (mut krate, _) =
            core::run_core(paths, cfgs, externs, Input::File(cratefile), triple, maybe_sysroot,
                           display_warnings, crate_name.clone(),
                           force_unstable_if_unmarked, edition, cg, error_format,
                           lint_opts, lint_cap, describe_lints);

        info!("finished with rustc");
        krate.name = crate_name.unwrap_or(krate.name);
        krate.version = crate_version;

        let module = schema::Module::scan(&krate);
        module.gen_swift_code();
        
        //tx.send(f(Output { krate: krate, renderinfo: renderinfo, passes: passes })).unwrap();
        tx.send(()).unwrap();
    }));
    rx.recv().unwrap();
}

