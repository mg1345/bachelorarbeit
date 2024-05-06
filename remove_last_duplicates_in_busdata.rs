use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

pub fn remove_last_duplicates_in_busdata(directory: &str) -> Result<(), Box<dyn Error>> {
    // Durchlaufen aller Dateien im Verzeichnis
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // Überprüfen, ob es sich um eine CSV-Datei handelt
        if let Some(extension) = path.extension() {
            if extension == "csv" {
                process_csv_file(&path)?;
            }
        }
    }

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        // Überprüfen, ob es sich um eine CSV-Datei handelt
        if let Some(file_name) = path.file_name() {
            let file_name = file_name.to_string_lossy();

            if !file_name.ends_with("_filtered.csv") {
                // Löschen der unfilterten Dateien
                fs::remove_file(&path)?;
            }
        }
    }
    
    Ok(())
}



pub fn process_csv_file(file_path: &Path) -> Result<(), Box<dyn Error>> {
    // Öffnen der Eingabedatei
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // Erstellen der Ausgabedatei im gleichen Verzeichnis mit "_filtered" im Dateinamen
    let output_file_path = file_path.with_extension("csv_filtered.csv");
    let mut output_file = File::create(output_file_path)?;

    // Durchlaufen der Zeilen der Eingabedatei
    let mut prev_line: Option<Vec<String>> = None;
    let mut next_line: Option<Vec<String>> = None;

    for line in reader.lines() {
        let current_line = line?;
        let current_fields: Vec<String> = current_line.split(';').map(|s| s.to_string()).collect();

        // Speichern der aktuellen Zeile für den nächsten Durchlauf
        next_line = Some(current_fields.clone());

        // Überprüfen, ob die vorherige Zeile existiert und die Bedingung erfüllt ist
        if let Some(prev_fields) = prev_line.take() {
            if let Some(next_fields) = next_line.clone() {
                if next_fields[12] == current_fields[12] && next_fields[9] == "ZAHLUNG" {
                    // Überspringen der aktuellen Zeile, da die nächste Zeile den Typ "ZAHLUNG" hat
                    prev_line = next_line;
                    continue;
                }
            }

            // Schreiben der vorherigen Zeile in die Ausgabedatei
            writeln!(&mut output_file, "{}", prev_fields.join(";"))?;
        }

        // Speichern der aktuellen Zeile für den nächsten Durchlauf
        prev_line = next_line;
    }

    // Überprüfen, ob eine vorherige Zeile übrig geblieben ist und sie in die Ausgabedatei schreiben
    if let Some(prev_fields) = prev_line {
        writeln!(&mut output_file, "{}", prev_fields.join(";"))?;
    }

    Ok(())
}