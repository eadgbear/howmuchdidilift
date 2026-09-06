use std::borrow::BorrowMut;

use loco_rs::schema::*;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                table_auto(Measures::Table)
                    .col(pk_auto(Measures::Id).borrow_mut())
                    .col(string_uniq(Measures::Name).not_null().borrow_mut())
                    .col(string_uniq(Measures::NamePlural).not_null().borrow_mut())
                    .col(ColumnDef::new(Measures::Grams).double().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Measures::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Measures {
    Table,
    Id,
    Name,
    NamePlural,
    Grams,
    Icon,
}
