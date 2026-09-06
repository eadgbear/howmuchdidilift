use crate::api::UnauthorizedApi;
use crate::clipboard::{copy_image, copy_text, download_image, is_mobile, share_card};
use leptos::*;
use leptos_router::use_query_map;

/// Split a share spec like `225lbs` / `100kg` into (amount, "Lbs"|"Kgs").
fn parse_spec(spec: &str) -> Option<(String, String)> {
    let s = spec.trim().to_lowercase();
    let idx = s
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(s.len());
    let (num, unit) = s.split_at(idx);
    if num.parse::<f64>().is_err() {
        return None;
    }
    let unit = match unit.trim() {
        "" | "lb" | "lbs" | "pound" | "pounds" => "Lbs",
        "kg" | "kgs" | "kilo" | "kilos" | "kilogram" | "kilograms" => "Kgs",
        _ => return None,
    };
    Some((num.to_string(), unit.to_string()))
}

#[component]
pub fn Convert(api: UnauthorizedApi, show_links: RwSignal<bool>) -> impl IntoView {
    let (error, set_error) = create_signal(None::<String>);
    let (amt, set_amt) = create_signal(String::new());
    let (input_type, set_input_type) = create_signal("Lbs".to_string());
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    // Latest card: (data-url image, optional share sentence).
    let result = create_rw_signal(None::<(String, Option<String>)>);
    let copied = create_rw_signal(None::<String>);

    let click_count = create_rw_signal(0);
    create_effect(move |_| {
        if click_count.get() >= 6 {
            show_links.update(|s| *s = !(*s));
            click_count.set(0);
        }
    });

    let convert_action = create_action(move |(weight, unit): &(String, String)| {
        let weight = weight.clone();
        let unit_lower = unit.to_lowercase();
        async move {
            if weight.parse::<f64>().is_err() {
                set_error.set(Some("Invalid weight".to_string()));
                return;
            }
            set_wait_for_response.set(true);
            copied.set(None);
            match api.fetch_card(&weight, &unit_lower).await {
                Ok(card) => {
                    set_error.set(None);
                    result.set(Some(card));
                }
                Err(e) => set_error.set(Some(e.to_string())),
            }
            set_wait_for_response.set(false);
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());
    let submit_disabled =
        Signal::derive(move || disabled.get() || error.get().is_some() || amt.get().is_empty());
    let dispatch_action = move || {
        if !submit_disabled.get() {
            convert_action.dispatch((amt.get(), input_type.get()));
        }
    };

    create_effect(move |_| {
        if !amt.get().is_empty() {
            if let Err(e) = amt.get().parse::<f64>() {
                set_error.set(Some(format!(
                    "Unable to parse {} as a number: {e:?}",
                    amt.get()
                )));
            } else {
                set_error.set(None);
            }
        }
    });

    // Deep-link support: `/?w=225lbs` pre-fills and auto-converts (once).
    let query = use_query_map();
    let did_init = create_rw_signal(false);
    create_effect(move |_| {
        if did_init.get_untracked() {
            return;
        }
        if let Some(spec) = query.with(|q| q.get("w").cloned()) {
            if let Some((amt_s, unit_s)) = parse_spec(&spec) {
                did_init.set(true);
                set_amt.set(amt_s.clone());
                set_input_type.set(unit_s.clone());
                convert_action.dispatch((amt_s, unit_s));
            }
        }
    });

    let card_src = Signal::derive(move || result.get().map(|(url, _)| url));

    let mobile = is_mobile();

    let do_share = move |_| {
        if let Some((url, share)) = result.get_untracked() {
            let text = share.unwrap_or_default();
            spawn_local(async move {
                let status = share_card(&url, &text, "How much did I lift?").await;
                copied.set((status != "cancelled").then_some(status)); // reset on cancel
            });
        }
    };
    let do_copy_image = move |_| {
        if let Some((url, _)) = result.get_untracked() {
            spawn_local(async move {
                let ok = copy_image(&url).await;
                copied.set(Some(if ok { "img" } else { "fail" }.to_string()));
            });
        }
    };
    let do_copy_text = move |_| {
        if let Some((_, share)) = result.get_untracked() {
            let text = share.unwrap_or_default();
            spawn_local(async move {
                let ok = copy_text(&text).await;
                copied.set(Some(if ok { "txt" } else { "fail" }.to_string()));
            });
        }
    };
    let do_download = move |_| {
        if let Some((url, _)) = result.get_untracked() {
            let name = format!(
                "hmdil-{}{}.png",
                amt.get_untracked().trim().replace(' ', ""),
                input_type.get_untracked().to_lowercase()
            );
            download_image(&url, &name);
        }
    };

    let share_label = move || match copied.get().as_deref() {
        Some("shared") => "Shared! ✓",
        Some("fail") => "Couldn't share — try again",
        _ => "Share",
    };
    let img_label = move || {
        if copied.get().as_deref() == Some("img") {
            "Image copied ✓"
        } else {
            "Copy image"
        }
    };
    let txt_label = move || {
        if copied.get().as_deref() == Some("txt") {
            "Text copied ✓"
        } else {
            "Copy text"
        }
    };

    view! {
        <section class="items-center justify-center py-12 px-4 sm:px-6 lg:px-8 hero min-h-screen flex">
            <div class="items-center flex flex-col w-full">
                <button class="btn no-animation text-center text-4xl lg:text-5xl leading-9 font-extrabold bg-base-100 hover:bg-base-100 outline-none" on:click=move |_| {
                    click_count.set(click_count.get() + 1);
                }>How much did I lift?</button>
                <form on:submit=|ev| ev.prevent_default() class="mt-8 w-full space-y-6">
                    <div>
                        <div class="items-center flex w-full md:w-2/3 mx-auto flex-col md:flex-row">
                            <input
                                type="number"
                                class="focus:border-indigo-700 focus:outline-none
                                    focus:shadow-outline flex-grow transition duration-200 appearance-none p-2 border-2 border-gray-300
                                    text-black bg-gray-100 font-normal w-full h-14 text-xl rounded-md shadow-sm"
                                placeholder="Amount lifted"
                                prop:value=amt
                                prop:disabled=move|| disabled.get()
                                on:keyup=move |ev: ev::KeyboardEvent| {
                                    match &*ev.key() {
                                        "Enter" => dispatch_action(),
                                        _ => {
                                            let val = event_target_value(&ev);
                                            set_amt.update(|v| *v = val);
                                        }
                                    }
                                }
                                on:change=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_amt.update(|v| *v = val);
                                }
                            />
                            <select
                                class="focus:border-indigo-700 focus:outline-none
                                    focus:shadow-outline flex-grow transition duration-200 appearance-none p-2 m-2 h-14 border-2
                                    font-normal w-1/6 min-w-14 max-w-14 h-12 text-xl rounded-md shadow-sm"
                                prop:value=input_type
                                prop:disabled=move|| disabled.get() on:change= move|ev| {
                                    set_input_type.update(|v| *v = event_target_value(&ev))
                            }>
                                <option value="Lbs" selected>"lbs"</option>
                                <option value="Kgs">"kgs"</option>
                            </select>
                            <button
                                class="btn btn-primary btn-lg flex p-2 h-14 w-auto justify-end
                                    items-center "
                                prop:disabled=submit_disabled
                                on:click=move|_| dispatch_action()
                            >"Convert"</button>
                        </div>
                        <p class="text-error text-center mt-2">{move || error.get()}</p>
                        <Show when=move || card_src.get().is_some() fallback=|| view!{}>
                            <div class="flex flex-col items-center gap-4 mt-8">
                                <img
                                    class="rounded-3xl w-full max-w-md shadow-xl shadow-slate-900"
                                    src=move || card_src.get().unwrap_or_default()
                                    alt="Your lift, measured"
                                />
                                <div class="flex flex-wrap gap-2 justify-center">
                                    {if mobile {
                                        view! {
                                            <button class="btn btn-primary" on:click=do_share>{share_label}</button>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <button class="btn btn-primary" on:click=do_copy_image>{img_label}</button>
                                            <button class="btn btn-secondary" on:click=do_copy_text>{txt_label}</button>
                                        }.into_view()
                                    }}
                                    <button class="btn btn-outline" on:click=do_download>"Download image"</button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </form>
            </div>
        </section>
    }
}
