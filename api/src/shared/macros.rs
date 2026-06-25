#[macro_export]
macro_rules! assign_patch {
    ($model:expr, $payload:expr, [ $( $field:ident ),+ $(,)? ]) => {
        $(
            if let Some(val) = $payload.$field {
                $model.$field = sea_orm::ActiveValue::Set(val);
            }
        )+
    };
}
