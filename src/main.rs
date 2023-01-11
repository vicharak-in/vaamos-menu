#![feature(const_str_from_utf8)]
#![feature(string_remove_matches)]
#![allow(non_upper_case_globals)]

mod alpm_helper;
mod application_browser;
mod config;
mod data_types;
mod pages;
mod utils;

use config::{APP_ID, GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR, VERSION};
use data_types::*;
use gettextrs::LocaleCategory;
use gtk::{gio, glib, Builder, Window};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use utils::*;

use gio::prelude::*;
use gtk::prelude::*;

use gdk_pixbuf::Pixbuf;

use serde_json::json;
use std::{fs, str};

static mut g_save_json: Lazy<Mutex<serde_json::Value>> = Lazy::new(|| Mutex::new(json!(null)));
static mut g_menu_window: Option<Arc<MenuWindow>> = None;

fn show_about_dialog() {
    let main_window: Window;
    unsafe {
        main_window = g_menu_window.clone().unwrap().window.clone();
    }
    let logo_path = format!("/usr/share/icons/hicolor/scalable/apps/{APP_ID}.svg");
    let mut logo = Pixbuf::from_file(logo_path).unwrap();
    // scale logo size
    logo = logo
        .scale_simple(128, 128, gdk_pixbuf::InterpType::Bilinear)
        .unwrap();

    let dialog = gtk::AboutDialog::builder()
        .transient_for(&main_window)
        .modal(true)
        .program_name(&gettextrs::gettext("VaamOS Menu"))
        .comments(&gettextrs::gettext("Welcome to VaamOS Menu"))
        .version(VERSION)
        .logo(&logo)
        .authors(vec!["Utsav Balar".into()])
        // Translators: Replace "translator-credits" with your names. Put a comma between.
        .translator_credits(&gettextrs::gettext("translator-credits"))
        .copyright("2023 Vicharak")
        .license_type(gtk::License::Gpl30)
        .website("https://github.com/vicharak-in/vaamos-updater")
        .website_label("GitHub")
        .build();

    dialog.run();
    dialog.hide();
}

fn main() {
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain.");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain.");

    glib::set_application_name("VaamOSMenu");

    gtk::init().expect("Unable to start GTK3.");

    let application = gtk::Application::new(
        Some(APP_ID),       // Application id
        Default::default(), // Using default flags
    );

    application.connect_activate(|application| {
        build_ui(application);
    });

    // Run the application and start the event loop
    application.run();
}

