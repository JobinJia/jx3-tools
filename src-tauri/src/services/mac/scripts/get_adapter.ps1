Get-NetAdapter | Where-Object {
    $_.Status -eq 'Up' -and
    $_.HardwareInterface -eq $true -and
    -not $_.Virtual
} | Sort-Object -Property InterfaceMetric |
Select-Object -First 1 -Property Name, InterfaceGuid, MacAddress |
ConvertTo-Json -Compress
