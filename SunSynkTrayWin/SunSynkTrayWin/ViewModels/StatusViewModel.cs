using System.Drawing;

namespace SunSynkTrayWin.ViewModels;

public sealed class StatusViewModel : ObservableObject
{
    private string _connectionStatus = "Not connected";
    private string _lastUpdated = "Waiting for first refresh";
    private string _nextUpdate = "--";
    private string _pollingState = "Not ready";
    private string _readinessText = "Missing credentials";
    private Color _readinessColor = Color.OrangeRed;

    public string ConnectionStatus
    {
        get => _connectionStatus;
        set => SetProperty(ref _connectionStatus, value);
    }

    public string LastUpdated
    {
        get => _lastUpdated;
        set => SetProperty(ref _lastUpdated, value);
    }

    public string NextUpdate
    {
        get => _nextUpdate;
        set => SetProperty(ref _nextUpdate, value);
    }

    public string PollingState
    {
        get => _pollingState;
        set => SetProperty(ref _pollingState, value);
    }

    public string ReadinessText
    {
        get => _readinessText;
        set => SetProperty(ref _readinessText, value);
    }

    public Color ReadinessColor
    {
        get => _readinessColor;
        set => SetProperty(ref _readinessColor, value);
    }
}
