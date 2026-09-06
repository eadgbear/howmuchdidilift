use insta::{assert_debug_snapshot, with_settings};
use interface::MeasureCreate;
use liftcalc::{app::App, models::measures};
use loco_rs::testing;
use serial_test::serial;

macro_rules! configure_insta {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("measures");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn test_model() {
    configure_insta!();

    let boot = testing::boot_test::<App>().await.unwrap();
    //testing::seed::<App>(&boot.app_context.db).await.unwrap();
    let create_params = MeasureCreate {
        name: "burgers".to_string(),
        grams: 500.0,
        icon: None,
    };
    let create_req = measures::ActiveModel::create(&boot.app_context.db, create_params).await;
    println!("{:?}", create_req);
    assert!(create_req.is_ok());
    let create_req = create_req.unwrap();

    let item = measures::Model::find_by_name(&boot.app_context.db, "Burgers").await;
    assert!(item.is_ok());
    let item = item.unwrap();
    assert_eq!(create_req.id.unwrap(), item.id);
    /*with_settings!({
            filters => {
                 let mut combined_filters = testing::CLEANUP_DATE.to_vec();
                    combined_filters.extend(vec![(r#"\"id\\":\d+"#, r#""id\":ID"#)]);
                    combined_filters
            }
        }, {
            assert_debug_snapshot!(item);
        }
    );*/
    // query your model, e.g.:
    //
    // let item = models::posts::Model::find_by_pid(
    //     &boot.app_context.db,
    //     "11111111-1111-1111-1111-111111111111",
    // )
    // .await;

    // snapshot the result:
    // assert_debug_snapshot!(item);
}
