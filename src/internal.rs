use crate::*;
use near_sdk::{CryptoHash};

pub(crate) fn hash_account_id(account_id: &AccountId) -> CryptoHash {
    //get the default hash
    let mut hash = CryptoHash::default();
    //we hash the account ID and return it
    hash.copy_from_slice(&env::sha256(account_id.as_bytes()));
    hash
}

pub(crate) fn get_new_game(player1: AccountId, player2: AccountId) -> Game{
    // Intialize the vector that will contain status of each space in the game as all empty spaces.
    // Each index of the vector is along the y axis and each char in the string is along the x axis
    let mut board: Vec<String> = Vec::new();
    let empty_row = "   ".to_string();
    for x in 0..ROWS{
        board.push(empty_row.clone());
    }
    // Return the new game
    Game{
        users_turn: player1.clone(), 
        x_player: player1.clone(), 
        o_player: player2, 
        board: board,
        game_complete_status: false,
        number_of_turns_played: 0,
    }
}

// Checks if the user selected placement is valid
pub(crate) fn check_if_placement_is_valid(x_placement: usize, y_placement: usize){
    // The tic tac toe board has a x and y axis with valid values only in the interval of 1-3
    if x_placement > 3 || y_placement > 3 {
        env::panic_str(
            &format!("Positions only go up to 3, {:?},{:?} is too high", x_placement, y_placement),
        );
    } else if x_placement < 1 || y_placement < 1 {
        env::panic_str(
            &format!("The lowest position is 1, {:?},{:?} is too low", x_placement, y_placement),
        );
    }
}