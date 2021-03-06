// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// This file is part of cancer.
//
// cancer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cancer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cancer.  If not, see <http://www.gnu.org/licenses/>.

#![feature(mpsc_select, conservative_impl_trait, slice_patterns, static_in_const)]
#![feature(trace_macros, type_ascription, inclusive_range_syntax, pub_restricted)]
#![feature(deque_extras, integer_atomics)]
#![recursion_limit="100"]

#[macro_use]
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate bitflags;
extern crate bit_vec;
extern crate fnv;
extern crate itertools;
extern crate lru_cache as lru;
extern crate shlex;
extern crate picto;
extern crate schedule_recv as timer;
#[macro_use]
extern crate control_code as control;

extern crate unicode_segmentation;
extern crate unicode_width;

extern crate regex;
extern crate app_dirs;
extern crate toml;
extern crate clap;
use clap::{App, Arg};

extern crate libc;

#[cfg(target_os = "linux")]
extern crate xcb;
#[cfg(target_os = "linux")]
extern crate xcb_util as xcbu;
#[cfg(target_os = "linux")]
extern crate xkbcommon;

#[cfg(target_os = "macos")]
extern crate cocoa;

#[macro_use]
mod util;
mod error;
mod ffi;
mod sys;

mod config;
use config::Config;

mod font;
use font::Font;

mod terminal;
use terminal::Terminal;

mod interface;
pub use interface::{Interface, Action};

mod overlay;
pub use overlay::Overlay;

mod style;

mod platform;
use platform::{Event, Window, Tty};
use platform::mouse::{self, Mouse};

mod renderer;
use renderer::Renderer;

use std::mem;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::io::Write;
use std::iter;

