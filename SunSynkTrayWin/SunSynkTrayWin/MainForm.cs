using System.Drawing;
using System.IO;
using System.Windows.Forms;
using Microsoft.Win32;
using SunSynkTrayWin.Api;
using SunSynkTrayWin.Storage;
using SunSynkTrayWin.Controls;
using SunSynkTrayWin.Theme;
using SunSynkTrayWin.ViewModels;

namespace SunSynkTrayWin;

public partial class MainForm : Form
{
    private bool _allowClose;
    private readonly SunSynkClient _apiClient;
    private readonly SecureSettingsStore _settingsStore;
    private bool _isLoadingSettings;
    private bool _isPopulatingPlants;
    private DayEnergyData? _lastDayEnergy;
    private PowerFlowData? _lastFlow;
    private readonly StatusViewModel _statusViewModel = new();
    private readonly AppState _appState;
    private Form? _settingsForm;
    private readonly ThemeManager _themeManager;
    private readonly Icon? _darkTrayIcon;
    private readonly Icon? _lightTrayIcon;
    private AppTheme _currentTheme = AppTheme.Dark;
    private readonly MonitoringController _monitoring;
    private Icon? _dynamicTrayIcon;

    public MainForm()
    {
        InitializeComponent();
        statusSummaryControl.Bind(_statusViewModel);
        _apiClient = SunSynkClient.Instance;
        _settingsStore = new SecureSettingsStore();
        _appState = new AppState(_statusViewModel, _apiClient);
        BindSettingsToState();
        Load += MainForm_Load;
        FormClosing += MainForm_FormClosing;
        _themeManager = new ThemeManager();
        var iconPath = Path.Combine(AppContext.BaseDirectory, "sunsynk.ico");
        if (File.Exists(iconPath))
        {
            using var tmp = new Icon(iconPath);
            _lightTrayIcon = new Icon(tmp, tmp.Size);
            _darkTrayIcon = _lightTrayIcon;
            trayIcon.Icon = _lightTrayIcon;
        }
        else
        {
            _lightTrayIcon = trayIcon.Icon;
            _darkTrayIcon = trayIcon.Icon;
        }

        ApplyTheme(_currentTheme);
        HandleCreated += (_, _) => ThemeManager.ApplyWindowTheme(Handle, _currentTheme);
        _monitoring = new MonitoringController(
            _apiClient,
            _appState,
            (title, message) => MessageBox.Show(this, message, title, MessageBoxButtons.OK, MessageBoxIcon.Warning),
            (flow, dayEnergy) =>
            {
                _lastFlow = flow;
                _lastDayEnergy = dayEnergy;
                flowView.SetFlow(flow);
                UpdateDayChart();
                UpdateDynamicTrayIcon(flow);
            });
        FormClosed += (_, _) =>
        {
            _monitoring.Dispose();
            _dynamicTrayIcon?.Dispose();
        };
    }

}
