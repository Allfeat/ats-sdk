use ats_sdk::verify_creator_inclusion;
use leptos::prelude::*;

use crate::form::{CreatorFormFields, CreatorSignals, HintBox};
use crate::proof::{parse_hex_hash, parse_merkle_proof_json};

#[derive(Clone, PartialEq, Eq)]
enum CreatorResult {
    Included,
    NotIncluded,
}

#[component]
pub fn CreatorPage() -> impl IntoView {
    let signals = CreatorSignals::new(0);

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

        let creator = signals.to_creator();

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
            <h2>"Prove Involvement"</h2>
            <p class="description">
                "Verify that a specific creator is part of an ATS commitment. This enables selective disclosure — a creator can prove their involvement without revealing the other participants."
            </p>

            <HintBox>
                {r#"The Merkle root is listed on your Allfeat certificate under "Cryptographic Proof". The Merkle proof for each creator is provided separately by the tool or service that generated the ATS."#}
            </HintBox>

            <h3>"Creator data"</h3>
            <CreatorFormFields signals=signals />

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
