using System;
using System.Runtime.InteropServices;
using System.Text.Json;
using System.Threading.Tasks;

namespace NML.Windows.Core;

/// <summary>
/// FFI wrapper for NML Core (Rust)
/// </summary>
public class NMLCore : IDisposable
{
    private IntPtr _handle;
    private bool _disposed;

    // ============================================================================
    // Native Imports
    // ============================================================================
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr nml_init(IntPtr configPath);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_shutdown(IntPtr handle);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr nml_get_last_error();
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_free_string(IntPtr str);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern IntPtr nml_version();

    // Version
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_version_get_installed(IntPtr handle, VersionCallback callback);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_version_get_remote(IntPtr handle, VersionCallback callback);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_version_install(IntPtr handle, IntPtr versionId, ProgressCallback progress);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_version_uninstall(IntPtr handle, IntPtr versionId);

    // Launch
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_launch(IntPtr handle, IntPtr versionId, IntPtr playerName, [MarshalAs(UnmanagedType.I1)] bool isOffline);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_kill_minecraft(IntPtr handle);

    // Account
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_account_add_offline(IntPtr handle, IntPtr username, AccountCallback callback);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_account_get_all(IntPtr handle, AccountCallback callback);

    // P2P
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_p2p_start(IntPtr handle, IntPtr configJson);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_p2p_stop(IntPtr handle);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_p2p_discover(IntPtr handle, WorldCallback callback);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_p2p_host(IntPtr handle, IntPtr worldName, ushort localPort, WorldIdCallback callback);

    // Download
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_download_file(IntPtr handle, IntPtr url, IntPtr destination, ProgressCallback progress);

    // MCJEBooster
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern int nml_mcje_enable(IntPtr handle, IntPtr mcVersion);
    
    [DllImport("nml_core.dll", CallingConvention = CallingConvention.Cdecl)]
    private static extern void nml_mcje_get_stats(IntPtr handle, StatsCallback callback);

    // Callbacks
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void VersionCallback(IntPtr json);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void ProgressCallback(float progress);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void AccountCallback(IntPtr json);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void WorldCallback(IntPtr json);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void WorldIdCallback(IntPtr worldId);
    
