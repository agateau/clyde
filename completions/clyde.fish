# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_clyde_global_optspecs
	string join \n h/help V/version
end

function __fish_clyde_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_clyde_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_clyde_using_subcommand
	set -l cmd (__fish_clyde_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c clyde -n "__fish_clyde_needs_command" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_needs_command" -s V -l version -d 'Print version'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "setup" -d 'Setup Clyde'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "update" -d 'Update Clyde store'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "install" -d 'Install applications'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "uninstall" -d 'Uninstall applications (alias: remove)'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "show" -d 'Show details about an application'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "search" -d 'Search for available applications'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "doc" -d 'Read documentation files provided by an application'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "list" -d 'List installed applications'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "upgrade" -d 'Upgrade all installed applications, enforcing pinning'
complete -c clyde -n "__fish_clyde_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c clyde -n "__fish_clyde_using_subcommand setup" -l url -d 'URL of the Git repository to use for the store' -r
complete -c clyde -n "__fish_clyde_using_subcommand setup" -s u -l update-scripts -d 'Update the activation scripts of an existing installation'
complete -c clyde -n "__fish_clyde_using_subcommand setup" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand update" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand install" -s r -l reinstall -d 'Uninstall then reinstall already installed packages'
complete -c clyde -n "__fish_clyde_using_subcommand install" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c clyde -n "__fish_clyde_using_subcommand uninstall" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand show" -s l -l list -d 'List application files instead of showing information'
complete -c clyde -n "__fish_clyde_using_subcommand show" -s j -l json -d 'Use JSON output'
complete -c clyde -n "__fish_clyde_using_subcommand show" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand search" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand doc" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand list" -s j -l json -d 'Use JSON output'
complete -c clyde -n "__fish_clyde_using_subcommand list" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand upgrade" -s h -l help -d 'Print help'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "setup" -d 'Setup Clyde'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "update" -d 'Update Clyde store'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "install" -d 'Install applications'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "uninstall" -d 'Uninstall applications (alias: remove)'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "show" -d 'Show details about an application'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "search" -d 'Search for available applications'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "doc" -d 'Read documentation files provided by an application'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "list" -d 'List installed applications'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "upgrade" -d 'Upgrade all installed applications, enforcing pinning'
complete -c clyde -n "__fish_clyde_using_subcommand help; and not __fish_seen_subcommand_from setup update install uninstall show search doc list upgrade help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
