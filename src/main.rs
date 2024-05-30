mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    mqtt::client::{EspMqttClient, MqttClientConfiguration, QoS},
    nvs::EspDefaultNvsPartition,
};
use log::info;
use std::{thread::sleep, time::Duration};
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let peripherals = Peripherals::take().context("ERROR: failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("ERROR: faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("ERROR: failed to 'take' NVS partition")?;

    let app_config = CONFIG;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    // WARN: does broker url require port??
    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_psk, app_config.mqtt_host,
        )
    } else {
        format!("mqtt://{}:1883", app_config.mqtt_host)
    };

    let mqtt_config = MqttClientConfiguration::default();

    let (mut mqtt_client, _mqtt_connection) =
        EspMqttClient::new(&broker_url, &mqtt_config).unwrap();

    loop {
        println!("in loop...");
        sleep(Duration::from_secs(1));
        println!("Publishing on: {broker_url}");
        let payload = "hello from esp32";
        let _ = mqtt_client
            .publish(
                "wrongcolor/home/default",
                QoS::AtLeastOnce,
                true,
                payload.as_bytes(),
            )
            .unwrap();
        println!("data published!");
    }
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
