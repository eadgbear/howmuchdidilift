use crate::{api::UnauthorizedApi, components::credentials::*, Page};
use interface::*;
use leptos::{logging::log, *};
use leptos_router::*;

#[component]
pub fn Register(api: UnauthorizedApi) -> impl IntoView {
    let (register_response, set_register_response) = create_signal(None::<()>);
    let (reg_error, set_reg_error) = create_signal(None::<String>);
    let (wait, set_wait) = create_signal(false);

    let register_action = create_action(
        move |(name, email, password, password_confirm): &(String, String, String, String)| {
            let name = name.to_string();
            let email = email.to_string();
            let password = password.to_string();
            let password_confirm = password_confirm.to_string();
            let creds = RegisterParams {
                name,
                email,
                password,
                password_confirm,
            };

            log!("Trying to register new account for {}", creds.email);
            async move {
                set_wait.update(|w| *w = true);
                let result = api.register(&creds).await;
                set_wait.update(|w| *w = false);
                match result {
                    Ok(res) => {
                        set_register_response.update(|v| *v = Some(res));
                        set_reg_error.update(|v| *v = None);
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        set_reg_error.update(|e| *e = Some(msg));
                    }
                }
            }
        },
    );

    let disabled = Signal::derive(move || wait.get());

    view! {
        <Show
            when=move || register_response.get().is_some()
            fallback=move || {
                view! {
                    <RegistrationForm
                        title="Create your account"
                        action_label="Register"
                        action=register_action
                        error=reg_error.into()
                        disabled
                    >
                        "Already have an account? "
                        <A class="link link-primary" href=Page::Login.path()>"Login"</A>
                    </RegistrationForm>
                }
            }
        >
            <section class="min-h-screen flex items-center justify-center px-4">
                <div class="card w-full max-w-sm bg-base-200 shadow-xl">
                    <div class="card-body items-center text-center gap-3">
                        <h2 class="card-title">"You're all set! 🎉"</h2>
                        <p>"You can now " <A class="link link-primary" href=Page::Login.path()>"log in"</A> " with your new account."</p>
                    </div>
                </div>
            </section>
        </Show>
    }
}
