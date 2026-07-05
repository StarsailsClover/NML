using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;

namespace NML.Windows.Views;

public sealed partial class AccountPage : Page
{
    public AccountPage()
    {
        this.InitializeComponent();
    }

    private void AddMicrosoftButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Implement Microsoft account login
    }

    private async void AddOfflineButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Show input dialog for username
        var core = App.Current.Services.GetService(typeof(Core.NMLCore)) as Core.NMLCore;
        if (core == null) return;

        try
        {
            var account = await core.AddOfflineAccountAsync("Player");
            // TODO: Refresh account list
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"Failed to add account: {ex.Message}");
        }
    }

    private void AddThirdPartyButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Implement third-party account
    }

    private void SelectButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Select account for launch
    }

    private void DeleteButton_Click(object sender, RoutedEventArgs e)
    {
        // TODO: Delete account
    }
}
