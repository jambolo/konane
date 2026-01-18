use konane::import::import_game_from_content;

#[test]
fn import_accepts_valid_opening_sequence() {
    let json = r#"{
        "board_size": 4,
        "moves": [
            {"OpeningRemoval": {"color": "Black", "position": {"row": 1, "col": 1}}},
            {"OpeningRemoval": {"color": "White", "position": {"row": 1, "col": 2}}}
        ]
    }"#;

    let result = import_game_from_content(json);
    assert!(result.is_ok());
}

#[test]
fn import_rejects_invalid_jump() {
    let json = r#"{
        "board_size": 4,
        "moves": [
            {"OpeningRemoval": {"color": "Black", "position": {"row": 1, "col": 1}}},
            {"OpeningRemoval": {"color": "White", "position": {"row": 1, "col": 2}}},
            {"Jump": {"color": "Black", "from": {"row": 0, "col": 0}, "to": {"row": 0, "col": 2}, "captured": [{"row": 0, "col": 1}]}}
        ]
    }"#;

    let result = import_game_from_content(json);
    assert!(result.is_err());
}

#[test]
fn import_rejects_winner_without_game_over() {
    let json = r#"{
        "board_size": 4,
        "winner": "White",
        "moves": [
            {"OpeningRemoval": {"color": "Black", "position": {"row": 1, "col": 1}}},
            {"OpeningRemoval": {"color": "White", "position": {"row": 1, "col": 2}}}
        ]
    }"#;

    let result = import_game_from_content(json);
    assert!(result.is_err());
}
