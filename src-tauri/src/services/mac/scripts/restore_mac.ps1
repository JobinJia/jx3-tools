# Remove MAC overrides from every physical adapter and restart the affected
# ones so they fall back to the permanent (burned-in) address.
# No-op when nothing is overridden, so the logon task never restarts adapters needlessly.
Assert-Admin
$restored = @()
foreach ($adapter in @(Get-NetAdapter -Physical -ErrorAction SilentlyContinue)) {
    $override = Get-MacOverride $adapter
    if (-not $override) { continue }
    $regKey = Get-AdapterRegKey $adapter
    Remove-ItemProperty -Path $regKey.PSPath -Name 'NetworkAddress'
    # 用户手动禁用的网卡只清除配置，不替用户启用
    if ($adapter.Status -ne 'Disabled') {
        Restart-TargetAdapter $adapter.Name
    }
    $restored += $adapter.Name
}

# 等待网卡恢复出厂 MAC（尽力而为，最多每块 10 秒）
foreach ($name in $restored) {
    for ($i = 0; $i -lt 20; $i++) {
        $current = Get-NetAdapter -Name $name -ErrorAction SilentlyContinue
        if ($current -and (("$($current.MacAddress)" -replace '-', '') -eq ("$($current.PermanentAddress)" -replace '-', ''))) { break }
        Start-Sleep -Milliseconds 500
    }
}

Emit-AdapterInfo (Get-TargetAdapter).Name
