using System;
using System.Windows.Forms;

namespace SunSynkTrayWin;

public partial class SettingsForm : Form
{
    public SettingsForm()
    {
        InitializeComponent();
        StartPosition = FormStartPosition.CenterParent;
    }

    public void WireEvents(
        EventHandler? onSave,
        EventHandler? onCancel,
        EventHandler? onReset,
        EventHandler? onTestConnection,
        FormClosingEventHandler? onClosing)
    {
        saveSettingsButton.Click += onSave;
        cancelSettingsButton.Click += onCancel;
        resetSettingsButton.Click += onReset;
        testConnectionButton.Click += onTestConnection;

        if (onClosing != null)
        {
            FormClosing += onClosing;
        }
    }

    private void pollIntervalNumeric_ValueChanged(object sender, EventArgs e)
    {

    }

    private void logGroupBox_Enter(object sender, EventArgs e)
    {

    }

    private void settingsLayout_Paint(object sender, PaintEventArgs e)
    {

    }

    private void cloudLayout_Paint(object sender, PaintEventArgs e)
    {

    }

    private void usernameLabel_Click(object sender, EventArgs e)
    {

    }

    private void testConnectionButton_Click(object sender, EventArgs e)
    {

    }

    private void SettingsForm_Load(object sender, EventArgs e)
    {

    }
}
