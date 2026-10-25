# Documentation helper for the v0.1.0 wire protocol. No capture or agent launch.
# Each call closes its connection so the single-connection daemon remains usable.
function Invoke-CausativeLensRpc {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [AllowNull()][object]$Params = @{},
        [ValidateRange(100, 60000)][int]$TimeoutMs = 5000
    )
    $utf8 = [Text.UTF8Encoding]::new($false)
    $request = [ordered]@{ id = 1; method = $Method; params = $Params } |
        ConvertTo-Json -Depth 30 -Compress
    if ($utf8.GetByteCount($request) -gt 65535) {
        throw 'RPC request exceeds the 64 KiB documentation limit.'
    }
    $pipe = [IO.Pipes.NamedPipeClientStream]::new(
        '.', 'causativelens-rpc', [IO.Pipes.PipeDirection]::InOut
    )
    $writer = $null
    $reader = $null
    try {
        $pipe.Connect($TimeoutMs)
        $writer = [IO.StreamWriter]::new($pipe, $utf8, 4096, $true)
        $reader = [IO.StreamReader]::new($pipe, $utf8, $false, 4096, $true)
        $writer.WriteLine($request)
        $writer.Flush()
        $readTask = $reader.ReadLineAsync()
        if (-not $readTask.Wait($TimeoutMs)) { throw 'RPC response timed out.' }
        $line = $readTask.Result
        if ($null -eq $line) { throw 'Daemon closed the pipe without a response.' }
        $response = $line | ConvertFrom-Json -ErrorAction Stop
        if ($response.id -ne 1) { throw 'RPC response ID mismatch.' }
        if ($null -ne $response.error) {
            throw ('Daemon error: ' + ($response.error | ConvertTo-Json -Compress))
        }
        return $response.result
    } finally {
        if ($null -ne $reader) { $reader.Dispose() }
        if ($null -ne $writer) { $writer.Dispose() }
        $pipe.Dispose()
    }
}
