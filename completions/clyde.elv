
use builtin;
use str;

set edit:completion:arg-completer[clyde] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'clyde'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'clyde'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand setup 'Setup Clyde'
            cand update 'Update Clyde store'
            cand install 'Install applications'
            cand uninstall 'Uninstall applications (alias: remove)'
            cand show 'Show details about an application'
            cand search 'Search for available applications'
            cand doc 'Read documentation files provided by an application'
            cand list 'List installed applications'
            cand upgrade 'Upgrade all installed applications, enforcing pinning'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'clyde;setup'= {
            cand --url 'URL of the Git repository to use for the store'
            cand -u 'Update the activation scripts of an existing installation'
            cand --update-scripts 'Update the activation scripts of an existing installation'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;update'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;install'= {
            cand -r 'Uninstall then reinstall already installed packages'
            cand --reinstall 'Uninstall then reinstall already installed packages'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'clyde;uninstall'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;show'= {
            cand -l 'List application files instead of showing information'
            cand --list 'List application files instead of showing information'
            cand -j 'Use JSON output'
            cand --json 'Use JSON output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;search'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;doc'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;list'= {
            cand -j 'Use JSON output'
            cand --json 'Use JSON output'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;upgrade'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'clyde;help'= {
            cand setup 'Setup Clyde'
            cand update 'Update Clyde store'
            cand install 'Install applications'
            cand uninstall 'Uninstall applications (alias: remove)'
            cand show 'Show details about an application'
            cand search 'Search for available applications'
            cand doc 'Read documentation files provided by an application'
            cand list 'List installed applications'
            cand upgrade 'Upgrade all installed applications, enforcing pinning'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'clyde;help;setup'= {
        }
        &'clyde;help;update'= {
        }
        &'clyde;help;install'= {
        }
        &'clyde;help;uninstall'= {
        }
        &'clyde;help;show'= {
        }
        &'clyde;help;search'= {
        }
        &'clyde;help;doc'= {
        }
        &'clyde;help;list'= {
        }
        &'clyde;help;upgrade'= {
        }
        &'clyde;help;help'= {
        }
    ]
    $completions[$command]
}
