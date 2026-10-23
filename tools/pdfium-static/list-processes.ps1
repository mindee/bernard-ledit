# Lists the processes started since the given time, oldest first, with their parent, CPU time and
# command line. watchdog.sh runs it to show what a stalled Windows build is stuck on.
param(
  [Parameter(Mandatory = $true)]
  [datetime] $Since
)

Get-CimInstance Win32_Process |
  Where-Object { $_.CreationDate -gt $Since } |
  Sort-Object CreationDate |
  ForEach-Object {
    $process = Get-Process -Id $_.ProcessId -ErrorAction SilentlyContinue
    [pscustomobject]@{
      Id         = $_.ProcessId
      Parent     = $_.ParentProcessId
      Name       = $_.Name
      Started    = $_.CreationDate.ToString('HH:mm:ss')
      CpuSeconds = if ($process) { [math]::Round($process.CPU) } else { $null }
      Command    = $_.CommandLine
    }
  } |
  Format-List |
  Out-String -Width 4096
