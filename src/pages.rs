use crate::application_browser::ApplicationBrowser;
use crate::utils;
use crate::utils::PacmanWrapper;
use gtk::{glib, Builder};
use std::fmt::Write as _;
use std::path::Path;

use gtk::prelude::*;

use std::str;
use subprocess::{Exec, Redirection};

fn create_fixes_section() -> gtk::Box {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let button_box_f = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let button_box_s = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let button_box_t = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text("Available Options:");

    let removelock_btn = gtk::Button::with_label("Remove db lock");
    let reinstall_btn = gtk::Button::with_label("Reinstall all packages");
    let update_system_btn = gtk::Button::with_label("System update");
    let remove_orphans_btn = gtk::Button::with_label("Remove orphans");
    let clear_pkgcache_btn = gtk::Button::with_label("Clear package cache");

    removelock_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            if Path::new("/var/lib/pacman/db.lck").exists() {
                let _ = Exec::cmd("/sbin/pkexec")
                    .arg("bash")
                    .arg("-c")
                    .arg("rm /var/lib/pacman/db.lck")
                    .join()
                    .unwrap();
                if !Path::new("/var/lib/pacman/db.lck").exists() {
                    let dialog = gtk::MessageDialog::builder()
                        .message_type(gtk::MessageType::Info)
                        .text("Pacman db lock was removed!")
                        .build();
                    dialog.show();
                }
            }
        });
    });
    reinstall_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            let _ = utils::run_cmd_terminal(String::from("pacman -S $(pacman -Qnq)"), true);
        });
    });
    update_system_btn.connect_clicked(on_update_system_btn_clicked);
    remove_orphans_btn.connect_clicked(move |_| {
        // Spawn child process in separate thread.
        std::thread::spawn(move || {
            let _ = utils::run_cmd_terminal(String::from("pacman -Rns $(pacman -Qtdq)"), true);
        });
    });
    clear_pkgcache_btn.connect_clicked(on_clear_pkgcache_btn_clicked);

    topbox.pack_start(&label, true, false, 1);
    button_box_f.pack_start(&update_system_btn, true, true, 2);
    button_box_f.pack_start(&reinstall_btn, true, true, 2);
    button_box_s.pack_start(&removelock_btn, true, true, 2);
    button_box_s.pack_start(&clear_pkgcache_btn, true, true, 2);
    button_box_s.pack_end(&remove_orphans_btn, true, true, 2);
    button_box_f.set_halign(gtk::Align::Fill);
    button_box_s.set_halign(gtk::Align::Fill);
    button_box_t.set_halign(gtk::Align::Fill);
    topbox.pack_end(&button_box_t, true, true, 5);
    topbox.pack_end(&button_box_s, true, true, 5);
    topbox.pack_end(&button_box_f, true, true, 5);

    topbox.set_hexpand(true);
    topbox
}

fn create_apps_section() -> Option<gtk::Box> {
    let topbox = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let box_collection = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let label = gtk::Label::new(None);
    label.set_line_wrap(true);
    label.set_justify(gtk::Justification::Center);
    label.set_text("Applications");

    // Check first btn.
    if Path::new("/sbin/vaamos-pi-bin").exists() {
        let vaamos_pi = gtk::Button::with_label("VaamOS PackageInstaller");
        vaamos_pi.connect_clicked(on_appbtn_clicked);
        box_collection.pack_start(&vaamos_pi, true, true, 2);
    }

    topbox.pack_start(&label, true, true, 5);

    box_collection.set_halign(gtk::Align::Fill);
    topbox.pack_end(&box_collection, true, true, 0);

    topbox.set_hexpand(true);
    match !box_collection.children().is_empty() {
        true => Some(topbox),
        _ => None,
    }
}

pub fn create_tweaks_page(builder: &Builder) {
    let install: gtk::Button = builder.object("tweaksBrowser").unwrap();
    install.set_visible(true);

    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let image = gtk::Image::from_icon_name(Some("go-previous"), gtk::IconSize::Button);
    let back_btn = gtk::Button::new();
    back_btn.set_image(Some(&image));
    back_btn.set_widget_name("home");

    back_btn.connect_clicked(glib::clone!(@weak builder => move |button| {
        let name = button.widget_name();
        let stack: gtk::Stack = builder.object("stack").unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    }));

    let fixes_section_box = create_fixes_section();
    let apps_section_box_opt = create_apps_section();

    let grid = gtk::Grid::new();
    grid.set_hexpand(true);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(5);
    grid.set_margin_bottom(5);
    grid.attach(&back_btn, 0, 1, 1, 1);
    let box_collection = gtk::Box::new(gtk::Orientation::Vertical, 5);

    box_collection.pack_start(&fixes_section_box, false, false, 10);

    if let Some(apps_section_box) = apps_section_box_opt {
        box_collection.pack_end(&apps_section_box, false, false, 10);
    }

    box_collection.set_valign(gtk::Align::Center);
    box_collection.set_halign(gtk::Align::Center);
    grid.attach(&box_collection, 1, 2, 5, 1);
    viewport.add(&grid);
    viewport.show_all();

    let stack: gtk::Stack = builder.object("stack").unwrap();
    let child_name = "tweaksBrowserpage";
    stack.add_named(&viewport, child_name);
}

