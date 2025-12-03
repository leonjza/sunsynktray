using System;
using System.Drawing;
using System.Threading.Tasks;
using System.Windows.Forms;
using SunSynkTrayWin.Controls;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private void TogglePolling()
    {
        _monitoring.TogglePause();
        var label = _appState.PauseResumeLabel;
        pauseResumeMenuItem.Text = label;
        pauseResumeButton.Text = label;
        UpdateTrayText();
    }

    private async void TriggerManualRefresh(string source)
    {
        await _monitoring.ManualRefreshAsync(source, showWarning: true);
    }

    private void UpdateTrayText()
    {
        // NotifyIcon text is limited to 63 characters.
        var pollingText = _monitoring.IsPaused ? "paused" : "running";
        trayIcon.Text = $"SunSynk Tray: polling {pollingText}";
    }

    private void UpdateDayChart()
    {
        if (_lastDayEnergy == null || _lastDayEnergy.Infos.Count == 0)
        {
            dayChart.SetSeries(Array.Empty<DayChartControl.ChartSeries>());
            return;
        }

        var seriesList = new List<DayChartControl.ChartSeries>();
        foreach (var info in _lastDayEnergy.Infos)
        {
            var points = new List<DayChartControl.ChartPoint>();
            foreach (var record in info.Records)
            {
                if (double.TryParse(record.Value, System.Globalization.NumberStyles.Any, System.Globalization.CultureInfo.InvariantCulture, out var val))
                {
                    var timeVal = record.Time ?? "00:00";
                    if (TimeSpan.TryParse(timeVal, out var ts))
                    {
                        points.Add(new DayChartControl.ChartPoint(ts.TotalMinutes, val, timeVal));
                    }
                }
            }

            seriesList.Add(new DayChartControl.ChartSeries(info.Label ?? "Series", points));
        }

        dayChart.SetSeries(seriesList);
    }
}
