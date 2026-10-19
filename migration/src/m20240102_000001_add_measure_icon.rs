use sea_orm_migration::prelude::*;

use crate::m20231224_205059_measures::Measures;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Nullable: stores a Noto emoji codepoint (e.g. "1f35a") to render on cards.
        manager
            .alter_table(
                Table::alter()
                    .table(Measures::Table)
                    .add_column_if_not_exists(ColumnDef::new(Measures::Icon).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Measures::Table)
                    .drop_column(Measures::Icon)
                    .to_owned(),
            )
            .await
    }
}
