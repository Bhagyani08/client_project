# Software-Defined Vehicle (SDV) Diagnostic Platform

A unified, end-to-end diagnostic and fault management ecosystem for Software-Defined Vehicles (SDVs), integrating modern **OpenSOVD (ASAM / Eclipse Foundation)** service-oriented diagnostics, **ISO 14229 (UDS)** classic diagnostic bridging, and high-performance **Diagnostic Fault Management (DFM)**.

---

## 🏛️ System Architecture

The platform connects offboard/onboard/cloud SOVD clients to High Performance Computers (HPC1, HPC2) and Classic Microcontroller ECUs (MCU1) through a centralized SOVD Gateway and translation layers:

```mermaid
graph TB
    subgraph Clients["Diagnostic Clients"]
        Client["SOVD Client<br/>(Offboard, Onboard, Cloud)"]
        UDSTester["UDS Tester<br/>(Legacy Tester)"]
    end

    subgraph HPC1["HPC 1 (High Performance Computer)"]
        Gateway["SOVD Gateway<br/>(:7690)"]
        UDSProxy["UDS2SOVD Proxy"]
        SOVDServer1["SOVD Server (HPC1)"]
        DFM1["Diagnostic Fault Manager (DFM)"]
        FaultLib1["Fault Lib"]
        DiagDB1[("Diagnostic DB")]
        ServiceApp1["Service Apps"]
        
        CDA["Classic Diagnostic Adapter (CDA)<br/>(:20002)"]
    end

    subgraph HPC2["HPC 2 (High Performance Computer)"]
        SOVDServer2["SOVD Server (HPC2)"]
        DFM2["Diagnostic Fault Manager (DFM)"]
        FaultLib2["Fault Lib"]
        DiagDB2[("Diagnostic DB")]
        ServiceApp2["Service Apps"]
    end

    subgraph MCUs["Classic ECUs / Microcontrollers"]
        MCU1["MCU 1 / CAN ECU (E400)<br/>Pure CAN / ISO-TP"]
        DoIPECUs["DoIP ECUs<br/>(FLXC1000, FSNR2000...)"]
    end

    %% Client connections
    Client -->|SOVD / HTTP REST| Gateway
    UDSTester -->|UDS| UDSProxy
    UDSProxy -->|SOVD| Gateway

    %% Gateway Routing
    Gateway -->|SOVD| SOVDServer1
    Gateway -->|SOVD| SOVDServer2
    Gateway -->|SOVD| CDA

    %% HPC1 Internal Flow
    FaultLib1 -->|IPC (iceoryx2)| DFM1
    DFM1 <-->|IPC| DiagDB1
    DFM1 -->|IPC| SOVDServer1
    ServiceApp1 <-->|IPC| SOVDServer1

    %% HPC2 Internal Flow
    FaultLib2 -->|IPC (iceoryx2)| DFM2
    DFM2 <-->|IPC| DiagDB2
    DFM2 -->|IPC| SOVDServer2
    ServiceApp2 <-->|IPC| SOVDServer2

    %% Classic Adapter to ECUs
    CDA -->|UDS over CAN / ISO-TP| MCU1
    CDA -->|UDS over DoIP| DoIPECUs
```

---

## 📦 Project Structure

```
client_project/
├── opensovd-core-main/               # OpenSOVD Core Implementation (Rust)
│   ├── opensovd-gateway/             # Central SOVD Gateway & dynamic routing
│   ├── opensovd-server/              # Native SOVD server implementation
│   ├── opensovd-client/              # SOVD client SDK & CLI
│   ├── opensovd-models/              # ASAM SOVD data models and schemas
│   ├── Dockerfile.gateway            # Docker container for SOVD Gateway
│   └── Dockerfile.client             # Docker container for SOVD Client
│
├── classic-diagnostic-adapter-main/  # Classic Diagnostic Adapter (CDA)
│   ├── cda-main/                     # Main CDA daemon & REST API (port 20002)
│   ├── cda-comm-can/                 # CAN bus transport (SocketCAN & socketcand)
│   ├── cda-comm-doip/                # DoIP transport (ISO 13400)
│   ├── cda-database/                 # ODX / MDD database engine & builder
│   └── testcontainer/                # End-to-end integration test environment
│       ├── docker-compose.yml        # Orchestrates all containers (profile: can)
│       ├── socketcand/               # socketcand daemon (vcan0 on 29536, vcan1 on 29537)
│       ├── can-ecu-sim/              # Rust standalone pure CAN ECU simulator (E400)
│       ├── ecu-sim/                  # Kotlin multi-ECU DoIP & CAN simulator
│       ├── cda-test-config.toml      # CDA runtime configuration for CAN testing
│       ├── odx/                      # MDD / ODX diagnostic databases (E400.mdd, FLXC1000...)
│       └── test_readvin.sh           # Automated integration test script
│
├── fault-lib-main/                   # Diagnostic Fault Library
│   ├── src/fault_lib/                # Client reporter API for applications (IPC)
│   ├── src/dfm_lib/                  # Diagnostic Fault Manager (ISO 14229 lifecycle)
│   ├── src/common/                   # FaultRecord, FaultCatalog, debounce algorithms
│   └── examples/                     # DFM and SOVD fault manager examples
│
└── README.md                         # Project documentation
```

