// Importieren der nötigen Bibliotheken
use std::collections::HashMap; // HashMap für die Speicherung von Zeilen für jedes Fahrzeug
use std::error::Error; // Trait für Fehlerbehandlung
use std::fs; // Modul für Dateioperationen
use std::fs::File; // Datei-Typ
use std::io::{self, BufRead}; // Eingabe-/Ausgabemodul für Zeilenbasiertes Lesen
use std::path::Path; // Pfadmodul für Pfadmanipulation


pub fn filter_bus_in_csv_rohdaten(output_folder: &str, input_file_path: &str) -> Result<(), Box<dyn Error>> {
    // Erstellen des Output-Ordners, falls er noch nicht vorhanden ist
    if !Path::new(output_folder).exists() {
        fs::create_dir_all(output_folder)?;
    }

    println!("Öffnen und Lesen der Rohdaten-CSV-Datei: {}", input_file_path);
    let file = File::open(input_file_path)?;
    let reader = io::BufReader::new(file);

    // HashMap zum Speichern von Zeilen für jedes Fahrzeug
    let mut lines_by_vehicle: HashMap<String, Vec<String>> = HashMap::new();
    let mut headers: Option<String> = None;

    // Iteriere über jede Zeile in der CSV-Datei
    for (index, line) in reader.lines().enumerate() {
        let line_content = line?;
        let columns: Vec<&str> = line_content.split(';').collect(); // Verwenden Sie ';' als Trennzeichen

        // Holen Sie den Wert der "vehicle"-Spalte
        let vehicle = columns[0].to_string(); // Annahme: Die Fahrzeugart ist in der ersten Spalte

        // Speichere die Spaltenüberschriften
        if index == 0 {
            headers = Some(line_content.clone());
        }

        // Fügen Sie die Zeile zur entsprechenden Fahrzeugart im HashMap hinzu
        lines_by_vehicle
            .entry(vehicle.clone())
            .or_insert(Vec::new())
            .push(line_content);
    }

    // Für jede Fahrzeugart eine Datei erstellen
    for (vehicle, lines) in lines_by_vehicle.iter() {
        // Dateiname für die aktuelle Fahrzeugart
        let filename = format!("{}/{}.csv", output_folder, vehicle.replace(";", "_")); // Ersetzen Sie ';' durch '_' im Fahrzeugtyp

        // Inhalt der Datei zusammenstellen
        let mut file_content = String::new();

        // Hinzufügen der Spaltenüberschriften
        if let Some(header_line) = headers.clone() {
            file_content.push_str(&header_line);
            file_content.push('\n');
        }

        // Hinzufügen der Zeileninhalte
        for line in lines {
            file_content.push_str(line);
            file_content.push('\n');
        }

        // Datei schreiben
        fs::write(&filename, &file_content)?;

        println!("Datei erstellt: {}", filename);
    }

    // Datei vehicle.csv löschen
    let vehicle_csv_path = format!("{}/vehicle.csv", output_folder);
    fs::remove_file(&vehicle_csv_path)?;

    println!("Datei gelöscht: {}", vehicle_csv_path);

    Ok(())
}