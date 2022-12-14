use dioxus::prelude::ScopeState;
use std::sync::Once;
use wasm_bindgen::{prelude::Closure, JsCast};

#[derive(Debug, Clone)]
pub enum PreferredColorScheme {
    Light,
    Dark,
    Err(String),
}

static INIT: Once = Once::new();

/// Retrieves (as well as listens for changes) to the user's preferred color scheme (dark or light) so your application can adapt accordingly.
pub fn use_preferred_color_scheme(cx: &ScopeState) -> PreferredColorScheme {
    // This code is kinda messy..
    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            return PreferredColorScheme::Err(
                "not running in wasm context: window doesn't exist".to_string(),
            )
        }
    };

    let media_query_list = match window.match_media("(prefers-color-scheme: dark)") {
        Ok(opt) => match opt {
            Some(m) => m,
            None => {
                return PreferredColorScheme::Err(
                    "failed to determine preferred scheme".to_string(),
                )
            }
        },
        Err(e) => {
            return PreferredColorScheme::Err(e.as_string().unwrap_or(
                "failed to determine preferred scheme and couldn't retrieve error".to_string(),
            ))
        }
    };

    let update_callback = cx.schedule_update();

    // Create closure that listens to the event of matchMedia and calls write to scheme
    INIT.call_once(|| {
        let listener = Closure::wrap(Box::new(move || {
            update_callback();
        }) as Box<dyn Fn()>);

        // Create a reference to the closure to pass to JavaScript.
        let cb = listener.as_ref().clone();

        // Prevent the closure from being dropped.
        // This normally isn't good practice, however the idea is that this callback should live forever.
        listener.forget();

        media_query_list.set_onchange(Some(cb.unchecked_ref()));
    });

    determine_scheme(media_query_list.matches())
}

fn determine_scheme(value: bool) -> PreferredColorScheme {
    match value {
        true => PreferredColorScheme::Dark,
        false => PreferredColorScheme::Light,
    }
}
