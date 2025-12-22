# Template script for restarting a network adapter
# {NAME} will be replaced with the adapter name at runtime
$name = '{NAME}'
Disable-NetAdapter -Name $name -Confirm:$false -ErrorAction Stop
Start-Sleep -Milliseconds 800
Enable-NetAdapter -Name $name -Confirm:$false -ErrorAction Stop
