# Shared prelude for MAC address scripts (PowerShell 5.1 compatible).
# Non-terminating errors must become terminating, otherwise powershell.exe
# exits 0 on failure and Rust would treat the operation as successful.
$ErrorActionPreference = 'Stop'
# Output/stderr must be UTF-8 so the Rust side can decode reliably (default is OEM/GBK).
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8

$classKey = 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002BE10318}'

function Assert-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
        throw '需要管理员权限，请以管理员身份运行本程序'
    }
}

function Get-TargetAdapter {
    $physical = @(Get-NetAdapter -Physical -ErrorAction SilentlyContinue)
    if ($physical.Count -eq 0) { throw '未找到物理网卡' }
    $up = @($physical | Where-Object { $_.Status -eq 'Up' })
    $pool = if ($up.Count -gt 0) { $up } else { $physical }
    # 优先选择承载默认路由的网卡（当前实际联网的那块）
    $route = Get-NetRoute -DestinationPrefix '0.0.0.0/0' -ErrorAction SilentlyContinue |
        Sort-Object -Property RouteMetric | Select-Object -First 1
    if ($route) {
        $hit = $pool | Where-Object { $_.ifIndex -eq $route.ifIndex } | Select-Object -First 1
        if ($hit) { return $hit }
    }
    return ($pool | Select-Object -First 1)
}

function Get-AdapterRegKey($adapter) {
    $guid = '{' + "$($adapter.InterfaceGuid)".Trim('{}').ToUpper() + '}'
    Get-ChildItem $classKey -ErrorAction SilentlyContinue | Where-Object {
        $props = Get-ItemProperty $_.PSPath -ErrorAction SilentlyContinue
        $props -and $props.PSObject.Properties['NetCfgInstanceId'] -and ($props.NetCfgInstanceId.ToUpper() -eq $guid)
    } | Select-Object -First 1
}

function Get-MacOverride($adapter) {
    $regKey = Get-AdapterRegKey $adapter
    if (-not $regKey) { return $null }
    $value = (Get-ItemProperty -Path $regKey.PSPath -ErrorAction SilentlyContinue).NetworkAddress
    if ($value -and "$value".Trim() -ne '') { return "$value" }
    return $null
}

function Restart-TargetAdapter($name) {
    Disable-NetAdapter -Name $name -Confirm:$false
    Start-Sleep -Milliseconds 500
    Enable-NetAdapter -Name $name -Confirm:$false
}

function Emit-AdapterInfo($name) {
    $adapter = Get-NetAdapter -Name $name
    [PSCustomObject]@{
        name         = "$($adapter.Name)"
        currentMac   = "$($adapter.MacAddress)"
        permanentMac = "$($adapter.PermanentAddress)"
        hasOverride  = [bool](Get-MacOverride $adapter)
    } | ConvertTo-Json -Compress
}
