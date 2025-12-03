using System.ComponentModel;
using System.Drawing;
using System.Windows.Forms;
using SunSynkTrayWin.Theme;
using SunSynkTrayWin.ViewModels;

namespace SunSynkTrayWin.Controls;

public class StatusSummaryControl : BindableControl<StatusViewModel>
{
    private readonly TableLayoutPanel _layout;
    private readonly TableLayoutPanel _statusLayout;
    private readonly FlowLayoutPanel _readinessPanel;
    private readonly Label _connectionLabel;
    private readonly Label _connectionValue;
    private readonly Label _lastUpdatedLabel;
    private readonly Label _lastUpdatedValue;
    private readonly Label _nextUpdateLabel;
    private readonly Label _nextUpdateValue;
    private readonly Label _pollingStateLabel;
    private readonly Label _pollingStateValue;
    private readonly Panel _readinessDot;
    private readonly Label _readinessStatusLabel;

    public StatusSummaryControl()
    {
        AutoSize = true;
        Margin = new Padding(0);

        _layout = new TableLayoutPanel
        {
            AutoSize = true,
            ColumnCount = 2,
            Dock = DockStyle.Fill
        };
        _layout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        _layout.ColumnStyles.Add(new ColumnStyle());
        Controls.Add(_layout);

        _statusLayout = new TableLayoutPanel
        {
            AutoSize = true,
            ColumnCount = 2,
            Dock = DockStyle.Top,
            Margin = new Padding(6)
        };
        _statusLayout.ColumnStyles.Add(new ColumnStyle());
        _statusLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        _layout.Controls.Add(_statusLayout, 0, 0);

        _connectionLabel = CreateTitleLabel("Connection:");
        _connectionValue = CreateValueLabel("Not connected");
        _lastUpdatedLabel = CreateTitleLabel("Last update:");
        _lastUpdatedValue = CreateValueLabel("Waiting for first refresh");
        _nextUpdateLabel = CreateTitleLabel("Next update:");
        _nextUpdateValue = CreateValueLabel("--");
        _pollingStateLabel = CreateTitleLabel("Polling state:");
        _pollingStateValue = CreateValueLabel("Running");

        _statusLayout.Controls.Add(_connectionLabel, 0, 0);
        _statusLayout.Controls.Add(_connectionValue, 1, 0);
        _statusLayout.Controls.Add(_lastUpdatedLabel, 0, 1);
        _statusLayout.Controls.Add(_lastUpdatedValue, 1, 1);
        _statusLayout.Controls.Add(_nextUpdateLabel, 0, 2);
        _statusLayout.Controls.Add(_nextUpdateValue, 1, 2);
        _statusLayout.Controls.Add(_pollingStateLabel, 0, 3);
        _statusLayout.Controls.Add(_pollingStateValue, 1, 3);

        _readinessPanel = new FlowLayoutPanel
        {
            Anchor = AnchorStyles.Top | AnchorStyles.Right,
            AutoSize = true,
            FlowDirection = FlowDirection.RightToLeft,
            Margin = new Padding(6),
            WrapContents = false
        };
        _layout.Controls.Add(_readinessPanel, 1, 0);

        _readinessDot = new Panel
        {
            BackColor = Color.OrangeRed,
            BorderStyle = BorderStyle.FixedSingle,
            Margin = new Padding(6, 6, 11, 6),
            Size = new Size(24, 28)
        };
        _readinessStatusLabel = new Label
        {
            AutoSize = true,
            Margin = new Padding(6, 4, 6, 0),
            Text = "Missing credentials"
        };

        _readinessPanel.Controls.Add(_readinessDot);
        _readinessPanel.Controls.Add(_readinessStatusLabel);
    }

    public void ApplyPalette(ThemePalette palette)
    {
        BackColor = palette.Surface;
        _layout.BackColor = palette.Surface;
        _statusLayout.BackColor = palette.Surface;
        _readinessPanel.BackColor = Color.Transparent;

        var labels = new[]
        {
            _connectionLabel, _connectionValue, _lastUpdatedLabel, _lastUpdatedValue,
            _nextUpdateLabel, _nextUpdateValue, _pollingStateLabel, _pollingStateValue,
            _readinessStatusLabel
        };

        foreach (var label in labels)
        {
            label.ForeColor = palette.Text;
            label.BackColor = Color.Transparent;
        }
    }

    protected override void OnViewModelPropertyChanged(PropertyChangedEventArgs e)
    {
        if (ViewModel == null || IsDisposed)
        {
            return;
        }

        switch (e.PropertyName)
        {
            case nameof(StatusViewModel.ConnectionStatus):
                _connectionValue.Text = ViewModel.ConnectionStatus;
                break;
            case nameof(StatusViewModel.LastUpdated):
                _lastUpdatedValue.Text = ViewModel.LastUpdated;
                break;
            case nameof(StatusViewModel.NextUpdate):
                _nextUpdateValue.Text = ViewModel.NextUpdate;
                break;
            case nameof(StatusViewModel.PollingState):
                _pollingStateValue.Text = ViewModel.PollingState;
                break;
            case nameof(StatusViewModel.ReadinessText):
            case nameof(StatusViewModel.ReadinessColor):
                _readinessStatusLabel.Text = ViewModel.ReadinessText;
                _readinessDot.BackColor = ViewModel.ReadinessColor;
                break;
            default:
                ApplyModelToUi();
                break;
        }
    }

    protected override void ApplyModelToUi()
    {
        if (ViewModel == null || IsDisposed)
        {
            return;
        }

        _connectionValue.Text = ViewModel.ConnectionStatus;
        _lastUpdatedValue.Text = ViewModel.LastUpdated;
        _nextUpdateValue.Text = ViewModel.NextUpdate;
        _pollingStateValue.Text = ViewModel.PollingState;
        _readinessStatusLabel.Text = ViewModel.ReadinessText;
        _readinessDot.BackColor = ViewModel.ReadinessColor;
    }

    private static Label CreateTitleLabel(string text) => new()
    {
        AutoSize = true,
        Font = new Font("Segoe UI", 9F, FontStyle.Bold),
        Margin = new Padding(6, 0, 6, 0),
        Text = text
    };

    private static Label CreateValueLabel(string text) => new()
    {
        AutoSize = true,
        Margin = new Padding(6, 0, 6, 0),
        Text = text
    };
}
