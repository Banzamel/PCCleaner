<#
    PC Cleaner - automatic cleaning at sign-in (Settings, off by default). A Task Scheduler task of the signed-in
    account starts the program with -AutoClean and the highest rights, so UAC does not ask. How often it really
    cleans (every sign-in, daily, weekly) is checked at start by Test-AutoCleanDue (lib\Core\Config.ps1).
    The uninstaller removes the task.
#>

$AutoCleanTask = 'PC Cleaner - automatic cleaning'

# Creates, updates or removes the task to match $Config.AutoClean. Returns an error message, or $null when it worked.
function Set-AutoCleanTask {
    try {
        if ("$($Config.AutoClean)" -in 'logon', 'daily', 'weekly') {
            $ps = Join-Path $env:SystemRoot 'System32\WindowsPowerShell\v1.0\powershell.exe'
            $arg = "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$(Join-Path $AppDir 'PCCleaner.ps1')`" -AutoClean"
            $action = New-ScheduledTaskAction -Execute $ps -Argument $arg -WorkingDirectory $AppDir
            $user = [Security.Principal.WindowsIdentity]::GetCurrent().Name
            $trigger = New-ScheduledTaskTrigger -AtLogOn -User $user
            $trigger.Delay = 'PT2M'   # let the sign-in settle first
            $principal = New-ScheduledTaskPrincipal -UserId $user -LogonType Interactive -RunLevel Highest
            # No time limit: after the cleaning the user may keep the window open
            $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero)
            Register-ScheduledTask -TaskName $AutoCleanTask -Action $action -Trigger $trigger -Principal $principal -Settings $settings `
                -Description 'PC Cleaner: automatic cleaning of the recommended categories at sign-in (Settings).' -Force -ErrorAction Stop | Out-Null
            Write-Log "Automatic cleaning: task set ($($Config.AutoClean))."
        }
        elseif (Get-ScheduledTask -TaskName $AutoCleanTask -ErrorAction SilentlyContinue) {
            Unregister-ScheduledTask -TaskName $AutoCleanTask -Confirm:$false -ErrorAction Stop
            Write-Log 'Automatic cleaning: task removed.'
        }
        $null
    }
    catch { Write-Log "Automatic cleaning task: $($_.Exception.Message)"; $_.Exception.Message }
}
