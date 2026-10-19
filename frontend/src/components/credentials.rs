use leptos::{ev, *};

/// Shared visual shell: a centered card with a title, optional error alert, the
/// form fields, a primary action button, and a footer slot (for cross-links).
#[component]
fn AuthCard(
    title: &'static str,
    error: Signal<Option<String>>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="min-h-screen flex items-center justify-center px-4 py-12">
            <div class="card w-full max-w-sm bg-base-200 shadow-xl">
                <div class="card-body gap-3">
                    <h2 class="card-title justify-center text-2xl mb-1">{title}</h2>
                    {move || {
                        error
                            .get()
                            .map(|err| view! { <div class="alert alert-error text-sm py-2">{err}</div> })
                    }}
                    {children()}
                </div>
            </div>
        </section>
    }
}

#[component]
pub fn CredentialsForm(
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let (pw, set_pw) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());

    let dispatch_action = move || action.dispatch((email.get(), pw.get()));
    let button_is_disabled =
        Signal::derive(move || disabled.get() || pw.get().is_empty() || email.get().is_empty());

    view! {
        <AuthCard title=title error=error>
            <form class="flex flex-col gap-3" on:submit=|ev| ev.prevent_default()>
                <input
                    type="email"
                    required
                    class="input input-bordered w-full"
                    placeholder="Email address"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| set_email.set(event_target_value(&ev))
                    on:change=move |ev| set_email.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    required
                    class="input input-bordered w-full"
                    placeholder="Password"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| {
                        if &*ev.key() == "Enter" { dispatch_action() } else { set_pw.set(event_target_value(&ev)) }
                    }
                    on:change=move |ev| set_pw.set(event_target_value(&ev))
                />
                <button
                    class="btn btn-primary w-full mt-1"
                    prop:disabled=move || button_is_disabled.get()
                    on:click=move |_| dispatch_action()
                >
                    {action_label}
                </button>
            </form>
        </AuthCard>
        <p class="text-center text-sm opacity-70 -mt-6 pb-12">{children()}</p>
    }
}

#[component]
pub fn RegistrationForm(
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String, String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let (pw, set_pw) = create_signal(String::new());
    let (pw_confirm, set_pw_confirm) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (name, set_name) = create_signal(String::new());

    let dispatch_action =
        move || action.dispatch((name.get(), email.get(), pw.get(), pw_confirm.get()));

    let mismatch =
        Signal::derive(move || !pw_confirm.get().is_empty() && pw.get() != pw_confirm.get());
    let button_is_disabled = Signal::derive(move || {
        disabled.get()
            || pw.get().is_empty()
            || email.get().is_empty()
            || pw_confirm.get().is_empty()
            || name.get().is_empty()
            || (pw.get() != pw_confirm.get())
    });

    view! {
        <AuthCard title=title error=error>
            <form class="flex flex-col gap-3" on:submit=|ev| ev.prevent_default()>
                <input
                    type="text"
                    required
                    class="input input-bordered w-full"
                    placeholder="Username"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| set_name.set(event_target_value(&ev))
                    on:change=move |ev| set_name.set(event_target_value(&ev))
                />
                <input
                    type="email"
                    required
                    class="input input-bordered w-full"
                    placeholder="Email address"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| set_email.set(event_target_value(&ev))
                    on:change=move |ev| set_email.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    required
                    class="input input-bordered w-full"
                    placeholder="Password"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| set_pw.set(event_target_value(&ev))
                    on:change=move |ev| set_pw.set(event_target_value(&ev))
                />
                <input
                    type="password"
                    required
                    class="input input-bordered w-full"
                    placeholder="Confirm password"
                    prop:disabled=move || disabled.get()
                    on:keyup=move |ev: ev::KeyboardEvent| {
                        if &*ev.key() == "Enter" { dispatch_action() } else { set_pw_confirm.set(event_target_value(&ev)) }
                    }
                    on:change=move |ev| set_pw_confirm.set(event_target_value(&ev))
                />
                <p class="text-error text-xs h-4">
                    {move || if mismatch.get() { "Passwords don't match" } else { "" }}
                </p>
                <button
                    class="btn btn-primary w-full"
                    prop:disabled=move || button_is_disabled.get()
                    on:click=move |_| dispatch_action()
                >
                    {action_label}
                </button>
            </form>
        </AuthCard>
        <p class="text-center text-sm opacity-70 -mt-6 pb-12">{children()}</p>
    }
}
