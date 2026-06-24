use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, Select};
use uuid::Uuid;

use crate::models::{workspace_invitation, workspace_membership};

pub trait WorkspaceBound: EntityTrait {
    fn workspace_column() -> Self::Column;

    fn find_by_workspace(workspace_id: Uuid) -> Select<Self> {
        Self::find().filter(Self::workspace_column().eq(workspace_id))
    }
}

macro_rules! impl_workspace_bound {
    ($p1:ident) => {
        impl WorkspaceBound for $p1::Entity {
            fn workspace_column() -> Self::Column {
                $p1::Column::WorkspaceId
            }
        }
    };
}

impl_workspace_bound!(workspace_invitation);
impl_workspace_bound!(workspace_membership);
