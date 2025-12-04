using System;
using System.Drawing;
using System.Threading.Tasks;
using System.Windows.Forms;
using Microsoft.Win32;
using SunSynkTrayWin.Api;
using SunSynkTrayWin.Storage;
using SunSynkTrayWin.Theme;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private async void MainForm_Load(object? sender, EventArgs e)
    {
        LoadStoredCredentials();
        // Start hidden in the tray.
        HideToTray();
        UpdateTrayText();
        await StartInitialLoginIfPossibleAsync();
    }

    private async Task StartMonitoringAsync(string source, bool resetCountdown = true)
    {
        await _monitoring.StartMonitoringAsync(source, resetCountdown);
    }

    private async Task<TokenData?> RunLoginAsync(bool requireCredentials = true, bool forceLoadPlants = false)
    {
        var username = _appState.Settings.Username.Trim();
        var password = _appState.Settings.Password;

        if (string.IsNullOrWhiteSpace(username) || string.IsNullOrWhiteSpace(password))
        {
            if (requireCredentials)
            {
                MessageBox.Show(this, "Please enter both username and password.", "Missing credentials",
                    MessageBoxButtons.OK, MessageBoxIcon.Warning);
            }
            return null;
        }

        ToggleSettingsUi(enabled: false);
        authLogTextBox.Clear();
        AppendAuthLog("Starting login...");

        TokenData? token = null;
        try
        {
            var progress = new Progress<string>(AppendAuthLog);
            await StatusOperationRunner.RunWithStatusAsync(
                _statusViewModel,
                context: "Authentication",
                workingStatus: "Connecting...",
                successStatus: "Authenticated",
                async () =>
                {
                    token = await _apiClient.LoginAsync(username, password, progress);
                    if (forceLoadPlants || !_appState.PlantSelection.Id.HasValue)
                    {
                        await LoadPlantsAsync();
                    }
                    _appState.UpdateAuthenticationState();
                    await _monitoring.StartMonitoringAsync("Initial");
                },
                onError: ex =>
                {
                    _apiClient.ClearCachedAuthentication();
                    MessageBox.Show(this, $"Login failed: {ex.Message}", "Test connection",
                        MessageBoxButtons.OK, MessageBoxIcon.Error);
                },
                log: AppendAuthLog);
        }
        catch
        {
            // Errors are handled in onError; keep flow going.
        }
        finally
        {
            ToggleSettingsUi(enabled: true);
            _appState.UpdateAuthenticationState();
        }

        return token;
    }

    private void ToggleSettingsUi(bool enabled)
    {
        testConnectionButton.Enabled = enabled;
        saveSettingsButton.Enabled = enabled;
        usernameTextBox.Enabled = enabled;
        passwordTextBox.Enabled = enabled;
        pollIntervalNumeric.Enabled = enabled;
        startOnBootCheckBox.Enabled = enabled;
    }

    private void AppendAuthLog(string message)
    {
        this.InvokeIfRequired(() =>
        {
            var line = $"[{DateTime.Now:HH:mm:ss}] {message}";
            authLogTextBox.AppendText(line + Environment.NewLine);
        });
    }

    private async Task StartInitialLoginIfPossibleAsync()
    {
        var username = _appState.Settings.Username;
        var password = _appState.Settings.Password;
        if (string.IsNullOrWhiteSpace(username) || string.IsNullOrWhiteSpace(password))
        {
            return;
        }

        await RunLoginAsync(requireCredentials: false);
    }

    private void LoadStoredCredentials()
    {
        _isLoadingSettings = true;
        try
        {
            var settings = _settingsStore.LoadSettings();
            _appState.ApplySettings(settings);
            ApplyTheme(_appState.Settings.UseDarkMode ? AppTheme.Dark : AppTheme.Light);
            ApplyStartupSetting(_appState.Settings.StartOnBoot);
        }
        finally
        {
            _isLoadingSettings = false;
            UpdateReadinessIndicator();
        }
    }

    private void PersistSettings()
    {
        if (_isLoadingSettings)
        {
            return;
        }

        var settings = _appState.ToSettingsData();
        _settingsStore.SaveSettings(settings);
        ApplyStartupSetting(settings.StartOnBoot);
        UpdateReadinessIndicator();
    }

    private async Task LoadPlantsAsync()
    {
        if (string.IsNullOrWhiteSpace(_apiClient.AccessToken))
        {
            return;
        }

        try
        {
            await StatusOperationRunner.RunWithStatusAsync(
                _statusViewModel,
                context: "Loading plants",
                workingStatus: "Loading plants...",
                successStatus: "Authenticated",
                async () =>
                {
                    var plantsPage = await _apiClient.GetPlantsAsync();
                    PopulatePlantList(plantsPage);
                },
                onError: _ => UpdateReadinessIndicator(),
                onSuccess: UpdateReadinessIndicator,
                log: AppendAuthLog);
        }
        catch
        {
            // Errors are logged; keep flow going so UI remains responsive.
        }
    }

    private void PopulatePlantList(PlantPage plantsPage)
    {
        _isPopulatingPlants = true;
        try
        {
            plantListBox.DataSource = null;
            plantListBox.DisplayMember = nameof(PlantInfo.Name);
            plantListBox.ValueMember = nameof(PlantInfo.Id);
            plantListBox.DataSource = plantsPage.Infos;
            plantListBox.ClearSelected();
            _appState.SetPlantSelection(_appState.PlantSelection.Id, _appState.PlantSelection.Name, available: false);

            if (_appState.PlantSelection.Id.HasValue)
            {
                foreach (var item in plantsPage.Infos)
                {
                    if (item.Id == _appState.PlantSelection.Id.Value)
                    {
                        plantListBox.SelectedItem = item;
                        SetSelectedPlant(item.Id, item.Name, available: true);
                        break;
                    }
                }
            }
        }
        finally
        {
            _isPopulatingPlants = false;
        }

        if (!_appState.PlantSelection.IsAvailable)
        {
            SetSelectedPlant(_appState.PlantSelection.Id, null, available: false, showMissingMessage: _appState.PlantSelection.Id.HasValue);
        }
    }

    private async void PlantListBox_SelectedIndexChanged(object? sender, EventArgs e)
    {
        if (_isPopulatingPlants)
        {
            return;
        }

        var selectedPlant = plantListBox.SelectedItem as PlantInfo;
        if (selectedPlant != null)
        {
            SetSelectedPlant(selectedPlant.Id, selectedPlant.Name, available: true);
            PersistSettings();
        }
        else
        {
            SetSelectedPlant(null, null, available: false);
        }
        UpdateReadinessIndicator();

        if (_appState.IsReady)
        {
            await StartMonitoringAsync("Plant change", resetCountdown: false);
        }
    }

    private void ResetSettingsButton_Click(object? sender, EventArgs e)
    {
        var defaults = _settingsStore.GetDefaultSettings();
        _appState.ApplySettings(defaults);
        _apiClient.ClearCachedAuthentication();
        _settingsStore.SaveSettings(defaults);
        _statusViewModel.ConnectionStatus = "Not connected";
        _monitoring.NotifyReadinessChanged();
        UpdateReadinessIndicator();
    }

    private void PlantListBox_Format(object? sender, ListControlConvertEventArgs e)
    {
        if (e.ListItem is PlantInfo plant)
        {
            var updated = plant.UpdatedAt?.ToLocalTime().ToString("g") ?? "n/a";
            var status = plant.Status == 1 ? "Online" : "Offline";
            var id = plant.Id;
            var pac = plant.Pac.HasValue ? $"{plant.Pac} W" : "n/a";
            e.Value = $"#{id} {plant.Name} — {status}, PAC: {pac}, Last update: {updated}";
        }
    }

    private void UpdateReadinessIndicator()
    {
        _appState.RefreshReadiness();
        var plantControlsVisible = _appState.IsAuthenticated;
        plantLabel.Visible = plantControlsVisible;
        plantListBox.Visible = plantControlsVisible;
        selectedPlantLabel.Text = _appState.PlantSelection.DisplayLabel;
        selectedPlantLabel.Visible = plantControlsVisible && _appState.PlantSelection.Id.HasValue;
    }

    private void EnsureSettingsWindow()
    {
        if (_settingsForm != null)
        {
            return;
        }

        _settingsForm = new Form
        {
            Text = "Settings",
            StartPosition = FormStartPosition.CenterParent,
            ClientSize = new Size(780, 650),
            MinimumSize = new Size(740, 600)
        };
        _settingsForm.FormClosing += SettingsForm_FormClosing;

        settingsLayout.Parent?.Controls.Remove(settingsLayout);
        settingsLayout.Dock = DockStyle.Fill;
        settingsLayout.Padding = new Padding(10);
        _settingsForm.Controls.Add(settingsLayout);

        // Scale settings window and key rows for current DPI so HiDPI isn't cramped and LoDPI isn't oversized.
        var scale = Math.Max(1f, _settingsForm.DeviceDpi / 96f);
        var effectiveScale = Math.Min(scale, 1.75f);
        _settingsForm.ClientSize = new Size(
            (int)Math.Round(640 * effectiveScale),
            (int)Math.Round(600 * effectiveScale));
        _settingsForm.MinimumSize = new Size(
            (int)Math.Round(620 * effectiveScale),
            (int)Math.Round(560 * effectiveScale));
        settingsLayout.Padding = new Padding((int)Math.Round(8 * effectiveScale));

        if (settingsLayout.RowStyles.Count > 10)
        {
            var plantHeight = (int)Math.Round(32f * 8f * effectiveScale);
            settingsLayout.RowStyles[7].Height = plantHeight;
            settingsLayout.RowStyles[7].SizeType = SizeType.Absolute;

            var lineHeight = authLogTextBox.Font?.GetHeight() ?? 16f;
            var logHeight = (int)Math.Round(lineHeight * 6.0f * effectiveScale);
            settingsLayout.RowStyles[10].Height = logHeight;
            settingsLayout.RowStyles[10].SizeType = SizeType.Absolute;
            authLogTextBox.Height = logHeight;
        }

        plantListBox.ItemHeight = (int)Math.Round(32 * effectiveScale);
        ApplyTheme(_currentTheme);
    }

    private void SettingsForm_FormClosing(object? sender, FormClosingEventArgs e)
    {
        if (e.CloseReason == CloseReason.UserClosing)
        {
            e.Cancel = true;
            _settingsForm?.Hide();
        }
    }

    private void ApplyStartupSetting(bool enable)
    {
        const string runKeyPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
        var appName = "SunSynkTrayWin";
        try
        {
            using var key = Registry.CurrentUser.OpenSubKey(runKeyPath, writable: true) ??
                            Registry.CurrentUser.CreateSubKey(runKeyPath, true);
            if (key == null)
            {
                return;
            }

            if (enable)
            {
                var exePath = Application.ExecutablePath;
                if (!string.IsNullOrWhiteSpace(exePath))
                {
                    key.SetValue(appName, $"\"{exePath}\"");
                }
            }
            else
            {
                key.DeleteValue(appName, false);
            }
        }
        catch
        {
            // Swallow errors to avoid crashing the UI; the setting simply won't apply.
        }
    }

    private void SetSelectedPlant(int? plantId, string? plantName, bool available, bool showMissingMessage = false)
    {
        _appState.SetPlantSelection(plantId, plantName, available);
        selectedPlantLabel.Text = _appState.PlantSelection.DisplayLabel;
        selectedPlantLabel.Visible = _appState.PlantSelection.IsAvailable || showMissingMessage;
    }
}
