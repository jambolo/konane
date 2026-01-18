use serde::Deserialize;

use crate::game::rules::Jump;
use crate::game::{GamePhase, GameState, MoveRecord, PieceColor, Position, Rules};

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct ImportedGame {
    pub board_size: usize,
    pub winner: Option<String>,
    pub moves: Vec<MoveRecord>,
}

pub fn import_game_from_path(path: &str) -> Result<(GameState, Vec<GameState>), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("Failed to read file: {}", err))?;
    import_game_from_content(&content)
}

pub fn import_game_from_content(content: &str) -> Result<(GameState, Vec<GameState>), String> {
    let imported: ImportedGame = serde_json::from_str(content)
        .map_err(|err| format!("Invalid JSON: {}", err))?;

    validate_board_size(imported.board_size)?;

    let mut state = GameState::new(imported.board_size, PieceColor::Black);
    let mut history = Vec::new();

    for (index, record) in imported.moves.into_iter().enumerate() {
        let move_number = index + 1;
        history.push(state.clone());
        validate_and_apply_move(&mut state, record, move_number)?;
    }

    validate_winner(&state, imported.winner)?;

    Ok((state, history))
}

fn validate_board_size(board_size: usize) -> Result<(), String> {
    if !(4..=16).contains(&board_size) || !board_size.is_multiple_of(2) {
        return Err("Invalid board_size: must be even and between 4 and 16".to_string());
    }
    Ok(())
}

fn validate_and_apply_move(
    state: &mut GameState,
    record: MoveRecord,
    move_number: usize,
) -> Result<(), String> {
    match record {
        MoveRecord::OpeningRemoval { color, position } => {
            validate_opening_removal(state, color, position, move_number)?;
            Rules::apply_opening_removal(state, position)
                .map_err(|err| format!("Move {}: {}", move_number, err))?;
        }
        MoveRecord::Jump {
            color,
            from,
            to,
            captured,
        } => {
            let jump = validate_jump(state, color, from, to, &captured, move_number)?;
            Rules::apply_jump(state, &jump);
        }
    }

    Ok(())
}

fn validate_opening_removal(
    state: &GameState,
    color: PieceColor,
    position: Position,
    move_number: usize,
) -> Result<(), String> {
    if !matches!(
        state.phase,
        GamePhase::OpeningBlackRemoval | GamePhase::OpeningWhiteRemoval
    ) {
        return Err(format!(
            "Move {}: Opening removal not allowed during {:?}",
            move_number, state.phase
        ));
    }

    if color != state.current_player {
        return Err(format!(
            "Move {}: Expected {} to move, got {}",
            move_number, state.current_player, color
        ));
    }

    validate_position_in_bounds(state, position, move_number, "Position")?;

    Ok(())
}

fn validate_jump(
    state: &GameState,
    color: PieceColor,
    from: Position,
    to: Position,
    captured: &[Position],
    move_number: usize,
) -> Result<Jump, String> {
    if !matches!(state.phase, GamePhase::Play) {
        return Err(format!(
            "Move {}: Jump not allowed during {:?}",
            move_number, state.phase
        ));
    }

    if color != state.current_player {
        return Err(format!(
            "Move {}: Expected {} to move, got {}",
            move_number, state.current_player, color
        ));
    }

    validate_position_in_bounds(state, from, move_number, "From position")?;
    validate_position_in_bounds(state, to, move_number, "To position")?;

    if captured.is_empty() {
        return Err(format!(
            "Move {}: Jump must capture at least one piece",
            move_number
        ));
    }

    for pos in captured {
        validate_position_in_bounds(state, *pos, move_number, "Captured position")?;
    }

    let valid_jumps = Rules::valid_jumps_from(state, from);
    let matching_jump = valid_jumps
        .into_iter()
        .find(|jump| jump.to == to && jump.captured == captured);

    let Some(jump) = matching_jump else {
        return Err(format!(
            "Move {}: Invalid jump from {} to {}",
            move_number, from, to
        ));
    };

    Ok(jump)
}

fn validate_position_in_bounds(
    state: &GameState,
    position: Position,
    move_number: usize,
    label: &str,
) -> Result<(), String> {
    let size = state.board.size();
    if position.row >= size || position.col >= size {
        return Err(format!(
            "Move {}: {} {} is out of bounds",
            move_number, label, position
        ));
    }
    Ok(())
}

fn validate_winner(state: &GameState, winner: Option<String>) -> Result<(), String> {
    let Some(winner) = winner else {
        return Ok(());
    };

    let winner_color = parse_winner_color(&winner)?;

    match state.phase {
        GamePhase::GameOver { winner: actual } => {
            if actual != winner_color {
                return Err(format!(
                    "Winner mismatch: expected {}, got {}",
                    winner_color, actual
                ));
            }
        }
        _ => {
            return Err("Winner specified but game is not over".to_string());
        }
    }

    Ok(())
}

fn parse_winner_color(winner: &str) -> Result<PieceColor, String> {
    match winner.to_lowercase().as_str() {
        "black" => Ok(PieceColor::Black),
        "white" => Ok(PieceColor::White),
        _ => Err("Invalid winner: must be \"Black\" or \"White\"".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::import_game_from_content;

    #[test]
    fn imports_valid_opening_moves() {
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
    fn rejects_invalid_board_size() {
        let json = r#"{
            "board_size": 5,
            "moves": []
        }"#;

        let result = import_game_from_content(json);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_wrong_opening_turn() {
        let json = r#"{
            "board_size": 4,
            "moves": [
                {"OpeningRemoval": {"color": "White", "position": {"row": 1, "col": 2}}}
            ]
        }"#;

        let result = import_game_from_content(json);
        assert!(result.is_err());
    }

    #[test]
    fn rejects_winner_when_not_game_over() {
        let json = r#"{
            "board_size": 4,
            "winner": "Black",
            "moves": [
                {"OpeningRemoval": {"color": "Black", "position": {"row": 1, "col": 1}}},
                {"OpeningRemoval": {"color": "White", "position": {"row": 1, "col": 2}}}
            ]
        }"#;

        let result = import_game_from_content(json);
        assert!(result.is_err());
    }
}
