<#
    PC Cleaner - icon in the notification area (tray). The X button can hide the window there (Settings),
    and the automatic cleaning at sign-in (-AutoClean) runs there without showing the window.
#>

$script:TrayState = 'Normal'
$script:TrayHintShown = $false
$script:Exiting = $false

function New-TrayIcon {
    $t = New-Object Windows.Forms.NotifyIcon
    $ico = Join-Path $AppDir 'PCCleaner.ico'
    $t.Icon = if (Test-Path -LiteralPath $ico) { New-Object Drawing.Icon($ico, [Windows.Forms.SystemInformation]::SmallIconSize) } else { [Drawing.SystemIcons]::Application }
    $t.Text = 'PC Cleaner'
    $menu = New-Object Windows.Forms.ContextMenuStrip
    [void]$menu.Items.Add((L 'Otwórz PC Cleaner' 'Open PC Cleaner'), $null, { Show-FromTray })
    [void]$menu.Items.Add((L 'Zakończ' 'Exit'), $null, { Exit-App })
    $t.ContextMenuStrip = $menu
    $t.Add_MouseClick({ if ($_.Button -eq 'Left') { Show-FromTray } })
    $t.Add_BalloonTipClicked({ Show-FromTray })
    $script:UI.Tray = $t
}

# Hides the window; the program keeps running in the notification area
function Hide-ToTray([bool]$quiet = $false) {
    $f = $script:UI.Window
    if ($f.WindowState -ne 'Minimized') { $script:TrayState = "$($f.WindowState)" }
    $script:UI.Tray.Visible = $true
    $f.Hide()
    if (-not $quiet -and -not $script:TrayHintShown) {
        $script:TrayHintShown = $true
        $script:UI.Tray.ShowBalloonTip(5000, 'PC Cleaner', (L 'Program działa dalej w zasobniku. Kliknij ikonę, żeby go otworzyć.' 'The program keeps running in the notification area. Click the icon to open it.'), 'Info')
    }
}

function Show-FromTray {
    $f = $script:UI.Window
    if ($f.IsDisposed) { return }
    $f.Show()
    if ($f.WindowState -eq 'Minimized') { $f.WindowState = $script:TrayState }
    $f.Activate()
    $script:UI.Tray.Visible = $false
}

# Really closes the program (tray menu, update, restart) - the X button may only hide the window
function Exit-App {
    $script:Exiting = $true
    if (-not $script:UI.Window.IsDisposed) { $script:UI.Window.Close() }
}

# X button: hides to the tray when Settings say so (not when Windows shuts down or the program restarts)
function Test-CloseToTray($e) {
    $Config.CloseToTray -eq $true -and $e.CloseReason -eq 'UserClosing' -and -not $script:Exiting -and -not $script:Restart
}

# -AutoClean (Task Scheduler at sign-in): scan, clean the categories the plan recommends, show the result.
# Leftovers and large files always wait for the user. The program closes itself unless the user opens it.
function Invoke-AutoClean {
    $Config.AutoCleanLast = (Get-Date).ToString('yyyy-MM-dd')
    try { Save-Config $Config } catch { Write-Log "Automatic cleaning - saving the date: $($_.Exception.Message)" }
    Write-Log "Automatic cleaning at sign-in ($($Config.AutoClean))."
    $t = $script:UI.Tray
    $t.Text = L 'PC Cleaner - automatyczne czyszczenie...' 'PC Cleaner - automatic cleaning...'
    Invoke-UiAction { Start-WindowScan }
    foreach ($p in $script:Leftovers) { $p.Selected = $false }
    Update-PlanPage
    $bytes = Get-SelectedTotal
    $msg = if ($Preview) { (L 'Tryb podglądu: do wyczyszczenia {0}, nic nie zostało usunięte.' 'Preview mode: {0} to clean, nothing was deleted.') -f (Format-Size $bytes) }
    elseif ($bytes -le 0) { L 'Nie było nic do wyczyszczenia.' 'There was nothing to clean.' }
    else {
        Invoke-UiAction { Start-WindowCleanup }
        if ($script:CleanupResult) { (L 'Zwolniono {0}. Kliknij, żeby zobaczyć szczegóły.' '{0} freed. Click to see the details.') -f (Format-Size $script:CleanupResult.Total) }
        else { L 'Czyszczenie nie powiodło się - szczegóły w Dzienniku.' 'The cleaning failed - see the Log.' }
    }
    Write-Log "Automatic cleaning: $msg"
    $t.Text = 'PC Cleaner'
    $t.ShowBalloonTip(10000, (L 'PC Cleaner - automatyczne czyszczenie' 'PC Cleaner - automatic cleaning'), $msg, 'Info')
    Wait-Responsive 15000
    if (-not $script:UI.Window.IsDisposed -and -not $script:UI.Window.Visible) { Exit-App }
}
