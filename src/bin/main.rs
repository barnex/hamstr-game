extern crate cpuprofiler;
extern crate rand;
use cpuprofiler::PROFILER;
use flux::editor::prelude::*;
use flux::game::sdl_interface;
use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() {
	let args: Vec<OsString> = env::args_os().skip(1).collect();

	let mut ed = match args.len() {
		0 => Editor::new(),
		1 => Editor::load(PathBuf::from(&args[0])).expect("loading level"),
		_ => {
			write!(io::stderr(), "need 0 or 1 arguments, got {}", args.len()).unwrap();
			std::process::exit(1);
		}
	};

	start_pprof();
	sdl_interface::mainloop(&mut ed).expect("initialize SDL");
	stop_pprof();
}

fn start_pprof() {
	PROFILER
		.lock()
		.unwrap()
		.start("./profile.pprof")
		.expect("start pprof");
}

fn stop_pprof() {
	PROFILER.lock().unwrap().stop().expect("stop pprof");
}
