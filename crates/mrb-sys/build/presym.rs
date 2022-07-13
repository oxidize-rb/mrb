use std::fmt::Write;
use std::{error::Error, path::Path};

/// Transpiles mruby/presym/table.h to Rust.
pub fn transpile(table_h: &Path) -> Result<String, Box<dyn Error>> {
    let mut table_rs_output = String::new();
    let mut in_presym_length_table = false;
    let mut in_presym_name_table = false;
    let mut presym_name_table_entries = Vec::new();
    let mut presym_length_table_entries = Vec::new();

    let table_h_string = std::fs::read_to_string(&table_h)?;

    for line in table_h_string.lines() {
        if in_presym_length_table && line.starts_with('}') {
            in_presym_length_table = false;
        } else if in_presym_name_table && line.starts_with('}') {
            in_presym_name_table = false;
        }

        if line.starts_with("static const uint16_t presym_length_table[]") {
            assert!(!in_presym_name_table);
            in_presym_length_table = true;
            continue;
        } else if line.starts_with("static const char * const presym_name_table[]") {
            assert!(!in_presym_length_table);
            in_presym_name_table = true;
            continue;
        }

        if in_presym_length_table {
            presym_length_table_entries.push(line);
        } else if in_presym_name_table {
            presym_name_table_entries.push(line);
        }
    }

    table_rs_output.push_str("/// Presym table length entries.\n");
    write!(
        table_rs_output,
        "pub const presym_length_table: [u16; {}] = [",
        presym_length_table_entries.len()
    )?;
    for line in presym_length_table_entries {
        writeln!(table_rs_output, "{}", line)?;
    }
    table_rs_output.push_str("];\n");

    table_rs_output.push_str("/// Presym name table entries.\n");
    writeln!(
        table_rs_output,
        "pub const presym_name_table: [&str; {}] = [",
        presym_name_table_entries.len(),
    )?;
    for line in presym_name_table_entries {
        writeln!(table_rs_output, "  {}", line)?;
    }
    table_rs_output.push_str("];\n");

    Ok(table_rs_output)
}