    [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
    private delegate void StatsCallback(IntPtr json);

    // ============================================================================
    // Public API
    // ============================================================================
    
    public NMLCore(string? configPath = null)
    {
        IntPtr pathPtr = IntPtr.Zero;
        if (configPath != null)
        {
            pathPtr = Marshal.StringToHGlobalAnsi(configPath);
        }
        
        _handle = nml_init(pathPtr);
        
        if (pathPtr != IntPtr.Zero)
        {
            Marshal.FreeHGlobal(pathPtr);
        }
        
        if (_handle == IntPtr.Zero)
        {
            throw new InvalidOperationException($"Failed to initialize NML Core: {GetLastError()}");
        }
    }

    public static string GetVersion()
    {
        var ptr = nml_version();
        return Marshal.PtrToStringAnsi(ptr) ?? "unknown";
    }

    // Version Management
    public async Task<string[]> GetInstalledVersionsAsync()
    {
        var tcs = new TaskCompletionSource<string[]>();
        
        VersionCallback callback = (jsonPtr) =>
        {
            if (jsonPtr == IntPtr.Zero)
            {
                tcs.SetResult(Array.Empty<string>());
                return;
            }
            
            var json = Marshal.PtrToStringAnsi(jsonPtr);
            nml_free_string(jsonPtr);
            
            try
            {
                var versions = JsonSerializer.Deserialize<string[]>(json ?? "[]");
                tcs.SetResult(versions ?? Array.Empty<string>());
            }
            catch
            {
                tcs.SetResult(Array.Empty<string>());
            }
        };
        
        nml_version_get_installed(_handle, callback);
        
        return await tcs.Task;
    }

    public async Task<string[]> GetRemoteVersionsAsync()
    {
        var tcs = new TaskCompletionSource<string[]>();
        
        VersionCallback callback = (jsonPtr) =>
        {
            if (jsonPtr == IntPtr.Zero)
            {
                tcs.SetResult(Array.Empty<string>());
                return;
            }
            
            var json = Marshal.PtrToStringAnsi(jsonPtr);
            nml_free_string(jsonPtr);
            
            try
            {
                var versions = JsonSerializer.Deserialize<string[]>(json ?? "[]");
                tcs.SetResult(versions ?? Array.Empty<string>());
            }
            catch
            {
                tcs.SetResult(Array.Empty<string>());
            }
        };
        
        nml_version_get_remote(_handle, callback);
        
        return await tcs.Task;
    }

    public async Task InstallVersionAsync(string versionId, IProgress<float> progress)
    {
        var versionPtr = Marshal.StringToHGlobalAnsi(versionId);
        
        var tcs = new TaskCompletionSource<bool>();
        
        ProgressCallback progressCb = (p) =>
        {
            progress.Report(p);
        };
        
        var result = nml_version_install(_handle, versionPtr, progressCb);
        
        Marshal.FreeHGlobal(versionPtr);
        
        if (result != 0)
        {
            throw new InvalidOperationException($"Install failed: {GetLastError()}");
        }
    }

    // Launch
    public void Launch(string versionId, string playerName, bool isOffline = false)
    {
        var versionPtr = Marshal.StringToHGlobalAnsi(versionId);
        var playerPtr = Marshal.StringToHGlobalAnsi(playerName);
        
        var result = nml_launch(_handle, versionPtr, playerPtr, isOffline);
        
        Marshal.FreeHGlobal(versionPtr);
        Marshal.FreeHGlobal(playerPtr);
        
        if (result != 0)
        {
            throw new InvalidOperationException($"Launch failed: {GetLastError()}");
        }
    }

    // Account
    public async Task<AccountInfo> AddOfflineAccountAsync(string username)
    {
        var namePtr = Marshal.StringToHGlobalAnsi(username);
        var tcs = new TaskCompletionSource<AccountInfo>();
        
        AccountCallback callback = (jsonPtr) =>
        {
            if (jsonPtr == IntPtr.Zero)
            {
                tcs.SetException(new InvalidOperationException("Failed to create account"));
                return;
            }
            
            var json = Marshal.PtrToStringAnsi(jsonPtr);
            nml_free_string(jsonPtr);
            
            try
            {
                var account = JsonSerializer.Deserialize<AccountInfo>(json ?? "{}");
                tcs.SetResult(account ?? new AccountInfo());
            }
            catch (Exception ex)
            {
                tcs.SetException(ex);
            }
        };
        
        nml_account_add_offline(_handle, namePtr, callback);
        
        Marshal.FreeHGlobal(namePtr);
        
        return await tcs.Task;
    }

    // P2P
    public void StartP2P()
    {
        var result = nml_p2p_start(_handle, IntPtr.Zero);
        if (result != 0)
        {
            throw new InvalidOperationException($"P2P start failed: {GetLastError()}");
        }
    }

    public void StopP2P()
    {
        nml_p2p_stop(_handle);
    }

    public async Task<WorldInfo[]> DiscoverWorldsAsync()
    {
        var tcs = new TaskCompletionSource<WorldInfo[]>();
        
        WorldCallback callback = (jsonPtr) =>
        {
            if (jsonPtr == IntPtr.Zero)
            {
                tcs.SetResult(Array.Empty<WorldInfo>());
                return;
            }
            
            var json = Marshal.PtrToStringAnsi(jsonPtr);
            nml_free_string(jsonPtr);
            
            try
            {
                var worlds = JsonSerializer.Deserialize<WorldInfo[]>(json ?? "[]");
                tcs.SetResult(worlds ?? Array.Empty<WorldInfo>());
            }
            catch
            {
                tcs.SetResult(Array.Empty<WorldInfo>());
            }
        };
        
        nml_p2p_discover(_handle, callback);
        
        return await tcs.Task;
    }

    // MCJEBooster
    public void EnableMCJEBooster(string mcVersion)
    {
        var versionPtr = Marshal.StringToHGlobalAnsi(mcVersion);
        var result = nml_mcje_enable(_handle, versionPtr);
        Marshal.FreeHGlobal(versionPtr);
        
        if (result != 0)
        {
            throw new InvalidOperationException($"MCJEBooster enable failed: {GetLastError()}");
        }
    }

    // Helpers
    private string GetLastError()
    {
        var ptr = nml_get_last_error();
        if (ptr == IntPtr.Zero) return "Unknown error";
        
        var msg = Marshal.PtrToStringAnsi(ptr);
        nml_free_string(ptr);
        
        return msg ?? "Unknown error";
    }

    public void Dispose()
    {
        if (!_disposed)
        {
            if (_handle != IntPtr.Zero)
            {
                nml_shutdown(_handle);
                _handle = IntPtr.Zero;
            }
            _disposed = true;
        }
    }
}

// Data Models
public class AccountInfo
{
    public string Id { get; set; } = "";
    public string Type { get; set; } = "";
    public string Username { get; set; } = "";
    public string UUID { get; set; } = "";
}

public class WorldInfo
{
    public string WorldId { get; set; } = "";
    public string WorldName { get; set; } = "";
    public string Motd { get; set; } = "";
    public int PlayerCount { get; set; }
    public int MaxPlayers { get; set; }
    public long LatencyMs { get; set; }
}

public class PerformanceStats
{
    public float TPS { get; set; }
    public float MSPT { get; set; }
    public int EntityCount { get; set; }
    public bool Optimized { get; set; }
}
