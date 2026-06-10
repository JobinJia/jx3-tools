# Apply a new MAC override and verify the driver actually accepted it.
# {MAC} is replaced with 12 uppercase hex chars at runtime.
Assert-Admin
$newMac = '{MAC}'
$adapter = Get-TargetAdapter
$regKey = Get-AdapterRegKey $adapter
if (-not $regKey) { throw '未找到网卡对应的注册表项' }

Set-ItemProperty -Path $regKey.PSPath -Name 'NetworkAddress' -Value $newMac -Force
Restart-TargetAdapter $adapter.Name

# 轮询验证驱动是否真的应用了新 MAC（最多 10 秒）
$expected = ($newMac -replace '(..)(?!$)', '$1-')
$applied = $false
for ($i = 0; $i -lt 20; $i++) {
    Start-Sleep -Milliseconds 500
    $current = (Get-NetAdapter -Name $adapter.Name -ErrorAction SilentlyContinue).MacAddress
    if ("$current" -eq $expected) { $applied = $true; break }
}

if (-not $applied) {
    # 驱动没接受：回滚注册表并恢复网卡，避免留下无效配置
    Remove-ItemProperty -Path $regKey.PSPath -Name 'NetworkAddress' -ErrorAction SilentlyContinue
    Restart-TargetAdapter $adapter.Name
    throw '网卡驱动未接受新的 MAC 地址，该网卡可能不支持修改（无线网卡尤其常见）'
}

Emit-AdapterInfo $adapter.Name
