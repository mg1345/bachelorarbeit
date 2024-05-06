use std::fs;
use std::io;
use std::collections::{HashMap};
use std::io::{Write, BufReader, BufRead};
use std::time::Duration;
use csv::{ReaderBuilder, WriterBuilder};
use std::fs::File;
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::thread;

// Implementierung von Eq und Hash für die Struktur Line
impl Eq for Line {}

impl Hash for Line {
    fn hash<H: Hasher>(&self, state: &mut H) {
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

use std::ops::Deref;

use std::f64::consts::PI;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Assignment {
    pub line_id: String,
    pub bus_id: String,
}

pub fn is_delay_acceptable(line_time: u64, bus_time: u64) -> bool {
    // Definiere die maximale Zeitabweichung in Sekunden (hier 10 Minuten)
    const MAX_DELAY_SECONDS: u64 = 5 * 60;

    // Berechne die Differenz zwischen den Zeitstempeln
    let time_difference = (line_time as i64 - bus_time as i64).abs();

    // Überprüfe, ob die Differenz innerhalb der maximalen Verspätung liegt
    time_difference <= MAX_DELAY_SECONDS as i64
}

pub fn read_assignment_from_csv_folder(assignment_file: &str) -> io::Result<HashMap<Assignment, String>> {
    // Öffne die angegebene Zuweisungsdatei
    let file = File::open(assignment_file)?;

    // Erstelle einen gepufferten Reader für effizientes Lesen
    let reader = BufReader::new(file);

    // Initialisiere eine HashMap zum Speichern von Zuweisungen und ihren zugehörigen Bus-IDs
    let mut assignment_map: HashMap<Assignment, String> = HashMap::new();

    // Erstelle einen CSV-Reader mit Headern und dem Trennzeichen als Komma
    let mut csv_reader = ReaderBuilder::new()
        .has_headers(true)
        .delimiter(b',')
        .from_reader(reader);

    // Iteriere über jeden Datensatz in der CSV-Datei und überspringe die Headerzeile
    for result in csv_reader.records().skip(1) {
        // Extrahiere den Datensatz
        let record = result?;

        // Hole das Line_ID-Feld oder gib einen Fehler zurück, wenn es fehlt
        let line_id = record.get(0)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Fehlende Line_ID"))?
            .to_string();

        // Hole das Bus_ID-Feld oder gib einen Fehler zurück, wenn es fehlt
        let bus_id = record.get(1)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Fehlende Bus_ID"))?
            .to_string();

        // Erstelle einen Zuweisungseintrag mit Line_ID und Bus_ID
        let assignment_entry = Assignment { line_id: line_id.clone(), bus_id: bus_id.clone() };

        // Füge die Zuweisung der HashMap hinzu
        assignment_map.insert(assignment_entry, bus_id);
    }

    // Gib die HashMap zurück, die Zuweisungen und ihre Bus-IDs enthält
    Ok(assignment_map)
}



pub fn calculate_distance(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    // Radius der Erde in Kilometern
    const R: f64 = 6371.0;

    // Umrechnung der Differenz der Breitengrade und Längengrade in Radiant
    let d_lat = (lat2 - lat1).to_radians();
    let d_lon = (lon2 - lon1).to_radians();

    // Berechnung der Formel für die haversine-Formel
    let a = (d_lat / 2.0).sin().powi(2) + lat1.to_radians().cos() * lat2.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    // Berechnung der Gesamtdistanz
    R * c
}

// Funktion zur Überprüfung, ob die Zeit kompatibel ist
pub fn is_time_compatible(line_time: u64, bus_time: u64) -> bool {
    // Konvertiere line_time von u32 nach u64
    let line_time_u64 = line_time as u64;

    // Definiere die maximale Verspätung oder Verfrühung in Sekunden (hier 10 Minuten)
    let time_tolerance_seconds = 10 * 60;

    // Berechne die Differenz zwischen den Zeitstempeln
    let time_difference = (line_time_u64 as i64 - bus_time as i64).abs();

    // Überprüfe, ob die Differenz innerhalb der Toleranz liegt
    time_difference <= time_tolerance_seconds
}

pub fn run(line_folder_path: &str, bus_folder_path: &str, assignment_file: &str) -> io::Result<()> {
    // Einlesen der Linien-, Busgraphen und der Zuordnung aus den CSV-Dateien
    let lines = read_lines_from_csv_folder(line_folder_path)?;
    let buses = read_buses_from_csv_folder(bus_folder_path)?;
    
    let assignment = read_assignment_from_csv_folder(assignment_file)?;
    println!("ZUORDNUNG BEENDET");
    // Finden der nächstgelegenen Koordinaten
    let nearest_coordinates = find_nearest_coordinates(&lines, &buses, &assignment);
    // Der Rückgabewert der Funktion write_nearest_coordinates_to_file ist ein Result, 
    // deshalb wird das Ergebnis mit '?' verarbeitet.
    write_nearest_coordinates_to_file(&nearest_coordinates, 
        "/Users/martin/Desktop/Bachelorarbeit/test/daten_filtern/ergebnis_vf2.csv");

    Ok(())
}

// Funktion zum Finden der nächsten Koordinate für jede Haltestelle
pub fn find_nearest_coordinates(lines: &HashMap<String, Vec<Line>>, buses: &HashMap<String, Vec<Bus>>, assignment: &HashMap<Assignment, String>) -> Vec<((
    String, String, u32, u32, u32, u32, u32, u32, u32, u32, u32, String, u64, String, u32, String, String
), (
    String, String, String, u64, f64, f64, f64, f64, String, u32, u32, String,
))> {
    let mut nearest_coordinates: Vec<_> = Vec::new();
    let mut output_file = "/Users/martin/Desktop/Bachelorarbeit/test/daten_filtern/ergebnis.csv";
    let mut i = 0;
    // Durchlaufen der Zuordnungen
    for (assignment_entry, bus_id) in assignment.iter() {
        i+=1;
        //println!("Linie: {:?}, {:?}", i, zuordnung_entry);
        let line_vec = lines.get(&assignment_entry.line_id).unwrap_or_else(|| panic!("Line with ID {} not found", &assignment_entry.line_id));
        let bus_vec = buses.get(bus_id).unwrap_or_else(|| panic!("Bus with ID {} not found", bus_id));
        
        // Durchlaufen der Linien und Finden der nächsten Koordinate für jede Haltestelle
        for line in line_vec {
            // Durchlaufen der Haltestellen
            let stop_coordinate = (line.lat, line.lon);
            let mut min_distance = f64::INFINITY;
            let mut nearest_bus_coordinate: Option<(f64, f64)> = None;
            let mut nearest_bus_info: Option<((String, String, u32, u32, u32, u32, u32, u32, u32, u32, u32, String, 
                                            u64, String, u32, String, String),
                                            (String, String, String, u64, f64, f64, f64, f64, String, u32, 
                                            u32, String))> = None;                                   

            // Durchlaufen der Bus-Koordinaten
            for bus in bus_vec {
                for bus_coordinate in &[(bus.lat, bus.lon)] {
                    let distance = calculate_distance(stop_coordinate.0, stop_coordinate.1, bus_coordinate.0, bus_coordinate.1);
                    
                    if distance < min_distance {
                        // Überprüfen, ob die Zeit kompatibel ist und die Verspätung nicht zu groß ist
                        if is_time_compatible(line.zeit, bus.unixzeit) && is_delay_acceptable(line.zeit, bus.unixzeit) {
                            min_distance = distance;
                            nearest_bus_info = Some(((line.schedule_id.clone(), line.datum.clone(), 
                                                    line.frt_fid.clone(), line.frt_start.clone(), line.line.clone(),
                                                    line.richtung.clone(), line.varianten.clone(), line.umlauf.clone(),
                                                    line.lfnr.clone(), line.ankunft.clone(), line.abfahrt.clone(),
                                                    line.zeitpkt.clone(), line.zeit.clone(), line.ort_name.clone(),
                                                    line.ort_nr.clone(), line.fahrt_start.clone(), line.fahrt_ende.clone()), 
                                                    (bus.vehicle.clone(), bus.zeit.clone(),
                                                    bus.zeit_next.clone(), bus.unixzeit.clone(), bus.lat.clone(), 
                                                    bus.lon.clone(), bus.x.clone(), bus.y.clone(), bus.typ.clone(), 
                                                    bus.einsteiger.clone(), bus.aussteiger.clone(), bus.wkt.clone())
                                                )); // Hier alle benötigten Informationen hinzufügen
                        }
                    }
                }
            }    
            if let Some(info) = nearest_bus_info {
                nearest_coordinates.push(info);
            }
        }
    }
    nearest_coordinates
}

pub fn write_nearest_coordinates_to_file(nearest_coordinates: &Vec<((
    String, String, u32, u32, u32, u32, u32, u32, u32, u32, u32, String, u64, String, u32, String, String
), (
    String, String, String, u64, f64, f64, f64, f64, String, u32, u32, String,
))>, output_file: &str) -> io::Result<()> {
    let mut file = File::create(output_file)?;
    println!("Datei schreiben");
    writeln!(file, "SCHEDULE_ID;Datum;Frt_Fid;Frt_Start;Line;Richtung;Varianten;Umlauf;LFD_NR;Ankunft;Abfahrt;Sollabfahrtzeit;Zeit;Ort_Name;Ort_Nr;Fahrt_Start;Fahrt_Ende;Vehicle;Zeit;Zeit_Next;Unixzeit;Lat;Lon;X;Y;Typ;Einsteiger;Aussteiger;WKT")?; // Header schreiben

    for (line_info, bus_info) in nearest_coordinates {

        // Schreibe die Details von Line und Bus in die Datei
        writeln!(
            file,
            "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
            line_info.0,
            line_info.1,
            line_info.2,
            line_info.3,
            line_info.4,
            line_info.5,
            line_info.6,
            line_info.7,
            line_info.8,
            line_info.9,
            line_info.10,
            line_info.11,
            line_info.12,
            line_info.13,
            line_info.14,
            line_info.15,
            line_info.16,
            bus_info.0,
            bus_info.1,
            bus_info.2,
            bus_info.3,
            bus_info.4,
            bus_info.5,
            bus_info.6,
            bus_info.7,
            bus_info.8,
            bus_info.9,
            bus_info.10,
            bus_info.11,
        )?;
    }

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
            let vehicle_name = file_stem.split('.').next().unwrap().to_string();
            
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
            buses_graph.entry(vehicle_name.clone()).or_insert(Vec::new()).extend(buses);
        }
    }
    
    Ok(buses_graph)
}

pub fn print_lines(lines_graph: &HashMap<String, Vec<Line>>) {
    for (file_name, lines) in lines_graph.iter() {
        println!("Linien aus der Datei {}: {:?}", file_name, lines.get(1));
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
        println!("ANZAHL: {:?}", buses_graph.len());
        println!("Busse aus der Datei {}: {:?}", file_name, buses.get(1));
        thread::sleep(Duration::from_secs(1));

        /*for (index, bus) in buses.iter().enumerate() {
            println!("Bus {}:", index + 1);
            println!("{:?}", bus);
            thread::sleep(Duration::from_secs(1));
        }*/

    }
}