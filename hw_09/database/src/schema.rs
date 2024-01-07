// @generated automatically by Diesel CLI.

diesel::table! {
    colors (id) {
        id -> Int4,
        r -> Int2,
        g -> Int2,
        b -> Int2,
    }
}

diesel::table! {
    messages (id) {
        id -> Int4,
        user_id -> Int4,
        text -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        color_id -> Nullable<Int4>,
    }
}

diesel::joinable!(messages -> users (user_id));
diesel::joinable!(users -> colors (color_id));

diesel::allow_tables_to_appear_in_same_query!(
    colors,
    messages,
    users,
);