---

## 🧩 Key Components

### 1. OpenSOVD Core (`opensovd-core-main`)
- **`opensovd-gateway`**: Acts as the central vehicle API entry point (`http://localhost:7690/sovd`). It handles authentication, aggregates components across multiple SOVD servers (HPC1, HPC2) and adapters (CDA), and proxies requests to the appropriate target backend.
- **`opensovd-server`**: Implements the ASAM OpenSOVD standard on High-Performance Computers. Exposes vehicle components, data properties, operations, locks, and fault logs.
- **`opensovd-client`**: Provides a programmatic Rust client and command-line interface to interact with OpenSOVD gateways and servers.

### 2. Classic Diagnostic Adapter (`classic-diagnostic-adapter-main`)
- Translates RESTful SOVD HTTP requests (`/vehicle/v15/...`) to Classic UDS (ISO 14229) diagnostic communication.
- **Transports**: Supports DoIP (ISO 13400) and CAN / ISO-TP (ISO 15765-2 / ISO 15765-3 via SocketCAN or `socketcand`).
- **Database Engine**: Ingests ODX 2.2 / MDD files to validate services, encode request parameters, and parse response payloads.
- **ECU Simulators**:
  - `can-ecu-sim`: Pure CAN ECU simulator for ECU `E400` listening on CAN IDs `0x7E0` (request) / `0x7E8` (response). Supports:
    - Diagnostic Session Control (`0x10`)
    - Read Data By Identifier (`0x22`, DID `0xF190` = VIN)
    - Write Data By Identifier (`0x2E`, DID `0x0100` = Config)
  - `ecu-sim`: Kotlin DoIP and CAN simulator simulating standard ECUs (`FLXC1000`, `FSNR2000`, `TMCC3000`).

### 3. Diagnostic Fault Library (`fault-lib-main`)
- Standardized fault management for applications running on Linux/QNX High Performance Computers.
- Implements the complete **ISO 14229 (UDS) fault lifecycle**:
  - Test Failed / Test Failed This Cycle
  - Pending DTC, Confirmed DTC
  - Warning Indicator Requested
  - Trip completion and auto-clearing
- Uses ultra-low-latency zero-copy shared memory IPC (**iceoryx2**) between Service Apps and the Diagnostic Fault Manager.

---

## 🚀 Quickstart & Testing

### Prerequisites
- Docker & Docker Compose
- `vcan` kernel module enabled on the Linux host (for CAN testing):
  ```bash
  sudo modprobe vcan
  ```
- `curl` and `jq`

---

### Step 1: Start the Integration Test Environment

From `classic-diagnostic-adapter-main/testcontainer`, start all services under the `can` profile:

```bash
cd classic-diagnostic-adapter-main/testcontainer
docker-compose --profile can up -d --build
```

This starts:
1. **`socketcand`**: Virtual CAN bus daemon (`vcan0` on port 29536)
2. **`socketcand-vcan1`**: Isolated virtual CAN bus daemon (`vcan1` on port 29537)
3. **`can-ecu-sim`**: Pure CAN ECU simulator for ECU `E400` (listening on `vcan1`)
4. **`ecu-sim`**: Kotlin DoIP/CAN simulator
5. **`cda`**: Classic Diagnostic Adapter listening on port 20002 (mapped to host)
6. **`gateway`**: OpenSOVD Gateway listening on port 7690
7. **`client`**: OpenSOVD client container

Check container status:
```bash
docker-compose --profile can ps
```

---

### Step 2: Test Direct Communication with CDA

