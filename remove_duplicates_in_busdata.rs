use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::Path;

pub fn remove_duplicates_except_payment(input_file_path: &str, output_file_path: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(input_file_path)?;
    let reader = io::BufReader::new(file);

    // Vector zum Speichern der Zeilen
    let mut lines: Vec<String> = Vec::new();
    let mut header = String::new();

    // Iteriere über jede Zeile in der CSV-Datei
    let mut previous_wkt = String::new();
    for (index, line) in reader.lines().enumerate() {
        let line_content = line?;
        if index == 0 {
            header = line_content.clone(); // Speichere die Überschrift
            continue; // Überspringe die erste Zeile
        }
        let columns: Vec<&str> = line_content.split(';').collect();
        let typ = columns[9]; // Index der Typ-Spalte (beginnt bei 0)
        let wkt = columns[12]; // Index der WKT-Spalte (beginnt bei 0)

        // Überprüfe, ob der WKT-Wert in dieser Zeile mit dem vorherigen übereinstimmt
        // und ob der Typ nicht ZAHLUNG ist
        if wkt == previous_wkt && typ != "ZAHLUNG" {
            continue; // Springe zur nächsten Iteration, um die Zeile zu überspringen
        }

        // Füge die Zeile hinzu
        lines.push(line_content.clone());
        previous_wkt = wkt.to_string(); // Aktualisiere den vorherigen WKT-Wert
    }

    // Schreibe die bereinigten Zeilen in die Ausgabedatei
    let mut output_file = File::create(output_file_path)?;
    writeln!(output_file, "{}", header)?;
    for line in &lines {
        writeln!(output_file, "{}", line)?;
    }

    Ok(())
}

pub fn process_csv_files_in_folder(folder_path: &str) -> Result<(), Box<dyn Error>> {
    // Liste alle Dateien im Ordner auf
    let paths = fs::read_dir(folder_path)?;

    // Durchlaufe alle Dateien im Ordner
    for path in paths {
        let entry = path?;
        let file_path = entry.path();
        
        // Extrahiere den Dateinamen aus dem DirEntry-Objekt
        if let Some(file_name) = file_path.file_name() {
            let file_name_str = file_name.to_string_lossy();

            // Überprüfe, ob die Datei eine CSV-Datei ist
            if let Some(extension) = file_path.extension() {
                if extension == "csv" {
                    // Erzeuge einen Ausgabepfad für die bereinigte CSV-Datei
                    let output_file_path = format!("{}/{}", folder_path, file_name_str);

                    // Entferne Duplikate außer für Zahlungen
                    remove_duplicates_except_payment(&file_path.to_string_lossy(), &output_file_path)?;
                    
                    println!("Duplicates removed for file: {}", file_name_str);
                }
            }
        }
    }

    Ok(())
}