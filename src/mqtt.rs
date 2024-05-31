// credits to: https://github.com/esp-rs/esp-idf-svc/blob/master/examples/mqtt_client.rs
use anyhow::Result;
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration, QoS};
use log::info;
use std::time::Duration;

pub fn mqtt_create(
    broker_url: &str,
    client_id: Option<&str>,
    username: Option<&str>,
    password: Option<&str>,
) -> Result<(EspMqttClient<'static>, EspMqttConnection)> {
    let mqtt_config = MqttClientConfiguration {
        client_id,
        username,
        password,
        ..Default::default()
    };
    let (mqtt_client, mqtt_connection) = EspMqttClient::new(&broker_url, &mqtt_config)?;

    Ok((mqtt_client, mqtt_connection))
}

pub fn mqtt_test(client: &mut EspMqttClient, connection: &mut EspMqttConnection) -> Result<()> {
    std::thread::scope(|s| {
        info!("Preparing to start MQTT Client");

        std::thread::Builder::new()
            .stack_size(6_000)
            .spawn_scoped(s, move || {
                info!("[MQTT] listening for event changes");

                while let Ok(event) = connection.next() {
                    info!("[Queue] Event: {}", event.payload());
                }

                info!("[MQTT] Connection closed!");
            })
            .unwrap();

        let topic = "home/default";
        let payload = "hello from MQTT";

        client.subscribe(topic, QoS::AtMostOnce)?;
        info!("[MQTT] Subscribed to topic: {topic}");
        std::thread::sleep(Duration::from_secs(2));

        loop {
            client.enqueue(topic, QoS::AtMostOnce, false, payload.as_bytes())?;
            info!("[MQTT] published \"{payload}\" to topic \"{topic}\"");
            std::thread::sleep(Duration::from_secs(2));
        }
    })
}
