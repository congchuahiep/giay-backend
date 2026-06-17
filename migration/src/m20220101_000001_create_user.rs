use sea_orm_migration::{prelude::*, sea_query::extension::postgres::Type};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TYPE user_role AS ENUM ('admin', 'user');
            "#,
        )
        .await?;

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TABLE "user" (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                email TEXT UNIQUE NOT NULL,
                password TEXT NOT NULL,
                first_name TEXT NOT NULL,
                last_name TEXT NOT NULL,
                role user_role DEFAULT 'user' NOT NULL,
                is_active BOOLEAN DEFAULT TRUE NOT NULL,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TABLE "user_session" (
                id UUID PRIMARY KEY,
                user_id UUID NOT NULL,
                expires_at TIMESTAMP WITH TIME ZONE NOT NULL
            );
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_type(Type::drop().name("user_role").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("user").to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table("user_session").to_owned())
            .await?;

        Ok(())
    }
}
