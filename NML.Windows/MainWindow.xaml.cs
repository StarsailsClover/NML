using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using NML.Windows.Views;

namespace NML.Windows;

public sealed partial class MainWindow : Window
{
    public MainWindow()
    {
        this.InitializeComponent();
        Title = "NML - Minecraft Launcher";
        
        ExtendsContentIntoTitleBar = true;
        SetTitleBar(NavView);
    }

    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.SelectedItem is NavigationViewItem item)
        {
            var tag = item.Tag?.ToString();
            
            switch (tag)
            {
                case "Home":
                    ContentFrame.Navigate(typeof(HomePage));
                    break;
                case "Download":
                    ContentFrame.Navigate(typeof(DownloadPage));
                    break;
                case "Multiplayer":
                    ContentFrame.Navigate(typeof(MultiplayerPage));
                    break;
                case "Mods":
                    ContentFrame.Navigate(typeof(ModsPage));
                    break;
                case "Server":
                    ContentFrame.Navigate(typeof(ServerPage));
                    break;
                case "Settings":
                    ContentFrame.Navigate(typeof(SettingsPage));
                    break;
                case "Account":
                    ContentFrame.Navigate(typeof(AccountPage));
                    break;
            }
        }
    }
}
