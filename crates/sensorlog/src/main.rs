use anyhow::{Result, Context};
use std::thread;
use std::time::Duration;
use std::sync::mpsc::{Receiver};
use std::process::Command;
use sensorlog::{SensorConfig, SensorAccessInfo, spawn_sensor_reader}; // Assuming you have your module imported here.

fn main() -> Result<()> {
    // Initialize the configuration with default values
    let cfg = SensorConfig::default();

    // Start the sensor reader in a separate thread and get the receiver
    let rx = spawn_sensor_reader(cfg);

    // Display events in real-time
    println!("Starting sensor log monitoring...\n");

    loop {
        match rx.recv() {
            Ok(event) => {
                // Print each event's details
                println!(
                    "[{}] Sensor: {}, Access Type: {}, Process: {:?}, Last Used: {:?}",
                    event.timestamp,
                    event.sensor,
                    event.access_type,
                    event.process,
                    event.last_used_unix
                );
            }
            Err(_) => {
                // If there's an issue receiving events, continue
                eprintln!("Error receiving event, retrying...");
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}