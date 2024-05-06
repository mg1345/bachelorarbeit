use std::fmt::write;
use std::fs;
use std::io;
use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use std::hash::{Hash, Hasher};
use csv::WriterBuilder;
use std::cmp::Eq;
use std::fs::File;
use std::f64::INFINITY;
use std::ops::Deref;
use std::f64::consts::PI;

// Implementierung von Eq und Hash für die Struktur Line
impl Eq for Line {}

impl Hash for Line {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Felder die für die Hashberechnung nötig sind
        self.schedule_id.hash(state);
        self.datum.hash(state);
    }
}

impl<'a> Eq for &'a Bus {}

impl<'a> Hash for &'a Bus {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vehicle.hash(state);
    }
}
use csv::ReaderBuilder;
// Struktur für eine Linie
#[derive(Debug, PartialEq)]
pub struct Line {
    schedule_id: String,
    datum: String,
    frt_fid: u32,
    frt_start: u32,
    line: u32,
    richtung: u32,
    varianten: u32,
    umlauf: u32,
    lfnr: u32,
    ankunft: u32,
    abfahrt: u32,
    zeitpkt: String,
    zeit: u64,
    ort_nr: u32,
    ort_name: String,
    lon: f64,
    lat: f64,
    x: f64,
    y: f64,
    fahrt_start: String,
    fahrt_ende: String,
    wkt: String,
}

// Struktur für einen Bus
#[derive(Debug, PartialEq)]
pub struct Bus {
    vehicle: String,
    datum: String,
    zeit: String,
    zeit_next: String,
    unixzeit: u64,
    lat: f64,
    lon: f64,
    x: f64,
    y: f64,
    typ: String,
    einsteiger: u32,
    aussteiger: u32,
    wkt: String,
}

// Struktur, um eine Kante im Graphen darzustellen
#[derive(Debug)]
pub struct Edge {
    edge_nummer: u64,
    edge_zuordnung: Vec<(String, String)>, // Liste von Haltestellen-Tupeln (Quelle, Ziel)
}

#[derive(Clone)]
pub struct VF2State<'a> {
    lines_graph: &'a HashMap<String, Vec<Line>>,
    buses_graph: &'a HashMap<String, Vec<Bus>>,
    mapping: HashMap<&'a Line, &'a Bus>,
    partial_match: HashMap<(&'a Line, &'a Bus), bool>,
}

pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    const R: f64 = 6371.0; // Radius der Erde in Kilometern

    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    R * c
}

pub fn is_time_compatible(line_time: u64, bus_time: u64) -> bool {
    // Konvertiere line_time von u32 nach u64
    let line_time_u64 = line_time as u64;

    // Definiere die maximale Verspätung oder Verfrühung in Sekunden (hier 10 Minuten)
    let time_tolerance_seconds = 10 * 60;

    // Berechne die Differenz zwischen den Zeitstempeln
    let time_difference = (line_time_u64 as i64 - bus_time as i64).abs();

    // Überprüfe, ob die Differenz innerhalb der Toleranz liegt
    //print!("Zeit:{:?}", time_difference);

    time_difference <= time_tolerance_seconds
}

// Funktion zum Zuordnen von Linien zu Bussen basierend auf den nächsten und zeitlich passenden Koordinaten
pub fn assign_buses_to_lines(lines: &HashMap<String, Vec<Line>>, buses: &HashMap<String, Vec<Bus>>) -> HashMap<String, String> {
    let mut assignment: HashMap<String, String> = HashMap::new();
    let mut i = 0;
    for (line_id, line_vec) in lines.iter() {
        i+=1;
        //println!("Linie: {:?}, {:?}", i, line_id);
        for line in line_vec {
            let mut min_distance = 1000.0;
            let mut closest_bus: Option<&Bus> = None;

            for (bus_id, bus_vec) in buses {
                let mut min_stop_distance = 1000.0;
                for bus_zustand in bus_vec {
                    
                    let distance = calculate_distance(line.lat, line.lon, bus_zustand.lat, bus_zustand.lon);
                    
                    if distance < min_distance && is_time_compatible(line.zeit, bus_zustand.unixzeit) {
                        min_distance = min_stop_distance;
                        //print!("Zeit und Distanz: {:?}:{:?}", line.ankunft, bus_zustand.unixzeit);
                        closest_bus = Some(bus_zustand);
                    }
                }
                
            }
            

            if let Some(bus) = closest_bus {
                //println!("{:?}:{:?}", line_id.clone(), bus.vehicle.clone());
                assignment.insert(line_id.clone(), bus.vehicle.clone());
            }
        }
    }

    assignment
}

pub fn run(line_folder_path: &str, bus_folder_path: &str, output_file: &str) -> io::Result<()> {
    // Einlesen der Linien- und Busdaten aus den CSV-Dateien
    let lines = read_lines_from_csv_folder(line_folder_path)?;
    let buses = read_buses_from_csv_folder(bus_folder_path)?;
    println!("Eingelesen der Busse und Linien");
    // Zuordnen von Linien zu Bussen basierend auf den nächsten und zeitlich passenden Koordinaten
    let assignment = assign_buses_to_lines(&lines, &buses);
    let output_datei_erstellen = write_assignment_to_csv(&assignment, output_file);
    print!("Erstellt der Zuordnung");
    // Ausgabe der zugeordneten Linien und Busse
    for (line_id, bus_id) in assignment {
        println!("Linien-ID: {}, Bus-ID: {}", line_id, bus_id);
    }
    Ok(())
}

