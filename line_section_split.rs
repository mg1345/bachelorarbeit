// Importieren der benötigten Standardbibliotheksmodule und externen Bibliotheken
use std::fs::{self, File}; // Modul für Dateioperationen
use std::io::{BufReader, BufWriter, Write}; // Modul für Ein- und Ausgabe
use std::error::Error; // Trait für Fehlerbehandlung
use csv::{ReaderBuilder, WriterBuilder}; // Externe CSV-Bibliothek für das Lesen und Schreiben von CSV-Dateien

pub fn run(csv_file_path: &str, output_folder: &str) -> Result<(), Box<dyn Error>> {
    let input_file = File::open(csv_file_path)?;
    let reader = BufReader::new(input_file);
    let mut csv_reader = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(reader);

    let mut current_section: Vec<String> = Vec::new();
    let mut section_identifiers: Option<(String, String, String, String, String)> = None;

    for result in csv_reader.records() {
        let record = result?;
        let lfnr = record[8].parse::<usize>()?;

        // Wenn lfnr != 1, füge Zeile zur aktuellen Sektion hinzu
        if lfnr != 1 {
            current_section.push(record.into_iter().collect::<Vec<&str>>().join(";"));
        } else {
            // Wenn lfnr == 1 und aktuelle Sektion nicht leer ist
            if !current_section.is_empty() {
                // Überprüfe, ob bereits eine Datei für diese Sektion existiert
                if let Some(section_identifiers) = &section_identifiers {
                    let file_exists = check_file_exists(output_folder, &section_identifiers)?;
                    if !file_exists {
                        // Datei erstellen und alle Zeilen in der Liste hinzufügen
                        write_section_to_file(output_folder, &current_section, &section_identifiers)?;
                        current_section.clear();
                    }
                }
            }

            // Füge Zeile mit lfnr 1 zur aktuellen Sektion hinzu
            current_section.push(record.into_iter().collect::<Vec<&str>>().join(";"));

            // Setze section_identifiers auf die Werte der aktuellen Zeile
            let line = record[4].to_string();
            let direction = record[5].to_string();
            let variant = record[6].to_string();
            let umlauf = record[7].to_string();
            let fahrtstart = record[19].to_string();
            section_identifiers = Some((line, direction, variant, umlauf, fahrtstart));
        }
    }

    // Schreiben der letzten Sektion, falls vorhanden
    if !current_section.is_empty() {
        if let Some(section_identifiers) = &section_identifiers {
            write_section_to_file(output_folder, &current_section, &section_identifiers)?;
        }
    }

    Ok(())
}

// Funktion zum Schreiben einer Sektion in eine separate CSV-Datei
fn write_section_to_file(output_folder: &str, section: &[String], section_identifiers: &(String, String, String, String, String)) -> Result<(), Box<dyn Error>> {
    // Extrahieren der Sektionsidentifikatoren
    let (line, direction, variant, umlauf, fahrtstart) = section_identifiers;
    let output_dir = output_folder; // Ordner für die Ausgabedateien
    fs::create_dir_all(output_dir)?; // Erstellen des Ausgabeverzeichnisses, falls es nicht existiert

    // Dateiname basierend auf den Werten von line, direction, variant und umlauf erstellen
    let filename = format!("{}_{}_{}_{}_{}.csv", line, direction, variant, umlauf, fahrtstart);
    let output_file_path = format!("{}/{}", output_dir, filename);

    // Öffnen der Ausgabedatei
    let output_file = File::create(output_file_path)?;
    let mut writer = BufWriter::new(output_file);
    let mut csv_writer = WriterBuilder::new()
        .delimiter(b';') // Festlegen des Trennzeichens für die CSV-Datei
        .from_writer(writer);

    // Schreiben aller Zeilen der Sektion in die Datei
    for line in section {
        csv_writer.write_record(line.split(';'))?;
    }

    csv_writer.flush()?; // Sicherstellen, dass alle Daten in die Datei geschrieben werden
    Ok(()) // Erfolgreicher Abschluss der Funktion
}

// Funktion zum Überprüfen, ob eine Datei für eine Sektion bereits existiert
fn check_file_exists(output_folder: &str, section_identifiers: &(String, String, String, String, String)) -> Result<bool, Box<dyn Error>> {
    // Extrahieren der Sektionsidentifikatoren
    let (line, direction, variant, umlauf, fahrtstart) = section_identifiers;
    let output_dir = output_folder; // Ordner für die Ausgabedateien
    let filename = format!("{}_{}_{}_{}_{}.csv", line, direction, variant, umlauf, fahrtstart);
    let file_path = format!("{}/{}", output_dir, filename);

    // Überprüfen, ob die Datei existiert
    Ok(std::path::Path::new(&file_path).exists())
}




