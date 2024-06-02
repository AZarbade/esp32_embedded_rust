//! MQTT helper module
//!
//! credits to: <https://github.com/esp-rs/esp-idf-svc/blob/master/examples/mqtt_client.rs>
use anyhow::Result;
use esp_idf_svc::mqtt::client::{EspMqttClient, EspMqttConnection, MqttClientConfiguration, QoS};
use log::info;
use std::time::Duration;

/// Creates a new MQTT client and connection with the provided configuration.
///
/// This function sets up an MQTT client and connection based on the specified broker URL and
/// configuration options.
///
/// # Arguments
///
/// * `broker_url` - A string slice representing the URL of the MQTT broker to connect to.
/// * `client_id` - An optional string slice specifying the client ID for the MQTT connection.
/// * `username` - An optional string slice specifying the username for authenticating with the MQTT broker.
/// * `password` - An optional string slice specifying the password for authenticating with the MQTT broker.
///
/// # Returns
///
/// * `Ok((EspMqttClient<'static>, EspMqttConnection))` - A tuple containing the `EspMqttClient` instance
///   and the `EspMqttConnection` instance, both wrapped in static lifetimes, if the client and connection
///   were created successfully.
/// * `Err(EspError)` - An `EspError` if there was an error creating the MQTT client or connection.
///
/// # Examples
///
/// ```
/// let (client, connection) = mqtt_create("mqtt://broker.example.com", Some("my_client"), None, None).unwrap();
/// ```
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

/// Tests the MQTT server status by publishing a message to a topic and subscribing to the same topic.
///
/// This function is intended for testing purposes only and should not be used in production code.
/// It creates a new thread to listen for MQTT events and then enters a loop where it publishes
/// a message to a topic and subscribes to the same topic.
///
/// # Arguments
///
/// * `client` - A mutable reference to an `EspMqttClient` instance.
/// * `connection` - A mutable reference to an `EspMqttConnection` instance.
///
/// # Returns
///
/// * `Ok(())` if the test was successful.
/// * `Err(EspError)` if there was an error during the test.
///
/// # Variables
///
/// * `topic` - A string literal representing the MQTT topic to publish and subscribe to. Its value is set to `"home/default"`.
/// * `payload` - A string literal representing the message payload to be published. Its value is set to `"hello from MQTT"`.
///
/// # Warning
///
/// This function is intended for testing purposes only and should not be used in production code.
///
/// # Examples
///
/// ```
/// let (mut client, mut connection) = mqtt_create("mqtt://broker.example.com", Some("test_client"), None, None).unwrap();
/// mqtt_test(&mut client, &mut connection).unwrap();
/// ```
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
