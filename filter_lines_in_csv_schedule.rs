use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{self, BufRead};  
    
// Eine Methode zum Filtern der Linien
pub fn filter_lines_in_csv_schedule(csv_file_path: &str, output_folder: &str) -> Result<(), Box<dyn Error>> {
    // Erstellen eines Ordners für die Ausgabedateien
    let output_folder = output_folder;
    fs::create_dir_all(output_folder)?;

    // CSV-Datei öffnen und lesen
    let file = File::open(csv_file_path)?;
    println!("Öffnen und Lesen der Schedule-CSV-Datei: {}", csv_file_path);
    let reader = io::BufReader::new(file);

    // Vector zum Speichern der Zeilen jeder Linie
    let mut lines_by_line: Vec<Vec<String>> = Vec::new();

    // Überschrift für die CSV-Datei
    let mut header = String::new();

    // Iteriere über jede Zeile in der CSV-Datei
    for (index, line) in reader.lines().enumerate() {
        let line_content = line?;
        if index == 0 {
            header = line_content.clone(); // Speichere die Überschrift
            continue; // Überspringe die erste Zeile
        }
        let columns: Vec<&str> = line_content.split(';').collect();

        // Holen Sie den Wert der "line"-Spalte
        let line_number: usize = columns[4].parse().unwrap(); // Annahme: Die Liniennummer ist in der fünften Spalte

        // Überprüfen, ob genügend Platz im Vektor vorhanden ist
        while lines_by_line.len() <= line_number {
            lines_by_line.push(Vec::new());
        }

        // Fügen Sie die Zeile zur entsprechenden Linie im Vector hinzu
        lines_by_line[line_number].push(line_content);
    }

    // Für jede Linie eine Datei erstellen
    for (line_number, lines) in lines_by_line.iter().enumerate() {
        // Überspringe Linien, die keine Zeilen haben
        if lines.is_empty() {
            continue;
        }

        // Dateiname für die aktuelle Linie
        let filename = format!("{}/line_{}.csv", output_folder, line_number);

        // Inhalt der Datei zusammenstellen
        let file_content = format!("{}\n{}", header, lines.join("\n"));

        // Datei schreiben
        fs::write(&filename, &file_content)?;

        //println!("Datei erstellt: {}", filename);
    }

    Ok(())
}