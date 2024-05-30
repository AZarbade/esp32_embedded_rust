mod wifi;
use anyhow::{Context, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        adc::{self, attenuation::DB_11},
        gpio::*,
        peripherals::Peripherals,
    },
    mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration, QoS},
    nvs::EspDefaultNvsPartition,
};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Duration;
use wifi::wifi;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    info!("Getting started...");
    let app_config = CONFIG;
    let peripherals = Peripherals::take().context("failed to 'take' peripheral control")?;
    let sysloop = EspSystemEventLoop::take().context("faild to 'take' event loop")?;
    let nvs = EspDefaultNvsPartition::take().context("failed to 'take' NVS partition")?;

    info!("Setting up ADC pin...");
    let mut adc1 = adc::AdcDriver::new(
        peripherals.adc1,
        &adc::config::Config::new().calibration(true),
    )
    .context("failed to new ADC Driver")?;
    let mut pin = adc::AdcChannelDriver::<'_, DB_11, Gpio36>::new(peripherals.pins.gpio36)
        .context("failed to set ADC Pin")?;

    info!("Setting up wifi...");
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
        Some(nvs),
    );

    info!("Setting up MQTT parameters...");
    let broker_url = if app_config.mqtt_user != "" {
        format!(
            "mqtt://{}:{}@{}",
            app_config.mqtt_user, app_config.mqtt_psk, app_config.mqtt_host,
        )
    } else {
        format!("mqtt://{}", app_config.mqtt_host)
    };

    let (mut client, mut connection) = mqtt_create(&broker_url, None)?;

    loop {
        let reading = adc1.read(&mut pin)?;
        let sensor_reading = SensorReading {
            heart_rate: reading,
        };

        let json_string = serde_json::to_string(&sensor_reading)?;

        mqtt_run(
            &mut client,
            &mut connection,
            "home/sensors/heart",
            json_string,
        )?;
    }
}

fn mqtt_run(
    client: &mut EspMqttClient,
    connection: &mut EspMqttConnection,
    topic: &str,
    payload: String,
) -> Result<()> {
    std::thread::scope(|s| {
        info!("Preparing to start MQTT Client");

        // TODO: wtf is stack_size?

        // This block is event watching
        std::thread::Builder::new()
            .stack_size(6_000)
            .spawn_scoped(s, move || {
                info!("[MQTT] listening for event changes");

                while let Ok(event) = connection.next() {
                    info!("[Queue] Event: {}", event.payload());
                }

                info!("Connection closed!");
            })
            .unwrap();

        client.subscribe(topic, QoS::AtMostOnce)?;
        info!("[MQTT] Subscribed to topic: {topic}");
        std::thread::sleep(Duration::from_millis(500));

        loop {
            client.enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())?;
            info!("[MQTT] published \"{payload}\" to topic \"{topic}\"");
            std::thread::sleep(Duration::from_millis(500));
        }
    })
}

fn mqtt_create(
    broker_url: &str,
    client_id: Option<&str>,
) -> Result<(EspMqttClient<'static>, EspMqttConnection)> {
    let mqtt_config = MqttClientConfiguration {
        client_id,
        ..Default::default()
    };
    let (mqtt_client, mqtt_connection) = EspMqttClient::new(&broker_url, &mqtt_config)?;

    Ok((mqtt_client, mqtt_connection))
}

#[derive(Serialize, Deserialize)]
struct SensorReading {
    heart_rate: u16,
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
