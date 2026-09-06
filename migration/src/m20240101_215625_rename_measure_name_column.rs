use sea_orm_migration::prelude::*;

use crate::m20231224_205059_measures::Measures;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let query = Table::alter()
            .table(Measures::Table)
            .rename_column(Measures::NamePlural, Measures::Name)
            .to_owned();

        println!("{:?}", query.to_string(PostgresQueryBuilder));
        manager.alter_table(query).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Measures::Table)
                    .rename_column(Measures::Name, Measures::NamePlural)
                    .to_owned(),
            )
            .await
    }
}
