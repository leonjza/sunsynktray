namespace SunSynkTrayWin.ViewModels;

public sealed class SettingsViewModel : ObservableObject
{
    private string _username = string.Empty;
    private string _password = string.Empty;
    private int _pollIntervalSeconds = 60;
    private bool _startOnBoot;
    private bool _useDarkMode = true;

    public string Username
    {
        get => _username;
        set => SetProperty(ref _username, value);
    }

    public string Password
    {
        get => _password;
        set => SetProperty(ref _password, value);
    }

    public int PollIntervalSeconds
    {
        get => _pollIntervalSeconds;
        set => SetProperty(ref _pollIntervalSeconds, Math.Max(1, value));
    }

    public bool StartOnBoot
    {
        get => _startOnBoot;
        set => SetProperty(ref _startOnBoot, value);
    }

    public bool UseDarkMode
    {
        get => _useDarkMode;
        set => SetProperty(ref _useDarkMode, value);
    }

    public bool HasCredentials =>
        !string.IsNullOrWhiteSpace(_username) && !string.IsNullOrWhiteSpace(_password);

    protected override void OnPropertyChangedExtended(string? propertyName)
    {
        if (propertyName is nameof(Username) or nameof(Password))
        {
            OnPropertyChanged(nameof(HasCredentials));
        }
    }
}
