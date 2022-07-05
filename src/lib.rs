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


#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Game {
    users_turn: AccountId,
    x_player: AccountId,
    o_player: AccountId,
    board: Vector<String>,
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
    let mut board = Vector::new(b"m");
    let mut b: Vec<String> = Vec::new();
    let empty_row = "   ".to_string();
    for x in 0..ROWS{
        board.push(&empty_row);
    }
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
        // Check if the method caller has a game already
        assert!(
            self.game_keys.contains_key(&env::signer_account_id()) == false, 
            "You must finish your current game before playing a new one."
        );
        
        // Check if the challenger is also in a game
        if self.game_keys.contains_key(&challenger) {
            env::panic_str(
                &format!("is currently in a game. {:?}", challenger),
            );
        }
        
        // Make a key to access the game and store it in the game_keys lookup map for each user
        // and store the game under the game_key
        let game_key = format!("{}{}", env::signer_account_id(), challenger);
        
        self.game_keys.insert(&env::signer_account_id(),&game_key);
        self.game_keys.insert(&challenger,&game_key);
        
        self.games.insert(&game_key, &get_new_game(env::signer_account_id(), challenger.clone()));
        
        // If a new game was started check if each user has stats and initialize them if not
        if !self.user_stats.contains_key(&challenger){
            self.user_stats.insert(&challenger, &Stats{ wins: 0, loses: 0, ties: 0 });
        }
        if !self.user_stats.contains_key(&env::signer_account_id()){
            self.user_stats.insert(&env::signer_account_id(), &Stats{ wins: 0, loses: 0, ties: 0 });
        }
    }


    pub fn play_turn(&mut self, x_placement: usize, y_placement: usize){
        let player = env::signer_account_id();
        assert!(
            self.game_keys.contains_key(&player) == true, 
            "You do not have an active game"
        );
        let game_key = self.game_keys.get(&player).unwrap();
        let mut game = self.games.get(&game_key).unwrap();
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
        assert_eq!(game.users_turn,player,"It is not your turn");
        check_if_placement_is_valid(x_placement, y_placement);
        let x_index = x_placement - 1;
        let y_index = y_placement - 1;
        let space_value = game.board.get(y_index.try_into().unwrap()).unwrap().get(x_index..x_index+1).unwrap().chars().nth(0).unwrap();
        if space_value != EMPTY_SPACE {
            env::panic_str(&format!("Position {:?},{:?} is already played", x_placement, y_placement));
        }
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
        // .replace_range(x_index..x_index+1, &new_marker.to_string())
        let mut new_board = game.board;
        let mut new_row = new_board.get(y_index.try_into().unwrap()).unwrap();
        new_row.replace_range(x_index..x_index+1, &new_marker.to_string());
        new_board.replace(y_index.try_into().unwrap(), &new_row);
        game.board = new_board;
        game.users_turn = next_player;
        // Increment number of turns played
        game.number_of_turns_played += 1;
        self.games.insert(&game_key, &game);
        self.view_game();
        // Check if game has been won
        if self.has_user_won_on_turn(game_key.clone(), new_marker.to_string(), x_index, y_index){
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
        let three_in_a_row = format!("{}{}{}", player_marker, player_marker, player_marker);
        if board.get(y_index.try_into().unwrap()).unwrap() == three_in_a_row{
            env::log_str("Game has been won");
            return true;
        }
        if format!("{}{}{}", board.get(0).unwrap().get(x_index..x_index+1).unwrap(), board.get(1).unwrap().get(x_index..x_index+1).unwrap(), board.get(2).unwrap().get(x_index..x_index+1).unwrap())
            == three_in_a_row{
            env::log_str("Game has been won");
            return true;
        }
        // If this is true the space is in position to be checked for the diagonal
        if (x_index + y_index)%2 == 0{
            // If centre tile check both diagonal directions
            if x_index == 1 && y_index == 1{
                if format!("{}{}{}", board.get(0).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(2).unwrap().get(2..3).unwrap())
                    == three_in_a_row || 
                    format!("{}{}{}", board.get(2).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(0).unwrap().get(2..3).unwrap())
                    == three_in_a_row {
                        return true;
                    }
            } else if (x_index + y_index) == 2{
                if format!("{}{}{}", board.get(2).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(0).unwrap().get(2..3).unwrap())
                    == three_in_a_row{
                    return true;
                }
            }else {
                if format!("{}{}{}", board.get(0).unwrap().get(0..1).unwrap(), board.get(1).unwrap().get(1..2).unwrap(), board.get(2).unwrap().get(2..3).unwrap())
                    == three_in_a_row {
                        return true;
                    }
            }
        }
        false
    }

    pub fn view_game(&self) {
        assert!(
            self.game_keys.contains_key(&env::signer_account_id()) == true, 
            "You do not have an active game"
        );
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
    // ADD CONTRACT METHODS HERE
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


    #[test]
    fn my_test() {
        let mut context = get_context("zdefranc.testnet".parse().unwrap());
        testing_env!(context.build());
        let mut contract = Contract::new();
        testing_env!(context.is_view(true).build());
        contract.new_game("mike.testnet".parse().unwrap());
        // assert_eq!(contract.view_game(), "         ");
    }
}
