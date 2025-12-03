using System;
using System.Threading.Tasks;
using System.Windows.Forms;
using SunSynkTrayWin.Api;
using SunSynkTrayWin.Controls;
using SunSynkTrayWin.ViewModels;

namespace SunSynkTrayWin;

/// <summary>
/// Handles polling lifecycle, readiness guarding, countdown updates, and data fetches.
/// Keeps MainForm focused on wiring UI instead of managing timers and state.
/// </summary>
public sealed class MonitoringController : IDisposable
{
    private readonly SunSynkClient _apiClient;
    private readonly AppState _state;
    private readonly System.Windows.Forms.Timer _pollTimer;
    private readonly Action<string, string> _showWarning;
    private readonly Action<PowerFlowData, DayEnergyData?> _onDataUpdated;

    private bool _isFetching;

    public MonitoringController(
        SunSynkClient apiClient,
        AppState state,
        Action<string, string> showWarning,
        Action<PowerFlowData, DayEnergyData?> onDataUpdated)
    {
        _apiClient = apiClient;
        _state = state;
        _showWarning = showWarning;
        _onDataUpdated = onDataUpdated;

        _pollTimer = new System.Windows.Forms.Timer { Interval = 1000 };
        _pollTimer.Tick += PollTimer_Tick;
    }

    public bool IsPaused => _state.IsPaused;

    public void TogglePause()
    {
        _state.TogglePaused();
        EnsurePollingTimer();
    }

    public async Task ManualRefreshAsync(string source, bool showWarning)
    {
        if (!EnsureReadyOrWarn(showWarning, source))
        {
            return;
        }

        await FetchFlowAsync($"{source} refresh");
        ResetCountdown();
    }

    public async Task StartMonitoringAsync(string source, bool resetCountdown = true)
    {
        if (!EnsureReadyOrWarn(showMessage: false, context: source))
        {
            return;
        }

        await FetchFlowAsync(source);
        if (resetCountdown)
        {
            ResetCountdown();
        }
        EnsurePollingTimer();
    }

    public void NotifyReadinessChanged()
    {
        _state.RefreshReadiness();
        EnsurePollingTimer();
    }

    public void ResetCountdown()
    {
        _state.ResetCountdown();
    }

    private void EnsurePollingTimer()
    {
        if (_state.IsReady && !_state.IsPaused)
        {
            if (!_pollTimer.Enabled)
            {
                ResetCountdown();
                _state.SetTimerActive(true);
                _pollTimer.Start();
            }
        }
        else
        {
            StopPolling();
        }
    }

    private void StopPolling()
    {
        _pollTimer.Stop();
        _state.SetTimerActive(false);
    }

    private async void PollTimer_Tick(object? sender, EventArgs e)
    {
        if (!_state.IsReady || _state.IsPaused)
        {
            StopPolling();
            return;
        }

        if (_state.CountdownSeconds > 0)
        {
            _state.DecrementCountdown();
            return;
        }

        await FetchFlowAsync("Scheduled");
        ResetCountdown();
    }

    private async Task FetchFlowAsync(string source)
    {
        if (_isFetching)
        {
            return;
        }

        if (!_state.IsReady || string.IsNullOrWhiteSpace(_apiClient.AccessToken) || !_state.PlantSelection.Id.HasValue)
        {
            _state.RefreshReadiness();
            return;
        }

        try
        {
            _isFetching = true;
            await StatusOperationRunner.RunWithStatusAsync(
                _state.Status,
                source,
                workingStatus: "Refreshing...",
                successStatus: "Online",
                async () =>
                {
                    var plantId = _state.PlantSelection.Id!.Value;
                    var flow = await _apiClient.GetPowerFlowAsync(plantId, DateTime.UtcNow);
                    var dayEnergy = await _apiClient.GetDayEnergyAsync(plantId, DateTime.UtcNow);

                    _onDataUpdated(flow, dayEnergy);
                    _state.RefreshReadiness();
                },
                onError: _ => _state.RefreshReadiness()).ConfigureAwait(false);
        }
        catch (Exception ex)
        {
            _state.Status.ConnectionStatus = "Error";
            _state.Status.LastUpdated = $"Error: {ex.Message}";
        }
        finally
        {
            _isFetching = false;
        }
    }

    private bool EnsureReadyOrWarn(bool showMessage, string context)
    {
        if (_state.IsReady)
        {
            return true;
        }

        _state.RefreshReadiness();
        if (showMessage)
        {
            _showWarning(context,
                "Not ready to monitor. Ensure credentials, authentication, and plant selection are set.");
        }

        return false;
    }

    public void Dispose()
    {
        _pollTimer.Dispose();
    }
}
