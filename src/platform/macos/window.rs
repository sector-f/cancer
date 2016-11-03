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

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{Receiver, channel};

use cocoa;
use cocoa::base::{selector, nil, NO};
use cocoa::foundation::{NSUInteger, NSRect, NSPoint, NSSize, NSAutoreleasePool, NSProcessInfo, NSString};
use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicyRegular, NSWindow, NSTitledWindowMask, NSBackingStoreBuffered, NSMenu, NSMenuItem, NSRunningApplication, NSApplicationActivateIgnoringOtherApps};

use error;
use sys::cairo::Surface;
use config::Config;
use font::Font;
use platform::Event;
use picto::Region;

pub struct Window {
	receiver: Option<Receiver<Event>>,
	surface:  Option<Surface>,

	width:  Arc<AtomicU32>,
	height: Arc<AtomicU32>,
	focus:  Arc<AtomicBool>,
}

impl Window {
	pub fn open(name: Option<&str>, config: &Config, font: &Font) -> error::Result<Self> {
		let margin  = config.style().margin();
		let spacing = config.style().spacing();

		let mut width  = (80 * font.width()) + (margin * 2);
		let mut height = (24 * (font.height() + spacing)) + (margin * 2);

		let (sender, receiver) = channel();

		let width  = Arc::new(AtomicU32::new(width));
		let height = Arc::new(AtomicU32::new(height));
		let focus  = Arc::new(AtomicBool::new(true));

		let (app, surface) = {
			let _pool = NSAutoreleasePool::new(nil);
			let app = NSApp();
			app.setActivationPolicy_(NSApplicationActivationPolicyRegular);
	
			// create Menu Bar
			let menubar = NSMenu::new(nil).autorelease();
			let app_menu_item = NSMenuItem::new(nil).autorelease();
			menubar.addItem_(app_menu_item);
			app.setMainMenu_(menubar);
	
			// create Application menu
			let app_menu = NSMenu::new(nil).autorelease();
			let quit_prefix = NSString::alloc(nil).init_str("Quit");
			let quit_title = quit_prefix.stringByAppendingString_(
				NSProcessInfo::processInfo(nil).processName());
			let quit_action = selector("terminate:");
			let quit_key = NSString::alloc(nil).init_str("q");
			let quit_item = NSMenuItem::alloc(nil).initWithTitle_action_keyEquivalent_(
				quit_title,
				quit_action,
				quit_key).autorelease();
			app_menu.addItem_(quit_item);
			app_menu_item.setSubmenu_(app_menu);
	
			// create Window
			let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
				NSRect::new(NSPoint::new(0., 0.), NSSize::new(200., 200.)),
				NSTitledWindowMask as NSUInteger,
				NSBackingStoreBuffered,
				NO
			).autorelease();
	
			window.cascadeTopLeftFromPoint_(NSPoint::new(20., 20.));
			window.center();
	
			let title = NSString::alloc(nil).init_str("Hello World!");
			window.setTitle_(title);
			window.makeKeyAndOrderFront_(nil);
	
			let current_app = NSRunningApplication::currentApplication(nil);
			current_app.activateWithOptions_(NSApplicationActivateIgnoringOtherApps);
		};

		unsafe {
			let width  = width.clone();
			let height = height.clone();
			let focus  = focus.clone();

			thread::spawn(move || {
				app.run();
			});
		}

		Ok(Window {
			receiver: Some(receiver),
			surface:  None,

			width:  width,
			height: height,
			focus:  focus,
		})
	}

	/// Get the width.
	pub fn width(&self) -> u32 {
		self.width.load(Ordering::Relaxed)
	}

	/// Get the height.
	pub fn height(&self) -> u32 {
		self.height.load(Ordering::Relaxed)
	}

	/// Check if the window has focus.
	pub fn has_focus(&self) -> bool {
		self.focus.load(Ordering::Relaxed)
	}

	/// Take the events sink.
	pub fn events(&mut self) -> Receiver<Event> {
		self.receiver.take().unwrap()
	}

	/// Take the surface.
	pub fn surface(&mut self) -> Surface {
		self.surface.take().unwrap()
	}

	/// Resize the window.
	pub fn resize(&mut self, width: u32, height: u32) {
	}

	/// Set the window title.
	pub fn set_title<T: Into<String>>(&self, title: T) {
	}

	/// Set the clipboard.
	pub fn clipboard<T1: Into<String>, T2: Into<String>>(&self, name: T1, value: T2) {
	}

	/// Flush the surface and connection.
	pub fn flush(&self) {
	}
}
