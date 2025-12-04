using System.Drawing;
using System.Windows.Forms;
using SunSynkTrayWin.Theme;

namespace SunSynkTrayWin;

public partial class MainForm
{
    private static void StyleLabel(Label label, ThemePalette palette)
    {
        label.ForeColor = palette.Text;
        label.BackColor = Color.Transparent;
    }

    private static void StyleCheckbox(CheckBox box, ThemePalette palette)
    {
        box.ForeColor = palette.Text;
        box.BackColor = Color.Transparent;
    }

    private void ApplyTheme(AppTheme theme)
    {
        _currentTheme = theme;
        var palette = _themeManager.GetPalette(theme);
        ThemeManager.ApplyWindowTheme(Handle, theme);

        Padding = new Padding(0);
        BackColor = palette.Window;
        liveLayout.Margin = new Padding(0);
        liveLayout.BackColor = palette.Window;
        liveTopBarLayout.BackColor = palette.Surface;
        statusSummaryControl.BackColor = palette.Surface;
        statusSummaryControl.ApplyPalette(palette);
        liveActionsPanel.BackColor = palette.Surface;
        if (_settingsForm != null)
        {
            _settingsForm.BackColor = palette.Window;
            _settingsForm.settingsLayout.BackColor = palette.Window;
            // Readiness UI removed from settings form.
            ThemeManager.ApplyWindowTheme(_settingsForm.Handle, theme);
            _settingsForm.cloudGroupBox.ForeColor = palette.Text;
            _settingsForm.dataGroupBox.ForeColor = palette.Text;
            _settingsForm.appGroupBox.ForeColor = palette.Text;
            _settingsForm.logGroupBox.ForeColor = palette.Text;
        }

        if (_settingsForm != null)
        {
            Control[] primaryLabels =
            {
                _settingsForm.usernameLabel, _settingsForm.passwordLabel, _settingsForm.pollIntervalLabel,
                _settingsForm.plantLabel, _settingsForm.selectedPlantLabel, _settingsForm.authLogLabel, _settingsForm.credentialsHintLabel
            };
            foreach (Label label in primaryLabels)
            {
                StyleLabel(label, palette);
            }

            StyleCheckbox(_settingsForm.startOnBootCheckBox, palette);
            StyleCheckbox(_settingsForm.themeToggleCheckBox, palette);
            StyleLabel(_settingsForm.selectedPlantLabel, palette);
            StyleLabel(_settingsForm.credentialsHintLabel, palette);

            StyleButton(_settingsForm.testConnectionButton, palette, isPrimary: true);
            StyleButton(_settingsForm.saveSettingsButton, palette, isPrimary: true);
            StyleButton(_settingsForm.cancelSettingsButton, palette);
            StyleButton(_settingsForm.resetSettingsButton, palette);

            StyleInput(_settingsForm.usernameTextBox, palette);
            StyleInput(_settingsForm.passwordTextBox, palette);
            StyleInput(_settingsForm.authLogTextBox, palette);
            StyleInput(_settingsForm.pollIntervalNumeric, palette);
            _settingsForm.credentialsHintLabel.ForeColor = palette.SubtleText;

            _settingsForm.plantListBox.BackColor = palette.InputBackground;
            _settingsForm.plantListBox.ForeColor = palette.Text;
            _settingsForm.plantListBox.BorderStyle = BorderStyle.FixedSingle;
        }

        StyleButton(refreshNowButton, palette, isPrimary: true);
        StyleButton(pauseResumeButton, palette);

        dayChart.SetPalette(palette);
        flowView.SetPalette(palette);

        // Menus
        var renderer = _themeManager.CreateRenderer(palette);
        mainMenu.Renderer = renderer;
        trayMenu.Renderer = renderer;
        mainMenu.BackColor = palette.Surface;
        mainMenu.ForeColor = palette.Text;
        trayMenu.BackColor = palette.Surface;
        trayMenu.ForeColor = palette.Text;
        StyleMenuItems(mainMenu.Items, palette);
        StyleMenuItems(trayMenu.Items, palette);

        var icon = _dynamicTrayIcon
                   ?? (theme == AppTheme.Dark ? _darkTrayIcon ?? _lightTrayIcon : _lightTrayIcon);
        if (icon != null)
        {
            trayIcon.Icon = icon;
        }
    }

