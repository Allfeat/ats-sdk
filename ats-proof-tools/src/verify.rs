use ats_sdk::{AtsInput, OnChainCommitment, verify_commitment};
use leptos::prelude::*;

use crate::form::{CreatorSignals, DataForm, HintBox};
use crate::proof::parse_hex_hash;

#[derive(Clone, PartialEq, Eq)]
enum VerifyResult {
    Match,
    Mismatch,
}

#[component]
pub fn VerifyPage() -> impl IntoView {
    let title = RwSignal::new(String::new());
    let next_id = RwSignal::new(1u64);
    let creators = RwSignal::new(vec![CreatorSignals::new(0)]);
    let media_bytes = RwSignal::new(Option::<Vec<u8>>::None);
    let media_name = RwSignal::new(String::new());

    let commitment_hex = RwSignal::new(String::new());
    let protocol_version = RwSignal::new(String::from("1"));

    let result = RwSignal::new(Option::<VerifyResult>::None);
    let error = RwSignal::new(Option::<String>::None);

    let verify = move |_| {
        let hash = match parse_hex_hash(&commitment_hex.get_untracked()) {
            Ok(h) => h,
            Err(e) => {
                error.set(Some(format!("Commitment hash: {e}")));
                result.set(None);
                return;
            }
        };

        let version: u8 = match protocol_version.get_untracked().parse() {
            Ok(v) => v,
            Err(_) => {
                error.set(Some("Protocol version must be a number (0-255).".into()));
                result.set(None);
                return;
            }
        };

        let creator_list: Vec<_> = creators
            .get_untracked()
            .iter()
            .map(|c| c.to_creator())
            .collect();

        let input = AtsInput {
            title: title.get_untracked(),
            creators: creator_list,
        };

        let expected = OnChainCommitment {
            commitment: hash,
            protocol_version: version,
        };

        match media_bytes.get_untracked() {
            None => {
                error.set(Some("Please select a media file.".into()));
                result.set(None);
            }
            Some(bytes) => match verify_commitment(&input, &bytes, &expected) {
                Ok(true) => {
                    result.set(Some(VerifyResult::Match));
                    error.set(None);
                }
                Ok(false) => {
                    result.set(Some(VerifyResult::Mismatch));
                    error.set(None);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                    result.set(None);
                }
            },
        }
    };

    view! {
        <section class="page">
            <h2>"Prove Ownership of a work"</h2>
            <p class="description">
                "Prove you hold the original data behind an on-chain ATS commitment. Fill in the fields from your certificate, provide the original media file, and the tool will reconstruct the commitment locally to verify it."
            </p>

            <HintBox>
                {r#"If you registered via the Allfeat platform, you can find all these values on your PDF certificate under "Cryptographic Proof" and "Creators"."#}
            </HintBox>

            <h3>"On-chain data"</h3>
            <div class="form-row form-row-onchain">
                <div class="form-field">
                    <label class="label-required">"Commitment"</label>
                    <input
                        type="text"
                        placeholder="64 hex characters"
                        prop:value=move || commitment_hex.get()
                        on:input=move |ev| commitment_hex.set(event_target_value(&ev))
                    />
                </div>
                <div class="form-field">
                    <label>"Protocol Version"</label>
                    <select
                        prop:value=move || protocol_version.get()
                        on:change=move |ev| protocol_version.set(event_target_value(&ev))
                    >
                        <option value="1">"1"</option>
                    </select>
                </div>
            </div>

            <h3>"Original work data"</h3>
            <DataForm title=title creators=creators next_id=next_id media_bytes=media_bytes media_name=media_name />

            <div class="btn-primary-wrapper">
                <button class="btn-primary" on:click=verify>"Prove Ownership"</button>
            </div>

            <Show when=move || error.get().is_some()>
                <div class="error">{move || error.get().unwrap_or_default()}</div>
            </Show>

            {move || result.get().map(|r| match r {
                VerifyResult::Match => view! {
                    <div class="verify-result success">
                        "Ownership confirmed — the provided data matches the on-chain commitment."
                    </div>
                }.into_any(),
                VerifyResult::Mismatch => view! {
                    <div class="verify-result failure">
                        "Ownership not confirmed — the provided data does not match the on-chain commitment."
                    </div>
                }.into_any(),
            })}
        </section>
    }
}
