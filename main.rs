// Importe der nötigen Bibliotheken
use std::error::Error;
use std::fs;
use std::fs::File;
use std::mem::align_of;
use std::{thread, time};
use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::path::PathBuf;

// Einbinden der Funktionen
// Daten filtern

// Liniendaten
mod filter_lines_in_csv_schedule;       // Importiert die filter_lines_in_csv_schedule
mod line_section_split;                 // Importiert die linienabschnitt_split

// Busdaten
mod filter_bus_in_csv_rohdaten;         // Importiert die filter_bus_in_csv_rohdaten
mod remove_duplicates_in_busdata;       // Importiert die remove_duplicates_in_busdaten
mod remove_last_duplicates_in_busdata;  // Importiert die remove_last_duplicates_in_busdaten

// Algorithmus

// Heuristischer Algorithmus
mod algorithm_heuristic;                // Importiert den ersten Algorithmus

// VF2 Algorithmus
mod algorithm_vf2_part_1;               // Importiert den ersten Teil des VF2-Algorithmus
mod algorithm_vf2_part_2;               // Importiert den zweiten Teil des VF2-Algorihtmus

// Ergebnistest
mod ergebnis_test;                      // Importiert den Test für die Ergebnisse

// Main Methode
fn main() -> Result<(), Box<dyn Error>> {

    // Pfad zur Eingabe-CSV-Datei
    println!("Starten des Programms mit einlesen der Dateien.");

    // Pfade zu Dateien/Ordnern erstellen:
    // Wichtig diese anzupassen!
    let input_file_path = "/Users/martin/Downloads/diagnose/rohdaten_20231025.csv";                         // Pfad zur CSV-Busdatei
    let csv_file_path = "/Users/martin/Downloads/diagnose/schedule_20231025.csv";                           // Pfad zur CSV-Fahrplandatei
    let output_data = "/Users/martin/Desktop/Bachelorarbeit/test/GA/output.csv";                 // Pfad zur Ausgabedatei für heuristischen Algorithmus
    let comparison_data = "/Users/martin/Downloads/diagnose/zahldaten_20231025.csv";                        // Pfad zur Vergleichsdatei
    let output_folder_fahrplandaten = "/Users/martin/Desktop/Bachelorarbeit/test/GA/fahrplan";   // Pfad zum Fahrplanordner
    let output_folder_busdaten = "/Users/martin/Desktop/Bachelorarbeit/test/GA/busdaten";        // Pfad zum Busdatenordner
    let line_folder_path = "/Users/martin/Desktop/Bachelorarbeit/test/GA/output";                // Pfad zum Ordner der gefilterten Linien
    let output_file = "/Users/martin/Desktop/Bachelorarbeit/test/GA/output_vf2.csv";             // Pfad zur Outputdatei des VF2 Algorithmus
    let interim_save = "/Users/martin/Desktop/Bachelorarbeit/test/GA/zwischenspeicher.csv";            // Pfad zur Zwischenspeicher datei



    //Starten der Datenfilterung
    println!("Starten der Datenfilterung.");



    // Schritt 1: Beginne mit dem Filtern der CSV-Fahrplan-Daten

    // Zeitmessung starten
    let start_time = Instant::now();
    println!("\nSchritt 1: Beginne mit dem Filtern der CSV-Fahrplan-Daten.");
    filter_lines_in_csv_schedule::filter_lines_in_csv_schedule(csv_file_path, output_folder_fahrplandaten);
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 1 abgeschlossen: Daten nach Linien gefiltert und in fahrplan gespeichert.");
   


    // Schritt 2: Beginne mit dem Filtern der CSV-Bus-Daten

    // Zeitmessung starten
    let start_time = Instant::now();
    println!("\nSchritt 2: Beginne mit dem Filtern der CSV-Bus-Daten.");
    filter_bus_in_csv_rohdaten::filter_bus_in_csv_rohdaten(output_folder_busdaten, input_file_path);
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 2 abgeschlossen: Daten nach Busse gefiltert und in busdaten gespeichert.");
    


    // Schritt 3: Funktion aufrufen, um die Duplikate aus den Busdaten zu entfernen

    // Zeitmessung starten
    let start_time = Instant::now();
    println!("Schritt 3: Funktion aufrufen, um die Duplikate aus den Busdaten zu entfernen");    
    remove_duplicates_in_busdata::process_csv_files_in_folder(output_folder_busdaten);
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 3 abgeschlossen: Dateien erstellt.");



    // Schritt 4: Aufruf der Funktion zur Filterung der CSV-Dateien im Verzeichnis

    // Zeitmessung starten
    let start_time = Instant::now();
    println!("Schritt 4: Funktion aufrufen, um die letzten Duplikate aus den Busdaten zu entfernen");    
    remove_last_duplicates_in_busdata::remove_last_duplicates_in_busdata(output_folder_busdaten)?;
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 4 abgeschlossen: Dateien erstellt.");



    // Schritt 5: Teilen der Linien in einzelne abschnitte und speichern in eigenen csv dateien

    // Zeitmessung starten
    let start_time = Instant::now();
    println!("Schritt 5: Funktion aufrufen, um die Linien in einzelne Abschnitte zu teilen und speichern in eigenen csv dateien");    
    line_section_split::run(csv_file_path, line_folder_path)?;   
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 5 abgeschlossen: Funktion aufrufen, um die Linien in einzelne Abschnitte zu teilen und speichern in eigenen csv dateien"); 
    



    // Starten der Algorithmen
    println!("\n Es folgen die Algorithmen.");

    // Schritt 6: Aufruf der Funktion zum starten des erweiterten ersten Heuristischen-Algorithmus
    println!("Schritt 6: Aufruf der Funktion zum starten des Heuristischen-Algorithmus");  
    // Zeitmessung starten
    let start_time = Instant::now();  
    let lines = algorithm_heuristic::read_lines(csv_file_path)?;
    println!("Schritt 6.1: Linien eingelesen");
    let bus= algorithm_heuristic::read_bus_files(output_folder_busdaten)?;
    println!("Schritt 6.2: Busse eingelesen");
    algorithm_heuristic::run_algorithm(lines, bus, output_data);
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 6 abgeschlossen: Heuristischer-Algorithmus durchgeführt.");
 


    // Schritt 7: Aufruf der Funktion zum starten des erweiterten ersten Heuristischen-Algorithmus
    println!("Schritt 7.1: Aufruf der ersten Funktion zum starten des VF2-Algorithmus");  
    // Zeitmessung starten
    let start_time = Instant::now();  
    if let Err(err) = algorithm_vf2_part_1::run(line_folder_path, output_folder_busdaten, &interim_save) {
        eprintln!("Fehler beim Ausführen des Programms: {}", err);
    }
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 7.1 abgeschlossen: Aufruf der ersten Funktion zum starten des VF2-Algorithmus");  
    
    println!("Schritt 7.2: Aufruf der zweiten Funktion zum starten des VF2-Algorithmus");  
    // Zeitmessung starten
    let start_time = Instant::now();  
    algorithm_vf2_part_2::run(line_folder_path, output_folder_busdaten, &interim_save);
    // Beende die Zeitmessung
    let end_time = Instant::now();
    // Berechne die Dauer der Funktion
    let duration = end_time - start_time;
    // Gib die Dauer aus
    println!("Die Funktion hat {} Sekunden gedauert.", duration.as_secs_f64());
    println!("Schritt 7.2 abgeschlossen: Aufruf der zweiten Funktion zum starten des VF2-Algorithmus");  
    


    // Starten der Tests
    // Vergleiche die Daten und gib die Abweichung in Prozent aus
    println!("Schritt 8: Testen der Ergebnisse gestartet.");
    let percentage_difference = ergebnis_test::test_results(&output_data, &comparison_data);
    println!("Schritt 8 abgeschlossen: Testen der Ergebnisse abgeschlossen.");

    Ok(())
}