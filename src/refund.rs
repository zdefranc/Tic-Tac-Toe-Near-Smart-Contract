use crate::*;
use near_sdk::Balance;
#[near_bindgen]
impl Contract {
    #[payable]
    pub fn refund_excess_storage(
        &mut self,
        initial_storage: u64,
    ) {
        //calculate the required storage which was the used - initial. Then find associated cost
        let storage_used = env::storage_usage() - initial_storage;
        let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
        //get ammount deposited
        let attached_deposit = env::attached_deposit();

        //make sure that the attached deposit is greater than or equal to the required cost
        assert!(
            required_cost <= attached_deposit,
            "Must attach {} yoctoNEAR to cover storage",
            required_cost,
        );

        //get the refund amount from the attached deposit - required cost
        let refund = attached_deposit - required_cost;

        //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
        if refund > 1 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
    }
    
}