fn build_ui(application: &gtk::Application) {
    let data = fs::read_to_string(format!("{PKGDATADIR}/data/preferences.json"))
        .expect("Unable to read file");
    let preferences: serde_json::Value = serde_json::from_str(&data).expect("Unable to parse");

    // Import Css
    let provider = gtk::CssProvider::new();
    provider
        .load_from_path(preferences["style_path"].as_str().unwrap())
        .expect("Failed to load CSS");
    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().expect("Error initializing gtk css provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Init window
    let builder: Builder = Builder::from_file(preferences["ui_path"].as_str().unwrap());
    builder.connect_signals(|_builder, handler_name| {
        match handler_name {
            // handler_name as defined in the glade file => handler function as defined above
            "on_start_menu" => Box::new(on_start_menu),
            "on_action_clicked" => Box::new(on_action_clicked),
            "on_btn_clicked" => Box::new(on_btn_clicked),
            "on_link_clicked" => Box::new(on_link_clicked),
            "on_link1_clicked" => Box::new(on_link1_clicked),
            "on_delete_window" => Box::new(|_| Some(false.to_value())),
            _ => Box::new(|_| None),
        }
    });

    let main_window: Window = builder
        .object("window")
        .expect("Could not get the object window");
    main_window.set_application(Some(application));

    unsafe {
        g_menu_window = Some(Arc::new(MenuWindow {
            window: main_window.clone(),
            builder: builder.clone(),
            preferences: preferences.clone(),
        }));
    };

    // Load images
    let logo_path = format!(
        "{}/{}.svg",
        preferences["logo_path"].as_str().unwrap(),
        APP_ID
    );
    if Path::new(&logo_path).exists() {
        let logo = Pixbuf::from_file(logo_path).unwrap();
        main_window.set_icon(Some(&logo));
    }

    let homepage_grid: gtk::Grid = builder.object("homepage").unwrap();
    for widget in homepage_grid.children() {
        let casted_widget = widget.downcast::<gtk::Button>();
        if casted_widget.is_err() {
            continue;
        }

        let btn = casted_widget.unwrap();
        if btn.image_position() != gtk::PositionType::Right {
            continue;
        }

        let image_path = format!("{PKGDATADIR}/data/img/external-link.png");
        let image = gtk::Image::new();
        image.set_from_file(Some(&image_path));
        image.set_margin_start(2);
        btn.set_image(Some(&image));
    }

    // Create pages
    let pages = format!("{PKGDATADIR}/data/pages/en");

    for page in fs::read_dir(pages).unwrap() {
        let scrolled_window =
            gtk::ScrolledWindow::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);

        let viewport = gtk::Viewport::new(gtk::Adjustment::NONE, gtk::Adjustment::NONE);
        viewport.set_border_width(10);

        let label = gtk::Label::new(None);
        label.set_line_wrap(true);
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
        grid.attach(&back_btn, 0, 1, 1, 1);
        grid.attach(&label, 1, 2, 1, 1);
        viewport.add(&grid);
        scrolled_window.add(&viewport);
        scrolled_window.show_all();

        let stack: gtk::Stack = builder.object("stack").unwrap();
        let child_name = format!(
            "{}page",
            page.unwrap().path().file_name().unwrap().to_str().unwrap()
        );
        stack.add_named(&scrolled_window, &child_name);
    }

    // Set autostart switcher state
    let autostart = Path::new(&fix_path(preferences["autostart_path"].as_str().unwrap())).exists();
    let autostart_switch: gtk::Switch = match builder.object("autostart") {
        Some(switch) => switch,
        None => gtk::Switch::new(),
    };
    autostart_switch.set_active(autostart);

    // Live systems
    let installlabel: gtk::Label = builder.object("installlabel").unwrap();
    installlabel.set_visible(false);

    let install: gtk::Button = builder.object("install").unwrap();
    install.set_visible(false);
    pages::create_appbrowser_page(&builder);
    pages::create_tweaks_page(&builder);

    // Show the UI
    main_window.show();
}

/// Returns the best locale, based on user's preferences.
/// Sets locale of ui and pages.
fn set_menu_ui(use_locale: &str) {
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain.");
    glib::setenv("LANGUAGE", use_locale, true).expect("Unable to change env variable.");

    unsafe {
        g_save_json.lock().unwrap()["locale"] = json!(use_locale);
    }

    // Real-time locale changing
    let elts: HashMap<String, serde_json::Value> = serde_json::from_str(&serde_json::to_string(&json!({
        "label": ["autostartlabel", "github", "software", "firstcategory", "forum", "install", "installlabel", "involved", "readme", "aboutus", "secondcategory", "thirdcategory", "welcomelabel", "welcometitle", "wiki"],
        "tooltip_text": ["about", "github", "software", "forum", "wiki"],
    })).unwrap()).unwrap();

    let mut default_texts = json!(null);
    for method in elts.iter() {
        if default_texts.get(method.0).is_none() {
            default_texts[method.0] = json![null];
        }

        for elt in elts[method.0].as_array().unwrap() {
            let elt_value = elt.as_str().unwrap();
            unsafe {
                let item: gtk::Widget = g_menu_window
                    .clone()
                    .unwrap()
                    .builder
                    .object(elt_value)
                    .unwrap();
                if default_texts[method.0].get(elt_value).is_none() {
                    let item_buf = item.property::<String>(method.0.as_str());
                    default_texts[method.0][elt_value] = json!(item_buf);
                }
                if method.0 == "tooltip_text" {
                    item.set_property(
                        method.0,
                        &gettextrs::gettext(default_texts[method.0][elt_value].as_str().unwrap()),
                    );
                }
            }
        }
    }

    unsafe {
        // Change content of pages
        let pages = format!("{PKGDATADIR}/data/pages/en");
        for page in fs::read_dir(pages).unwrap() {
            let stack: gtk::Stack = g_menu_window
                .clone()
                .unwrap()
                .builder
                .object("stack")
                .unwrap();
            let child = stack.child_by_name(&format!(
                "{}page",
                page.as_ref()
                    .unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            ));
            if child.is_none() {
                eprintln!("child not found");
                continue;
            }
            let first_child = &child
                .unwrap()
                .downcast::<gtk::Container>()
                .unwrap()
                .children();
            let second_child = &first_child[0]
                .clone()
                .downcast::<gtk::Container>()
                .unwrap()
                .children();
            let third_child = &second_child[0]
                .clone()
                .downcast::<gtk::Container>()
                .unwrap()
                .children();

            let label = &third_child[0].clone().downcast::<gtk::Label>().unwrap();
            label.set_markup(
                get_page(page.unwrap().path().file_name().unwrap().to_str().unwrap()).as_str(),
            );
        }
    }
}

