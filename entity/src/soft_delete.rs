use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};

use crate::workspace;

pub trait SoftDeletableEntity: EntityTrait {
    type SoftDeleteColumn: ColumnTrait;

    fn delete_at_col() -> Self::SoftDeleteColumn;
}

pub trait SoftDeleteQueryExt<E: SoftDeletableEntity> {
    fn active(self) -> Self;
}

impl<E> SoftDeleteQueryExt<E> for Select<E>
where
    E: SoftDeletableEntity,
{
    fn active(self) -> Self {
        self.filter(E::delete_at_col().is_null())
    }
}

impl SoftDeletableEntity for workspace::Entity {
    type SoftDeleteColumn = workspace::Column;

    fn delete_at_col() -> Self::SoftDeleteColumn {
        workspace::Column::DeletedAt
    }
}
