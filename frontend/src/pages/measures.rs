use crate::api::AuthorizedApi;
use crate::Page;
use interface::{Measure, MeasureCreate};
use leptos::*;
use leptos_router::*;

/// Turn an icon spec ("1f34c", "1f1e9_1f1ea+1f9d1") into native emoji glyphs for
/// preview in the browser (the card itself renders bundled Noto SVGs server-side).
fn emoji_glyphs(spec: &str) -> String {
    spec.split('+')
        .flat_map(|tok| tok.split('_'))
        .filter_map(|h| u32::from_str_radix(h.trim(), 16).ok())
        .filter_map(char::from_u32)
        .collect()
}

/// Searchable emoji picker. Queries the server (only matches come back) and
/// appends the chosen codepoint to `icon` (up to two, `+`-joined).
#[component]
fn IconPicker(api: Signal<Option<AuthorizedApi>>, icon: RwSignal<String>) -> impl IntoView {
    let query = create_rw_signal(String::new());
    let results = create_resource(
        move || query.get(),
        move |q| async move {
            match api.get_untracked() {
                Some(a) => a.search_emoji(&q).await.unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );
    let add = move |cp: String| {
        let mut tokens: Vec<String> = icon
            .get_untracked()
            .split('+')
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        if !tokens.contains(&cp) && tokens.len() < 2 {
            tokens.push(cp);
            icon.set(tokens.join("+"));
        }
    };

    view! {
        <div class="form-control">
            <span class="label-text mb-1">
                "Emoji (up to 2) "
                <span class="text-2xl align-middle">{move || emoji_glyphs(&icon.get())}</span>
                <button
                    type="button" class="btn btn-xs btn-ghost"
                    on:click=move |_| icon.set(String::new())
                >"clear"</button>
            </span>
            <input
                class="input input-bordered w-72" type="text"
                placeholder="Search: banana, flag, cat…"
                prop:value=query
                on:input=move |ev| query.set(event_target_value(&ev))
            />
            <div class="flex flex-wrap gap-1 mt-2 max-h-44 overflow-y-auto w-72 p-1 bg-base-100 rounded-box">
                <For
                    each=move || results.get().unwrap_or_default()
                    key=|e| e.codepoint.clone()
                    children=move |e| {
                        let cp = e.codepoint.clone();
                        view! {
                            <button
                                type="button" class="btn btn-ghost btn-sm text-2xl px-2"
                                title=e.name
                                on:click=move |_| add(cp.clone())
                            >{e.glyph}</button>
                        }
                    }
                />
            </div>
        </div>
    }
}

#[component]
pub fn MeasureForm(
    api: Signal<Option<AuthorizedApi>>,
    existing: Signal<Option<Measure>>,
    #[prop(into)] on_save: Callback<()>,
) -> impl IntoView {
    let name = create_rw_signal(String::new());
    let grams = create_rw_signal(String::new());
    let icon = create_rw_signal(String::new());
    let submit_error = create_rw_signal(None::<String>);
    let waiting = create_rw_signal(false);

    let grams_valid = Signal::derive(move || grams.get().parse::<f64>().is_ok_and(|f| f > 0.0));

    create_effect(move |_| match existing.get() {
        None => {
            name.set(String::new());
            grams.set(String::new());
            icon.set(String::new());
        }
        Some(e) => {
            name.set(e.name);
            grams.set(e.grams.to_string());
            icon.set(e.icon.unwrap_or_default());
        }
    });

    let editing = Signal::derive(move || existing.get().is_some());
    let disabled =
        Signal::derive(move || waiting.get() || name.get().is_empty() || !grams_valid.get());

    let submit_action = create_action(move |()| async move {
        let icon_val = {
            let i = icon.get_untracked();
            (!i.trim().is_empty()).then(|| i.trim().to_string())
        };
        let grams_val = grams.get_untracked().parse::<f64>().unwrap_or(0.0);
        waiting.set(true);
        let a = api.get_untracked();
        let result = if let Some(ex) = existing.get_untracked() {
            a.as_ref()
                .unwrap()
                .update_one(Measure {
                    id: ex.id,
                    name: name.get_untracked(),
                    grams: grams_val,
                    icon: icon_val,
                })
                .await
                .map(|_| ())
        } else {
            a.as_ref()
                .unwrap()
                .add(MeasureCreate {
                    name: name.get_untracked(),
                    grams: grams_val,
                    icon: icon_val,
                })
                .await
                .map(|_| ())
        };
        waiting.set(false);
        match result {
            Ok(()) => {
                name.set(String::new());
                grams.set(String::new());
                icon.set(String::new());
                submit_error.set(None);
                on_save.call(());
            }
            Err(err) => submit_error.set(Some(err.to_string())),
        }
    });

    view! {
        <form
            class="flex flex-col md:flex-row gap-3 md:items-end bg-base-200 p-4 rounded-box"
            on:submit=|ev| ev.prevent_default()
        >
            <label class="form-control flex-grow">
                <span class="label-text mb-1">"Name (plural)"</span>
                <input
                    class="input input-bordered w-full" type="text" placeholder="Bowling Balls"
                    prop:value=name
                    on:input=move |ev| name.set(event_target_value(&ev))
                />
            </label>
            <label class="form-control">
                <span class="label-text mb-1">"Grams (each)"</span>
                <input
                    class="input input-bordered w-40" type="number" attr:step="0.0001" attr:min="0"
                    placeholder="7000"
                    prop:value=grams
                    on:input=move |ev| grams.set(event_target_value(&ev))
                />
            </label>
            <IconPicker api=api icon=icon/>
            <button
                class="btn btn-primary"
                prop:disabled=move || disabled.get()
                on:click=move |_| submit_action.dispatch(())
            >
                {move || if editing.get() { "Save" } else { "Add measure" }}
            </button>
        </form>
        <p class="text-error mt-2">{move || submit_error.get()}</p>
    }
}

#[component]
pub fn MeasureList(#[prop(into)] api: Signal<Option<AuthorizedApi>>) -> impl IntoView {
    if api.get().is_none() {
        use_navigate()(Page::Login.path(), Default::default());
    }

    let fetch_error = create_rw_signal(None::<String>);
    let measures = create_resource(
        || (),
        move |()| async move {
            match api.get_untracked().as_ref().unwrap().list().await {
                Ok(m) => m,
                Err(err) => {
                    fetch_error.set(Some(err.to_string()));
                    vec![]
                }
            }
        },
    );

    let edit_measure = create_rw_signal(None::<Measure>);
    let delete_action = create_action(move |id: &i32| {
        let id = *id;
        async move {
            match api.get().as_ref().unwrap().delete_one(id).await {
                Ok(()) => measures.refetch(),
                Err(err) => fetch_error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <section class="container mx-auto p-4 max-w-4xl flex flex-col gap-4">
            <h1 class="text-2xl font-bold">"Measures"</h1>
            <MeasureForm
                api=api
                existing=edit_measure.into()
                on_save=move |()| {
                    measures.refetch();
                    edit_measure.set(None);
                }
            />
            <p class="text-error">{move || fetch_error.get()}</p>
            <Transition fallback=move || view! { <span class="loading loading-spinner"></span> }>
                <div class="overflow-x-auto">
                    <table class="table table-zebra">
                        <thead>
                            <tr>
                                <th>"Icon"</th>
                                <th>"Name"</th>
                                <th class="text-right">"Grams"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || measures.get().unwrap_or_default()
                                key=move |m| m.id
                                children=move |m| {
                                    let for_edit = m.clone();
                                    let id = m.id;
                                    let glyphs = m.icon.as_deref().map(emoji_glyphs).unwrap_or_default();
                                    view! {
                                        <tr>
                                            <td class="text-2xl">{glyphs}</td>
                                            <td>{m.name}</td>
                                            <td class="text-right font-mono">{m.grams}</td>
                                            <td class="text-right whitespace-nowrap">
                                                <button
                                                    class="btn btn-sm btn-ghost"
                                                    on:click=move |_| edit_measure.set(Some(for_edit.clone()))
                                                >"Edit"</button>
                                                <button
                                                    class="btn btn-sm btn-error btn-outline"
                                                    on:click=move |_| delete_action.dispatch(id)
                                                >"Delete"</button>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>
            </Transition>
        </section>
    }
}