fn set_autostart(autostart: bool) {
    let autostart_path: String;
    let desktop_path: String;
    unsafe {
        autostart_path = fix_path(
            g_menu_window.clone().unwrap().preferences["autostart_path"]
                .as_str()
                .unwrap(),
        );
        desktop_path = g_menu_window.clone().unwrap().preferences["desktop_path"]
            .as_str()
            .unwrap()
            .to_string();
    }
    let config_dir = Path::new(&autostart_path).parent().unwrap();
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).unwrap();
    }
    if autostart && !check_regular_file(autostart_path.as_str()) {
        std::os::unix::fs::symlink(desktop_path, autostart_path).unwrap();
    } else if !autostart && check_regular_file(autostart_path.as_str()) {
        std::fs::remove_file(autostart_path).unwrap();
    }
}

#[inline]
fn get_page(name: &str) -> String {
    let mut filename = format!("{PKGDATADIR}/data/pages/en/{name}");
    if !check_regular_file(filename.as_str()) {
        filename = format!("{PKGDATADIR}/data/pages/en/{name}");
    }

    fs::read_to_string(filename).unwrap()
}

/// Handlers
fn on_start_menu(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::ComboBox>().unwrap();
    let active_id = &widget.active_id().expect("active_id read failed!");
    set_menu_ui(active_id);

    None
}

fn on_action_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Widget>().unwrap();
    return match widget.widget_name().as_str() {
        "install" => None,
        "autostart" => {
            let action = widget.downcast::<gtk::Switch>().unwrap();
            set_autostart(action.is_active());
            None
        }
        _ => {
            show_about_dialog();
            None
        }
    };
}

fn on_btn_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Button>().unwrap();
    let name = widget.widget_name();

    unsafe {
        let stack: gtk::Stack = g_menu_window
            .clone()
            .unwrap()
            .builder
            .object("stack")
            .unwrap();
        stack.set_visible_child_name(&format!("{name}page"));
    };

    None
}

fn on_link_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Widget>().unwrap();
    let name = widget.widget_name();

    unsafe {
        let preferences = &g_menu_window.clone().unwrap().preferences["urls"];

        let uri = preferences[name.as_str()].as_str().unwrap();
        let _ = gtk::show_uri_on_window(gtk::Window::NONE, uri, 0);
    }

    None
}

fn on_link1_clicked(param: &[glib::Value]) -> Option<glib::Value> {
    let widget = param[0].get::<gtk::Widget>().unwrap();
    let name = widget.widget_name();

    unsafe {
        let preferences = &g_menu_window.clone().unwrap().preferences["urls"];

        let uri = preferences[name.as_str()].as_str().unwrap();
        let _ = gtk::show_uri_on_window(gtk::Window::NONE, uri, 0);
    }

    Some(false.to_value())
}
