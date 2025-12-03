using System.ComponentModel;
using System.Drawing;
using SunSynkTrayWin.Api;
using SunSynkTrayWin.Storage;

namespace SunSynkTrayWin.ViewModels;

/// <summary>
/// Centralised application state that owns settings, plant selection, and monitoring flags.
/// It keeps StatusViewModel in sync so UI surfaces stay dumb wiring-only.
/// </summary>
public sealed class AppState
{
    private readonly SunSynkClient _apiClient;
    private bool _pollTimerActive;

    public AppState(StatusViewModel status, SunSynkClient apiClient)
    {
        Status = status;
        _apiClient = apiClient;
        Settings = new SettingsViewModel();
        PlantSelection = new PlantSelectionViewModel();
        Settings.PropertyChanged += SettingsOnPropertyChanged;
        PlantSelection.PropertyChanged += PlantSelectionOnPropertyChanged;
    }

    public SettingsViewModel Settings { get; }
    public PlantSelectionViewModel PlantSelection { get; }
    public StatusViewModel Status { get; }

    public bool IsAuthenticated => !string.IsNullOrWhiteSpace(_apiClient.AccessToken);
    public bool IsReady => Settings.HasCredentials && IsAuthenticated && PlantSelection.IsAvailable;
    public bool IsPaused { get; private set; }
    public int CountdownSeconds { get; private set; }

    public string PauseResumeLabel => IsPaused ? "Resume polling" : "Pause polling";

    public event EventHandler<PropertyChangedEventArgs>? StateChanged;
    public event EventHandler? ReadinessChanged;
    public event EventHandler? MonitoringChanged;

    public void RefreshReadiness()
    {
        var (text, color) = ComputeReadiness();
        Status.ReadinessText = text;
        Status.ReadinessColor = color;
        UpdateMonitoringDerived();
        ReadinessChanged?.Invoke(this, EventArgs.Empty);
    }

    public void UpdateAuthenticationState()
    {
        RefreshReadiness();
    }

    public void SetPlantSelection(int? id, string? name, bool available)
    {
        PlantSelection.Id = id;
        PlantSelection.Name = name;
        PlantSelection.IsAvailable = available && id.HasValue;
        RefreshReadiness();
    }

    public void ResetCountdown()
    {
        CountdownSeconds = Math.Max(1, Settings.PollIntervalSeconds);
        UpdateMonitoringDerived();
    }

    public void DecrementCountdown()
    {
        if (CountdownSeconds > 0)
        {
            CountdownSeconds--;
        }
        UpdateMonitoringDerived();
    }

    public void SetTimerActive(bool active)
    {
        _pollTimerActive = active;
        UpdateMonitoringDerived();
    }

    public void SetPaused(bool paused)
    {
        IsPaused = paused;
        UpdateMonitoringDerived();
    }

    public void TogglePaused() => SetPaused(!IsPaused);

    public SettingsData ToSettingsData() => new()
    {
        Username = Settings.Username,
        Password = Settings.Password,
        PlantId = PlantSelection.Id,
        PlantName = PlantSelection.Name,
        PollIntervalSeconds = Settings.PollIntervalSeconds,
        StartOnBoot = Settings.StartOnBoot,
        UseDarkMode = Settings.UseDarkMode
    };

    public void ApplySettings(SettingsData settings)
    {
        var normalized = settings.Normalize();
        Settings.Username = normalized.Username ?? string.Empty;
        Settings.Password = normalized.Password ?? string.Empty;
        Settings.PollIntervalSeconds = normalized.PollIntervalSeconds;
        Settings.StartOnBoot = normalized.StartOnBoot;
        Settings.UseDarkMode = normalized.UseDarkMode;
        PlantSelection.Id = normalized.PlantId;
        PlantSelection.Name = normalized.PlantName;
        PlantSelection.IsAvailable = normalized.PlantId.HasValue;
        RefreshReadiness();
        ResetCountdown();
    }

    private void SettingsOnPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        switch (e.PropertyName)
        {
            case nameof(SettingsViewModel.Username):
            case nameof(SettingsViewModel.Password):
                _apiClient.ClearCachedAuthentication();
                break;
            case nameof(SettingsViewModel.PollIntervalSeconds):
                ResetCountdown();
                break;
        }

        RefreshReadiness();
        StateChanged?.Invoke(this, e);
    }

    private void PlantSelectionOnPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        RefreshReadiness();
        StateChanged?.Invoke(this, e);
    }

    private (string Text, Color Color) ComputeReadiness()
    {
        if (!Settings.HasCredentials)
        {
            return ("Missing credentials", Color.OrangeRed);
        }

        if (!IsAuthenticated)
        {
            return ("Not authenticated", Color.DarkOrange);
        }

        if (!PlantSelection.IsAvailable)
        {
            return ("Select a plant", Color.Goldenrod);
        }

        var displayName = PlantSelection.Name ?? (PlantSelection.Id.HasValue ? $"Plant {PlantSelection.Id}" : "plant");
        var status = PlantSelection.Id.HasValue ? $"Monitoring plant: {displayName} (ID: {PlantSelection.Id})" : "Ready to monitor";
        return (status, Color.SeaGreen);
    }

    private void UpdateMonitoringDerived()
    {
        Status.PollingState = ComputePollingState();
        Status.NextUpdate = ComputeNextUpdate();
        MonitoringChanged?.Invoke(this, EventArgs.Empty);
    }

    private string ComputePollingState() => !IsReady
        ? "Not ready"
        : IsPaused ? "Paused" : "Running";

    private string ComputeNextUpdate() => _pollTimerActive && IsReady && !IsPaused
        ? $"{CountdownSeconds}s"
        : "--";
}