#### 1. Retrieve an Authorization Token
```bash
export CDA_PORT=36095 # Or host port mapped to 20002 (see `docker ps`)
export TOKEN=$(curl -s -X POST "http://localhost:${CDA_PORT}/vehicle/v15/authorize" \
  -H "Content-Type: application/json" \
  -d '{"client_id": "test_client", "client_secret": "test_secret"}' | jq -r .access_token)

echo "Token: $TOKEN"
```

#### 2. Discover Available Components
```bash
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://localhost:${CDA_PORT}/vehicle/v15/components" | jq .
```

#### 3. Acquire Diagnostic Lock
```bash
LOCK_RESP=$(curl -s -X POST "http://localhost:${CDA_PORT}/vehicle/v15/components/e400/locks" \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"lock_expiration": 60000, "reason": "testing"}')

LOCK_ID=$(echo $LOCK_RESP | jq -r .id)
echo "Lock ID: $LOCK_ID"
```

#### 4. Read VIN over CAN (UDS 0x22 DID 0xF190)
```bash
curl -s -H "Authorization: Bearer $TOKEN" \
  "http://localhost:${CDA_PORT}/vehicle/v15/components/e400/data/ReadVIN" | jq .
```
*Expected Response:* Returns the simulated VIN value (e.g. `"E400-PURE-CAN-SIM"` or value configured via `OVERRIDE_VIN`).

#### 5. Release Diagnostic Lock
```bash
curl -s -X DELETE "http://localhost:${CDA_PORT}/vehicle/v15/components/e400/locks/${LOCK_ID}" \
  -H "Authorization: Bearer $TOKEN"
```

---

### Step 3: Test Communication Through SOVD Gateway

To test the complete routing flow:
**Client ➔ SOVD Gateway (port 7690) ➔ CDA / SOVD Servers ➔ CAN ECU**:

```bash
# 1. Authorize via Gateway
export GW_TOKEN=$(curl -s -X POST "http://localhost:7690/vehicle/v15/authorize" \
  -H "Content-Type: application/json" \
  -d '{"client_id": "test_client", "client_secret": "test_secret"}' | jq -r .access_token)

# 2. Query components aggregated by the Gateway
curl -s -H "Authorization: Bearer $GW_TOKEN" \
  "http://localhost:7690/vehicle/v15/components" | jq .

# 3. Request E400 data through the Gateway
curl -s -H "Authorization: Bearer $GW_TOKEN" \
  "http://localhost:7690/vehicle/v15/components/e400/data/ReadVIN" | jq .
```

---

### Step 4: Run the Automated Test Script

A ready-to-run bash script is located in `classic-diagnostic-adapter-main/testcontainer`:

```bash
chmod +x classic-diagnostic-adapter-main/testcontainer/test_readvin.sh
./classic-diagnostic-adapter-main/testcontainer/test_readvin.sh
```

---

## ⚙️ Configuration Reference

### CDA Configuration (`cda-test-config.toml`)

- **Server Bind**:
  ```toml
  [server]
  transport = "tcp"
  address = "0.0.0.0"
  port = 20002
  ```

- **CAN Bus Mapping**:
  ```toml
  [can]
  interface = "socketcand:socketcand-vcan1:29537:vcan1"
  response_timeout_ms = 2000
  keepalive_interval_ms = 2000

  [[can.ecu_mappings]]
  ecu_name = "E400"
  request_id = 0x7E0
  response_id = 0x7E8
  ```

- **Per-ECU Protocol Override**:
  For pure CAN ECUs using ISO-15765 rather than DoIP:
  ```toml
  [ecu.E400]
  protocol = "ISO_15765_3_on_ISO_15765_2"
  ```

- **Simulator VIN Override**:
  In `can-ecu-sim`, specify the `OVERRIDE_VIN` environment variable in `docker-compose.yml`:
  ```yaml
  can-ecu-sim:
    environment:
      - SIM_CAN_INTERFACE=socketcand:socketcand-vcan1:29537:vcan1
      - OVERRIDE_VIN=MYCUSTOMVIN123456
  ```

---

## 📜 Standards & Compliance
- **ASAM OpenSOVD**: API specification for service-oriented vehicle diagnostics over HTTP/REST.
- **ISO 14229-1 (UDS)**: Unified Diagnostic Services.
- **ISO 13400-2 (DoIP)**: Diagnostic communication over Internet Protocol.
- **ISO 15765-2 / ISO 15765-3**: Diagnostics on Controller Area Network (CAN / ISO-TP).