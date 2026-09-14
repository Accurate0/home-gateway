function __home_entity_ids
    if set -q argv[1]
        home ls --json --category $argv[1] 2>/dev/null | jq -r '.[] | "\(.id)\t\(.name)"'
    else
        home ls --json 2>/dev/null | jq -r '.[] | "\(.id)\t\(.name)"'
    end
end

function __home_workflow_slugs
    home workflow list --json 2>/dev/null | jq -r '.[] | "\(.slug)\t\(.name)"'
end

function __home_adhoc_cron_names
    home adhoc cron --json 2>/dev/null | jq -r '.[] | "\(.name)\t\(.schedule)"'
end

function __home_transperth_routes
    home transperth --json 2>/dev/null | jq -r '.[] | "\(.id)\t\(.origin) -> \(.destination)"'
end

function __home_seen_path
    set -l tokens (commandline -opc)
    set -l index 2

    for word in $argv
        while test $index -le (count $tokens); and string match -q -- '-*' $tokens[$index]
            set index (math $index + 1)
        end

        if test $index -gt (count $tokens); or test "$tokens[$index]" != "$word"
            return 1
        end

        set index (math $index + 1)
    end

    return 0
end

for action in on off toggle set
    complete -c home -n "__home_seen_path light $action" -f -a '(__home_entity_ids lights)'
end

for action in run enable disable
    complete -c home -n "__home_seen_path workflow $action" -f -a '(__home_workflow_slugs)'
end

complete -c home -n '__home_seen_path workflow runs' -l slug -f -a '(__home_workflow_slugs)'
complete -c home -n '__home_seen_path adhoc run' -f -a '(__home_adhoc_cron_names)'
complete -c home -n '__home_seen_path transperth' -f -a '(__home_transperth_routes)'
complete -c home -n '__home_seen_path mode set' -f -a 'home away vacation guest'
complete -c home -n '__home_seen_path events' -f -a '"*"'

for domain in presence door switch environment cron light unifi sun mode home_assistant woolworths device_battery jellyfin media_player solar weather fuelwatch command_failed custom
    complete -c home -n '__home_seen_path events' -f -a "$domain:\*"
end
