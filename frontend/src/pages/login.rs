use crate::{
    api::{AuthorizedApi, UnauthorizedApi},
    components::credentials::*,
    Page,
};
use interface::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn Login(
    api: UnauthorizedApi,
    #[prop(into)] on_success: Callback<AuthorizedApi>,
) -> impl IntoView {
    let (login_error, set_login_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let login_action = create_action(move |(email, password): &(String, String)| {
        log::debug!("Try to login with {email}");
        let email = email.to_string();
        let password = password.to_string();
        let credentials = LoginParams { email, password };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = api.login(&credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    set_login_error.update(|e| *e = None);
                    on_success.call(res);
                }
                Err(err) => {
                    log::error!(
                        "Unable to login with {}: {}",
                        credentials.email,
                        err.to_string()
                    );
                    set_login_error.update(|e| *e = Some(err.to_string()));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <CredentialsForm
            title="Log in"
            action_label="Login"
            action=login_action
            error=login_error.into()
            disabled
        >
            "Don't have an account? "
            <A class="link link-primary" href=Page::Register.path()>"Register"</A>
        </CredentialsForm>
    }
}
