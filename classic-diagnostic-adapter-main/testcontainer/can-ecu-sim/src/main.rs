
use std::env;
use tokio_socketcan_isotp::IsoTpSocket;
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let interface = env::var("SIM_CAN_INTERFACE").unwrap_or_else(|_| "socketcand:socketcand:29536:vcan0".to_string());
    let vin = env::var("OVERRIDE_VIN").unwrap_or_else(|_| "E400-PURE-CAN-SIM".to_string());
    
    // The CDA sends requests to 0x7E0, so our RX is 0x7E0.
    // The CDA expects responses from 0x7E8, so our TX is 0x7E8.
    let rx_id = tokio_socketcan_isotp::StandardId::new(0x7E0).unwrap();
    let tx_id = tokio_socketcan_isotp::StandardId::new(0x7E8).unwrap();

    info!("Starting E400 Pure CAN Simulator on interface {}", interface);

    // Open ISO-TP Socket
    let mut socket = IsoTpSocket::open(&interface, rx_id, tx_id)?;
    info!("Socket opened successfully. Waiting for UDS requests...");

    let mut session: u8 = 0x01; // Default session

    loop {
        if let Ok(data) = socket.read_packet().await {
            if data.is_empty() {
                continue;
            }

            let sid = data[0];
            info!("Received UDS SID 0x{:02X}, len: {}", sid, data.len());

            match sid {
                // 0x10 Diagnostic Session Control
                0x10 => {
                    if data.len() >= 2 {
                        session = data[1];
                        info!("Session changed to 0x{:02X}", session);
                        let response = vec![0x50, session, 0x00, 0x32, 0x01, 0xF4];
                        socket.write_packet(&response).await?;
                    }
                }
                // 0x22 Read Data By Identifier
                0x22 => {
                    if data.len() >= 3 {
                        let did = ((data[1] as u16) << 8) | (data[2] as u16);
                        if did == 0xF190 {
                            // VIN DID
                            info!("Read VIN DID 0xF190 requested.");
                            let mut response = vec![0x62, 0xF1, 0x90];
                            response.extend_from_slice(vin.as_bytes());
                            socket.write_packet(&response).await?;
                        } else {
                            warn!("Unknown DID 0x{:04X} requested.", did);
                            // NRC 31: Request out of range
                            socket.write_packet(&[0x7F, 0x22, 0x31]).await?;
                        }
                    }
                }
                // 0x2E Write Data By Identifier
                0x2E => {
                    if data.len() >= 3 {
                        let did = ((data[1] as u16) << 8) | (data[2] as u16);
                        if did == 0x0100 {
                            if session == 0x03 || session == 0x02 { // Extended or Programming
                                info!("Write Config DID 0x0100 accepted in session 0x{:02X}.", session);
                                socket.write_packet(&[0x6E, 0x01, 0x00]).await?;
                            } else {
                                warn!("Write Config DID 0x0100 rejected (Session 0x{:02X} != Extended).", session);
                                // NRC 7F: Service Not Supported In Active Session
                                socket.write_packet(&[0x7F, 0x2E, 0x7F]).await?;
                            }
                        } else {
                            socket.write_packet(&[0x7F, 0x2E, 0x31]).await?;
                        }
                    }
                }
                // 0x3E Tester Present
                0x3E => {
                    info!("Tester Present received.");
                    socket.write_packet(&[0x7E, 0x00]).await?;
                }
                _ => {
                    warn!("Unsupported SID 0x{:02X}", sid);
                    // NRC 11: Service Not Supported
                    socket.write_packet(&[0x7F, sid, 0x11]).await?;
                }
            }
        }
    }
}
