# MQTT Driver Quick Start Guide

## 5-Minute Setup

### Prerequisites
- ESNODE agent installed
- MQTT broker (e.g., Mosquitto) running
- IoT sensors publishing to MQTT topics

### Step 1: Install Mosquitto (if needed)

**macOS:**
```bash
brew install mosquitto
brew services start mosquitto
```

**Ubuntu/Debian:**
```bash
sudo apt update
sudo apt install mosquitto mosquitto-clients
sudo systemctl start mosquitto
```

**Docker:**
```bash
docker run -d -p 1883:1883 eclipse-mosquitto
```

### Step 2: Configure ESNODE

Create or edit `esnode.toml`:

```toml
[[drivers]]
protocol = "mqtt"
id = "mqtt-sensors"
target = "localhost:1883"

[drivers.params]
topics = "sensors/#"
mappings = "sensors/+/temperature:temperature:celsius:value:1.0"
```

### Step 3: Start ESNODE

```bash
./esnode-core daemon --config esnode.toml
```

### Step 4: Publish Test Data

```bash
mosquitto_pub -h localhost -t "sensors/room1/temperature" -m '{"value": 23.5}'
mosquitto_pub -h localhost -t "sensors/room2/temperature" -m '{"value": 25.2}'
```

### Step 5: View Metrics

```bash
curl http://localhost:9100/metrics | grep iot_sensor
```

**Expected Output:**
```
esnode_iot_sensor_value{driver_id="mqtt-sensors",sensor_type="temperature",unit="celsius",param="topic",topic="sensors/room1/temperature"} 23.5
esnode_iot_sensor_value{driver_id="mqtt-sensors",sensor_type="temperature",unit="celsius",param="topic",topic="sensors/room2/temperature"} 25.2
```

## Common Use Cases

### 1. Temperature Monitoring

```toml
[[drivers]]
protocol = "mqtt"
id = "mqtt-temperature"
target = "mqtt.local:1883"

[drivers.params]
topics = "datacenter/temperature/#"
mappings = "datacenter/temperature/+:temperature:celsius:value:1.0"
```

```bash
# Publish
mosquitto_pub -t "datacenter/temperature/row1" -m '{"value": 24.5}'
```

### 2. Multi-Sensor Dashboard

```toml
[[drivers]]
protocol = "mqtt"
id = "mqtt-multi"
target = "mqtt.local:1883"

[drivers.params]
topics = "sensors/#"
mappings = "sensors/+/temp:temperature:celsius:value:1.0,sensors/+/humidity:other:percent:value:1.0,sensors/+/power:power:watts:value:1.0"
```

```bash
# Publish multiple sensors
mosquitto_pub -t "sensors/rack1/temp" -m '{"value": 25.0}'
mosquitto_pub -t "sensors/rack1/humidity" -m '{"value": 45.0}'
mosquitto_pub -t "sensors/rack1/power" -m '{"value": 1250.0}'
```

### 3. Nested JSON Data

```toml
[[drivers]]
protocol = "mqtt"
id = "mqtt-nested"
target = "mqtt.local:1883"

[drivers.params]
topics = "devices/#"
mappings = "devices/+/status:temperature:celsius:data.temperature:1.0"
```

```bash
# Publish nested JSON
mosquitto_pub -t "devices/sensor1/status" -m '{"data": {"temperature": 26.5, "timestamp": "2026-02-10T22:00:00Z"}}'
```

## Troubleshooting

### Issue: No metrics showing
**Check:**
1. MQTT broker is running: `mosquitto -v`
2. Topics match configuration: `mosquitto_sub -t "sensors/#" -v`
3. JSON structure matches `value_path`

### Issue: Connection refused
**Solution:**
```bash
# Check broker is listening
netstat -an | grep 1883

# Test connection
mosquitto_pub -h localhost -t test -m "hello"
```

### Issue: Authentication failed
**Solution:**
```toml
[drivers.params]
username = "your_username"
password = "your_password"
```

## Next Steps

1. **Add authentication** - Secure your MQTT broker
2. **Set up TLS** - Encrypt MQTT traffic (coming soon)
3. **Create dashboards** - Visualize in Grafana
4. **Scale up** - Add more sensors and topics

## Resources

- **MQTT Specification**: [MQTT.org](https://mqtt.org/)
- **Mosquitto Docs**: [mosquitto.org/documentation](https://mosquitto.org/documentation/)
- **ESNODE Docs**: `README.md`

---

**Questions?** Check the full documentation in `crates/esnode-mqtt/README.md`
