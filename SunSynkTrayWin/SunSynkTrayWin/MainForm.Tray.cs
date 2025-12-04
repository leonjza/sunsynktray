using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Windows.Forms;
using SunSynkTrayWin.Api;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private void TrayIcon_DoubleClick(object? sender, EventArgs e) => ShowMainWindow();

    private void OpenMenuItem_Click(object? sender, EventArgs e) => ShowMainWindow();

    private void RefreshMenuItem_Click(object? sender, EventArgs e) => TriggerManualRefresh("Tray menu");

    private void PauseResumeMenuItem_Click(object? sender, EventArgs e)
    {
        TogglePolling();
    }

    private void ExitMenuItem_Click(object? sender, EventArgs e)
    {
        _allowClose = true;
        trayIcon.Visible = false;
        Application.Exit();
    }

    private void MainForm_FormClosing(object? sender, FormClosingEventArgs e)
    {
        if (_allowClose)
        {
            return;
        }

        if (e.CloseReason == CloseReason.UserClosing)
        {
            e.Cancel = true;
            HideToTray();
        }
    }

    private void RefreshNowButton_Click(object? sender, EventArgs e) => TriggerManualRefresh("Refresh button");

    private void PauseResumeButton_Click(object? sender, EventArgs e)
    {
        TogglePolling();
    }

    private void SettingsMenuItem_Click(object? sender, EventArgs e)
    {
        EnsureSettingsWindow();
        LoadStoredCredentials();
        _settingsForm!.Show(this);
        _settingsForm.BringToFront();
    }

    private void CancelSettingsButton_Click(object? sender, EventArgs e)
    {
        LoadStoredCredentials();
        _settingsForm?.Hide();
    }

    private async void TestConnectionButton_Click(object? sender, EventArgs e)
    {
        await RunLoginAsync(forceLoadPlants: true);
    }

    private async void SaveSettingsButton_Click(object? sender, EventArgs e)
    {
        if (!_appState.PlantSelection.Id.HasValue || !_appState.PlantSelection.IsAvailable)
        {
            MessageBox.Show(this, "Please select a plant before saving settings.", "Plant required",
                MessageBoxButtons.OK, MessageBoxIcon.Warning);
            return;
        }

        if (string.IsNullOrWhiteSpace(_apiClient.AccessToken))
        {
            var token = await RunLoginAsync();
            if (token == null)
            {
                return;
            }
        }

        PersistSettings();
        _settingsForm?.Hide();
    }

    private void AboutMenuItem_Click(object? sender, EventArgs e)
    {
        ShowAboutDialog();
    }

    private void ShowMainWindow()
    {
        Show();
        WindowState = FormWindowState.Normal;
        ShowInTaskbar = true;
        Activate();
    }

    private void HideToTray()
    {
        Hide();
        ShowInTaskbar = false;
    }

    private void UpdateDynamicTrayIcon(PowerFlowData? flow)
    {
        if (flow?.Soc is not double soc)
        {
            return;
        }

        var charging = flow.BattPower < 0;
        _dynamicTrayIcon?.Dispose();
        _dynamicTrayIcon = RenderSocIcon(soc, charging);
        trayIcon.Icon = _dynamicTrayIcon ?? trayIcon.Icon;
        trayIcon.Text = $"SunSynk Tray: SOC {soc:0}% ({(charging ? "charging" : "discharging")})";
    }

    private Icon? RenderSocIcon(double soc, bool charging)
    {
        const int size = 32;
        var socColor = StatusColorHelper.GetSocColor(soc);
        using var bmp = new Bitmap(size, size);
        using var g = Graphics.FromImage(bmp);
        //g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
        g.Clear(Color.Transparent);

        var text = $"{soc:0}";
        using var font = new Font("Arial Narrow", 14, FontStyle.Regular);
        var textSize = g.MeasureString(text, font);
        var textPoint = new PointF(
            (size - textSize.Width) / 2f,
            (size - textSize.Height) / 2f);

        // Soft outline for contrast on both light/dark backgrounds.
        using var outlineBrush = new SolidBrush(Color.FromArgb(180, Color.Black));
        for (var dx = -1; dx <= 1; dx++)
        {
            for (var dy = -1; dy <= 1; dy++)
            {
                if (dx == 0 && dy == 0) continue;
                g.DrawString(text, font, outlineBrush, textPoint.X + dx, textPoint.Y + dy);
            }
        }

        using var textBrush = new SolidBrush(socColor);
        g.DrawString(text, font, textBrush, textPoint);

        var handle = bmp.GetHicon();
        return Icon.FromHandle(handle);
    }

    // Hover preview intentionally removed to keep tray interaction minimal.
}
