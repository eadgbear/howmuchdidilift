use insta::{assert_debug_snapshot, with_settings};
use interface::{LoginParams, RegisterParams};
use liftcalc::{app::App, models::users};
use loco_rs::testing;
use rstest::rstest;
use serial_test::serial;

// TODO: see how to dedup / extract this to app-local test utils
// not to framework, because that would require a runtime dep on insta
macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("auth_request");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_register() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        let email = "test@loco.com";
        let payload = RegisterParams {
            email: email.to_string(),
            password: "12341234".to_string(),
            password_confirm: "12341234".to_string(),
            name: "loco".to_string(),
        };

        let _response = request.post("/api/auth/register").json(&payload).await;
        let saved_user = users::Model::find_by_email(&ctx.db, email).await;

        with_settings!({
            filters => testing::cleanup_user_model()
        }, {
            assert_debug_snapshot!(saved_user);
        });

        with_settings!({
            filters => testing::cleanup_email()
        }, {
            assert_debug_snapshot!(ctx.mailer.unwrap().deliveries());
        });
    })
    .await;
}

#[rstest]
#[case("login_with_valid_password", "12341234")]
#[case("login_with_invalid_password", "invalid-password")]
#[tokio::test]
#[serial]
async fn can_login_with_verify(#[case] test_name: &str, #[case] password: &str) {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        let email = "test@loco.com";
        let payload = RegisterParams {
            email: email.to_string(),
            password: "12341234".to_string(),
            password_confirm: "12341234".to_string(),
            name: "loco".to_string(),
        };

        //Creating a new user
        _ = request.post("/api/auth/register").json(&payload).await;

        let user = users::Model::find_by_email(&ctx.db, email).await.unwrap();
        let verify_payload = serde_json::json!({
            "token": user.email_verification_token,
        });
        request.post("/api/auth/verify").json(&verify_payload).await;

        let login_payload = LoginParams {
            email: email.to_string(),
            password: password.to_string(),
        };
        //verify user request
        let response = request.post("/api/auth/login").json(&login_payload).await;

        // Make sure email_verified_at is set
        assert!(users::Model::find_by_email(&ctx.db, email)
            .await
            .unwrap()
            .email_verified_at
            .is_some());

        with_settings!({
            filters => testing::cleanup_user_model()
        }, {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()));
        });
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_login_without_verify() {
    configure_insta!();

    testing::request::<App, _, _>(|request, _ctx| async move {
        let email = "test@loco.com";
        let password = "12341234";
        let payload = RegisterParams {
            email: email.to_string(),
            password: password.to_string(),
            password_confirm: password.to_string(),
            name: "loco".to_string(),
        };

        //Creating a new user
        _ = request.post("/api/auth/register").json(&payload).await;

        let login_payload = LoginParams {
            email: email.to_string(),
            password: password.to_string(),
        };

        //verify user request
        let response = request.post("/api/auth/login").json(&login_payload).await;

        with_settings!({
            filters => testing::cleanup_user_model()
        }, {
            assert_debug_snapshot!((response.status_code(), response.text()));
        });
    })
    .await;
}

/*#[tokio::test]
#[serial]
async fn can_reset_password() {
    configure_insta!();

    testing::request::<App, _, _>(|request, ctx| async move {
        let login_data = prepare_data::init_user_login(&request, &ctx).await;

        let forgot_payload = serde_json::json!({
            "email": login_data.user.email,
        });
        _ = request.post("/api/auth/forgot").json(&forgot_payload).await;

        let user = users::Model::find_by_email(&ctx.db, &login_data.user.email)
            .await
            .unwrap();
        assert!(user.reset_token.is_some());

        let new_password = "new-password";
        let reset_payload = serde_json::json!({
            "token": user.reset_token,
            "password": new_password,
        });

        let reset_response = request.post("/api/auth/reset").json(&reset_payload).await;

        let user = users::Model::find_by_email(&ctx.db, &user.email)
            .await
            .unwrap();
        assert!(user.reset_sent_at.is_some());

        assert_debug_snapshot!((reset_response.status_code(), reset_response.text()));

        let response = request
            .post("/api/auth/login")
            .json(&serde_json::json!({
                "email": user.email,
                "password": new_password
            }))
            .await;

        assert_eq!(response.status_code(), 200);

        with_settings!({
            filters => testing::cleanup_email()
        }, {
            assert_debug_snapshot!(ctx.mailer.unwrap().deliveries());
        });
    })
    .await;
}
*/
