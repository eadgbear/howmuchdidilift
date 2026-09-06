use gloo_storage::{LocalStorage, Storage};
use interface::*;
use leptos::*;
use leptos_meta::provide_meta_context;
use leptos_router::*;

mod api;
mod clipboard;
mod components;
mod pages;

use self::{components::*, pages::*};

const DEFAULT_API_URL: &str = "/api";
const API_TOKEN_STORAGE_KEY: &str = "api-token";

#[component]
pub fn App() -> impl IntoView {
    // -- signals -- //

    let authorized_api = create_rw_signal(None::<api::AuthorizedApi>);
    let user_info = create_rw_signal(None::<CurrentResponse>);
    let logged_in = Signal::derive(move || authorized_api.get().is_some());
    let title_clicked = create_rw_signal(false);

    let show_links = Signal::derive(move || title_clicked.get() || authorized_api.get().is_some());

    provide_context(authorized_api);

    // -- actions -- //

    let fetch_user_info = create_action(move |_| async move {
        match authorized_api.get_untracked() {
            Some(api) => match api.current_user().await {
                Ok(info) => user_info.update(|i| *i = Some(info)),
                Err(err) => {
                    log::error!("Unable to fetch user info: {err}")
                }
            },
            None => {
                log::error!("Unable to fetch user info: not logged in")
            }
        }
    });

    let logout = create_action(move |_| async move {
        match authorized_api.get() {
            Some(_api) => {
                authorized_api.update(|a| *a = None);
                user_info.update(|a| *a = None);
            }
            None => {
                log::error!("Unable to logout user: not logged in")
            }
        }
    });

    // -- callbacks -- //
    let on_logout = move |_| {
        logout.dispatch(());
    };

    let unauthorized_api = api::UnauthorizedApi::new(DEFAULT_API_URL);
    if let Ok(token) = LocalStorage::get(API_TOKEN_STORAGE_KEY) {
        let api = api::AuthorizedApi::new(DEFAULT_API_URL, token);
        authorized_api.update(|a| *a = Some(api));
        fetch_user_info.dispatch(());
    }

    log::debug!("User is logged in: {}", logged_in.get_untracked());

    create_effect(move |_| {
        log::debug!("API authorization state changed");
        match authorized_api.get() {
            Some(api) => {
                log::debug!("API is now authorized");
                LocalStorage::set(API_TOKEN_STORAGE_KEY, api.token).expect("LocalStorage::set");
            }
            None => {
                log::debug!("API is no longer authorized");
                LocalStorage::delete(API_TOKEN_STORAGE_KEY);
            }
        }
    });
    let on_success = move |api| {
        log::info!("Successfully logged in");
        authorized_api.update(|v| *v = Some(api));
        let navigate = use_navigate();
        navigate(Page::MeasureList.path(), Default::default());
        fetch_user_info.dispatch(());
    };

    provide_meta_context();

    view! {
        <div data-theme="dracula">

                <Router>
                    <NavBar logged_in on_logout show_links=show_links.into() />
                        <main>
                            <Routes>
                                <Route
                                    path=Page::Convert.path()
                                    view=move || {
                                        view! { <Convert api=unauthorized_api show_links=title_clicked/>}
                                    }
                                />
                                <Route
                                    path=Page::Login.path()
                                    view=move|| {
                                        view! {
                                            <Login
                                                api=unauthorized_api
                                                on_success=on_success
                                            />
                                        }
                                    }
                                />
                                <Route
                                    path=Page::Register.path()
                                    view=move || {
                                        view! { <Register api=unauthorized_api/> }
                                    }
                                />
                                <ProtectedRoute
                                    path=Page::MeasureList.path()
                                    condition=move || authorized_api.get().is_some()
                                    redirect_path=Page::Login.path()
                                    view=move || {
                                        view! { <MeasureList api=authorized_api/> }
                                    }
                                />
                            </Routes>
                        </main>
                </Router>
        </div>
    }
}
