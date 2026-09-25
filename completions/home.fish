# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_home_global_optspecs
    string join \n base-url= api-key= issuer= client-id= json h/help V/version
end

function __fish_home_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_home_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_home_using_subcommand
    set -l cmd (__fish_home_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c home -n "__fish_home_needs_command" -l base-url -r
complete -c home -n "__fish_home_needs_command" -l api-key -r
complete -c home -n "__fish_home_needs_command" -l issuer -r
complete -c home -n "__fish_home_needs_command" -l client-id -r
complete -c home -n "__fish_home_needs_command" -l json
complete -c home -n "__fish_home_needs_command" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_needs_command" -s V -l version -d 'Print version'
complete -c home -n "__fish_home_needs_command" -f -a "login"
complete -c home -n "__fish_home_needs_command" -f -a "logout"
complete -c home -n "__fish_home_needs_command" -f -a "whoami"
complete -c home -n "__fish_home_needs_command" -f -a "ls"
complete -c home -n "__fish_home_needs_command" -f -a "light"
complete -c home -n "__fish_home_needs_command" -f -a "workflow"
complete -c home -n "__fish_home_needs_command" -f -a "mode"
complete -c home -n "__fish_home_needs_command" -f -a "keys"
complete -c home -n "__fish_home_needs_command" -f -a "lua"
complete -c home -n "__fish_home_needs_command" -f -a "push"
complete -c home -n "__fish_home_needs_command" -f -a "adhoc"
complete -c home -n "__fish_home_needs_command" -f -a "eink"
complete -c home -n "__fish_home_needs_command" -f -a "synergy"
complete -c home -n "__fish_home_needs_command" -f -a "weather"
complete -c home -n "__fish_home_needs_command" -f -a "solar"
complete -c home -n "__fish_home_needs_command" -f -a "energy"
complete -c home -n "__fish_home_needs_command" -f -a "transperth"
complete -c home -n "__fish_home_needs_command" -f -a "fuel"
complete -c home -n "__fish_home_needs_command" -f -a "events"
complete -c home -n "__fish_home_needs_command" -f -a "curl"
complete -c home -n "__fish_home_needs_command" -f -a "completions"
complete -c home -n "__fish_home_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand login" -l base-url -r
complete -c home -n "__fish_home_using_subcommand login" -l api-key -r
complete -c home -n "__fish_home_using_subcommand login" -l issuer -r
complete -c home -n "__fish_home_using_subcommand login" -l client-id -r
complete -c home -n "__fish_home_using_subcommand login" -l json
complete -c home -n "__fish_home_using_subcommand login" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand logout" -l base-url -r
complete -c home -n "__fish_home_using_subcommand logout" -l api-key -r
complete -c home -n "__fish_home_using_subcommand logout" -l issuer -r
complete -c home -n "__fish_home_using_subcommand logout" -l client-id -r
complete -c home -n "__fish_home_using_subcommand logout" -l json
complete -c home -n "__fish_home_using_subcommand logout" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand whoami" -l base-url -r
complete -c home -n "__fish_home_using_subcommand whoami" -l api-key -r
complete -c home -n "__fish_home_using_subcommand whoami" -l issuer -r
complete -c home -n "__fish_home_using_subcommand whoami" -l client-id -r
complete -c home -n "__fish_home_using_subcommand whoami" -l json
complete -c home -n "__fish_home_using_subcommand whoami" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand ls" -l category -r -f -a "lights\t''
doors\t''
presence\t''
environment\t''
displays\t''
vacuums\t''
media\t''"
complete -c home -n "__fish_home_using_subcommand ls" -l base-url -r
complete -c home -n "__fish_home_using_subcommand ls" -l api-key -r
complete -c home -n "__fish_home_using_subcommand ls" -l issuer -r
complete -c home -n "__fish_home_using_subcommand ls" -l client-id -r
complete -c home -n "__fish_home_using_subcommand ls" -l json
complete -c home -n "__fish_home_using_subcommand ls" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -l json
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -f -a "on"
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -f -a "off"
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -f -a "toggle"
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -f -a "set"
complete -c home -n "__fish_home_using_subcommand light; and not __fish_seen_subcommand_from on off toggle set help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -l base-url -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -l api-key -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -l issuer -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -l client-id -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -l json
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from on" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -l base-url -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -l api-key -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -l issuer -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -l client-id -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -l json
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from off" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -l base-url -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -l api-key -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -l issuer -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -l client-id -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -l json
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from toggle" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l brightness -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l colour-temperature -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l colour -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l base-url -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l api-key -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l issuer -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l client-id -r
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l on
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l off
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -l json
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from help" -f -a "on"
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from help" -f -a "off"
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from help" -f -a "toggle"
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from help" -f -a "set"
complete -c home -n "__fish_home_using_subcommand light; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "exec"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "enable"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "disable"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "runs"
complete -c home -n "__fish_home_using_subcommand workflow; and not __fish_seen_subcommand_from list run exec enable disable runs help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l input -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from run" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l input -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from exec" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from enable" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from disable" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l slug -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l limit -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l base-url -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l api-key -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l issuer -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l client-id -r
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -l json
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from runs" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "exec"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "enable"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "disable"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "runs"
complete -c home -n "__fish_home_using_subcommand workflow; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -l json
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -f -a "show"
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -f -a "set"
complete -c home -n "__fish_home_using_subcommand mode; and not __fish_seen_subcommand_from show set help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -l base-url -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -l api-key -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -l issuer -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -l client-id -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -l json
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -l base-url -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -l api-key -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -l issuer -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -l client-id -r
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -l json
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from help" -f -a "show"
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from help" -f -a "set"
complete -c home -n "__fish_home_using_subcommand mode; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -l json
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "create"
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "update"
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "regenerate"
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "revoke"
complete -c home -n "__fish_home_using_subcommand keys; and not __fish_seen_subcommand_from create list update regenerate revoke help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l expires-at -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -l json
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from create" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -l json
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l name -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l scopes -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l expires-at -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -l json
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -l json
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from regenerate" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -l base-url -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -l api-key -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -l issuer -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -l client-id -r
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -l json
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from revoke" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "create"
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "update"
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "regenerate"
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "revoke"
complete -c home -n "__fish_home_using_subcommand keys; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -l json
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -f -a "repl"
complete -c home -n "__fish_home_using_subcommand lua; and not __fish_seen_subcommand_from run repl help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -s e -l expression -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l var -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l base-url -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l api-key -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l issuer -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l client-id -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l dry-run
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -l json
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from run" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l base-url -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l api-key -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l issuer -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l client-id -r
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l dry-run
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -l json
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from repl" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from help" -f -a "repl"
complete -c home -n "__fish_home_using_subcommand lua; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -l json
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -f -a "send"
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand push; and not __fish_seen_subcommand_from send list help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l title -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l category -r -f -a "alarm\t''
door\t''
watchdog\t''
general\t''"
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l tag -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l action -d 'repeatable; KIND is acknowledge, dismiss, snooze:SECONDS or workflow:SLUG' -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l remind-after -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l reminders -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l base-url -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l api-key -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l issuer -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l client-id -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -l json
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from send" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l limit -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l base-url -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l api-key -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l issuer -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l client-id -r
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -l json
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from help" -f -a "send"
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand push; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -l json
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -f -a "cron"
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -f -a "run-pending"
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand adhoc; and not __fish_seen_subcommand_from list cron run-pending run help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -l base-url -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -l api-key -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -l issuer -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -l client-id -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -l json
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -l base-url -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -l api-key -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -l issuer -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -l client-id -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -l json
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from cron" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -l base-url -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -l api-key -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -l issuer -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -l client-id -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -l json
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run-pending" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -l base-url -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -l api-key -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -l issuer -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -l client-id -r
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -l json
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from run" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from help" -f -a "list"
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from help" -f -a "cron"
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from help" -f -a "run-pending"
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from help" -f -a "run"
complete -c home -n "__fish_home_using_subcommand adhoc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -l json
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -f -a "screenshot"
complete -c home -n "__fish_home_using_subcommand eink; and not __fish_seen_subcommand_from screenshot help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -l base-url -r
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -l api-key -r
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -l issuer -r
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -l client-id -r
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -l json
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from screenshot" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from help" -f -a "screenshot"
complete -c home -n "__fish_home_using_subcommand eink; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -l base-url -r
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -l api-key -r
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -l issuer -r
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -l client-id -r
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -l json
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -f -a "upload"
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -f -a "gaps"
complete -c home -n "__fish_home_using_subcommand synergy; and not __fish_seen_subcommand_from upload gaps help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -l base-url -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -l api-key -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -l issuer -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -l client-id -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -l json
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from upload" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l since -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l interval -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l base-url -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l api-key -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l issuer -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l client-id -r
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -l json
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from gaps" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from help" -f -a "upload"
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from help" -f -a "gaps"
complete -c home -n "__fish_home_using_subcommand synergy; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand weather" -l base-url -r
complete -c home -n "__fish_home_using_subcommand weather" -l api-key -r
complete -c home -n "__fish_home_using_subcommand weather" -l issuer -r
complete -c home -n "__fish_home_using_subcommand weather" -l client-id -r
complete -c home -n "__fish_home_using_subcommand weather" -l json
complete -c home -n "__fish_home_using_subcommand weather" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand solar" -l base-url -r
complete -c home -n "__fish_home_using_subcommand solar" -l api-key -r
complete -c home -n "__fish_home_using_subcommand solar" -l issuer -r
complete -c home -n "__fish_home_using_subcommand solar" -l client-id -r
complete -c home -n "__fish_home_using_subcommand solar" -l json
complete -c home -n "__fish_home_using_subcommand solar" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand energy" -l since -r
complete -c home -n "__fish_home_using_subcommand energy" -l base-url -r
complete -c home -n "__fish_home_using_subcommand energy" -l api-key -r
complete -c home -n "__fish_home_using_subcommand energy" -l issuer -r
complete -c home -n "__fish_home_using_subcommand energy" -l client-id -r
complete -c home -n "__fish_home_using_subcommand energy" -l json
complete -c home -n "__fish_home_using_subcommand energy" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand transperth" -l base-url -r
complete -c home -n "__fish_home_using_subcommand transperth" -l api-key -r
complete -c home -n "__fish_home_using_subcommand transperth" -l issuer -r
complete -c home -n "__fish_home_using_subcommand transperth" -l client-id -r
complete -c home -n "__fish_home_using_subcommand transperth" -l json
complete -c home -n "__fish_home_using_subcommand transperth" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand fuel" -l postcode -r
complete -c home -n "__fish_home_using_subcommand fuel" -l limit -r
complete -c home -n "__fish_home_using_subcommand fuel" -l base-url -r
complete -c home -n "__fish_home_using_subcommand fuel" -l api-key -r
complete -c home -n "__fish_home_using_subcommand fuel" -l issuer -r
complete -c home -n "__fish_home_using_subcommand fuel" -l client-id -r
complete -c home -n "__fish_home_using_subcommand fuel" -l json
complete -c home -n "__fish_home_using_subcommand fuel" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand events" -l base-url -r
complete -c home -n "__fish_home_using_subcommand events" -l api-key -r
complete -c home -n "__fish_home_using_subcommand events" -l issuer -r
complete -c home -n "__fish_home_using_subcommand events" -l client-id -r
complete -c home -n "__fish_home_using_subcommand events" -l json
complete -c home -n "__fish_home_using_subcommand events" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand curl" -l base-url -r
complete -c home -n "__fish_home_using_subcommand curl" -l api-key -r
complete -c home -n "__fish_home_using_subcommand curl" -l issuer -r
complete -c home -n "__fish_home_using_subcommand curl" -l client-id -r
complete -c home -n "__fish_home_using_subcommand curl" -l json
complete -c home -n "__fish_home_using_subcommand curl" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand completions" -l base-url -r
complete -c home -n "__fish_home_using_subcommand completions" -l api-key -r
complete -c home -n "__fish_home_using_subcommand completions" -l issuer -r
complete -c home -n "__fish_home_using_subcommand completions" -l client-id -r
complete -c home -n "__fish_home_using_subcommand completions" -l json
complete -c home -n "__fish_home_using_subcommand completions" -s h -l help -d 'Print help'
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "login"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "logout"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "whoami"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "ls"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "light"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "workflow"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "mode"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "keys"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "lua"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "push"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "adhoc"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "eink"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "synergy"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "weather"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "solar"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "energy"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "transperth"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "fuel"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "events"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "curl"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "completions"
complete -c home -n "__fish_home_using_subcommand help; and not __fish_seen_subcommand_from login logout whoami ls light workflow mode keys lua push adhoc eink synergy weather solar energy transperth fuel events curl completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from light" -f -a "on"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from light" -f -a "off"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from light" -f -a "toggle"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from light" -f -a "set"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "list"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "run"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "exec"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "enable"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "disable"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from workflow" -f -a "runs"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from mode" -f -a "show"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from mode" -f -a "set"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from keys" -f -a "create"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from keys" -f -a "list"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from keys" -f -a "update"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from keys" -f -a "regenerate"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from keys" -f -a "revoke"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from lua" -f -a "run"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from lua" -f -a "repl"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from push" -f -a "send"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from push" -f -a "list"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from adhoc" -f -a "list"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from adhoc" -f -a "cron"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from adhoc" -f -a "run-pending"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from adhoc" -f -a "run"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from eink" -f -a "screenshot"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from synergy" -f -a "upload"
complete -c home -n "__fish_home_using_subcommand help; and __fish_seen_subcommand_from synergy" -f -a "gaps"
