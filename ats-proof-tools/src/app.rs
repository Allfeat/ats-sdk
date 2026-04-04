use leptos::prelude::*;

use crate::creator::CreatorPage;
use crate::verify::VerifyPage;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Ownership,
    Creator,
}

#[component]
pub fn App() -> impl IntoView {
    let active_tab = RwSignal::new(Tab::Ownership);
    let show_banner = RwSignal::new(true);
    let is_dark = RwSignal::new(initial_dark_mode());

    // Apply dark class on mount and on toggle.
    Effect::new(move |_| {
        let document = web_sys::window().unwrap().document().unwrap();
        let html = document.document_element().unwrap();
        let class_list = html.class_list();
        if is_dark.get() {
            let _ = class_list.add_1("dark");
        } else {
            let _ = class_list.remove_1("dark");
        }
        if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
            let _ = storage.set_item("theme", if is_dark.get() { "dark" } else { "light" });
        }
    });

    let toggle_theme = move |_| {
        is_dark.set(!is_dark.get_untracked());
    };

    view! {
        <header class="header">
            <div class="header-left">
                <AllfeatLogo />
                <span class="header-separator"></span>
                <span class="header-app-name">"ATS Proof Tools"</span>
            </div>
            <div class="header-right">
                <span class="header-badge">"Open-Source"</span>
                <span class="header-badge badge-client">"100% Client-Side"</span>
                <button class="theme-toggle" on:click=toggle_theme title="Toggle theme">
                    {move || if is_dark.get() {
                        view! {
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M12 3v2.25m6.364.386-1.591 1.591M21 12h-2.25m-.386 6.364-1.591-1.591M12 18.75V21m-4.773-4.227-1.591 1.591M5.25 12H3m4.227-4.773L5.636 5.636M15.75 12a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0Z" />
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M21.752 15.002A9.72 9.72 0 0 1 18 15.75c-5.385 0-9.75-4.365-9.75-9.75 0-1.33.266-2.597.748-3.752A9.753 9.753 0 0 0 3 11.25C3 16.635 7.365 21 12.75 21a9.753 9.753 0 0 0 9.002-5.998Z" />
                            </svg>
                        }.into_any()
                    }}
                </button>
            </div>
        </header>

        <Show when=move || show_banner.get()>
            <div class="trust-banner">
                <svg class="trust-icon" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z" />
                </svg>
                <span class="trust-text">"This tool runs entirely in your browser. No data is sent to any server — your files and metadata never leave your device."</span>
                <button class="trust-close" on:click=move |_| show_banner.set(false)>
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        </Show>

        <nav class="tabs">
            <button
                class:active=move || active_tab.get() == Tab::Ownership
                on:click=move |_| active_tab.set(Tab::Ownership)
            >
                "Prove Ownership of a work"
            </button>
            <button
                class:active=move || active_tab.get() == Tab::Creator
                on:click=move |_| active_tab.set(Tab::Creator)
            >
                "Prove Involvement"
            </button>
        </nav>
        <main class="content">
            {move || match active_tab.get() {
                Tab::Ownership => view! { <VerifyPage /> }.into_any(),
                Tab::Creator => view! { <CreatorPage /> }.into_any(),
            }}
        </main>

        <footer class="footer">
            <div class="footer-left">
                <span class="footer-built">
                    "Built with the "
                    <a href="https://github.com/Allfeat/ats-sdk" target="_blank" rel="noopener">"ats-sdk"</a>
                </span>
                <span class="footer-version">"v" {env!("CARGO_PKG_VERSION")}</span>
                <span class="footer-license">"MIT License"</span>
            </div>
            <div class="footer-right">
                <a class="footer-link" href="https://github.com/Allfeat/ats-sdk/issues/new" target="_blank" rel="noopener">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z" />
                    </svg>
                    "Report Issue"
                </a>
                <a class="footer-link" href="https://github.com/Allfeat/ats-sdk" target="_blank" rel="noopener">
                    <svg viewBox="0 0 16 16" fill="currentColor">
                        <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z" />
                    </svg>
                    "Source"
                </a>
                <a class="footer-link" href="https://allfeat.org" target="_blank" rel="noopener">"allfeat.org"</a>
            </div>
        </footer>
    }
}