pub fn create_appbrowser_page(builder: &Builder) {
    let install: gtk::Button = builder.object("appBrowser").unwrap();
    install.set_visible(true);

    let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
    let image = gtk::Image::from_icon_name(Some("go-previous"), gtk::IconSize::Button);
    let back_btn = gtk::Button::new();
    back_btn.set_image(Some(&image));
    back_btn.set_widget_name("home");

    back_btn.connect_clicked(glib::clone!(@weak builder => move |button| {
        let name = button.widget_name();
        let stack: gtk::Stack = builder.object("stack").unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    }));

    let grid = gtk::Grid::new();
    grid.set_hexpand(true);
    grid.set_margin_start(10);
    grid.set_margin_end(10);
    grid.set_margin_top(5);
    grid.set_margin_bottom(5);
    grid.attach(&back_btn, 0, 1, 1, 1);

    let app_browser_ref = ApplicationBrowser::default_impl().lock().unwrap();
    let app_browser_box = app_browser_ref.get_page();
    grid.attach(app_browser_box, 0, 2, 1, 1);

    // Add grid to the viewport
    // NOTE: we might eliminate that?
    viewport.add(&grid);
    viewport.show_all();

    let stack: gtk::Stack = builder.object("stack").unwrap();
    let child_name = "appBrowserpage";
    stack.add_named(&viewport, child_name);
}

fn on_update_system_btn_clicked(_: &gtk::Button) {
    let (cmd, escalate) = match utils::get_pacman_wrapper() {
        PacmanWrapper::Pak => ("pak -Syu", false),
        PacmanWrapper::Yay => ("yay -Syu", false),
        PacmanWrapper::Paru => ("paru --removemake -Syu", false),
        _ => ("pacman -Syu", true),
    };
    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let _ = utils::run_cmd_terminal(String::from(cmd), escalate);
    });
}

fn on_clear_pkgcache_btn_clicked(_: &gtk::Button) {
    let (cmd, escalate) = match utils::get_pacman_wrapper() {
        PacmanWrapper::Pak => ("pak -Sc", false),
        PacmanWrapper::Yay => ("yay -Sc", false),
        PacmanWrapper::Paru => ("paru -Sc", false),
        _ => ("pacman -Sc", true),
    };
    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let _ = utils::run_cmd_terminal(String::from(cmd), escalate);
    });
}

fn on_appbtn_clicked(button: &gtk::Button) {
    // Get button label.
    let name = button.label().unwrap();
    let (binname, is_sudo) = if name == "VaamOS PackageInstaller" {
        ("vaamos-pi-bin", true)
    } else if name == "VaamOS Kernel Manager" {
        ("vaamos-kernel-manager", false)
    } else {
        ("", false)
    };

    // Check if executable exists.
    let exit_status = Exec::cmd("which").arg(binname).join().unwrap();
    if !exit_status.success() {
        return;
    }

    let mut envs = String::new();
    for env in glib::listenv() {
        if env == "PATH" {
            envs += "PATH=/sbin:/bin:/usr/local/sbin:/usr/local/bin:/usr/bin:/usr/sbin ";
            continue;
        }
        let _ = write!(
            envs,
            "{}=\"{}\" ",
            env.to_str().unwrap(),
            glib::getenv(&env).unwrap().to_str().unwrap()
        );
    }

    // Get executable path.
    let mut exe_path = Exec::cmd("which")
        .arg(binname)
        .stdout(Redirection::Pipe)
        .capture()
        .unwrap()
        .stdout_str();
    exe_path.pop();
    let bash_cmd = format!("{} {}", &envs, &exe_path);

    // Create context channel.
    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);

    // Spawn child process in separate thread.
    std::thread::spawn(move || {
        let exit_status = if is_sudo {
            Exec::cmd("/sbin/pkexec")
                .arg("bash")
                .arg("-c")
                .arg(bash_cmd)
                .join()
                .unwrap()
        } else {
            Exec::shell(bash_cmd).join().unwrap()
        };
        tx.send(format!(
            "Exit status successfully? = {:?}",
            exit_status.success()
        ))
        .expect("Couldn't send data to channel");
    });

    rx.attach(None, move |text| {
        println!("{text}");
        glib::Continue(true)
    });
}
