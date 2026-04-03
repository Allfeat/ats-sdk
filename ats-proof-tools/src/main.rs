mod app;
mod creator;
mod form;
mod proof;
mod verify;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(app::App);
}
