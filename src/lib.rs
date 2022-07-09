use std::convert::TryInto;
use std::ops::Index;

use near_sdk::collections::{
    LookupMap,
    Vector,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    serde::{Deserialize, Serialize},
    AccountId, PanicOnDefault, Promise,
};
use near_sdk::env::{self, signer_account_id};
use near_sdk::{near_bindgen, serde};
use String;

const ROWS: u128 = 3;
const EMPTY_SPACE: char = ' ';
const X_SPACE: char = 'X';
const O_SPACE: char = 'O';


pub use crate::refund::*;
mod refund;


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Game {
    users_turn: AccountId,
    x_player: AccountId,
    o_player: AccountId,
    board: Vec<String>,
    game_complete_status: bool,
    number_of_turns_played: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Stats {
    wins: u128,
    loses: u128,
    ties: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    game_keys: LookupMap<AccountId, String>,
    games: LookupMap<String, Game>,
    user_stats: LookupMap<AccountId, Stats>,
}

fn get_new_game(player1: AccountId, player2: AccountId) -> Game{
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
pub fn check_if_placement_is_valid(x_placement: usize, y_placement: usize){
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

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self{
        Self{
            game_keys: LookupMap::new(b"u"),
            games: LookupMap::new(b"c"),
            user_stats: LookupMap::new(b"d"),
        }
    }
    
    pub fn new_game(&mut self, challenger: AccountId){
        let method_caller = env::signer_account_id();
        // Check if the method caller has a game already
        assert!(
            self.game_keys.contains_key(&method_caller) == false, 
            "You must finish your current game before playing a new one."
        );
        
        // Check if the challenger is also in a game
        if self.game_keys.contains_key(&challenger) {
            env::panic_str(
                &format!("is currently in a game. {:?}", challenger),
            );
        }
        
        //Measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();


        // Make a key to access the game and store it in the game_keys lookup map for each user
        // and store the game under the game_key
        let game_key = format!("{}{}", method_caller.clone(), challenger);
        
        self.game_keys.insert(&method_caller,&game_key);
        self.game_keys.insert(&challenger,&game_key);
        
        self.games.insert(&game_key, &get_new_game(method_caller.clone(), challenger.clone()));
        
        // If a new game was started check if each user has stats and initialize them if not
        if !self.user_stats.contains_key(&challenger){
            self.user_stats.insert(&challenger, &Stats{ wins: 0, loses: 0, ties: 0 });
        }
        if !self.user_stats.contains_key(&method_caller){
            self.user_stats.insert(&method_caller, &Stats{ wins: 0, loses: 0, ties: 0 });
        }
        self.refund_excess_storage(initial_storage_usage);
    }


    pub fn play_turn(&mut self, x_placement: usize, y_placement: usize){
        let player = env::signer_account_id();
        // Panic if the user does not have a game
        assert!(
            self.game_keys.contains_key(&player) == true, 
            "You do not have an active game"
        );
        // Get the players game
        let game_key = self.game_keys.get(&player).unwrap();
        let mut game = self.games.get(&game_key).unwrap();
        // If the game is completed before the player can perform their turn it means that they have lost game
        // or if the number of turns played is greater than or equal to 9 then there are no more spaces to play on 
        // and therefore they have tied
        if game.game_complete_status {
            self.view_game();
            self.game_keys.remove(&player);
            self.games.remove(&game_key);
            env::log_str("You have lost :'(");
            self.increment_loses(player);
            return
        } else if game.number_of_turns_played >= 9 {
            self.view_game();
            self.game_keys.remove(&player);
            self.games.remove(&game_key);
            env::log_str("You've tied :|");
            self.increment_ties(player);
            return
        }
        // Panic if it is not the players turn
        assert_eq!(game.users_turn,player,"It is not your turn");
        check_if_placement_is_valid(x_placement, y_placement);
        // Convert the entered placements into indexable values 
        let x_index = x_placement - 1;
        let y_index = y_placement - 1;
        // Get the value of the space being played on, if it is not epty panic
        let space_value = game.board[y_index.clone()].clone().get(x_index..x_index+1).unwrap().chars().nth(0).unwrap();
        if space_value != EMPTY_SPACE {
            env::panic_str(&format!("Position {:?},{:?} is already played", x_placement, y_placement));
        }
        // Get the value of the players marker and record who the next player will be
        let mut new_marker = ' ';
        let mut next_player = "placeholder.near".parse().unwrap();
        if player == game.x_player{
            new_marker = X_SPACE;
            next_player = game.o_player.clone();
        }
        else {
            new_marker = O_SPACE;
            next_player = game.x_player.clone();
        } 
        // Take the board and add the players marker in the entered location and increment the number of turns played.
        // Then, insert the new board back into the games lookup map.
        let mut new_board = game.board.clone();
        let mut new_row = game.board[y_index.clone()].clone();
        new_row.replace_range(x_index..x_index+1, &new_marker.to_string());
        new_board[y_index.clone()] = new_row;
        game.board = new_board;
        game.users_turn = next_player;
        // Increment number of turns played
        game.number_of_turns_played += 1;
        self.games.insert(&game_key, &game);
        self.view_game();
        // Check if game has been won if the user won update that the game has been completed, remove the game from the users game_key
        // and increment the users wins.  Else if the number of placements has reached 9 thena tie has been reached. 
        if self.has_user_won_on_turn(game_key.clone(), new_marker.to_string(), x_index, y_index.clone()){
            game.game_complete_status = true;
            self.games.insert(&game_key, &game);
            self.game_keys.remove(&player);
            env::log_str("You've won!!!!");
            self.increment_wins(player);
        } else if game.number_of_turns_played == 9{
            self.game_keys.remove(&player);
            env::log_str("You've tied :|");
            self.increment_ties(player);
        }
    }

    

    // Checks if a user's move has won them the match
    #[private]
    pub fn has_user_won_on_turn(&mut self, game_key: String, player_marker: String, x_index: usize, y_index: usize) -> bool {
        check_if_placement_is_valid(x_index + 1, y_index + 1);

        let board = self.games.get(&game_key).unwrap().board;
        let three_in_a_row = format!("{}{}{}", player_marker, player_marker, player_marker); // Assembles a winning three in a row string of the player's markers to be used for comparison
        // Checks the horizontal case of winning by comparing a row at the y index to the three_in_a_row
        if board[y_index] == three_in_a_row{
            env::log_str("Game has been won");
            return true;
        }
        // Checks the vertical case of winning by comparing a column at the x index to the three_in_a_row
        if format!("{}{}{}", board.get(0).unwrap().get(x_index..x_index+1).unwrap(), board.get(1).unwrap().get(x_index..x_index+1).unwrap(), board.get(2).unwrap().get(x_index..x_index+1).unwrap())
            == three_in_a_row{
            env::log_str("Game has been won");
            return true;
        }
        // If this is true the space is in position to be checked for the diagonal.
        // A space is deemed to be in position to be checked for the diagonal if it is any of the corners or is the middle position as these are the only posiitons that
        // can be won in diagonal.  Summing the x and y index will return a even value if it is in any of the positions.
        if (x_index + y_index)%2 == 0{
            // If the position is the centre tile check both diagonal directions
            if x_index == 1 && y_index == 1{
                if format!("{}{}{}", board.get(0).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(2).unwrap().get(2..3).unwrap())
                    == three_in_a_row || 
                    format!("{}{}{}", board.get(2).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(0).unwrap().get(2..3).unwrap())
                    == three_in_a_row {
                        return true;
                    }
            // If the x and y index are summed and the value is 2 the position is either the top right corner or the bottom left corner and therefore
            // The positive slop diagonal must be checked.
            } else if (x_index + y_index) == 2{
                if format!("{}{}{}", board.get(2).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(0).unwrap().get(2..3).unwrap())
                    == three_in_a_row{
                    return true;
                }
            // Else the position is either the bottom right corner or the top left cornerand therefore
            // The negative slop diagonal must be checked.
            }else {
                if format!("{}{}{}", board.get(0).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(2).unwrap().get(2..3).unwrap())
                    == three_in_a_row {
                        return true;
                    }
            }
        }
        false
    }

    // Constructs the tic tac toe board to visually see the progress of the users game.
    pub fn view_game(&self) {
        assert!(
            self.game_keys.contains_key(&env::signer_account_id()) == true, 
            "You do not have an active game"
        );
        // Get the board seperate the rows and print each position.
        let game_key = self.game_keys.get(&env::signer_account_id());
        let board = self.games.get(&game_key.unwrap()).unwrap().board;
        // continue here
        let first_row = board.get(0).unwrap();
        let second_row = board.get(1).unwrap();
        let third_row = board.get(2).unwrap();
        
        env::log_str(&format!("\n {} | {} | {} \n-----------\n {} | {} | {} \n-----------\n {} | {} | {} ", 
        first_row.get(0..1).unwrap(), first_row.get(1..2).unwrap(), first_row.get(2..3).unwrap(), second_row.get(0..1).unwrap(), second_row.get(1..2).unwrap(), second_row.get(2..3).unwrap(), third_row.get(0..1).unwrap(), 
        third_row.get(1..2).unwrap(), third_row.get(2..3).unwrap()));
    }

    // Returns the stats of a user that is or has played games
    pub fn view_user_stats(&self, user: AccountId){
        self.panic_if_user_does_not_have_stats(user.clone());
        let stats = self.user_stats.get(&user).unwrap();
        env::log_str(&format!("{} has {} wins, {} ties, and {} loses.",
            user.to_string(), stats.wins.to_string(), stats.ties.to_string(), stats.loses.to_string()));
    }

    #[private]
    pub fn panic_if_user_does_not_have_stats(&self, user: AccountId) {
        // If the player does not have statistics panic
        if !self.user_stats.contains_key(&user) {
            env::panic_str(
                &format!("{} does not have any statistics", user.to_string()),
            );
        }
    }

    #[private]
    pub fn increment_wins(&mut self, user: AccountId){
        self.panic_if_user_does_not_have_stats(user.clone());
        let mut stats = self.user_stats.get(&user).unwrap();
        stats.wins += 1;
        self.user_stats.insert(&user, &stats);
    }

    #[private]
    pub fn increment_loses(&mut self, user: AccountId){
        self.panic_if_user_does_not_have_stats(user.clone());
        let mut stats = self.user_stats.get(&user).unwrap();
        stats.loses += 1;
        self.user_stats.insert(&user, &stats);
    }

    #[private]
    pub fn increment_ties(&mut self, user: AccountId){
        self.panic_if_user_does_not_have_stats(user.clone());
        let mut stats = self.user_stats.get(&user).unwrap();
        stats.ties += 1;
        self.user_stats.insert(&user, &stats);
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.signer_account_id(predecessor.clone()).predecessor_account_id(predecessor);
        builder
    }
}
