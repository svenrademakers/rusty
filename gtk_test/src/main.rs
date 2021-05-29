extern crate gtk;
use gtk::prelude::*;
use gtk::{ButtonsType, DialogFlags, MessageDialog, MessageType, Window};

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let win = gtk::Window::new(gtk::WindowType::Toplevel);

    // Then we set its size and a title.
    win.set_default_size(320, 200);
    win.set_title("Basic example");

    // Don't forget to make all widgets visible.
    win.show_all();
    loop {
        unsafe {
            if gtk_sys::gtk_main_iteration_do(glib_sys::GFALSE) == glib_sys::GFALSE {
                break;
            }
        }
    }
}
