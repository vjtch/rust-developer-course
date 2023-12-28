pub mod builder;
pub mod errors;
pub mod message;
pub mod receiver;
pub mod sender;

pub fn remove_new_line(string: &mut String) {
    *string = string
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string();
}
