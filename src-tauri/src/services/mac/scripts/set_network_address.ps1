# Template script for setting/removing NetworkAddress registry value
# {GUID} will be replaced with the interface GUID at runtime
# {ACTION} will be replaced with Set-ItemProperty or Remove-ItemProperty at runtime

$guid = '{GUID}'
$guidTrimmed = $guid.Trim('{}').ToUpper()
$normalizedGuid = '{' + $guidTrimmed + '}'
$classKey = 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4D36E972-E325-11CE-BFC1-08002BE10318}'

$target = Get-ChildItem $classKey | Where-Object {
    try {
        $props = Get-ItemProperty $_.PSPath
        if ($null -ne $props -and $props.PSObject.Properties['NetCfgInstanceId']) {
            $props.NetCfgInstanceId.ToUpper() -eq $normalizedGuid
        } else {
            $false
        }
    } catch { $false }
} | Select-Object -First 1

if (-not $target) {
    throw '未找到对应的网络适配器注册表项'
}

{ACTION}
