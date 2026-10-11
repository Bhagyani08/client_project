// SPDX-FileCopyrightText: 2025 Contributors to the Eclipse Foundation
//
// SPDX-License-Identifier: Apache-2.0
//
// Generates a native CAN UDS MDD file containing:
//   - DiagnosticSessionControl (SID 0x10) — Default / Extended / Programming
//   - ReadDataByIdentifier      (SID 0x22) — reads VIN (DID 0xF190)
//   - WriteDataByIdentifier     (SID 0x2E) — writes config DID 0x0100
//   - TesterPresent             (SID 0x3E) — keep-alive
//
// Usage (from the workspace root):
//   cargo run -p cda-database --example generate_can_mdd --features database-builder \
//       -- testcontainer/odx/E400.mdd

use std::path::PathBuf;

use cda_database::datatypes::{
    DataType, ResponseType,
    database_builder::{
        Addressing, DiagClassType, DiagCommParams, DiagLayerParams, DiagServiceParams,
        EcuDataBuilder, EcuDataParams, TransmissionMode,
    },
};
use cda_interfaces::uds::service_ids;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output_path: PathBuf = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("testcontainer/odx/E400.mdd"));

    let ecu_name = output_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("CAN_ECU")
        .to_owned();

    let mut b = EcuDataBuilder::new();

    // ── Protocol (CAN / ISO-15765) ─────────────────────────────────────────
    let protocol = b.create_protocol("ISO_15765_3_on_ISO_15765_2", None, None, None);

    // ═══════════════════════════════════════════════════════════════════════
    // SESSION STATE CHART
    // ═══════════════════════════════════════════════════════════════════════
    let default_state     = b.create_state("DefaultSession",     Some("Default Diagnostic Session"));
    let extended_state    = b.create_state("ExtendedSession",    Some("Extended Diagnostic Session"));
    let programming_state = b.create_state("ProgrammingSession", Some("Programming Session"));

    let t_def_to_ext  = b.create_state_transition(
        "DefaultToExtended",   Some("DefaultSession"),  Some("ExtendedSession"));
    let t_def_to_prog = b.create_state_transition(
        "DefaultToProgramming", Some("DefaultSession"),  Some("ProgrammingSession"));
    let t_ext_to_prog = b.create_state_transition(
        "ExtendedToProgramming", Some("ExtendedSession"), Some("ProgrammingSession"));

    let session_chart = b.create_state_chart(
        "SessionControl",
        Some("SESSION"),
        Some(vec![t_def_to_ext, t_def_to_prog, t_ext_to_prog]),
        Some("DefaultSession"),
        Some(vec![default_state, extended_state, programming_state]),
    );

    // ═══════════════════════════════════════════════════════════════════════
    // DiagnosticSessionControl  (SID 0x10)
    // ═══════════════════════════════════════════════════════════════════════
    let session_sub_dct   = b.create_diag_coded_type_standard_length(8, DataType::UInt32);
    let session_sub_compu = b.create_compu_method_identical();
    let session_sub_dop   = b.create_regular_normal_dop("SessionType", session_sub_dct, session_sub_compu);

    let session_req = {
        let sid = b.create_coded_const_param(
            "SID_RQ", &service_ids::SESSION_CONTROL.to_string(), 0, 0, 8, DataType::UInt32);
        let sub = b.create_value_param("SessionType", session_sub_dop, 1, 0);
        b.create_request(Some(vec![sid, sub]), None)
    };
    let session_pos = {
        let sid = b.create_coded_const_param(
            "SID_PR", &(service_ids::SESSION_CONTROL | 0x40u8).to_string(), 0, 0, 8, DataType::UInt32);
        let sub = b.create_value_param("SessionType", session_sub_dop, 1, 0);
        b.create_response(ResponseType::Positive, Some(vec![sid, sub]), None)
    };
    let session_neg = {
        let sid = b.create_coded_const_param(
            "SID_NR", &service_ids::NEGATIVE_RESPONSE.to_string(), 0, 0, 8, DataType::UInt32);
        b.create_response(ResponseType::Negative, Some(vec![sid]), None)
    };

    let t_def_to_ext_ref = b.create_state_transition_ref(t_def_to_ext);
    let t_def_to_prog_ref = b.create_state_transition_ref(t_def_to_prog);
    let session_dc = b.create_diag_comm(DiagCommParams {
        short_name: "DiagnosticSessionControl",
        semantic: Some("SESSION"),
        diag_class_type: DiagClassType::START_COMM,
        state_transition_refs: Some(vec![t_def_to_ext_ref, t_def_to_prog_ref]),
        protocols: Some(vec![protocol]),
        ..Default::default()
    });
    let session_svc = b.create_diag_service(DiagServiceParams {
        diag_comm: Some(session_dc),
        request: Some(session_req),
        pos_responses: vec![session_pos],
        neg_responses: vec![session_neg],
        addressing: *Addressing::PHYSICAL,
        transmission_mode: *TransmissionMode::SEND_AND_RECEIVE,
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // ReadDataByIdentifier  (SID 0x22 / DID 0xF190 = VIN, 17 ASCII bytes)
    // ═══════════════════════════════════════════════════════════════════════
    let vin_dct   = b.create_diag_coded_type_standard_length(136, DataType::AsciiString); // 17 × 8
    let vin_compu = b.create_compu_method_identical();
    let vin_dop   = b.create_regular_normal_dop("VIN", vin_dct, vin_compu);

    let rdbi_req = {
        let sid = b.create_coded_const_param(
            "SID_RQ", &service_ids::READ_DATA_BY_IDENTIFIER.to_string(), 0, 0, 8, DataType::UInt32);
        let did = b.create_coded_const_param(
            "DataIdentifier", "61840", // 0xF190
            1, 0, 16, DataType::UInt32);
        b.create_request(Some(vec![sid, did]), None)
    };
    let rdbi_pos = {
        let sid = b.create_coded_const_param(
            "SID_PR", &(service_ids::READ_DATA_BY_IDENTIFIER | 0x40u8).to_string(), 0, 0, 8, DataType::UInt32);
        let did = b.create_coded_const_param(
            "DataIdentifier", "61840", 1, 0, 16, DataType::UInt32);
        let val = b.create_value_param("VINValue", vin_dop, 3, 0);
        b.create_response(ResponseType::Positive, Some(vec![sid, did, val]), None)
    };
    let rdbi_neg = {
        let sid = b.create_coded_const_param(
            "SID_NR", &service_ids::NEGATIVE_RESPONSE.to_string(), 0, 0, 8, DataType::UInt32);
        b.create_response(ResponseType::Negative, Some(vec![sid]), None)
    };

    let rdbi_dc = b.create_diag_comm(DiagCommParams {
        short_name: "ReadVIN",
        semantic: Some("DATA"),
        diag_class_type: DiagClassType::START_COMM,
        protocols: Some(vec![protocol]),
        ..Default::default()
    });
    let rdbi_svc = b.create_diag_service(DiagServiceParams {
        diag_comm: Some(rdbi_dc),
        request: Some(rdbi_req),
        pos_responses: vec![rdbi_pos],
        neg_responses: vec![rdbi_neg],
        addressing: *Addressing::PHYSICAL,
        transmission_mode: *TransmissionMode::SEND_AND_RECEIVE,
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // WriteDataByIdentifier  (SID 0x2E / DID 0x0100 — config DID, 2 bytes)
    // Precondition: Extended session must be active.
    // ═══════════════════════════════════════════════════════════════════════
    let cfg_dct   = b.create_diag_coded_type_standard_length(16, DataType::UInt32);
    let cfg_compu = b.create_compu_method_identical();
    let cfg_dop   = b.create_regular_normal_dop("ConfigValue", cfg_dct, cfg_compu);

    let wdbi_req = {
        let sid = b.create_coded_const_param(
            "SID_RQ", &service_ids::WRITE_DATA_BY_IDENTIFIER.to_string(), 0, 0, 8, DataType::UInt32);
        let did = b.create_coded_const_param(
            "DataIdentifier", "256", // 0x0100
            1, 0, 16, DataType::UInt32);
        let val = b.create_value_param("ConfigValue", cfg_dop, 3, 0);
        b.create_request(Some(vec![sid, did, val]), None)
    };
    let wdbi_pos = {
        let sid = b.create_coded_const_param(
            "SID_PR", &(service_ids::WRITE_DATA_BY_IDENTIFIER | 0x40u8).to_string(), 0, 0, 8, DataType::UInt32);
        let did = b.create_coded_const_param(
            "DataIdentifier", "256", 1, 0, 16, DataType::UInt32);
        b.create_response(ResponseType::Positive, Some(vec![sid, did]), None)
    };
    let wdbi_neg = {
        let sid = b.create_coded_const_param(
            "SID_NR", &service_ids::NEGATIVE_RESPONSE.to_string(), 0, 0, 8, DataType::UInt32);
        b.create_response(ResponseType::Negative, Some(vec![sid]), None)
    };

    let wdbi_pre_cond = b.create_pre_condition_state_ref(extended_state);
    let wdbi_dc = b.create_diag_comm(DiagCommParams {
        short_name: "WriteConfigValue",
        semantic: Some("DATA"),
        diag_class_type: DiagClassType::START_COMM,
        protocols: Some(vec![protocol]),
        pre_condition_state_refs: Some(vec![wdbi_pre_cond]),
        ..Default::default()
    });
    let wdbi_svc = b.create_diag_service(DiagServiceParams {
        diag_comm: Some(wdbi_dc),
        request: Some(wdbi_req),
        pos_responses: vec![wdbi_pos],
        neg_responses: vec![wdbi_neg],
        addressing: *Addressing::PHYSICAL,
        transmission_mode: *TransmissionMode::SEND_AND_RECEIVE,
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // TesterPresent  (SID 0x3E)
    // ═══════════════════════════════════════════════════════════════════════
    let tp_req = {
        let sid = b.create_coded_const_param(
            "SID_RQ", &service_ids::TESTER_PRESENT.to_string(), 0, 0, 8, DataType::UInt32);
        b.create_request(Some(vec![sid]), None)
    };
    let tp_pos = {
        let sid = b.create_coded_const_param(
            "SID_PR", &(service_ids::TESTER_PRESENT | 0x40u8).to_string(), 0, 0, 8, DataType::UInt32);
        b.create_response(ResponseType::Positive, Some(vec![sid]), None)
    };
    let tp_neg = {
        let sid = b.create_coded_const_param(
            "SID_NR", &service_ids::NEGATIVE_RESPONSE.to_string(), 0, 0, 8, DataType::UInt32);
        b.create_response(ResponseType::Negative, Some(vec![sid]), None)
    };
    let tp_dc = b.create_diag_comm(DiagCommParams {
        short_name: "TesterPresent",
        semantic: Some("DATA"),
        diag_class_type: DiagClassType::START_COMM,
        protocols: Some(vec![protocol]),
        ..Default::default()
    });
    let tp_svc = b.create_diag_service(DiagServiceParams {
        diag_comm: Some(tp_dc),
        request: Some(tp_req),
        pos_responses: vec![tp_pos],
        neg_responses: vec![tp_neg],
        addressing: *Addressing::PHYSICAL,
        transmission_mode: *TransmissionMode::SEND_AND_RECEIVE,
        ..Default::default()
    });

    // ═══════════════════════════════════════════════════════════════════════
    // Assemble: DiagLayer → Variant → EcuData → MddFile
    // ═══════════════════════════════════════════════════════════════════════
    let cp_ref = b.create_com_param_ref(None, None, None, Some(protocol), None);

    let diag_layer = b.create_diag_layer(DiagLayerParams {
        short_name: &ecu_name,
        com_param_refs: Some(vec![cp_ref]),
        diag_services: Some(vec![session_svc, rdbi_svc, wdbi_svc, tp_svc]),
        state_charts: Some(vec![session_chart]),
        ..Default::default()
    });

    let variant = b.create_variant(diag_layer, true, None, None);

    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write the actual MDD file using the new helper
    b.write_mdd_file(
        EcuDataParams {
            ecu_name: &ecu_name,
            revision: "1",
            version: "1.0.0",
            variants: Some(vec![variant]),
            ..Default::default()
        },
        output_path.to_str().unwrap()
    )?;

    println!(
        "✅  Generated CAN MDD: {}",
        output_path.display()
    );
    println!("   Services included:");
    println!("     • DiagnosticSessionControl (SID 0x10) — Default / Extended / Programming");
    println!("     • ReadDataByIdentifier     (SID 0x22) — VIN (DID 0xF190)");
    println!("     • WriteDataByIdentifier    (SID 0x2E) — Config DID 0x0100  [requires Extended session]");
    println!("     • TesterPresent            (SID 0x3E)");

    Ok(())
}
