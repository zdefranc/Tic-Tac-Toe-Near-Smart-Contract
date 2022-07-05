set -e




near create-account ttt.zdefranc.testnet --masterAccount zdefranc.testnet

./build.sh

near deploy ttt.zdefranc.testnet --wasmFile res/ttt.wasm

near call ttt.zdefranc.testnet new --accountId ttt.zdefranc.testnet

near call ttt.zdefranc.testnet new_game "{\"challenger\": \"ttt.zdefranc.testnet\"}" --accountId zdefranc.testnet

near call ttt.zdefranc.testnet new_game "{\"challenger\": \"a.zdefranc.testnet\"}" --accountId b.zdefranc.testnet

near call ttt.zdefranc.testnet view_game --accountId zdefranc.testnet

near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 3, \"y_placement\": 1}" --accountId zdefranc.testnet

near call ttt.zdefranc.testnet view_game --accountId a.zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 2, \"y_placement\": 1}" --accountId ttt.zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 1, \"y_placement\": 3}" --accountId zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 1, \"y_placement\": 2}" --accountId ttt.zdefranc.testnet

# near call ttt.zdefranc.testnet view_user_stats "{\"user\": \"ttt.zdefranc.testnet\"}" --accountId ttt.zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 3, \"y_placement\": 2}" --accountId zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 3, \"y_placement\": 3}" --accountId ttt.zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 1, \"y_placement\": 1}" --accountId zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 2, \"y_placement\": 2}" --accountId ttt.zdefranc.testnet

# near call ttt.zdefranc.testnet play_turn "{\"x_placement\": 2, \"y_placement\": 3}" --accountId zdefranc.testnet

