use ats_sdk::{Creator, Role};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

/// Reactive state for a single creator in the form.
#[derive(Clone)]
pub struct CreatorSignals {
    pub id: u64,
    pub full_name: RwSignal<String>,
    pub email: RwSignal<String>,
    pub author: RwSignal<bool>,
    pub composer: RwSignal<bool>,
    pub arranger: RwSignal<bool>,
    pub adapter: RwSignal<bool>,
    pub ipi: RwSignal<String>,
    pub isni: RwSignal<String>,
}

impl PartialEq for CreatorSignals {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl CreatorSignals {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            full_name: RwSignal::new(String::new()),
            email: RwSignal::new(String::new()),
            author: RwSignal::new(false),
            composer: RwSignal::new(false),
            arranger: RwSignal::new(false),
            adapter: RwSignal::new(false),
            ipi: RwSignal::new(String::new()),
            isni: RwSignal::new(String::new()),
        }
    }

    pub fn to_creator(&self) -> Creator {
        let mut roles = Vec::new();
        if self.author.get_untracked() {
            roles.push(Role::Author);
        }
        if self.composer.get_untracked() {
            roles.push(Role::Composer);
        }
        if self.arranger.get_untracked() {
            roles.push(Role::Arranger);
        }
        if self.adapter.get_untracked() {
            roles.push(Role::Adapter);
        }
        Creator {
            full_name: self.full_name.get_untracked(),
            email: self.email.get_untracked(),
            roles,
            ipi: {
                let v = self.ipi.get_untracked();
                if v.is_empty() { None } else { Some(v) }
            },
            isni: {
                let v = self.isni.get_untracked();
                if v.is_empty() { None } else { Some(v) }
            },
        }
    }
}

/// Full data form with title, creators list, and media file input.
/// Shared between the Generate and Verify pages.
#[component]
pub fn DataForm(
    title: RwSignal<String>,
    creators: RwSignal<Vec<CreatorSignals>>,
    next_id: RwSignal<u64>,
    media_bytes: RwSignal<Option<Vec<u8>>>,
    media_name: RwSignal<String>,
) -> impl IntoView {
    let add_creator = move |_| {
        let id = next_id.get_untracked();
        next_id.set(id + 1);
        creators.update(|list| list.push(CreatorSignals::new(id)));
    };

    let on_file_change = move |ev: leptos::ev::Event| {
        let input: web_sys::HtmlInputElement = ev.target().unwrap().unchecked_into();
        if let Some(files) = input.files() {
            if let Some(file) = files.get(0) {
                media_name.set(file.name());
                wasm_bindgen_futures::spawn_local(async move {
                    let buf = wasm_bindgen_futures::JsFuture::from(file.array_buffer())
                        .await
                        .unwrap();
                    let arr = js_sys::Uint8Array::new(&buf);
                    media_bytes.set(Some(arr.to_vec()));
                });
            }
        }
    };

    view! {
        <div class="form-field">
            <label>"Title"</label>
            <input
                type="text"
                placeholder="Title of the musical work"
                prop:value=move || title.get()
                on:input=move |ev| title.set(event_target_value(&ev))
            />
        </div>

        <div class="creators-section">
            <div class="section-header">
                <h3>"Creators"</h3>
                <button class="btn-add" on:click=add_creator>"+ Add Creator"</button>
            </div>
            <For
                each=move || creators.get()
                key=|c| c.id
                children=move |c| {
                    let id = c.id;
                    view! { <CreatorEntry c=c creators=creators id=id /> }
                }
            />
        </div>

        <div class="form-field">
            <label>"Media File"</label>
            <input type="file" on:change=on_file_change />
            <Show when=move || !media_name.get().is_empty()>
                <span class="file-name">{move || media_name.get()}</span>
            </Show>
        </div>
    }
}

#[component]
fn CreatorEntry(
    c: CreatorSignals,
    creators: RwSignal<Vec<CreatorSignals>>,
    id: u64,
) -> impl IntoView {
    let index = move || {
        creators
            .get()
            .iter()
            .position(|x| x.id == id)
            .map(|i| i + 1)
            .unwrap_or(0)
    };

    view! {
        <div class="creator-entry">
            <div class="creator-header">
                <span class="creator-index">"Creator #" {index}</span>
                <button
                    class="btn-remove"
                    on:click=move |_| creators.update(|list| list.retain(|x| x.id != id))
                >
                    "Remove"
                </button>
            </div>
            <div class="form-row">
                <div class="form-field">
                    <label>"Full Name"</label>
                    <input
                        type="text"
                        placeholder="Full legal name"
                        prop:value=move || c.full_name.get()
                        on:input=move |ev| c.full_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label>"Email"</label>
                    <input
                        type="email"
                        placeholder="email@example.com"
                        prop:value=move || c.email.get()
                        on:input=move |ev| c.email.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="form-field">
                <label>"Roles"</label>
                <div class="role-cards">
                    <button
                        class="role-card"
                        class:selected=move || c.author.get()
                        on:click=move |_| c.author.set(!c.author.get_untracked())
                    >"Author"</button>
                    <button
                        class="role-card"
                        class:selected=move || c.composer.get()
                        on:click=move |_| c.composer.set(!c.composer.get_untracked())
                    >"Composer"</button>
                    <button
                        class="role-card"
                        class:selected=move || c.arranger.get()
                        on:click=move |_| c.arranger.set(!c.arranger.get_untracked())
                    >"Arranger"</button>
                    <button
                        class="role-card"
                        class:selected=move || c.adapter.get()
                        on:click=move |_| c.adapter.set(!c.adapter.get_untracked())
                    >"Adapter"</button>
                </div>
            </div>
            <div class="form-row">
                <div class="form-field">
                    <label>"IPI (optional)"</label>
                    <input
                        type="text"
                        placeholder="1-11 digits"
                        prop:value=move || c.ipi.get()
                        on:input=move |ev| c.ipi.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label>"ISNI (optional)"</label>
                    <input
                        type="text"
                        placeholder="16 characters [0-9X]"
                        prop:value=move || c.isni.get()
                        on:input=move |ev| c.isni.set(event_target_value(&ev))
                    />
                </div>
            </div>
        </div>
    }
}
