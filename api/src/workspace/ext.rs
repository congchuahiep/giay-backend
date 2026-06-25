use entity::workspace_invitation;
use uuid::Uuid;

use crate::lookup;

lookup! {
    InvitationByToken => workspace_invitation::Entity {
        column: workspace_invitation::Column::Token,
        value_type: Uuid,
        param: "token",
    }
}
