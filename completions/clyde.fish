complete -c clyde -n "__fish_use_subcommand" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_use_subcommand" -s V -l version -d 'Print version'
complete -c clyde -n "__fish_use_subcommand" -f -a "setup" -d 'Setup Clyde'
complete -c clyde -n "__fish_use_subcommand" -f -a "update" -d 'Update Clyde store'
complete -c clyde -n "__fish_use_subcommand" -f -a "install" -d 'Install applications'
complete -c clyde -n "__fish_use_subcommand" -f -a "uninstall" -d 'Uninstall applications (alias: remove)'
complete -c clyde -n "__fish_use_subcommand" -f -a "show" -d 'Show details about an application'
complete -c clyde -n "__fish_use_subcommand" -f -a "search" -d 'Search for available applications'
complete -c clyde -n "__fish_use_subcommand" -f -a "list" -d 'List installed applications'
complete -c clyde -n "__fish_use_subcommand" -f -a "upgrade" -d 'Upgrade all installed applications, enforcing pinning'
complete -c clyde -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c clyde -n "__fish_seen_subcommand_from setup" -l url -d 'URL of the Git repository to use for the store' -r
complete -c clyde -n "__fish_seen_subcommand_from setup" -s u -l update-scripts -d 'Update the activation scripts of an existing installation'
complete -c clyde -n "__fish_seen_subcommand_from setup" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from update" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from install" -s r -l reinstall -d 'Uninstall then reinstall already installed packages'
complete -c clyde -n "__fish_seen_subcommand_from install" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c clyde -n "__fish_seen_subcommand_from uninstall" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from show" -s l -l list -d 'List application files instead of showing information'
complete -c clyde -n "__fish_seen_subcommand_from show" -s j -l json -d 'Use JSON output'
complete -c clyde -n "__fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from search" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from list" -s j -l json -d 'Use JSON output'
complete -c clyde -n "__fish_seen_subcommand_from list" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from upgrade" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "setup" -d 'Setup Clyde'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "update" -d 'Update Clyde store'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "install" -d 'Install applications'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "uninstall" -d 'Uninstall applications (alias: remove)'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "show" -d 'Show details about an application'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "search" -d 'Search for available applications'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "list" -d 'List installed applications'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "upgrade" -d 'Upgrade all installed applications, enforcing pinning'
complete -c clyde -n "__fish_seen_subcommand_from help; and not __fish_seen_subcommand_from setup; and not __fish_seen_subcommand_from update; and not __fish_seen_subcommand_from install; and not __fish_seen_subcommand_from uninstall; and not __fish_seen_subcommand_from show; and not __fish_seen_subcommand_from search; and not __fish_seen_subcommand_from list; and not __fish_seen_subcommand_from upgrade; and not __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