/// Allfeat logo SVG (uses currentColor, works in both themes).
#[component]
fn AllfeatLogo() -> impl IntoView {
    view! {
        <svg class="header-allfeat-logo" viewBox="17 27 170 36" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M50.7782 42.3274C51.591 40.3981 50.6592 38.1863 48.6972 37.3872C46.7351 36.588 44.4857 37.5042 43.6729 39.4335C42.8602 41.3628 43.792 43.5746 45.754 44.3737C47.7161 45.1728 49.9655 44.2567 50.7782 42.3274Z" fill="currentColor" />
            <path d="M25.7138 44.6594C27.8375 44.6594 29.5592 42.9665 29.5592 40.8783C29.5592 38.7901 27.8375 37.0972 25.7138 37.0972C23.5901 37.0972 21.8685 38.7901 21.8685 40.8783C21.8685 42.9665 23.5901 44.6594 25.7138 44.6594Z" fill="currentColor" />
            <path d="M36.4128 63C38.5366 63 40.2582 61.3072 40.2582 59.2189C40.2582 57.1307 38.5366 55.4378 36.4128 55.4378C34.2891 55.4378 32.5675 57.1307 32.5675 59.2189C32.5675 61.3072 34.2891 63 36.4128 63Z" fill="currentColor" />
            <path d="M52.16 49.5522C50.9778 48.8786 49.6565 48.6886 48.4022 48.8735C47.1196 49.0584 45.618 49.9954 43.931 49.0381C42.5762 48.2657 42.3727 46.774 42.0173 45.6268C41.8885 45.1785 41.7082 44.7353 41.461 44.3124C41.1648 43.811 40.8042 43.3678 40.387 42.993C39.5757 42.18 38.5609 41.2962 38.5609 39.8906C38.5609 37.976 40.1371 37.1656 40.9433 36.1678C41.7288 35.1852 42.2182 33.9645 42.2182 32.6172C42.2182 29.5148 39.6606 27 36.5055 27C33.3504 27 30.7929 29.5148 30.7929 32.6172C30.7929 33.9493 31.2282 35.1117 32.0549 36.1349C32.7735 37.0263 34.458 38.2065 34.458 39.883C34.458 41.4532 33.0414 42.3776 32.0807 43.5856C31.7175 44.0288 31.4291 44.5226 31.2024 45.0317C30.6796 46.2068 30.6796 48.0935 29.0878 48.9925C27.4034 49.9498 25.8993 49.0128 24.6166 48.8279C23.3623 48.6431 22.0384 48.8355 20.8588 49.5066C18.1235 51.0692 17.1886 54.5008 18.7674 57.1878C20.3463 59.8749 23.8414 60.7967 26.5741 59.2392C27.7485 58.5731 28.5547 57.6234 29.0441 56.4078C29.4665 55.3517 29.6673 53.3257 31.1457 52.4874C32.3331 51.8138 33.6286 52.2696 34.9576 52.5457C34.9988 52.5584 35.0375 52.566 35.0787 52.5786C35.0915 52.5786 35.1044 52.5862 35.1122 52.5862C35.1714 52.5989 35.2332 52.6115 35.3002 52.6267C35.3465 52.6343 35.3929 52.647 35.4418 52.6521C35.4624 52.6521 35.483 52.6597 35.4959 52.6597C35.8256 52.7179 36.1681 52.7508 36.5159 52.7508H36.6627C36.7167 52.7508 36.7708 52.7508 36.8301 52.7432C36.9923 52.7356 37.1597 52.723 37.3194 52.7027C38.9034 52.5102 40.4668 51.7125 41.8834 52.5178C43.3592 53.3561 43.5627 55.3821 43.9851 56.4382C44.4744 57.6538 45.2806 58.6035 46.4551 59.2696C49.1878 60.8195 52.6828 59.8976 54.2617 57.2182C55.8379 54.5388 54.8927 51.0996 52.16 49.5497V49.5522Z" fill="currentColor" />
            <path d="M73.5477 29.7858C71.3739 29.7858 69.4448 31.1559 68.7623 33.1845L59.7322 59.9711H65.0869L67.6342 52.0316H79.3789L81.9699 59.9711H87.3246L78.3358 33.1921C77.6558 31.161 75.7241 29.7883 73.5503 29.7883L73.5477 29.7858ZM69.1023 47.4452L73.2799 34.5014C73.3468 34.2836 73.6611 34.2836 73.7306 34.5014L77.9082 47.4452H69.0997H69.1023Z" fill="currentColor" />
            <path d="M93.9902 54.0703V29.7858H88.8957V54.5363C88.8957 57.8893 90.4925 59.9711 94.2503 59.9711H98.1369V55.8532H95.7622C94.4255 55.8532 93.9928 55.4708 93.9928 54.0703H93.9902Z" fill="currentColor" />
            <path d="M104.983 54.0703V29.7858H99.8883V54.5363C99.8883 57.8893 101.485 59.9711 105.243 59.9711H109.13V55.8532H106.755C105.418 55.8532 104.985 55.4708 104.985 54.0703H104.983Z" fill="currentColor" />
            <path d="M112.424 35.2206V38.7434H109.66V42.9879H112.424V59.9686H117.518V42.9904H121.232V38.7459H117.518V35.6891C117.518 34.2886 117.951 33.9062 119.288 33.9062H121.232V29.7858H117.778C114.023 29.7858 112.424 31.865 112.424 35.2206Z" fill="currentColor" />
            <path d="M132.663 38.3179C126.401 38.3179 121.912 42.5219 121.912 49.3547C121.912 56.1875 126.445 60.3915 132.879 60.3915C138.146 60.3915 141.729 57.5474 142.55 53.4295H137.585C137.111 55.1289 135.555 56.4863 132.879 56.4863C129.425 56.4863 127.179 54.1944 127.179 50.7121V50.6691H142.852V49.3522C142.852 42.2636 138.708 38.3154 132.663 38.3154V38.3179ZM127.179 47.2324C127.439 44.0921 129.598 42.2231 132.575 42.2231C135.553 42.2231 137.713 44.2187 137.713 47.2324H127.179Z" fill="currentColor" />
            <path d="M155.096 38.3179C149.829 38.3179 145.899 40.9492 145.554 46.0877H150.692C150.779 43.4969 152.245 42.2256 154.88 42.2256C157.342 42.2256 158.851 43.2867 158.851 46.1308V46.5132L151.727 48.253C148.402 49.1014 145.466 51.054 145.466 54.7059C145.466 57.8893 147.753 60.394 152.116 60.394C155.354 60.394 157.512 58.9505 158.939 56.4458V59.9686H163.946V46.7234C163.946 41.5013 160.793 38.3179 155.096 38.3179ZM158.851 51.7758C158.851 54.8756 156.822 56.7852 153.929 56.7852C151.9 56.7852 150.561 55.8937 150.561 54.0677C150.561 52.5406 151.555 51.5631 153.713 51.054L158.851 49.8232V51.7758Z" fill="currentColor" />
            <path d="M173.823 32.7995L168.729 33.6074V38.7434H166.009V42.9879H168.729V54.6198C168.729 58.1856 170.197 59.9686 174.212 59.9686H177.882V55.724H175.636C174.212 55.724 173.823 55.3847 173.823 53.9411V42.9879H177.882V38.7434H173.823V32.7995Z" fill="currentColor" />
            <path d="M183.878 59.9686C185.602 59.9686 187 58.5943 187 56.8991C187 55.2039 185.602 53.8297 183.878 53.8297C182.154 53.8297 180.757 55.2039 180.757 56.8991C180.757 58.5943 182.154 59.9686 183.878 59.9686Z" fill="currentColor" />
        </svg>
    }
}

/// Determine initial dark mode from localStorage or system preference.
fn initial_dark_mode() -> bool {
    let window = web_sys::window().unwrap();
    if let Ok(Some(storage)) = window.local_storage() {
        if let Ok(Some(theme)) = storage.get_item("theme") {
            return theme == "dark";
        }
    }
    window
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}
