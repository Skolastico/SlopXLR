use goxlr_ipc::DaemonRequest;

#[test]
fn redesigned_controls_deserialize_using_the_actual_daemon_contract() {
    let packets: Vec<serde_json::Value> = serde_json::from_str(include_str!("../../tests/redesign-commands.json")).unwrap();
    assert!(packets.len() > 70, "expected the migrated controls");
    let mut failures = Vec::new();
    for packet in packets {
        let result = serde_json::from_str::<DaemonRequest>(&packet.to_string());
        if result.is_err() { failures.push(format!("{packet}: {result:?}")); }
    }
    assert!(failures.is_empty(), "invalid frontend commands: {}", failures.join("\n"));
}
