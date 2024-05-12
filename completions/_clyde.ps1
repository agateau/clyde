
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'clyde' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'clyde'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'clyde' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Setup Clyde')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update Clyde store')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install applications')
            [CompletionResult]::new('uninstall', 'uninstall', [CompletionResultType]::ParameterValue, 'Uninstall applications (alias: remove)')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show details about an application')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for available applications')
            [CompletionResult]::new('doc', 'doc', [CompletionResultType]::ParameterValue, 'Read documentation files provided by an application')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List installed applications')
            [CompletionResult]::new('upgrade', 'upgrade', [CompletionResultType]::ParameterValue, 'Upgrade all installed applications, enforcing pinning')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'clyde;setup' {
            [CompletionResult]::new('--url', 'url', [CompletionResultType]::ParameterName, 'URL of the Git repository to use for the store')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'Update the activation scripts of an existing installation')
            [CompletionResult]::new('--update-scripts', 'update-scripts', [CompletionResultType]::ParameterName, 'Update the activation scripts of an existing installation')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;update' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;install' {
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Uninstall then reinstall already installed packages')
            [CompletionResult]::new('--reinstall', 'reinstall', [CompletionResultType]::ParameterName, 'Uninstall then reinstall already installed packages')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help (see more with ''--help'')')
            break
        }
        'clyde;uninstall' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;show' {
            [CompletionResult]::new('-l', 'l', [CompletionResultType]::ParameterName, 'List application files instead of showing information')
            [CompletionResult]::new('--list', 'list', [CompletionResultType]::ParameterName, 'List application files instead of showing information')
            [CompletionResult]::new('-j', 'j', [CompletionResultType]::ParameterName, 'Use JSON output')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Use JSON output')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;search' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;doc' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;list' {
            [CompletionResult]::new('-j', 'j', [CompletionResultType]::ParameterName, 'Use JSON output')
            [CompletionResult]::new('--json', 'json', [CompletionResultType]::ParameterName, 'Use JSON output')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;upgrade' {
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            break
        }
        'clyde;help' {
            [CompletionResult]::new('setup', 'setup', [CompletionResultType]::ParameterValue, 'Setup Clyde')
            [CompletionResult]::new('update', 'update', [CompletionResultType]::ParameterValue, 'Update Clyde store')
            [CompletionResult]::new('install', 'install', [CompletionResultType]::ParameterValue, 'Install applications')
            [CompletionResult]::new('uninstall', 'uninstall', [CompletionResultType]::ParameterValue, 'Uninstall applications (alias: remove)')
            [CompletionResult]::new('show', 'show', [CompletionResultType]::ParameterValue, 'Show details about an application')
            [CompletionResult]::new('search', 'search', [CompletionResultType]::ParameterValue, 'Search for available applications')
            [CompletionResult]::new('doc', 'doc', [CompletionResultType]::ParameterValue, 'Read documentation files provided by an application')
            [CompletionResult]::new('list', 'list', [CompletionResultType]::ParameterValue, 'List installed applications')
            [CompletionResult]::new('upgrade', 'upgrade', [CompletionResultType]::ParameterValue, 'Upgrade all installed applications, enforcing pinning')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'clyde;help;setup' {
            break
        }
        'clyde;help;update' {
            break
        }
        'clyde;help;install' {
            break
        }
        'clyde;help;uninstall' {
            break
        }
        'clyde;help;show' {
            break
        }
        'clyde;help;search' {
            break
        }
        'clyde;help;doc' {
            break
        }
        'clyde;help;list' {
            break
        }
        'clyde;help;upgrade' {
            break
        }
        'clyde;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