// Funktion zum Speichern der Zuordnung in eine CSV-Datei
pub fn save_assignment_to_csv(assignment: &HashMap<&Line, &Bus>, output_file: &str) -> io::Result<()> {
    // Erstellen des CSV-Writers
    let file = fs::File::create(output_file)?;
    let mut writer = WriterBuilder::new().delimiter(b';').from_writer(file);

    // Schreiben der Header-Zeile
    writer.write_record(&["Line_Schedule_ID", "Bus_Vehicle"])?;

    // Schreiben der Zuordnungen
    for (line, bus) in assignment {
        writer.serialize((&line.schedule_id, &bus.vehicle))?;
    }

    // Abschließen des Schreibvorgangs
    writer.flush()?;
    Ok(())
}

pub fn write_assignment_to_csv(assignment: &HashMap<String, String>, output_file: &str) -> io::Result<()> {
    let mut writer = csv::Writer::from_writer(File::create(output_file)?);

    // Schreibe die Überschrift
    writer.write_record(&["Line_ID", "Bus_ID"])?;

    // Schreibe die Zuordnungen
    for (line_id, bus_id) in assignment {
        writer.write_record(&[line_id, bus_id])?;
    }

    writer.flush()?;
    Ok(())
}

// Funktion zum Einlesen von Linieninformationen aus CSV-Dateien und Erstellen eines Graphen
pub fn read_lines_from_csv_folder(folder_path: &str) -> io::Result<HashMap<String, Vec<Line>>> {
    let mut lines_graph: HashMap<String, Vec<Line>> = HashMap::new();
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.is_file() && file_path.extension().unwrap_or_default() == "csv" {
            let mut lines = Vec::new();
            let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();
            let mut rdr = ReaderBuilder::new().delimiter(b';').from_path(&file_path)?;
            for result in rdr.records() {
                let record = result?;
                let line = Line {
                    schedule_id: record[0].to_string(),
                    datum: record[1].to_string(),
                    frt_fid: record[2].parse().unwrap(),
                    frt_start: record[3].parse().unwrap(),
                    line: record[4].parse().unwrap(),
                    richtung: record[5].parse().unwrap(),
                    varianten: record[6].parse().unwrap(),
                    umlauf: record[7].parse().unwrap(),
                    lfnr: record[8].parse().unwrap(),
                    ankunft: record[9].parse().unwrap(),
                    abfahrt: record[10].parse().unwrap(),
                    zeitpkt: record[11].to_string(),
                    zeit: record[12].parse().unwrap(),
                    ort_nr: record[13].parse().unwrap(),
                    ort_name: record[14].to_string(),
                    lon: record[15].parse().unwrap(),
                    lat: record[16].parse().unwrap(),
                    x: record[17].parse().unwrap(),
                    y: record[18].parse().unwrap(),
                    fahrt_start: record[19].to_string(),
                    fahrt_ende: record[20].to_string(),
                    wkt: record[21].to_string(),
                };
                lines.push(line);
            }
            lines_graph.entry(file_stem).or_insert(Vec::new()).extend(lines);
        }
    }
    Ok(lines_graph)
}

pub fn read_buses_from_csv_folder(folder_path: &str) -> io::Result<HashMap<String, Vec<Bus>>> {
    let mut buses_graph: HashMap<String, Vec<Bus>> = HashMap::new();
    
    // Durchlaufe alle Dateien im angegebenen Ordner
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let file_path = entry.path();
        
        // Prüfe, ob die Datei eine CSV-Datei ist
        if file_path.is_file() && file_path.extension().unwrap_or_default() == "csv" {
            let mut buses = Vec::new();
            let file_stem = file_path.file_stem().unwrap().to_string_lossy().to_string();
            
            // Öffne die CSV-Datei und lese die Zeilen ein
            let mut rdr = ReaderBuilder::new().delimiter(b';').from_path(&file_path)?;
            for result in rdr.records().skip(1) { // Überspringe Header-Zeile
                let record = result?;
                
                // Erstelle ein Bus-Objekt aus den Daten der Zeile und füge es zum Vektor hinzu
                let bus = Bus {
                    vehicle: record[0].to_string(),
                    datum: record[1].to_string(),
                    zeit: record[2].to_string(),
                    zeit_next: record[3].to_string(),
                    unixzeit: record[4].parse().unwrap(),
                    lat: record[5].parse().unwrap(),
                    lon: record[6].parse().unwrap(),
                    x: record[7].parse().unwrap(),
                    y: record[8].parse().unwrap(),
                    typ: record[9].to_string(),
                    einsteiger: record[10].parse().unwrap(),
                    aussteiger: record[11].parse().unwrap(),
                    wkt: record[12].to_string(),
                };
                buses.push(bus);
            }
            
            // Füge den Vektor zum Graphen hinzu, wobei der Dateiname als Schlüssel verwendet wird
            buses_graph.entry(file_stem).or_insert(Vec::new()).extend(buses);
        }
    }
    
    Ok(buses_graph)
}

pub fn print_lines(lines_graph: &HashMap<String, Vec<Line>>) {
    for (file_name, lines) in lines_graph.iter() {
        println!("Linien aus der Datei {}: ", file_name);
        thread::sleep(Duration::from_secs(1));
        /*for (index, line) in buses.iter().enumerate() {
            println!("Bus {}:", index + 1);
            println!("{:?}", bus);
            thread::sleep(Duration::from_secs(1));
        }*/
    }
}

pub fn print_buses(buses_graph: &HashMap<String, Vec<Bus>>) {
    for (file_name, buses) in buses_graph.iter() {
        println!("Busse aus der Datei {}: ", file_name);
        thread::sleep(Duration::from_secs(1));

        /*for (index, bus) in buses.iter().enumerate() {
            println!("Bus {}:", index + 1);
            println!("{:?}", bus);
            thread::sleep(Duration::from_secs(1));
        }*/

    }
}