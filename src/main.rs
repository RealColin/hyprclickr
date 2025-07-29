use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

use uinput::event::controller::Mouse;
use uinput::event::{Controller};

use gtk4::{gdk, prelude::*, Align, Box, ComboBoxText, CssProvider, DropDown, EventControllerKey, Expression, Frame, GestureClick, Orientation, Overlay, StringList, StyleContext, ToggleButton};
use gtk4::{Application, ApplicationWindow, Button, Label};
use glib::{clone, GString};

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
enum Activation {
    Toggle,
    Hold,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Modifier {
    Ctrl,
    Shift,
    Alt,
}

impl Modifier {
    fn to_string(&self) -> &str {
        match self {
            Self::Ctrl => "CTRL",
            Self::Shift => "SHIFT",
            Self::Alt => "ALT",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Key {
    Char(char),
    F(u8),
    Escape,
    Enter,
    Space,
    Backspace,
    Control_L,
    Control_R,
    Shift_L,
    Shift_R,
    Alt_L,
    Alt_R,
}

impl Key {
    fn to_string(&self) -> String {
        match self {
            Self::Char(c) => c.to_string(),
            Self::F(f) => format!("F{f}"),
            Self::Escape => "ESCAPE".to_string(),
            Self::Enter => "ENTER".to_string(),
            Self::Space => "SPACE".to_string(),
            Self::Backspace => "BACKSPACE".to_string(),
            Self::Control_L => "CTRL_L".to_string(),
            Self::Control_R => "CTRL_R".to_string(),
            Self::Shift_L => "SHIFT_L".to_string(),
            Self::Shift_R => "SHIFT_R".to_string(),
            Self::Alt_L => "ALT_L".to_string(),
            Self::Alt_R => "ALT_R".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Hotkey {
    modifiers: Vec<Modifier>,
    key: Key,
}

impl Hotkey {
    fn to_string(&self) -> String {
        let mut ret = String::from("");
        
        for modifier in self.modifiers.clone() {
            ret += modifier.to_string();
            ret += " + ";
        }

        ret += &self.key.to_string();

        ret
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Profile {
    name: String,
    mouse_button: MouseButton,
    click_pattern: ClickPattern,
    activation: Activation,
    hotkey: Hotkey,
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

        row.append(&name_label);
        row.append(&activate_button);

        container.append(&row);

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
            activation: Activation::Toggle,
            hotkey: Hotkey { modifiers: (vec![]), key: (Key::F(8)) },
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

    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    
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
    container.set_focusable(true);
    container.set_can_focus(true);

    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let frame = Frame::builder()
        .build();

    frame.set_width_request(400);
    frame.set_height_request(340);
    frame.set_margin_start(10);
    frame.set_margin_top(7);

    let selected_index = *app_state.selected_profile_index.borrow();
    let profiles = load_profiles();

    if let Some(profile) = selected_index.and_then(|i| profiles.get(i)) {
        let options_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();

        options_box.set_margin_start(5);
        options_box.set_margin_top(20);

        let mouse_row = Box::builder().orientation(Orientation::Horizontal).spacing(200).build();
        let mouse_label = Label::new(Some("Mouse Button"));
        let mouse_options = vec!["Left", "Right", "Middle"];
        let mouse_dropdown = DropDown::from_strings(&mouse_options);
        let mouse_selected = match profile.mouse_button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
        };
        mouse_dropdown.set_selected(mouse_selected);
        mouse_dropdown.connect_selected_notify(move |dd| {
            let selected = match dd.selected() {
                0 => MouseButton::Left,
                1 => MouseButton::Right,
                2 => MouseButton::Middle,
                _ => MouseButton::Right,
            };

            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles[index].mouse_button = selected;
                save_profiles(&profiles);
            }
        });

        mouse_row.append(&mouse_label);
        mouse_row.append(&mouse_dropdown);

        let click_row = Box::builder().orientation(Orientation::Horizontal).spacing(200).build();
        let click_label = Label::new(Some("Click Pattern"));
        let click_options = vec!["Normal", "Jitter", "Butterfly", "Drag"];
        let click_dropdown = DropDown::from_strings(&click_options);
        let click_selected = match profile.click_pattern {
            ClickPattern::Normal => 0,
            ClickPattern::Jitter => 1,
            ClickPattern::Butterfly => 2,
            ClickPattern::Drag => 3,
        };
        click_dropdown.set_selected(click_selected);
        click_dropdown.connect_selected_notify(move |dd| {
            let selected = match dd.selected() {
                0 => ClickPattern::Normal,
                1 => ClickPattern::Jitter,
                2 => ClickPattern::Butterfly,
                3 => ClickPattern::Drag,
                _ => ClickPattern::Normal
            };

            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles[index].click_pattern = selected;
                save_profiles(&profiles);
            }
        });

        click_row.append(&click_label);
        click_row.append(&click_dropdown);

        let activation_row = Box::builder().orientation(Orientation::Horizontal).spacing(200).build();
        let activation_label = Label::new(Some("Activation"));
        let activation_options = vec!["Toggle", "Hold"];
        let activation_dropdown = DropDown::from_strings(&activation_options);
        let activation_selected = match profile.activation {
            Activation::Toggle => 0,
            Activation::Hold => 1,
        };
        activation_dropdown.set_selected(activation_selected);
        activation_dropdown.connect_selected_item_notify(move |dd| {
            let selected = match dd.selected() {
                0 => Activation::Toggle,
                1 => Activation::Hold,
                _ => Activation::Toggle,
            };

            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles[index].activation = selected;
                save_profiles(&profiles);
            }
        });

        activation_row.append(&activation_label);
        activation_row.append(&activation_dropdown);

        let hotkey_row = Box::builder().orientation(Orientation::Horizontal).spacing(200).build();
        let hotkey_label = Label::new(Some("Hotkey"));
        let hotkey_button = Button::with_label(&profile.hotkey.to_string());
        let hotkey_button_clone = hotkey_button.clone();

        hotkey_button.connect_clicked(move |_| {
            let controller = EventControllerKey::new();
            let controller_clone = controller.clone();
            let button_inner = hotkey_button_clone.clone();

            controller.connect_key_released(move |_, keyval, _, state| {
                if let Some(keyname) = gdk::Key::from(keyval).name() {
                    let mut modifiers = vec![];

                    if state.contains(gdk::ModifierType::CONTROL_MASK) {
                        modifiers.push(Modifier::Ctrl);
                    }
                    if state.contains(gdk::ModifierType::SHIFT_MASK) {
                        modifiers.push(Modifier::Shift);
                    }
                    if state.contains(gdk::ModifierType::ALT_MASK) {
                        modifiers.push(Modifier::Alt);
                    }

                    match keyname.as_str() {
                        "Control_L" => modifiers.clear(),
                        "Control_R" => modifiers.clear(),
                        "Alt_L" => modifiers.clear(),
                        "Alt_R" => modifiers.clear(),
                        "Shift_L" => modifiers.clear(),
                        "Shift_R" => modifiers.clear(),
                        _ => (),
                    }

                    let okey = gtk_key_to_custom_key(keyname);

                    if let Some(key) = okey {
                        let hotkey = Hotkey {
                            modifiers,
                            key,
                        };

                        let cloned_hotkey = hotkey.clone();

                        if let Some(index) = selected_index {
                            let mut profiles = load_profiles();
                            profiles[index].hotkey = cloned_hotkey;
                            save_profiles(&profiles);

                            button_inner.set_label(&hotkey.to_string());
                        }
                    }
                }

                button_inner.remove_controller(&controller_clone);
            });

            hotkey_button_clone.add_controller(controller);
        });

        hotkey_row.append(&hotkey_label);
        hotkey_row.append(&hotkey_button);

        let delete_button = Button::with_label("Delete");
        // let profiles_clone = &app_state.profile_list_box.clone();
        let capp_state = app_state.clone();

        delete_button.connect_clicked(move |_| {
            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles.remove(index);
                save_profiles(&profiles);
                // change selection index to none
                *capp_state.selected_profile_index.borrow_mut() = None;
                // re-render profiles list
                build_profiles_box(&capp_state);
                // re-build settings box
                build_settings_box(&capp_state);
            }
        });
        
        options_box.append(&mouse_row);
        options_box.append(&click_row);
        options_box.append(&activation_row);
        options_box.append(&hotkey_row);
        options_box.append(&delete_button);
        frame.set_child(Some(&options_box));
    } else {
         let label = Label::new(Some("Nothing Selected."));
         frame.set_child(Some(&label));
    }
    
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

fn gtk_key_to_custom_key(keyname: GString) -> Option<Key> {
    match keyname.as_str() {
        "Escape" => Some(Key::Escape),
        "Return" => Some(Key::Enter),
        "space" => Some(Key::Space),
        "BackSpace" => Some(Key::Backspace),
        "Control_L" => Some(Key::Control_L),
        "Control_R" => Some(Key::Control_R),
        "Shift_L" => Some(Key::Shift_L),
        "Shift_R" => Some(Key::Shift_R),
        "Alt_L" => Some(Key::Alt_L),
        "Alt_R" => Some(Key::Alt_R),
        f if f.starts_with('F') => {
            let num_str = &f[1..];
            if let Ok(num) = num_str.parse::<u8>() {
                Some(Key::F(num))
            } else {
                None
            }
        }
        c if c.len() == 1 => c.chars().next().map(Key::Char),
        _ => None,
    }

    // if let Some(name) = key.name() {
    //     match name.as_str() {
    //         "Escape" => Some(Key::Escape),
    //         "Return" => Some(Key::Enter),
    //         "Tab" => Some(Key::Tab),
    //         "space" => Some(Key::Space),
    //         "BackSpace" => Some(Key::Backspace),
    //         f if f.starts_with('F') => {
    //             let num_str = &f[1..];
    //             if let Ok(num) = num_str.parse::<u8>() {
    //                 Some(Key::F(num))
    //             } else {
    //                 None
    //             }
    //         }
    //         c if c.len() == 1 => c.chars().next().map(Key::Char),
    //         _ => None,
    //     }
    // } else {
    //     None
    // }
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