    private static void StyleButton(Button button, ThemePalette palette, bool isPrimary = false)
    {
        button.FlatStyle = FlatStyle.Flat;
        button.FlatAppearance.BorderColor = palette.Border;
        button.FlatAppearance.MouseOverBackColor = isPrimary ? palette.Accent : palette.Border;
        button.FlatAppearance.MouseDownBackColor = isPrimary ? palette.Accent : palette.Muted;
        button.BackColor = isPrimary ? palette.Accent : palette.Surface;
        button.ForeColor = isPrimary ? Color.White : palette.Text;
        button.FlatAppearance.BorderSize = 1;
    }

    private static void StyleInput(Control control, ThemePalette palette)
    {
        control.BackColor = palette.InputBackground;
        control.ForeColor = palette.Text;
        if (control is TextBox textBox)
        {
            textBox.BorderStyle = BorderStyle.FixedSingle;
        }
        if (control is NumericUpDown numeric)
        {
            numeric.BorderStyle = BorderStyle.FixedSingle;
            numeric.BackColor = palette.InputBackground;
            numeric.ForeColor = palette.Text;
        }
    }

    private static void StyleMenuItems(ToolStripItemCollection items, ThemePalette palette)
    {
        foreach (ToolStripItem item in items)
        {
            item.BackColor = palette.Surface;
            item.ForeColor = palette.Text;
            if (item is ToolStripMenuItem menuItem && menuItem.DropDownItems.Count > 0)
            {
                StyleMenuItems(menuItem.DropDownItems, palette);
            }
        }
    }

    private void ShowAboutDialog()
    {
        var palette = _themeManager.GetPalette(_currentTheme);

        using var dialog = new Form
        {
            Text = "About SunSynk Tray",
            StartPosition = FormStartPosition.CenterParent,
            ClientSize = new Size(620, 260),
            FormBorderStyle = FormBorderStyle.FixedDialog,
            MaximizeBox = false,
            MinimizeBox = false,
            BackColor = palette.Window
        };

        var title = new Label
        {
            Text = "SunSynk Tray",
            Font = new Font(Font.FontFamily, 14, FontStyle.Bold),
            AutoSize = true,
            ForeColor = palette.Text,
            BackColor = Color.Transparent,
            Location = new Point(20, 20)
        };
        var author = new Label
        {
            Text = "Author: leonjza",
            AutoSize = true,
            ForeColor = palette.Text,
            BackColor = Color.Transparent,
            Location = new Point(20, 70)
        };
        var repo = new LinkLabel
        {
            Text = "GitHub: https://github.com/leonjza/sunsynktray",
            AutoSize = true,
            ForeColor = palette.Text,
            LinkColor = palette.Accent,
            ActiveLinkColor = palette.Accent,
            BackColor = Color.Transparent,
            Location = new Point(20, 110)
        };
        repo.LinkClicked += (_, _) => System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
        {
            FileName = "https://github.com/leonjza/sunsynktray",
            UseShellExecute = true
        });
        var version = new Label
        {
            Text = $"Version: {Application.ProductVersion}",
            AutoSize = true,
            ForeColor = palette.Text,
            BackColor = Color.Transparent,
            Location = new Point(20, 150)
        };

        dialog.Controls.Add(title);
        dialog.Controls.Add(author);
        dialog.Controls.Add(repo);
        dialog.Controls.Add(version);
        ThemeManager.ApplyWindowTheme(dialog.Handle, _currentTheme);
        dialog.ShowDialog(this);
    }
}
