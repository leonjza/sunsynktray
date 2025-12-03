using System.ComponentModel;
using System.Windows.Forms;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private void BindSettingsToState()
    {
        usernameTextBox.DataBindings.Add("Text", _appState.Settings, nameof(_appState.Settings.Username), false, DataSourceUpdateMode.OnPropertyChanged);
        passwordTextBox.DataBindings.Add("Text", _appState.Settings, nameof(_appState.Settings.Password), false, DataSourceUpdateMode.OnPropertyChanged);
        pollIntervalNumeric.DataBindings.Add("Value", _appState.Settings, nameof(_appState.Settings.PollIntervalSeconds), true, DataSourceUpdateMode.OnPropertyChanged);
        startOnBootCheckBox.DataBindings.Add("Checked", _appState.Settings, nameof(_appState.Settings.StartOnBoot), false, DataSourceUpdateMode.OnPropertyChanged);
        themeToggleCheckBox.DataBindings.Add("Checked", _appState.Settings, nameof(_appState.Settings.UseDarkMode), false, DataSourceUpdateMode.OnPropertyChanged);

        readinessStatusLabel.DataBindings.Add("Text", _statusViewModel, nameof(_statusViewModel.ReadinessText), false, DataSourceUpdateMode.OnPropertyChanged);
        readinessDot.DataBindings.Add("BackColor", _statusViewModel, nameof(_statusViewModel.ReadinessColor), false, DataSourceUpdateMode.OnPropertyChanged);

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
