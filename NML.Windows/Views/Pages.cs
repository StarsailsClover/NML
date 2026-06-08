using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NML.Windows.Views;

public sealed partial class DownloadPage : Page
{
    public DownloadPage()
    {
        this.InitializeComponent();
    }

    private void SearchButton_Click(object sender, RoutedEventArgs e) { }
}

public sealed partial class MultiplayerPage : Page
{
    public MultiplayerPage()
    {
        this.InitializeComponent();
    }

    private void HostButton_Click(object sender, RoutedEventArgs e) { }
    private void RefreshButton_Click(object sender, RoutedEventArgs e) { }
    private void JoinButton_Click(object sender, RoutedEventArgs e) { }
}

public sealed partial class ModsPage : Page
{
    public ModsPage()
    {
        this.InitializeComponent();
    }

    private void SearchButton_Click(object sender, RoutedEventArgs e) { }
    private void InstallButton_Click(object sender, RoutedEventArgs e) { }
    private void ModToggle_Click(object sender, RoutedEventArgs e) { }
    private void UpdateButton_Click(object sender, RoutedEventArgs e) { }
    private void DeleteButton_Click(object sender, RoutedEventArgs e) { }
}

public sealed partial class ServerPage : Page
{
    public ServerPage()
    {
        this.InitializeComponent();
    }

    private void CreateServerButton_Click(object sender, RoutedEventArgs e) { }
    private void ImportServerButton_Click(object sender, RoutedEventArgs e) { }
    private void StartButton_Click(object sender, RoutedEventArgs e) { }
    private void ConfigButton_Click(object sender, RoutedEventArgs e) { }
    private void ConsoleButton_Click(object sender, RoutedEventArgs e) { }
}

public sealed partial class SettingsPage : Page
{
    public SettingsPage()
    {
        this.InitializeComponent();
    }

    private void BrowsePath_Click(object sender, RoutedEventArgs e) { }
}

public sealed partial class AccountPage : Page
{
    public AccountPage()
    {
        this.InitializeComponent();
    }

    private void AddMicrosoftButton_Click(object sender, RoutedEventArgs e) { }
    private void AddOfflineButton_Click(object sender, RoutedEventArgs e) { }
    private void AddThirdPartyButton_Click(object sender, RoutedEventArgs e) { }
    private void SelectButton_Click(object sender, RoutedEventArgs e) { }
    private void DeleteButton_Click(object sender, RoutedEventArgs e) { }
}
