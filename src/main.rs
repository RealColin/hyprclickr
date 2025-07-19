use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

use uinput::event::controller::Mouse;
use uinput::event::{Controller};

use gtk4::{gdk, prelude::*, Align, Box, CssProvider, Frame, GestureClick, Orientation, Overlay, StyleContext, ToggleButton};
use gtk4::{Application, ApplicationWindow, Button, Label};
use glib::clone;

use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

#[derive(Serialize, Deserialize, Debug, Clone)]

enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ClickPattern {
    Normal,
    Jitter,
    Butterfly,
    Drag,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Profile {
    name: String,
    mouse_button: MouseButton,
    click_pattern: ClickPattern,
    active: bool,
}

struct AppState {
    selected_profile_index: Rc<RefCell<Option<usize>>>,
    profile_list_box: Box,
    settings_box: Box,
}

fn main() {
    let app = Application::builder()
        .application_id("com.colin.hyprclickr")
        .build();

    

    // let test = vec![
    //     Profile {
    //         name: "Joe".to_string(),
    //         mouse_button: MouseButton::Left,
    //         click_pattern: ClickPattern::Butterfly,
    //     },
    //     Profile {
    //         name: "Donald".to_string(),
    //         mouse_button: MouseButton::Right,
    //         click_pattern: ClickPattern::Drag,
    //     },
    // ];

    // save_profiles(&test);

    app.connect_activate(build_ui);

    app.run();

    // let mut device = uinput::default().unwrap()
    //     .name("fakemouse").unwrap()
    //     .event(Controller::Mouse(Mouse::Left)).unwrap()
    //     .event(Controller::Mouse(Mouse::Right)).unwrap()
    //     .event(keyboard::Key::D).unwrap()
    //     .create().unwrap();
        
    // sleep(Duration::from_millis(3000));


    // for _i in 0..100 {
    //     click(&mut device, 10, 40);
    // }
}

fn build_ui(app: &Application) {
    let provider = CssProvider::new();
    provider.load_from_data("
        .selected-profile {
            background-color: #555;
            border-radius: 4px;
        }
    ");

    StyleContext::add_provider_for_display(
        &gdk::Display::default().unwrap(),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let profile_list_box = Box::new(Orientation::Vertical, 10);
    let settings_box = Box::new(Orientation::Vertical, 10);

    let app_state = Rc::new(AppState {
        selected_profile_index: Rc::new(RefCell::new(None)),
        profile_list_box: profile_list_box.clone(),
        settings_box: settings_box.clone(),
    });

    build_profiles_box(&app_state);
    build_settings_box(&app_state);


    let inner = Box::new(Orientation::Horizontal, 10);
    inner.append(&app_state.profile_list_box);
    inner.append(&app_state.settings_box);
    // inner.append(&build_profiles_box(&app_state));
    // inner.append(&build_settings_box(&app_state));

    let title_label = Label::new(Some("Hyprclickr"));
    let outer = Box::new(Orientation::Vertical, 10);
    outer.set_margin_top(5);
    outer.set_margin_bottom(5);
    outer.set_margin_start(5);
    outer.set_margin_end(5);
    outer.append(&title_label);
    outer.append(&inner);

    // let fixed = Fixed::new();
    // fixed.put(&profiles_label, 10.0, 20.0);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Hyprclickr")
        .default_width(600)
        .default_height(400)
        .resizable(false)
        .child(&outer)
        .build();

    window.present();
}

fn render_profiles_list(container: &Box, profiles: &[Profile], app_state: Rc<AppState>) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    for (i, profile) in profiles.iter().enumerate() {
        let row = Box::new(Orientation::Horizontal, 2);

        let name_label = Label::new(Some(&profile.name));
        name_label.set_hexpand(true);
        name_label.set_halign(Align::Start);

        // ✅ Selection only when clicking the label
        let gesture = GestureClick::new();
        gesture.connect_pressed(clone!(@strong app_state, @strong container => move |_, _, _, _| {
            *app_state.selected_profile_index.borrow_mut() = Some(i);
            render_profiles_list(&container, &load_profiles(), app_state.clone());
            build_settings_box(&app_state);
        }));
        name_label.add_controller(gesture);

        let is_selected = app_state.selected_profile_index.borrow().map_or(false, |selected| selected == i);

        if is_selected {
            row.set_css_classes(&["selected-profile"]);
        } else {
            row.remove_css_class("selected-profile");
        }

        let activate_label = Label::new(Some("A"));
        activate_label.set_margin_top(0);
        activate_label.set_margin_bottom(0);
        activate_label.set_margin_start(0);
        activate_label.set_margin_end(0);
        
        let activate_button = ToggleButton::new();
        activate_button.set_child(Some(&activate_label));
        activate_button.set_size_request(5, 5);
        activate_button.set_hexpand(false);
        activate_button.set_vexpand(false);
        activate_button.set_has_frame(false);
        activate_button.set_halign(Align::End);
        activate_button.set_active(profile.active);

        let delete_label = Label::new(Some("D"));
        delete_label.set_margin_top(0);
        delete_label.set_margin_bottom(0);
        delete_label.set_margin_start(0);
        delete_label.set_margin_end(0);
        
        let delete_button = Button::new();
        delete_button.set_child(Some(&delete_label));
        delete_button.set_size_request(5, 5);
        delete_button.set_hexpand(false);
        delete_button.set_vexpand(false);
        delete_button.set_has_frame(false);
        delete_button.set_halign(Align::End);


        row.append(&name_label);
        row.append(&activate_button);
        row.append(&delete_button);

        container.append(&row);

        let apst = app_state.clone();
        let container_clone = container.clone();
        delete_button.connect_clicked(move |_| {
            let mut profiles = load_profiles().to_vec();
            profiles.remove(i);
            save_profiles(&profiles);
            render_profiles_list(&container_clone, &profiles, apst.clone());
            build_settings_box(&apst.clone());
        });

        let apst = app_state.clone();
        let container_clone = container.clone();
        activate_button.connect_toggled(move |btn| {
            let mut profiles = load_profiles();
            profiles[i].active = btn.is_active();
            save_profiles(&profiles);
            render_profiles_list(&container_clone, &profiles, apst.clone());
        });
    }

    let new_button = Button::with_label("+ New Profile");
    container.append(&new_button);

    let container_clone = container.clone();
    new_button.connect_clicked(move |_| {
        let mut profiles = load_profiles();
        profiles.push(Profile {
            name: format!("Profile {}", profiles.len() + 1),
            mouse_button: MouseButton::Left,
            click_pattern: ClickPattern::Normal,
            active: false,
        });
        save_profiles(&profiles);
        *app_state.selected_profile_index.borrow_mut() = Some(profiles.len() - 1);
        render_profiles_list(&container_clone, &profiles, app_state.clone());
        build_settings_box(&app_state.clone());
    });

}

fn build_profiles_box(app_state: &Rc<AppState>){
    let container = &app_state.profile_list_box;
    
    let label = Label::new(Some("Profiles"));
    label.set_valign(Align::Start);
    label.set_halign(Align::Start);
    label.set_margin_top(0);
    label.set_margin_start(8);

    let list = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .build();

    list.set_margin_top(10);

    let list_clone = list.clone();

    let frame = Frame::builder()
        .child(&list)
        .build();

    frame.set_width_request(150);
    frame.set_height_request(340);
    frame.set_margin_start(10);
    frame.set_margin_top(7);

    let overlay = Overlay::new();
    overlay.set_child(Some(&frame));
    overlay.add_overlay(&label);

    container.append(&overlay);

    let profiles = load_profiles();
    render_profiles_list(&list_clone, &profiles, app_state.clone());

}

fn build_settings_box(app_state: &Rc<AppState>){
    let container = &app_state.settings_box;

    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let options_box = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    options_box.set_margin_start(5);
    options_box.set_margin_top(20);

    let mouse_button = Label::new(Some("Mouse Button"));
    mouse_button.set_valign(Align::Start);
    mouse_button.set_halign(Align::Start);

    let click_pattern = Label::new(Some("Click Pattern"));
    click_pattern.set_valign(Align::Start);
    click_pattern.set_halign(Align::Start);

    let activation = Label::new(Some("Activation"));
    activation.set_valign(Align::Start);
    activation.set_halign(Align::Start);

    let hotkey = Label::new(Some("Hotkey"));
    hotkey.set_valign(Align::Start);
    hotkey.set_halign(Align::Start);

    let joe = Label::new(Some(""));
    joe.set_valign(Align::Center);
    joe.set_halign(Align::Center);

    let selected_index = *app_state.selected_profile_index.borrow();

    if let Some(index) = selected_index {
        let profiles = load_profiles();
        if let Some(profile) = profiles.get(index) {
            joe.set_text(&format!("Editing: {}", profile.name));
        }
    } else {
        joe.set_text("Nothing selected.");
    }

    options_box.append(&mouse_button);
    options_box.append(&click_pattern);
    options_box.append(&activation);
    options_box.append(&hotkey);
    options_box.append(&joe);

    let frame = Frame::builder()
        .child(&options_box)
        .build();

    frame.set_width_request(400);
    frame.set_height_request(340);
    frame.set_margin_start(10);
    frame.set_margin_top(7);

    let settings_label = Label::new(Some("Settings"));
    settings_label.set_valign(Align::Start);
    settings_label.set_halign(Align::Start);
    settings_label.set_margin_top(0);
    settings_label.set_margin_start(8);

    let overlay = Overlay::new();
    overlay.set_child(Some(&frame));
    overlay.add_overlay(&settings_label);

    container.append(&overlay);
}

fn get_profiles_path() -> PathBuf {
    let xdg_dirs = BaseDirectories::with_prefix("hyprclickr");
    xdg_dirs.place_config_file("profiles.json").expect("Cannot create config path")
}

fn load_profiles() -> Vec<Profile> {
    let path = get_profiles_path();

    if path.exists() {
        let data = fs::read_to_string(&path).expect("Failed to read profile file");
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        vec![]
    }
}

fn save_profiles(profiles: &[Profile]) {
    let path = get_profiles_path();
    let data = serde_json::to_string_pretty(profiles).expect("Failed to serialize profiles");
    fs::write(path, data).expect("Failed to write profile file");
}

fn click(dev: &mut uinput::Device, release_delay: u64, press_delay: u64) {
    dev.press(&Controller::Mouse(Mouse::Left)).unwrap();
    dev.synchronize().unwrap();
    sleep(Duration::from_millis(release_delay));
    dev.release(&Controller::Mouse(Mouse::Left)).unwrap();
    dev.synchronize().unwrap();
    sleep(Duration::from_millis(press_delay));
}