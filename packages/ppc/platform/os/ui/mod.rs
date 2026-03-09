// Minimal cross-platform UI component
use dioxus::prelude::*;

pub fn hello_world() -> Element {
    rsx! {
        div {
            style: "text-align: center; padding: 2rem;",
            h1 { "Hello from Mize!" }
            p { "This UI works on Web, Desktop, and Obsidian plugin" }
            p { "Current platform: {std::env::consts::OS}" }
        }
    }
}

pub fn launch_desktop_app() {
    dioxus::launch(hello_world);
}
