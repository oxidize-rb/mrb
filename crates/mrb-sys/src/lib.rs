#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/mruby_bindings.rs"));
include!(concat!(env!("OUT_DIR"), "/presym.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_a_valid_presym_table() {
        assert_eq!(presym_name_table.len(), presym_length_table.len());

        for (name, length) in presym_name_table.iter().zip(presym_length_table.iter()) {
            assert_eq!(name.len() as u16, *length);
        }
    }
}
