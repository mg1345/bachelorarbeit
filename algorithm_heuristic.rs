use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;
use std::fs;
use std::path::PathBuf;

// Datenstruktur für Koordinaten
#[derive(Debug, Clone, Copy)]
pub struct Coordinate {
    lat: f64,
    lon: f64,
}

// Datenstruktur für eine Haltestelle
#[derive(Debug)] // Diese Zeile hinzufügen
pub struct Stop {
    coordinate: Coordinate,
    time: u64, // Unix-Zeitstempel
}

// Datenstruktur für eine Linie
#[derive(Debug)] // Diese Zeile hinzufügen
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
    stops: Vec<Stop>, // Vektorfeld für Stopps
}

// Datenstruktur für eine Buslinie
#[derive(Debug)] 
pub struct BusLine {
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
    stops: Vec<Stop>,
}

pub fn calculate_optimal_bus_line<'a>(line: &'a Line, bus_lines: &'a [BusLine]) -> Option<&'a BusLine> {
    // Initialisierung der minimalen Distanz-Zeit-Produkt-Variable als unendlich
    let mut min_distance_time_product = std::f64::INFINITY;
    // Initialisierung des am nächsten gelegenen Busses als Option ohne Wert
    let mut closest_bus: Option<&BusLine> = None;

    // Durchlaufen aller Busdaten
    for bus_line in bus_lines {
        // Berechnung der Distanz zwischen den Koordinaten des Busses und der Linie
        let distance = calculate_distance(
            &Coordinate { lat: bus_line.lat, lon: bus_line.lon },
            &Coordinate { lat: line.lat, lon: line.lon }
        );
        // Berechnung des Zeitunterschieds zwischen der Zeit des Busses und der Linie
        let time_difference = (bus_line.unixzeit as i64 - line.zeit as i64).abs() as u64;
        // Berechnung des Distanz-Zeit-Produkts
        let distance_time_product = distance * time_difference as f64;

        // Überprüfung, ob das aktuelle Distanz-Zeit-Produkt kleiner als das bisher kleinste ist
        if distance_time_product < min_distance_time_product {
            // Aktualisierung des minimalen Distanz-Zeit-Produkts und des am nächsten gelegenen Busses
            min_distance_time_product = distance_time_product;
            closest_bus = Some(bus_line);
        }
    }

    // Rückgabe des am nächsten gelegenen Busses
    closest_bus
}

// Funktion zur Berechnung der Entfernung zwischen zwei Koordinaten mit der euklidischen Distanzformel
pub fn calculate_distance(coord1: &Coordinate, coord2: &Coordinate) -> f64 {
    const R: f64 = 6371.0; // Radius der Erde in Kilometern

    let d_lat = (coord2.lat - coord1.lat).to_radians();
    let d_lon = (coord2.lon - coord1.lon).to_radians();

    let a = (d_lat / 2.0).sin().powi(2) + coord1.lat.to_radians().cos() * coord2.lat.to_radians().cos() * (d_lon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    let distance = R * c;
    distance
}

// Hauptfunktion des Algorithmus
pub fn run_algorithm(lines: Vec<Line>, bus_lines: Vec<BusLine>, output_file: &str) -> io::Result<()> {
    // Öffnen Sie die Ausgabedatei im Schreibmodus
    let mut output_file = File::create(output_file)?;
    // Schreiben Sie die Header-Zeile in die Datei
    writeln!(output_file, "SCHEDULE_ID;DATUM;FRT_ID;FRT_START;LINE;RICHTUNG;VARIANTE;UMLAUF;LFD_NR;SOLL_ABFAHRT_ZEIT_TIME;ORT_NR;ORT_NAME;LON;LAT;X;Y;FRT_START_TIME;FRT_END_TIME;FAHRZEUG;LFD_NR;IST_ANKUNFT_TIME;IST_ABFAHRT_TIME;Einsteiger;Aussteiger;WKT")?;

    // Iterieren Sie über alle Linien
    for line in lines {
        // Überprüfen Sie, ob eine optimale Buslinie für die aktuelle Linie gefunden wurde
        if let Some(optimal_bus_line) = calculate_optimal_bus_line(&line, &bus_lines) {
            // Schreiben Sie die Daten der aktuellen Linie und der optimalen Buslinie in die Datei
            writeln!(output_file, "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
         line.schedule_id, line.datum, line.frt_fid, line.frt_start, line.line, line.richtung, line.varianten, line.umlauf, line.lfnr,
         line.abfahrt, line.ort_nr, line.ort_name, line.lon, line.lat, line.x, line.y, line.fahrt_start, line.fahrt_ende,
         optimal_bus_line.vehicle, line.lfnr, optimal_bus_line.zeit, optimal_bus_line.zeit_next,
         optimal_bus_line.einsteiger, optimal_bus_line.aussteiger, optimal_bus_line.wkt)?;
        }
    }
    // Rückgabe eines Erfolgsindikators
    Ok(())
}

// Funktion zum Lesen der Linien-CSV-Datei und Erstellen der Linienstruktur
pub fn read_lines(file_path: &str) -> io::Result<Vec<Line>> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    let mut lines: Vec<Line> = Vec::new();

    for line in reader.lines().skip(1) {
        let line_data = line?;
        let fields: Vec<&str> = line_data.split(';').collect();

        // Extrahiere die Spaltenwerte
        let schedule_id = fields[0].to_string();
        let datum = fields[1].to_string();
        let frt_fid = fields[2].parse().unwrap();
        let frt_start = fields[3].parse().unwrap();
        let line_num = fields[4].parse().unwrap();
        let richtung = fields[5].parse().unwrap();
        let varianten = fields[6].parse().unwrap();
        let umlauf = fields[7].parse().unwrap();
        let lfnr = fields[8].parse().unwrap();
        let ankunft = fields[9].parse().unwrap();
        let abfahrt = fields[10].parse().unwrap();
        let zeitpkt = fields[11].to_string();
        let zeit = fields[12].parse().unwrap();
        let ort_nr = fields[13].parse().unwrap();
        let ort_name = fields[14].to_string();
        let lon = fields[15].parse().unwrap();
        let lat = fields[16].parse().unwrap();
        let x = fields[17].parse().unwrap();
        let y = fields[18].parse().unwrap();
        let fahrt_start = fields[19].to_string();
        let fahrt_ende = fields[20].to_string();
        let wkt = fields[21].to_string();

        // Erstelle ein neues Line-Objekt und füge es zum Vector hinzu
        let line = Line {
            schedule_id,
            datum,
            frt_fid,
            frt_start,
            line: line_num,
            richtung,
            varianten,
            umlauf,
            lfnr,
            ankunft,
            abfahrt,
            zeitpkt,
            zeit,
            ort_nr,
            ort_name,
            lon,
            lat,
            x,
            y,
            fahrt_start,
            fahrt_ende,
            wkt,
            stops: Vec::new(), // Leeres Vektorfeld für Stopps
        };
        lines.push(line);
    }

    Ok(lines)
}