fn main() {
	env_logger::init().unwrap();

	let matches = App::new("cancer")
		.version(env!("CARGO_PKG_VERSION"))
		.author("meh. <meh@schizofreni.co>")
		.arg(Arg::with_name("config")
			.short("c")
			.long("config")
			.help("The path to the configuration file.")
			.takes_value(true))
		.arg(Arg::with_name("display")
			.short("d")
			.long("display")
			.takes_value(true)
			.help("The X11 display."))
		.arg(Arg::with_name("execute")
			.short("e")
			.long("execute")
			.takes_value(true)
			.help("Program to execute."))
		.arg(Arg::with_name("font")
			.short("f")
			.long("font")
			.takes_value(true)
			.help("Font to use with the terminal."))
		.arg(Arg::with_name("name")
			.short("n")
			.long("name")
			.takes_value(true)
			.help("Name for the window."))
		.arg(Arg::with_name("term")
			.short("t")
			.long("term")
			.takes_value(true).
			help("Specify the TERM environment variable to use."))
		.arg(Arg::with_name("tic")
			.short("T")
			.long("tic")
			.help("Print the terminfo database to stdout and exit."))
		.get_matches();

	if matches.is_present("tic") {
		print!("{}", include_str!("../assets/cancer.info"));
		return;
	}

	let config = Arc::new(Config::load(matches.value_of("config")).unwrap());
	let font   = Arc::new(Font::load(matches.value_of("font").unwrap_or(config.style().font())).unwrap());

	let mut window    = Window::new(matches.value_of("name"), config.clone(), &font).unwrap();
	let mut surface   = window.surface();
	let mut renderer  = Renderer::new(config.clone(), font.clone(), &surface, window.width(), window.height());
	let mut interface = Interface::from(Terminal::new(config.clone(), renderer.columns(), renderer.rows()).unwrap());
	let mut tty       = Tty::spawn(renderer.columns(), renderer.rows(),
	                               matches.value_of("term").or_else(|| config.environment().term()),
	                               matches.value_of("execute").or_else(|| config.environment().program())).unwrap();

	let     blink    = timer::periodic_ms(config.style().blink());
	let mut blinking = true;

	let (_k, mut batch) = channel();
	let mut batching    = None;
	let mut batched     = None;

	let input  = tty.output();
	let events = window.events();

	macro_rules! render {
		(options) => ({
			let mut options = renderer::Options::empty();

			if interface.mode().contains(terminal::mode::BLINK) {
				options.insert(renderer::option::BLINKING);
			}

			if window.has_focus() {
				options.insert(renderer::option::FOCUS);
			}

			if interface.mode().contains(terminal::mode::REVERSE) {
				options.insert(renderer::option::REVERSE);
			}

			if interface.cursor().is_visible() {
				options.insert(renderer::option::CURSOR);
			}

			options
		});

		(options!) => ({
			let mut options = render!(options);
			options.insert(renderer::option::DAMAGE);

			options
		});

		(handle $what:expr) => ({
			let (actions, touched) = try!(continue $what);

			if touched.is_total() && batched.is_none() && config.environment().batch().is_some() {
				batching = Some(true);
			}
			else if batched.is_none() && !touched.is_empty() {
				render!(touched);
			}

			for action in actions {
				match action {
					Action::Urgent => {
						window.urgent();
					}

					Action::Overlay(true) => {
						interface = Overlay::new(try!(return interface.into_inner(tty.by_ref()))).into();
						render!(interface.region().absolute());
					}

					Action::Overlay(false) => {
						interface = try!(return interface.into_inner(tty.by_ref())).into();
						render!(interface.region().absolute());
					}

					Action::Title(string) => {
						window.set_title(string);
					}

					Action::Resize(columns, rows) => {
						let (width, height) = Renderer::dimensions(columns, rows, &config, &font);
						window.resize(width, height);
					}

					Action::Copy(name, value) => {
						window.copy(name, value);
					}

					Action::Paste(name) => {
						window.paste(name)
					}

					Action::Open(what) => {
						window.open(what).unwrap();
					}
				}
			}

			try!(return tty.flush());
		});

		($iter:expr) => ({
			let iter = $iter;

			if window.is_visible() {
				let options = render!(options);

				renderer.batch(|mut o| {
					o.update(&interface, iter, options);
				});

				surface.flush();
				window.flush();
			}
		});
	}

	loop {
		match batching.take() {
			Some(true) => {
				batched = Some(mem::replace(&mut batch,
					timer::oneshot_ms(config.environment().batch().unwrap())));
			}

			Some(false) => {
				batch = batched.take().unwrap();
				render!(interface.region().absolute());
			}

			None => ()
		}

		select! {
			_ = batch.recv() => {
				batching = Some(false);
			},

			_ = blink.recv() => {
				blinking = !blinking;

				let blinked = interface.blinking(blinking);
				if (!blinked.is_empty() || interface.cursor().blink()) && batched.is_none() {
					render!(blinked);
				}
			},

			event = events.recv() => {
				match try!(return event) {
					Event::Show(_) => (),

					Event::Redraw(region) => {
						let options = render!(options!);

						renderer.batch(|mut o| {
							// Redraw margins.
							o.margin(&region);
			
							// Redraw the cells that fall within the damaged region.
							let damaged = o.damaged(&region).relative();
							o.update(&interface, damaged, options);
						});
			
						surface.flush();
						window.flush();
					}

					Event::Focus(focus) => {
						try!(return interface.focus(focus, tty.by_ref()));
						render!(iter::empty());
					}

					Event::Resize(width, height) => {
						if interface.overlay() {
							interface = try!(return interface.into_inner(tty.by_ref())).into();
						}

						renderer.resize(width, height);
						surface.resize(width, height);

						let rows    = renderer.rows();
						let columns = renderer.columns();

						if interface.columns() != columns || interface.rows() != rows {
							try!(return tty.resize(columns, rows));
							interface.resize(columns, rows);
						}
					}

					Event::Paste(value) => {
						try!(return interface.paste(&value, tty.by_ref()));
						try!(return tty.flush());
					}

					Event::Key(key) => {
						render!(handle interface.key(key, tty.by_ref()));
					}

					Event::Mouse(mut event) => {
						match event {
							Mouse::Click(mouse::Click { ref mut position, .. }) |
							Mouse::Motion(mouse::Motion { ref mut position, .. }) => {
								if let Some((x, y)) = renderer.position(position.x, position.y) {
									position.x = x;
									position.y = y;
								}
								else {
									continue;
								}
							}
						}

						render!(handle interface.mouse(event, tty.by_ref()));
					}
				}
			},

			input = input.recv() => {
				render!(handle interface.input(&try!(return input), tty.by_ref()));
			}
		}
	}
}
