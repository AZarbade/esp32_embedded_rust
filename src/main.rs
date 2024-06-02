//! This project is experimental only and should not be used in production code.
//! This tracks my learning progress for embedded Rust on ESP32.
pub mod mqtt;
pub mod wifi;

use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{self, attenuation::DB_11},
        gpio::*,
        peripherals::Peripherals,
    },
    mqtt::client::QoS,
    nvs::EspDefaultNvsPartition,
};
use log::info;
use mqtt::mqtt_create;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let app_config = CONFIG;
    let peripherals = Peripherals::take().context("failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("failed to 'take' NVS partition")?;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    info!("Setting up heart-rate sensor pin...");
    let mut pin_heart = adc::AdcDriver::new(
        peripherals.adc1,
        &adc::config::Config::new().calibration(true),
    )
    .context("failed to new ADC Driver")?;
    let mut pin = adc::AdcChannelDriver::<'_, DB_11, Gpio36>::new(peripherals.pins.gpio36)
        .context("failed to set ADC Pin")?;

    let sensor_reading = SensorReading {
        reading_1: pin_heart.read(&mut pin)?,
        reading_2: pin_heart.read(&mut pin)?,
    };

    info!("Setting up MQTT parameters...");
    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_psk, app_config.mqtt_host,
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    let (client, mut connection) = mqtt_create(&broker_url, None, None, None)?;
    let client = Arc::new(Mutex::new(client));

    std::thread::scope(|s| {
        info!("[MQTT] starting event listner");

        // TODO: what is stack_size?
        std::thread::Builder::new()
            .stack_size(6_000)
            .spawn_scoped(s, move || {
                info!("[MQTT: Event] listening for event changes");
                while let Ok(event) = connection.next() {
                    info!("[MQTT: Queue] Event: {}", event.payload());
                }
                info!("[MQTT: Event] Connection closed!");
            })
            .unwrap();

        // Mandetory waiting, to give event thread time for setup
        info!("[MQTT] waiting for event thread to setup...");
        std::thread::sleep(std::time::Duration::from_millis(500));

        let client_clone_1 = Arc::clone(&client);
        std::thread::Builder::new()
            .stack_size(6_000)
            .spawn_scoped(s, move || {
                let topic = "sensor/foo_1";
                let payload = serde_json::to_string(&sensor_reading.reading_1).unwrap();
                loop {
                    info!("[MQTT: Publisher] initializing publisher on topic: {topic}");
                    client_clone_1
                        .lock()
                        .unwrap()
                        .enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())
                        .unwrap();
                    info!("[MQTT: Publisher] published \"{payload}\" to topic \"{topic}\"");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            })
            .unwrap();

        let client_clone_2 = Arc::clone(&client);
        std::thread::Builder::new()
            .stack_size(6_000)
            .spawn_scoped(s, move || {
                let topic = "sensor/foo_2";
                let payload = serde_json::to_string(&sensor_reading.reading_2).unwrap();
                loop {
                    info!("[MQTT: Publisher] initializing publisher on topic: {topic}");
                    client_clone_2
                        .lock()
                        .unwrap()
                        .enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())
                        .unwrap();
                    info!("[MQTT: Publisher] published \"{payload}\" to topic \"{topic}\"");
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            })
            .unwrap();
    });

    Ok(())
}

#[derive(Serialize, Deserialize)]
struct SensorReading {
    reading_1: u16,
    reading_2: u16,
}

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    mqtt_user: &'static str,
    #[default("")]
    mqtt_psk: &'static str,
    #[default("broker.local")]
    mqtt_host: &'static str,
}
