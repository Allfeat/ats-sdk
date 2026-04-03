use ats_sdk::{Creator, MAX_CREATORS, Role};
use leptos::prelude::*;
use wasm_bindgen::JsCast;

const ROLE_CHECK_ICON: &str = r#"<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg>"#;

fn empty_to_none(s: String) -> Option<String> {
    if s.is_empty() { None } else { Some(s) }
}

/// Reactive state for a single creator in the form.
#[derive(Clone, Copy)]
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
        if self.author.get_untracked() { roles.push(Role::Author); }
        if self.composer.get_untracked() { roles.push(Role::Composer); }
        if self.arranger.get_untracked() { roles.push(Role::Arranger); }
        if self.adapter.get_untracked() { roles.push(Role::Adapter); }
        Creator {
            full_name: self.full_name.get_untracked(),
            email: self.email.get_untracked(),
            roles,
            ipi: empty_to_none(self.ipi.get_untracked()),
            isni: empty_to_none(self.isni.get_untracked()),
        }
    }
}

/// Hint box with info icon.
#[component]
pub fn HintBox(children: Children) -> impl IntoView {
    view! {
        <div class="hint-box">
            <svg class="hint-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
            </svg>
            {children()}
        </div>
    }
}

/// Form fields for a single creator (name, email, roles, IPI, ISNI).
#[component]
pub fn CreatorFormFields(signals: CreatorSignals) -> impl IntoView {
    view! {
        <div class="form-row">
            <div class="form-field">
                <label>"Full name"</label>
                <input
                    type="text"
                    placeholder="Enter full name"
                    prop:value=move || signals.full_name.get()
                    on:input=move |ev| signals.full_name.set(event_target_value(&ev))
                />
            </div>
            <div class="form-field">
                <label>"Email"</label>
                <input
                    type="email"
                    placeholder="Enter email address"
                    prop:value=move || signals.email.get()
                    on:input=move |ev| signals.email.set(event_target_value(&ev))
                />
            </div>
        </div>
        <div class="form-field">
            <label>"Roles"</label>
            <div class="role-cards">
                <button
                    class="role-card"
                    class:selected=move || signals.author.get()
                    on:click=move |_| signals.author.set(!signals.author.get_untracked())
                >
                    <Show when=move || signals.author.get()><span class="role-check" inner_html=ROLE_CHECK_ICON /></Show>
                    "Author"
                </button>
                <button
                    class="role-card"
                    class:selected=move || signals.composer.get()
                    on:click=move |_| signals.composer.set(!signals.composer.get_untracked())
                >
                    <Show when=move || signals.composer.get()><span class="role-check" inner_html=ROLE_CHECK_ICON /></Show>
                    "Composer"
                </button>
                <button
                    class="role-card"
                    class:selected=move || signals.arranger.get()
                    on:click=move |_| signals.arranger.set(!signals.arranger.get_untracked())
                >
                    <Show when=move || signals.arranger.get()><span class="role-check" inner_html=ROLE_CHECK_ICON /></Show>
                    "Arranger"
                </button>
                <button
                    class="role-card"
                    class:selected=move || signals.adapter.get()
                    on:click=move |_| signals.adapter.set(!signals.adapter.get_untracked())
                >
                    <Show when=move || signals.adapter.get()><span class="role-check" inner_html=ROLE_CHECK_ICON /></Show>
                    "Adapter"
                </button>
            </div>
        </div>
        <hr class="creator-divider" />
        <p class="creator-optional-subtitle">"Optional Information"</p>
        <div class="form-row">
            <div class="form-field">
                <label>"IPI (optional)"</label>
                <input
                    type="text"
                    prop:value=move || signals.ipi.get()
                    on:input=move |ev| signals.ipi.set(event_target_value(&ev))
                />
                <span class="field-hint">"Format: 1-11 digits"</span>
            </div>
            <div class="form-field">
                <label>"ISNI (optional)"</label>
                <input
                    type="text"
                    prop:value=move || signals.isni.get()
                    on:input=move |ev| signals.isni.set(event_target_value(&ev))
                />
                <span class="field-hint">"Format: 16 characters: 15 digits and one digit or X"</span>
            </div>
        </div>
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
                placeholder="Enter the title of your work"
                prop:value=move || title.get()
                on:input=move |ev| title.set(event_target_value(&ev))
            />
        </div>

        <div class="creators-section">
            <div class="section-header">
                <h3>"Creators"</h3>
                <button class="btn-add" on:click=add_creator>
                    {move || format!("Add Creator ({}/{})", creators.get().len(), MAX_CREATORS)}
                </button>
            </div>
            <For
                each=move || creators.get()
                key=|c| c.id
                children=move |c| {
                    let id = c.id;
                    view! { <CreatorEntry signals=c creators=creators id=id /> }
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
    signals: CreatorSignals,
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
            <CreatorFormFields signals=signals />
        </div>
    }
}
