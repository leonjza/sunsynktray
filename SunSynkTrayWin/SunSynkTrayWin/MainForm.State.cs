using System.ComponentModel;
using System.Windows.Forms;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private void BindSettingsToState()
    {
        SettingsForm.usernameTextBox.DataBindings.Add("Text", _appState.Settings, nameof(_appState.Settings.Username), false, DataSourceUpdateMode.OnPropertyChanged);
        SettingsForm.passwordTextBox.DataBindings.Add("Text", _appState.Settings, nameof(_appState.Settings.Password), false, DataSourceUpdateMode.OnPropertyChanged);
        SettingsForm.pollIntervalNumeric.DataBindings.Add("Value", _appState.Settings, nameof(_appState.Settings.PollIntervalSeconds), true, DataSourceUpdateMode.OnPropertyChanged);
        SettingsForm.startOnBootCheckBox.DataBindings.Add("Checked", _appState.Settings, nameof(_appState.Settings.StartOnBoot), false, DataSourceUpdateMode.OnPropertyChanged);
        SettingsForm.themeToggleCheckBox.DataBindings.Add("Checked", _appState.Settings, nameof(_appState.Settings.UseDarkMode), false, DataSourceUpdateMode.OnPropertyChanged);

        // Readiness UI was removed from SettingsForm; no bindings needed.

        _appState.StateChanged += AppStateOnStateChanged;
    }

    private void AppStateOnStateChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (_isLoadingSettings)
        {
            return;
        }

        switch (e.PropertyName)
        {
            case nameof(ViewModels.SettingsViewModel.PollIntervalSeconds):
                break;
            case nameof(ViewModels.SettingsViewModel.UseDarkMode):
                _currentTheme = _appState.Settings.UseDarkMode ? Theme.AppTheme.Dark : Theme.AppTheme.Light;
                ApplyTheme(_currentTheme);
                break;
        }

        PersistSettings();
        UpdateReadinessIndicator();
    }
}
