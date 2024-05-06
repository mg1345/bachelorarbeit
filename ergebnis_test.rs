use std::error::Error;
use std::fs;
use std::io::{self, BufRead};
extern crate geoutils; // Externes Paket für geografische Berechnungen
use geoutils::{Location, Distance};

#[derive(Debug)]
pub struct Record {
    schedule_id: String,
    fahrzeug: String,
    lon: f64,
    lat: f64,
    // Weitere Felder hinzufügen, die in den Dateien vorhanden sind
}

pub fn load_data_output(filename: &str) -> io::Result<Vec<Record>> {
    let file = fs::File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut records = Vec::new();

    // Iteriere über die Datensätze
    for line in reader.lines().skip(1) {
        let line = line?;
        let fields: Vec<&str> = line.split(';').collect();

        let record = Record {
            schedule_id: fields[0].to_string(),
            fahrzeug: fields[18].to_string(), //17
            lon: fields[13].parse().unwrap(), //22
            lat: fields[14].parse().unwrap(), //21
            
            // Hier weitere Felder entsprechend einfügen
        };
        records.push(record);
    }

    Ok(records)
}

pub fn load_data_comparison(filename: &str) -> io::Result<Vec<Record>> {
    let file = fs::File::open(filename)?;
    let reader = io::BufReader::new(file);
    let mut records = Vec::new();

    // Iteriere über die Datensätze
    for line in reader.lines().skip(1) {
        let line = line?;
        let fields: Vec<&str> = line.split(';').collect();

        let record = Record {
            schedule_id: fields[0].to_string(),
            fahrzeug: fields[20].to_string(), 
            lon: fields[14].parse().unwrap(),
            lat: fields[15].parse().unwrap(), 
        };
        records.push(record);
    }

    Ok(records)
}


pub fn test_results(output_data: &str, comparison_data: &str) -> io::Result<f64> {
    // Laden der Daten aus den Dateien
    let output_data = load_data_output(output_data)?;
    let comparison_data = load_data_comparison(comparison_data)?;

    // Gesamtanzahl der Datensätze
    let total_records = output_data.len();
    let mut matched_records = 0.0;

    // Durchlaufen der Datensätze in der Ausgabedatei
    for output_record in &output_data {
        for comparison_record in &comparison_data {
            // Überprüfung der Übereinstimmung der ersten Spalte
            if output_record.schedule_id == comparison_record.schedule_id {
                matched_records += 0.333333;
                // Überprüfung der Übereinstimmung der zweiten Spalte
                if output_record.fahrzeug == comparison_record.fahrzeug {
                    matched_records += 0.333333;
                    // Extrahieren der Breitengrade und Längengrade aus der dritten und vierten Spalte
                    let output_lat_lon = Location::new(output_record.lat, output_record.lon);
                    let comparison_lat_lon = Location::new(comparison_record.lat, comparison_record.lon);

                    // Berechnung der Distanz zwischen den Koordinaten und Überprüfung der Toleranz von 50 Metern
                    if let Ok(output_distance) = output_lat_lon.distance_to(&comparison_lat_lon) {
                        if output_distance.meters() <= 20.0 {
                            matched_records += 0.333333;
                            // Behandlung des Falls, wenn die Distanz weniger als oder gleich 20,0 Meter beträgt
                        }
                    }
                }
            }
        }
    }
    // Ausgabe der Anzahl der übereinstimmenden Paare und der Gesamtanzahl der Daten
    println!("Gleiche Paare: {:?}. Gesamte Daten: {:}", matched_records, total_records);
    // Berechnung des prozentualen Anteils der übereinstimmenden Paare an der Gesamtanzahl der Daten
    let percentage = (matched_records as f64 / total_records as f64) * 100.0;
    println!("Prozentuale Differenz: {:.2}%", percentage);
    Ok(percentage)
}

