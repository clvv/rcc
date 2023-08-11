use std::error::Error;
use std::fs::{read_to_string, write};
use std::io::Read;
use std::process::Stdio;
use std::{path::PathBuf, process::Command};
use toml_edit::{ArrayOfTables, Document, InlineTable, Value};

pub const RCC_CIRCUIT_NAME_PREFIX: &str = "rcc-";

fn ensure_path_is_rcc_circuit(path: &PathBuf) -> Result<(), Box<dyn Error>> {
    if path.is_file() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Target file not found",
        )))
    }
}

fn get_manifest_path() -> PathBuf {
    use serde_json::{Value, Value::Object};

    let output = Command::new("cargo")
        .args(["locate-project"])
        .output()
        .expect("Couldn't run cargo.");
    let result: Value =
        serde_json::from_str(String::from_utf8(output.stdout).unwrap().as_str()).unwrap();
    let path_string: String = match result {
        Object(map) => map.get("root").unwrap().as_str().unwrap().into(),
        _ => todo!(),
    };
    path_string.into()
}

fn circuit_path_to_name(circuit: &PathBuf) -> String {
    let no_extension_path = circuit.with_extension("");
    format!(
        "{RCC_CIRCUIT_NAME_PREFIX}{}",
        no_extension_path.file_name().unwrap().to_str().unwrap()
    )
}

fn modify_manifest(op: impl FnOnce(&mut ArrayOfTables) -> ()) -> Result<(), Box<dyn Error>> {
    let manifest_path = get_manifest_path();

    // Read the existing content of the TOML file into a string.
    let mut content = read_to_string(manifest_path.to_str().unwrap())?;

    // Parse the TOML content into a `toml_edit::Document`.
    let mut doc = content.parse::<Document>()?;

    // Check if the table already exists in the array.
    let array = doc
        .as_table_mut()
        .entry("bin")
        .or_insert(toml_edit::Item::ArrayOfTables(ArrayOfTables::new()))
        .as_array_of_tables_mut()
        .expect("Table exists, but it's not an array");

    // Apply the modification
    op(array);

    // Serialize the modified TOML document back to a string.
    content = doc.to_string();

    // Write the updated content back to the TOML file.
    write(manifest_path, content)?;

    Ok(())
}

/// This ensures that there is a binary target for a given circuit
fn ensure_circuit_in_manifest(circuit: &PathBuf) -> Result<(), Box<dyn Error>> {
    let circuit_name = circuit_path_to_name(circuit);
    let circuit_path = circuit.to_str().unwrap();

    modify_manifest(move |array| {
        let circuit_already_exists = array.iter().any(|table| {
            if let Some(name) = table.get("name") {
                name.as_str().unwrap() == circuit_name
            } else {
                false
            }
        });

        // If the table doesn't exist, append it to the array.
        if !circuit_already_exists {
            let mut table = InlineTable::new();
            table.insert("name", Value::from(circuit_name));
            table.insert("path", Value::from(circuit_path));
            array.push(table.into_table());
        }
    })?;

    Ok(())
}

/// Clean up a given `rcc-` binary target
pub fn clean_manifest_for(circuit: &PathBuf) -> Result<(), Box<dyn Error>> {
    let circuit_name = circuit_path_to_name(circuit);

    modify_manifest(move |array| {
        array.retain(|table| table.get("name").unwrap().as_str().unwrap() != circuit_name);
    })?;

    Ok(())
}

/// Clean up all generated `rcc-` binary targets
pub fn clean_manifest_forall() -> Result<(), Box<dyn Error>> {
    modify_manifest(|array| {
        array.retain(|table| {
            !table
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
                .starts_with(RCC_CIRCUIT_NAME_PREFIX)
        });
    })?;

    Ok(())
}

fn run_cargo(args: &[&str]) -> Result<(), std::io::Error> {
    let mut cmd = Command::new("cargo");
    cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    // Forward stdout and stderr concurrently
    let stdout = child.stdout.take().expect("Failed to capture stdout");
    let stderr = child.stderr.take().expect("Failed to capture stderr");

    let stdout_handle = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stdout);
        let mut buffer = String::new();
        while reader.read_to_string(&mut buffer).unwrap() > 0 {
            print!("{}", buffer);
            buffer.clear();
        }
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut reader = std::io::BufReader::new(stderr);
        let mut buffer = String::new();
        while reader.read_to_string(&mut buffer).unwrap() > 0 {
            eprint!("{}", buffer);
            buffer.clear();
        }
    });

    // Wait for the cargo process to complete
    let status = child.wait()?;
    let _stdout_result = stdout_handle.join().unwrap();
    let _stderr_result = stderr_handle.join().unwrap();

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Cargo process failed.",
        ));
    }

    Ok(())
}

pub fn build_circuit(circuit: &PathBuf, args: Vec<&str>) -> Result<(), Box<dyn Error>> {
    ensure_path_is_rcc_circuit(circuit)?;
    ensure_circuit_in_manifest(circuit)?;

    let circuit_name = circuit_path_to_name(circuit);
    println!("Building {circuit_name} via cargo..");

    let mut run = vec![
        "run",
        "--color",
        "always",
        "--release",
        "--bin",
        circuit_name.as_str(),
        "--",
    ];

    run.extend(args);

    let result = run_cargo(&run);

    clean_manifest_for(circuit)?;

    result?;

    Ok(())
}