// Funktion zum Einlesen aller Busdateien in einem Ordner
pub fn read_bus_files(folder_path: &str) -> io::Result<Vec<BusLine>> {
    let mut all_bus_data: Vec<BusLine> = Vec::new();

    // Durchlaufe alle Einträge im Ordner
    for entry in fs::read_dir(folder_path)? {
        let entry = entry?;
        let path = entry.path();

        // Überprüfe, ob der Eintrag eine Datei ist
        if path.is_file() {
            // Überprüfe die Dateierweiterung
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if extension == "csv" {
                    let file = File::open(&path)?;
                    let reader = io::BufReader::new(file);
                    
                    // Vektor für die Buslinien dieser Datei
                    let mut bus_lines: Vec<BusLine> = Vec::new();

                    for line in reader.lines().skip(1) {
                        let line_data = line?;
                        let fields: Vec<&str> = line_data.split(';').collect();

                        // Extrahiere die Spaltenwerte wie zuvor
                        let vehicle = fields[0].to_string();
                        let datum = fields[1].to_string();
                        let zeit = fields[2].to_string();
                        let zeit_next = fields[3].to_string();
                        let unixzeit = fields[4].parse().unwrap();
                        let lat = fields[5].parse().unwrap();
                        let lon = fields[6].parse().unwrap();
                        let x = fields[7].parse().unwrap();
                        let y = fields[8].parse().unwrap();
                        let typ = fields[9].to_string();
                        let einsteiger = fields[10].parse().unwrap();
                        let aussteiger = fields[11].parse().unwrap();
                        let wkt = fields[12].to_string();

                        // Erstelle ein neues BusLine-Objekt und füge es zum Vector hinzu
                        let bus_line = BusLine {
                            vehicle,
                            datum,
                            zeit,
                            zeit_next,
                            unixzeit,
                            lat,
                            lon,
                            x,
                            y,
                            typ,
                            einsteiger,
                            aussteiger,
                            wkt,
                            stops: Vec::new(),
                        };
                        //println!("{:?}", bus_line);
                        bus_lines.push(bus_line);
                    }

                    // Füge alle Buslinien dieser Datei zum Vektor aller Buslinien hinzu
                    all_bus_data.extend(bus_lines);
                }
            }
        }
    }

    Ok(all_bus_data)
}

// Funktion zum Einlesen einer einzelnen Busdatei
pub fn read_bus_file(file_path: &Path) -> io::Result<Vec<BusLine>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut bus_lines: Vec<BusLine> = Vec::new();

    for line in reader.lines().skip(1) {
        let line_data = line?;
        let fields: Vec<&str> = line_data.split(';').collect();

        // Extrahiere die Spaltenwerte wie zuvor
        let vehicle = fields[0].to_string();
        let datum = fields[1].to_string();
        let zeit = fields[2].to_string();
        let zeit_next = fields[3].to_string();
        let unixzeit = fields[4].parse().unwrap();
        let lat = fields[5].parse().unwrap();
        let lon = fields[6].parse().unwrap();
        let x = fields[7].parse().unwrap();
        let y = fields[8].parse().unwrap();
        let typ = fields[9].to_string();
        let einsteiger = fields[10].parse().unwrap();
        let aussteiger = fields[11].parse().unwrap();
        let wkt = fields[12].to_string();

        // Erstelle ein neues BusLine-Objekt und füge es zum Vector hinzu
        let bus_line = BusLine {
            vehicle,
            datum,
            zeit,
            zeit_next,
            unixzeit,
            lat,
            lon,
            x,
            y,
            typ,
            einsteiger,
            aussteiger,
            wkt,
            stops: Vec::new(),
        };
        println!("{:?}", bus_line);
        bus_lines.push(bus_line);
    }
    
    Ok(bus_lines)
}
