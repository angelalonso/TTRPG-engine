# How to manually test the plugins:

## Race Results
printf '%s\n' '{"interactive":true,"event":{"id":"race_thruxton_open","name":"Thruxton Open Race","quest_id":"","position_rewards":"1:5000;2:3000;3:2000"},"max_reward_position":3,"player_name":"Player","driver_names":[],"competitors":[],"damage_options":[],"results_directory":"/tmp"}' | python3 gtr2career/plugins/race_results.py

## Sponsor negotiation
python3 gtr2career/plugins/sponsor_negotiator.py --dataset-path ./gtr2career --results 80 --charisma 5 --has-agent --agent-level 2 --race-tier local --scope race --approach proposal --target-id race_thruxton_open --target-name "Thruxton Open Race" --seed 1
