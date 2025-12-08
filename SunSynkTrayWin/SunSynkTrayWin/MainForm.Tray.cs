using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
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
        var socColor = StatusColorHelper.GetSocColor(soc);
        using var referenceGraphics = Graphics.FromHwnd(IntPtr.Zero);
        var dpiScale = referenceGraphics.DpiX / 96f;
        var targetSize = (int)Math.Max(16, Math.Round(SystemInformation.SmallIconSize.Width * dpiScale));

        using var bmp = new Bitmap(targetSize, targetSize);
        bmp.SetResolution(referenceGraphics.DpiX, referenceGraphics.DpiY);

        using var g = Graphics.FromImage(bmp);
        g.Clear(Color.Transparent);
        g.SmoothingMode = SmoothingMode.None;
        g.PixelOffsetMode = PixelOffsetMode.HighQuality;
        g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;

        var fontSize = Math.Max(8f, (float)Math.Round(targetSize * 0.62f));
        var text = $"{soc:0}";
        using var font = new Font("Segoe UI Semibold", fontSize, FontStyle.Regular, GraphicsUnit.Pixel);
        var textRect = new Rectangle(0, 0, targetSize, targetSize);
        var flags = TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.NoPadding;

        // Light outline for contrast on mixed backgrounds.
        TextRenderer.DrawText(g, text, font,
            new Rectangle(textRect.X + 1, textRect.Y + 1, textRect.Width, textRect.Height),
            Color.FromArgb(180, Color.Black), Color.Transparent, flags);
        TextRenderer.DrawText(g, text, font, textRect, socColor, Color.Transparent, flags);

        var handle = bmp.GetHicon();
        return Icon.FromHandle(handle);
    }

    // Hover preview intentionally removed to keep tray interaction minimal.
}
