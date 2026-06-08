using Microsoft.UI.Xaml;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using NML.Windows.Core;
using System;

namespace NML.Windows;

public partial class App : Application
{
    public new static App Current => (App)Application.Current;
    public IServiceProvider Services { get; private set; } = null!;
    public Window? MainWindow { get; private set; }

    public App()
    {
        this.InitializeComponent();
        ConfigureServices();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        MainWindow = new MainWindow();
        MainWindow.Activate();
    }

    private void ConfigureServices()
    {
        var services = new ServiceCollection();
        
        services.AddLogging(builder =>
        {
            builder.AddDebug();
            builder.SetMinimumLevel(LogLevel.Debug);
        });

        // NML Core via FFI
        services.AddSingleton<NMLCore>(provider =>
        {
            return new NMLCore();
        });

        // ViewModels
        services.AddTransient<HomeViewModel>();
        services.AddTransient<DownloadViewModel>();
        services.AddTransient<MultiplayerViewModel>();
        services.AddTransient<SettingsViewModel>();
        services.AddTransient<ModsViewModel>();
        services.AddTransient<ServerViewModel>();
        services.AddTransient<AccountViewModel>();

        Services = services.BuildServiceProvider();
    }
}

// ViewModels with Core integration
public class HomeViewModel
{
    private readonly NMLCore _core;

    public HomeViewModel(NMLCore core)
    {
        _core = core;
    }

    public async Task<string[]> GetVersionsAsync()
    {
        return await _core.GetInstalledVersionsAsync();
    }

    public void LaunchGame(string versionId, string playerName)
    {
        _core.Launch(versionId, playerName, false);
    }
}

public class DownloadViewModel
{
    public string Title => "下载";
    
    public DownloadViewModel()
    {
    }
}

public class MultiplayerViewModel
{
    private readonly NMLCore _core;

    public MultiplayerViewModel(NMLCore core)
    {
        _core = core;
    }

    public void StartP2P()
    {
        _core.StartP2P();
    }

    public void StopP2P()
    {
        _core.StopP2P();
    }

    public async Task<WorldInfo[]> DiscoverWorldsAsync()
    {
        return await _core.DiscoverWorldsAsync();
    }
}

public class SettingsViewModel
{
    public string Title => "设置";
}

public class ModsViewModel
{
    public string Title => "Mod管理";
}

public class ServerViewModel
{
    public string Title => "服务器";
}

public class AccountViewModel
{
    private readonly NMLCore _core;

    public AccountViewModel(NMLCore core)
    {
        _core = core;
    }

    public async Task<AccountInfo> AddOfflineAccountAsync(string username)
    {
        return await _core.AddOfflineAccountAsync(username);
    }
}
