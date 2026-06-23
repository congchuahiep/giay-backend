use sea_orm_migration::prelude::*;

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
                id UUID PRIMARY KEY,
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

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TABLE workspace (
                id UUID PRIMARY KEY,
                name TEXT NOT NULL,
                slug TEXT UNIQUE NOT NULL,
                icon TEXT NULL,
                owner_id UUID REFERENCES "user"(id) ON DELETE CASCADE,
                created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
                deleted_at TIMESTAMP WITH TIME ZONE NULL
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TYPE workspace_role AS ENUM ('owner', 'moderator', 'member', 'viewer');
            "#,
        )
        .await?;

        db.execute_unprepared(
            /* sql */
            r#"
            CREATE TABLE workspace_membership (
                workspace_id UUID REFERENCES workspace(id) ON DELETE CASCADE,
                user_id UUID REFERENCES "user"(id) ON DELETE CASCADE,
                role workspace_role NOT NULL,
                joined_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,

                PRIMARY KEY (workspace_id, user_id)
            );
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TABLE IF EXISTS workspace_membership;
            DROP TYPE IF EXISTS workspace_role;
            DROP TABLE IF EXISTS workspace;
            DROP TABLE IF EXISTS user_session;
            DROP TABLE IF EXISTS "user";
            DROP TYPE IF EXISTS user_role;
            "#,
        )
        .await?;

        Ok(())
    }
}
