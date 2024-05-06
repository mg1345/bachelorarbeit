import csv
import os
import time

def read_and_sort_csv(input_file):
    # Dictionary zur Aufbewahrung der Daten für jedes Fahrzeug
    vehicle_data = {}

    with open(input_file, 'r', newline='') as csvfile:
        reader = csv.reader(csvfile, delimiter=';')
        header = next(reader)  # Überspringen der Kopfzeile

        for row in reader:
            vehicle = row[header.index('vehicle')]  # Fahrzeug aus der Zeile extrahieren

            # Überprüfen, ob das Fahrzeug bereits im Dictionary vorhanden ist
            if vehicle in vehicle_data:
                vehicle_data[vehicle].append(row)
            else:
                vehicle_data[vehicle] = [row]

    # Ordner für die sortierten CSV-Dateien erstellen, falls nicht vorhanden
    output_folder = '/Users/martin/Downloads/diagnose/sorted_csv_python_comparison'
    if not os.path.exists(output_folder):
        os.makedirs(output_folder)

    # Für jedes Fahrzeug die Daten in eine separate CSV-Datei schreiben
    for vehicle, data in vehicle_data.items():
        output_file = os.path.join(output_folder, f'{vehicle}.csv')
        with open(output_file, 'w', newline='') as csvfile:
            writer = csv.writer(csvfile, delimiter=';')
            writer.writerow(header)  # Kopfzeile schreiben
            writer.writerows(data)   # Daten für das jeweilige Fahrzeug schreiben

    print("Sortierung und Schreiben der CSV-Dateien abgeschlossen.")

# Beispielaufruf

start_time = time.time()  # Startzeit messen
input_file = '/Users/martin/Downloads/diagnose/rohdaten_20231025.csv'  # Name der Eingabedatei
read_and_sort_csv(input_file)
end_time = time.time()  # Endzeit messen
elapsed_time = end_time - start_time  # Gesamtdauer berechnen
print(f"Dauer: {elapsed_time:.2f} Sekunden")