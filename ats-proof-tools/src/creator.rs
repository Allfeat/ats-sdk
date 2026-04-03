use ats_sdk::{Creator, Role, verify_creator_inclusion};
use leptos::prelude::*;

use crate::proof::{parse_hex_hash, parse_merkle_proof_json};

#[derive(Clone, PartialEq, Eq)]
enum CreatorResult {
    Included,
    NotIncluded,
}

#[component]
pub fn CreatorPage() -> impl IntoView {
    let full_name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let author = RwSignal::new(false);
    let composer = RwSignal::new(false);
    let arranger = RwSignal::new(false);
    let adapter = RwSignal::new(false);
    let ipi = RwSignal::new(String::new());
    let isni = RwSignal::new(String::new());

    let merkle_root_hex = RwSignal::new(String::new());
    let proof_json = RwSignal::new(String::new());

    let result = RwSignal::new(Option::<CreatorResult>::None);
    let error = RwSignal::new(Option::<String>::None);

    let verify = move |_| {
        let root = match parse_hex_hash(&merkle_root_hex.get_untracked()) {
            Ok(h) => h,
            Err(e) => {
                error.set(Some(format!("Merkle root: {e}")));
                result.set(None);
                return;
            }
        };

        let proof = match parse_merkle_proof_json(&proof_json.get_untracked()) {
            Ok(p) => p,
            Err(e) => {
                error.set(Some(e));
                result.set(None);
                return;
            }
        };

        let mut roles = Vec::new();
        if author.get_untracked() {
            roles.push(Role::Author);
        }
        if composer.get_untracked() {
            roles.push(Role::Composer);
        }
        if arranger.get_untracked() {
            roles.push(Role::Arranger);
        }
        if adapter.get_untracked() {
            roles.push(Role::Adapter);
        }

        let creator = Creator {
            full_name: full_name.get_untracked(),
            email: email.get_untracked(),
            roles,
            ipi: {
                let v = ipi.get_untracked();
                if v.is_empty() { None } else { Some(v) }
            },
            isni: {
                let v = isni.get_untracked();
                if v.is_empty() { None } else { Some(v) }
            },
        };

        if verify_creator_inclusion(&creator, &proof, &root) {
            result.set(Some(CreatorResult::Included));
            error.set(None);
        } else {
            result.set(Some(CreatorResult::NotIncluded));
            error.set(None);
        }
    };

    view! {
        <section class="page">
            <h2>"Creator Proof"</h2>
            <p class="description">
                "Verify that a specific creator is part of an ATS commitment. This enables selective disclosure — a creator can prove their involvement without revealing the other participants."
            </p>

            <div class="hint-box">
                <svg class="hint-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="m11.25 11.25.041-.02a.75.75 0 0 1 1.063.852l-.708 2.836a.75.75 0 0 0 1.063.853l.041-.021M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9-3.75h.008v.008H12V8.25Z" />
                </svg>
                {r#"The Merkle root is listed on your Allfeat certificate under "Cryptographic Proof". The Merkle proof for each creator is provided separately by the tool or service that generated the ATS."#}
            </div>

            <h3>"Creator data"</h3>
            <div class="form-row">
                <div class="form-field">
                    <label>"Full Name"</label>
                    <input
                        type="text"
                        placeholder="Full legal name"
                        prop:value=move || full_name.get()
                        on:input=move |ev| full_name.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label>"Email"</label>
                    <input
                        type="email"
                        placeholder="email@example.com"
                        prop:value=move || email.get()
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </div>
            </div>
            <div class="form-field">
                <label>"Roles"</label>
                <div class="role-cards">
                    <button
                        class="role-card"
                        class:selected=move || author.get()
                        on:click=move |_| author.set(!author.get_untracked())
                    >"Author"</button>
                    <button
                        class="role-card"
                        class:selected=move || composer.get()
                        on:click=move |_| composer.set(!composer.get_untracked())
                    >"Composer"</button>
                    <button
                        class="role-card"
                        class:selected=move || arranger.get()
                        on:click=move |_| arranger.set(!arranger.get_untracked())
                    >"Arranger"</button>
                    <button
                        class="role-card"
                        class:selected=move || adapter.get()
                        on:click=move |_| adapter.set(!adapter.get_untracked())
                    >"Adapter"</button>
                </div>
            </div>
            <div class="form-row">
                <div class="form-field">
                    <label>"IPI (optional)"</label>
                    <input
                        type="text"
                        placeholder="1-11 digits"
                        prop:value=move || ipi.get()
                        on:input=move |ev| ipi.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label>"ISNI (optional)"</label>
                    <input
                        type="text"
                        placeholder="16 characters [0-9X]"
                        prop:value=move || isni.get()
                        on:input=move |ev| isni.set(event_target_value(&ev))
                    />
                </div>
            </div>

            <h3>"Proof data"</h3>
            <div class="form-field">
                <label>"Merkle Root (hex)"</label>
                <input
                    type="text"
                    placeholder="64 hex characters"
                    prop:value=move || merkle_root_hex.get()
                    on:input=move |ev| merkle_root_hex.set(event_target_value(&ev))
                />
            </div>
            <div class="form-field">
                <label>"Merkle Proof (JSON)"</label>
                <textarea
                    placeholder=r#"[{"sibling": "abcd...1234", "is_left": true}, ...]"#
                    prop:value=move || proof_json.get()
                    on:input=move |ev| proof_json.set(event_target_value(&ev))
                />
            </div>

            <div class="btn-primary-wrapper">
                <button class="btn-primary" on:click=verify>"Verify Creator"</button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            {move || result.get().map(|r| match r {
                CreatorResult::Included => view! {
                    <div class="verify-result success">
                        "Verified — this creator is part of the ATS commitment."
                    </div>
                }.into_any(),
                CreatorResult::NotIncluded => view! {
                    <div class="verify-result failure">
                        "Not verified — the creator data or proof does not match the commitment."
                    </div>
                }.into_any(),
            })}
        </section>
    }
}
