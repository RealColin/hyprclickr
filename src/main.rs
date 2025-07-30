use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::thread::sleep;

use uinput::event::controller::{self, Mouse};
use uinput::event::{Controller};

use gtk4::{gdk, prelude::*, Align, Box, CssProvider, DropDown, EventControllerKey, Frame, GestureClick, Orientation, Overlay, Scale, StyleContext, ToggleButton};
use gtk4::{Application, ApplicationWindow, Button, Label};
use glib::{GString};

use serde::{Deserialize, Serialize};
use xdg::BaseDirectories;

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MouseButton {
    Left,
    Right,
    Middle,
}

impl MouseButton {
    fn to_int(&self) -> u32 {
        match self {
            Self::Left => 0,
            Self::Right => 1,
            Self::Middle => 2,
        }
    }

    fn from_int(int: u32) -> MouseButton {
        match int {
            0 => Self::Left,
            1 => Self::Right,
            2 => Self::Middle,
            _ => Self::Left
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum ClickPattern {
    Normal,
    Jitter,
    Butterfly,
    Drag,
}

impl ClickPattern {
    fn to_int(&self) -> u32 {
        match self {
            Self::Normal => 0,
            Self::Jitter => 1,
            Self::Butterfly => 2,
            Self::Drag => 3,
        }
    }

    fn from_int(int: u32) -> ClickPattern {
        match int {
            0 => Self::Normal,
            1 => Self::Jitter,
            2 => Self::Butterfly,
            3 => Self::Drag,
            _ => Self::Normal,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Activation {
    Toggle,
    Hold,
}

impl Activation {
    fn to_int(&self) -> u32 {
        match self {
            Self::Toggle => 0,
            Self::Hold => 1,
        }
    }

    fn from_int(int: u32) -> Activation {
        match int {
            0 => Self::Toggle,
            1 => Self::Hold,
            _ => Self::Toggle,
        }
    }
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
    ControlL,
    ControlR,
    ShiftL,
    ShiftR,
    AltL,
    AltR,
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
            Self::ControlL => "CTRL_L".to_string(),
            Self::ControlR => "CTRL_R".to_string(),
            Self::ShiftL => "SHIFT_L".to_string(),
            Self::ShiftR => "SHIFT_R".to_string(),
            Self::AltL => "ALT_L".to_string(),
            Self::AltR => "ALT_R".to_string(),
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
    cps: u8,
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
        // gesture.connect_pressed(clone!(@strong app_state, @strong container => move |_, _, _, _| {
        //     *app_state.selected_profile_index.borrow_mut() = Some(i);
        //     render_profiles_list(&container, &load_profiles(), app_state.clone());
        //     build_settings_box(&app_state);
        // }));
        let capp_state = app_state.clone();
        let con_clone = container.clone();

        gesture.connect_pressed(move |_, _, _, _| {
            *capp_state.selected_profile_index.borrow_mut() = Some(i);
            render_profiles_list(&con_clone, &load_profiles(), capp_state.clone());
            build_settings_box(&capp_state);
        });
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
            cps: 15,
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

fn create_dropdown_row<T, F>(
    label_text: &str, 
    options: &[&str], 
    selected_index: u32,
    spacing: i32, 
    on_select: F,
) -> Box
where
    T: 'static,
    F: Fn(u32) + 'static,
{
    let row = Box::builder().orientation(Orientation::Horizontal).spacing(200).build();
    let label = Label::new(Some(&label_text));
    let dropdown = DropDown::from_strings(options);
    dropdown.set_selected(selected_index);

    dropdown.connect_selected_notify(move |dd| {
        let selected = dd.selected();
        on_select(selected);
    });

    row.append(&label);
    row.append(&dropdown);
    row
}

fn create_hotkey_row(curr_key: String, index: usize, spacing: i32) -> Box{
    let row = Box::builder().orientation(Orientation::Horizontal).spacing(spacing).build();
    let label = Label::new(Some("Hotkey"));
    let button = Button::with_label(&curr_key);
    let button_cloned = button.clone();


    button.connect_clicked(move |_| {
        let controller = EventControllerKey::new();
        let controller_clone = controller.clone();
        let button_inner = button_cloned.clone();

        controller.connect_key_released(move |_, keyval, _, state| {
            if let Some(name) = keyval.name() {
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

                match name.as_str() {
                    "Control_L" => modifiers.clear(),
                    "Control_R" => modifiers.clear(),
                    "Alt_L" => modifiers.clear(),
                    "Alt_R" => modifiers.clear(),
                    "Shift_L" => modifiers.clear(),
                    "Shift_R" => modifiers.clear(),
                    _ => (),
                }

                let key = gtk_key_to_custom_key(name);

                let hotkey = Hotkey {
                    modifiers,
                    key
                };

                let mut tmp = load_profiles();
                tmp[index].hotkey = hotkey.clone();
                save_profiles(&tmp);
                button_inner.set_label(&hotkey.to_string());
            }

            button_inner.remove_controller(&controller_clone);
        });

        button_cloned.add_controller(controller);
    });

    row.append(&label);
    row.append(&button);
    row
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

    if let Some(index) = selected_index {
        let profile = &profiles[index];

        let options_box = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build();

        options_box.set_margin_start(5);
        options_box.set_margin_top(20);

        let mouse_row = create_dropdown_row::<MouseButton, _>(
            "Mouse Button",
            &["Left", "Right", "Middle"],
            profile.mouse_button.to_int(),
            200,
            move |selected| {
                let selected = MouseButton::from_int(selected);
                let mut tmp = load_profiles();
                tmp[index].mouse_button = selected;
                save_profiles(&tmp);
            }
        );

        let click_row = create_dropdown_row::<ClickPattern, _>(
            "Click Pattern",
            &["Normal", "Jitter", "Butterfly", "Drag"],
            profile.click_pattern.to_int(),
            200,
            move |selected| {
                let selected = ClickPattern::from_int(selected);
                let mut tmp = load_profiles();
                tmp[index].click_pattern = selected;
                save_profiles(&tmp);
            }
        );

        let activation_row = create_dropdown_row::<ClickPattern, _>(
            "Activation",
            &["Toggle", "Hold"],
            profile.activation.to_int(),
            200,
            move |selected| {
                let selected = Activation::from_int(selected);
                let mut tmp = load_profiles();
                tmp[index].activation = selected;
                save_profiles(&tmp);
            }
        );

        let hotkey_row = create_hotkey_row(profile.hotkey.to_string(), index, 200);

        let cps_row = Box::builder().orientation(Orientation::Horizontal).spacing(100).build();
        let cps_label = Label::new(Some("CPS"));
        let cps_slider = Scale::with_range(Orientation::Horizontal, 0.0, 50.0, 1.0);
        cps_slider.set_value(profile.cps as f64);
        cps_slider.set_hexpand(true);
        cps_slider.set_draw_value(true);

        cps_slider.connect_value_changed(move |val| {
            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles[index].cps = val.value() as u8;
                save_profiles(&profiles);
            }
        });
    
        cps_row.append(&cps_label);
        cps_row.append(&cps_slider);

        let delete_button = Button::with_label("Delete");
        let capp_state = app_state.clone();

        delete_button.connect_clicked(move |_| {
            if let Some(index) = selected_index {
                let mut profiles = load_profiles();
                profiles.remove(index);
                save_profiles(&profiles);
                *capp_state.selected_profile_index.borrow_mut() = None;
                build_profiles_box(&capp_state);
                build_settings_box(&capp_state);
            }
        });
        
        options_box.append(&mouse_row);
        options_box.append(&click_row);
        options_box.append(&activation_row);
        options_box.append(&hotkey_row);
        options_box.append(&cps_row);
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

// fn gtk_key_to_custom_key(keyname: GString) -> Option<Key> {
//     match keyname.as_str() {
//         "Escape" => Some(Key::Escape),
//         "Return" => Some(Key::Enter),
//         "space" => Some(Key::Space),
//         "BackSpace" => Some(Key::Backspace),
//         "Control_L" => Some(Key::ControlL),
//         "Control_R" => Some(Key::ControlR),
//         "Shift_L" => Some(Key::ShiftL),
//         "Shift_R" => Some(Key::ShiftR),
//         "Alt_L" => Some(Key::AltL),
//         "Alt_R" => Some(Key::AltR),
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
// }

fn gtk_key_to_custom_key(keyname: GString) -> Key {
    match keyname.as_str() {
        "Escape" => Key::Escape,
        "Return" => Key::Enter,
        "space" => Key::Space,
        "BackSpace" => Key::Backspace,
        "Control_L" => Key::ControlL,
        "Control_R" => Key::ControlR,
        "Shift_L" => Key::ShiftL,
        "Shift_R" => Key::ShiftR,
        "Alt_L" => Key::AltL,
        "Alt_R" => Key::AltR,
        f if f.starts_with('F') => {
            let num_str = &f[1..];
            if let Ok(num) = num_str.parse::<u8>() {
                Key::F(num)
            } else {
                Key::Escape
            }
        }
        c if c.len() == 1 => c.chars().next().map(Key::Char).expect("Weird Crash"),
        _ => Key::Escape,
    }
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