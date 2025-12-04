using System.Drawing;
using System.Windows.Forms;

namespace SunSynkTrayWin;

partial class SettingsForm
{
    private System.ComponentModel.IContainer? components = null;

    protected override void Dispose(bool disposing)
    {
        if (disposing && components != null)
        {
            components.Dispose();
        }
        base.Dispose(disposing);
    }

    #region Windows Form Designer generated code

    private void InitializeComponent()
    {
            this.settingsLayout = new System.Windows.Forms.TableLayoutPanel();
            this.cloudGroupBox = new System.Windows.Forms.GroupBox();
            this.cloudLayout = new System.Windows.Forms.TableLayoutPanel();
            this.usernameLabel = new System.Windows.Forms.Label();
            this.usernameTextBox = new System.Windows.Forms.TextBox();
            this.passwordLabel = new System.Windows.Forms.Label();
            this.passwordTextBox = new System.Windows.Forms.TextBox();
            this.credentialsHintLabel = new System.Windows.Forms.Label();
            this.testConnectionButton = new System.Windows.Forms.Button();
            this.dataGroupBox = new System.Windows.Forms.GroupBox();
            this.dataLayout = new System.Windows.Forms.TableLayoutPanel();
            this.pollIntervalLabel = new System.Windows.Forms.Label();
            this.pollIntervalNumeric = new System.Windows.Forms.NumericUpDown();
            this.plantLabel = new System.Windows.Forms.Label();
            this.plantListBox = new System.Windows.Forms.ListBox();
            this.selectedPlantLabel = new System.Windows.Forms.Label();
            this.appGroupBox = new System.Windows.Forms.GroupBox();
            this.appLayout = new System.Windows.Forms.TableLayoutPanel();
            this.startOnBootCheckBox = new System.Windows.Forms.CheckBox();
            this.themeToggleCheckBox = new System.Windows.Forms.CheckBox();
            this.logGroupBox = new System.Windows.Forms.GroupBox();
            this.logLayout = new System.Windows.Forms.TableLayoutPanel();
            this.authLogLabel = new System.Windows.Forms.Label();
            this.authLogTextBox = new System.Windows.Forms.TextBox();
            this.settingsButtonRow = new System.Windows.Forms.TableLayoutPanel();
            this.settingsButtonPanel = new System.Windows.Forms.FlowLayoutPanel();
            this.saveSettingsButton = new System.Windows.Forms.Button();
            this.cancelSettingsButton = new System.Windows.Forms.Button();
            this.resetSettingsButton = new System.Windows.Forms.Button();
            this.settingsLayout.SuspendLayout();
            this.cloudGroupBox.SuspendLayout();
            this.cloudLayout.SuspendLayout();
            this.dataGroupBox.SuspendLayout();
            this.dataLayout.SuspendLayout();
            ((System.ComponentModel.ISupportInitialize)(this.pollIntervalNumeric)).BeginInit();
            this.appGroupBox.SuspendLayout();
            this.appLayout.SuspendLayout();
            this.logGroupBox.SuspendLayout();
            this.logLayout.SuspendLayout();
            this.settingsButtonRow.SuspendLayout();
            this.settingsButtonPanel.SuspendLayout();
            this.SuspendLayout();
            // 
            // settingsLayout
            // 
            this.settingsLayout.ColumnCount = 1;
            this.settingsLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.settingsLayout.Controls.Add(this.cloudGroupBox, 0, 0);
            this.settingsLayout.Controls.Add(this.dataGroupBox, 0, 1);
            this.settingsLayout.Controls.Add(this.appGroupBox, 0, 2);
            this.settingsLayout.Controls.Add(this.logGroupBox, 0, 3);
            this.settingsLayout.Controls.Add(this.settingsButtonRow, 0, 4);
            this.settingsLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsLayout.Location = new System.Drawing.Point(8, 8);
            this.settingsLayout.Name = "settingsLayout";
            this.settingsLayout.RowCount = 5;
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.Size = new System.Drawing.Size(593, 658);
            this.settingsLayout.TabIndex = 0;
            // 
            // cloudGroupBox
            // 
            this.cloudGroupBox.AutoSize = true;
            this.cloudGroupBox.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.cloudGroupBox.Controls.Add(this.cloudLayout);
            this.cloudGroupBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.cloudGroupBox.Location = new System.Drawing.Point(3, 3);
            this.cloudGroupBox.Margin = new System.Windows.Forms.Padding(3, 3, 3, 8);
            this.cloudGroupBox.Name = "cloudGroupBox";
            this.cloudGroupBox.Padding = new System.Windows.Forms.Padding(8, 8, 8, 8);
            this.cloudGroupBox.Size = new System.Drawing.Size(587, 136);
            this.cloudGroupBox.TabIndex = 1;
            this.cloudGroupBox.TabStop = false;
            this.cloudGroupBox.Text = "SunSynk cloud access";
            // 
            // cloudLayout
            // 
            this.cloudLayout.AutoSize = true;
            this.cloudLayout.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.cloudLayout.ColumnCount = 2;
            this.cloudLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 16.63748F));
            this.cloudLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 83.36252F));
            this.cloudLayout.Controls.Add(this.usernameLabel, 0, 0);
            this.cloudLayout.Controls.Add(this.usernameTextBox, 1, 0);
            this.cloudLayout.Controls.Add(this.passwordLabel, 0, 1);
            this.cloudLayout.Controls.Add(this.passwordTextBox, 1, 1);
            this.cloudLayout.Controls.Add(this.credentialsHintLabel, 1, 2);
            this.cloudLayout.Controls.Add(this.testConnectionButton, 1, 3);
            this.cloudLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.cloudLayout.Location = new System.Drawing.Point(8, 21);
            this.cloudLayout.Margin = new System.Windows.Forms.Padding(0);
            this.cloudLayout.Name = "cloudLayout";
            this.cloudLayout.RowCount = 4;
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.Size = new System.Drawing.Size(571, 107);
            this.cloudLayout.TabIndex = 0;
            // 
            // usernameLabel
            // 
            this.usernameLabel.AutoSize = true;
            this.usernameLabel.Location = new System.Drawing.Point(3, 0);
            this.usernameLabel.Margin = new System.Windows.Forms.Padding(3, 0, 6, 4);
            this.usernameLabel.Name = "usernameLabel";
            this.usernameLabel.Size = new System.Drawing.Size(55, 13);
            this.usernameLabel.TabIndex = 0;
            this.usernameLabel.Text = "Username";
            // 
            // usernameTextBox
            // 
            this.usernameTextBox.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.usernameTextBox.Location = new System.Drawing.Point(98, 3);
            this.usernameTextBox.Margin = new System.Windows.Forms.Padding(3, 3, 3, 8);
            this.usernameTextBox.Name = "usernameTextBox";
            this.usernameTextBox.Size = new System.Drawing.Size(470, 20);
            this.usernameTextBox.TabIndex = 1;
            // 
            // passwordLabel
            // 
            this.passwordLabel.AutoSize = true;
            this.passwordLabel.Location = new System.Drawing.Point(3, 31);
            this.passwordLabel.Margin = new System.Windows.Forms.Padding(3, 0, 6, 4);
            this.passwordLabel.Name = "passwordLabel";
            this.passwordLabel.Size = new System.Drawing.Size(53, 13);
            this.passwordLabel.TabIndex = 2;
            this.passwordLabel.Text = "Password";
            // 
            // passwordTextBox
            // 
            this.passwordTextBox.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.passwordTextBox.Location = new System.Drawing.Point(98, 34);
            this.passwordTextBox.Margin = new System.Windows.Forms.Padding(3, 3, 3, 8);
            this.passwordTextBox.Name = "passwordTextBox";
            this.passwordTextBox.Size = new System.Drawing.Size(470, 20);
            this.passwordTextBox.TabIndex = 3;
            this.passwordTextBox.UseSystemPasswordChar = true;
            // 
            // credentialsHintLabel
            // 
            this.credentialsHintLabel.AutoSize = true;
            this.credentialsHintLabel.ForeColor = System.Drawing.SystemColors.GrayText;
            this.credentialsHintLabel.Location = new System.Drawing.Point(98, 62);
            this.credentialsHintLabel.Margin = new System.Windows.Forms.Padding(3, 0, 3, 6);
            this.credentialsHintLabel.Name = "credentialsHintLabel";
            this.credentialsHintLabel.Size = new System.Drawing.Size(169, 13);
            this.credentialsHintLabel.TabIndex = 4;
            this.credentialsHintLabel.Text = "Use your SunSynk portal account.";
            // 
            // testConnectionButton
            // 
            this.testConnectionButton.AutoSize = true;
            this.testConnectionButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.testConnectionButton.Location = new System.Drawing.Point(98, 81);
            this.testConnectionButton.Margin = new System.Windows.Forms.Padding(3, 0, 3, 3);
            this.testConnectionButton.Name = "testConnectionButton";
            this.testConnectionButton.Size = new System.Drawing.Size(65, 23);
            this.testConnectionButton.TabIndex = 5;
            this.testConnectionButton.Text = "Get plants";
            this.testConnectionButton.UseVisualStyleBackColor = true;
            // 
            // dataGroupBox
            // 
            this.dataGroupBox.AutoSize = true;
            this.dataGroupBox.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.dataGroupBox.Controls.Add(this.dataLayout);
            this.dataGroupBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.dataGroupBox.Location = new System.Drawing.Point(3, 150);
            this.dataGroupBox.Margin = new System.Windows.Forms.Padding(3, 3, 3, 8);
            this.dataGroupBox.Name = "dataGroupBox";
            this.dataGroupBox.Padding = new System.Windows.Forms.Padding(8, 8, 8, 8);
            this.dataGroupBox.Size = new System.Drawing.Size(587, 218);
            this.dataGroupBox.TabIndex = 2;
            this.dataGroupBox.TabStop = false;
            this.dataGroupBox.Text = "Sync and sites";
            // 
            // dataLayout
            // 
            this.dataLayout.AutoSize = true;
            this.dataLayout.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.dataLayout.ColumnCount = 2;
            this.dataLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 16.63748F));
            this.dataLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 83.36252F));
            this.dataLayout.Controls.Add(this.pollIntervalLabel, 0, 0);
            this.dataLayout.Controls.Add(this.pollIntervalNumeric, 1, 0);
            this.dataLayout.Controls.Add(this.plantLabel, 0, 1);
            this.dataLayout.Controls.Add(this.plantListBox, 1, 1);
            this.dataLayout.Controls.Add(this.selectedPlantLabel, 1, 2);
            this.dataLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.dataLayout.Location = new System.Drawing.Point(8, 21);
            this.dataLayout.Margin = new System.Windows.Forms.Padding(0);
            this.dataLayout.Name = "dataLayout";
            this.dataLayout.RowCount = 3;
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Absolute, 150F));
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.dataLayout.Size = new System.Drawing.Size(571, 189);
            this.dataLayout.TabIndex = 0;
            // 
            // pollIntervalLabel
            // 
            this.pollIntervalLabel.AutoSize = true;
            this.pollIntervalLabel.Location = new System.Drawing.Point(3, 0);
            this.pollIntervalLabel.Margin = new System.Windows.Forms.Padding(3, 0, 6, 6);
            this.pollIntervalLabel.Name = "pollIntervalLabel";
            this.pollIntervalLabel.Size = new System.Drawing.Size(61, 13);
            this.pollIntervalLabel.TabIndex = 0;
            this.pollIntervalLabel.Text = "Poll interval";
            // 
            // pollIntervalNumeric
            // 
            this.pollIntervalNumeric.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Left | System.Windows.Forms.AnchorStyles.Right)));
            this.pollIntervalNumeric.Location = new System.Drawing.Point(98, 3);
            this.pollIntervalNumeric.Maximum = new decimal(new int[] {
            900,
            0,
            0,
            0});
            this.pollIntervalNumeric.Minimum = new decimal(new int[] {
            30,
            0,
            0,
            0});
            this.pollIntervalNumeric.Name = "pollIntervalNumeric";
            this.pollIntervalNumeric.Size = new System.Drawing.Size(470, 20);
            this.pollIntervalNumeric.TabIndex = 1;
            this.pollIntervalNumeric.Value = new decimal(new int[] {
            60,
            0,
            0,
            0});
            // 
            // plantLabel
            // 
            this.plantLabel.AutoSize = true;
            this.plantLabel.Location = new System.Drawing.Point(3, 32);
            this.plantLabel.Margin = new System.Windows.Forms.Padding(3, 6, 6, 6);
            this.plantLabel.Name = "plantLabel";
            this.plantLabel.Size = new System.Drawing.Size(74, 13);
            this.plantLabel.TabIndex = 2;
            this.plantLabel.Text = "Available sites";
            // 
            // plantListBox
            // 
            this.plantListBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.plantListBox.FormattingEnabled = true;
            this.plantListBox.IntegralHeight = false;
            this.plantListBox.Location = new System.Drawing.Point(98, 29);
            this.plantListBox.Margin = new System.Windows.Forms.Padding(3, 3, 3, 6);
            this.plantListBox.Name = "plantListBox";
            this.plantListBox.Size = new System.Drawing.Size(470, 141);
            this.plantListBox.TabIndex = 3;
            // 
            // selectedPlantLabel
            // 
            this.selectedPlantLabel.AutoSize = true;
            this.selectedPlantLabel.Location = new System.Drawing.Point(98, 176);
            this.selectedPlantLabel.Name = "selectedPlantLabel";
            this.selectedPlantLabel.Size = new System.Drawing.Size(111, 13);
            this.selectedPlantLabel.TabIndex = 4;
            this.selectedPlantLabel.Text = "Selected plant: (none)";
            // 
            // appGroupBox
            // 
            this.appGroupBox.AutoSize = true;
            this.appGroupBox.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.appGroupBox.Controls.Add(this.appLayout);
            this.appGroupBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.appGroupBox.Location = new System.Drawing.Point(3, 376);
            this.appGroupBox.Margin = new System.Windows.Forms.Padding(3, 0, 3, 8);
            this.appGroupBox.Name = "appGroupBox";
            this.appGroupBox.Padding = new System.Windows.Forms.Padding(8, 8, 8, 8);
            this.appGroupBox.Size = new System.Drawing.Size(587, 75);
            this.appGroupBox.TabIndex = 3;
            this.appGroupBox.TabStop = false;
            this.appGroupBox.Text = "App and OS";
            // 
            // appLayout
            // 
            this.appLayout.AutoSize = true;
            this.appLayout.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.appLayout.ColumnCount = 1;
            this.appLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.appLayout.Controls.Add(this.startOnBootCheckBox, 0, 0);
            this.appLayout.Controls.Add(this.themeToggleCheckBox, 0, 1);
            this.appLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.appLayout.Location = new System.Drawing.Point(8, 21);
            this.appLayout.Margin = new System.Windows.Forms.Padding(0);
            this.appLayout.Name = "appLayout";
            this.appLayout.RowCount = 2;
            this.appLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.appLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.appLayout.Size = new System.Drawing.Size(571, 46);
            this.appLayout.TabIndex = 0;
            // 
            // startOnBootCheckBox
            // 
            this.startOnBootCheckBox.AutoSize = true;
            this.startOnBootCheckBox.Location = new System.Drawing.Point(3, 3);
            this.startOnBootCheckBox.Name = "startOnBootCheckBox";
            this.startOnBootCheckBox.Size = new System.Drawing.Size(162, 17);
            this.startOnBootCheckBox.TabIndex = 0;
            this.startOnBootCheckBox.Text = "Start when Windows signs in";
            this.startOnBootCheckBox.UseVisualStyleBackColor = true;
            // 
            // themeToggleCheckBox
            // 
            this.themeToggleCheckBox.AutoSize = true;
            this.themeToggleCheckBox.Location = new System.Drawing.Point(3, 26);
            this.themeToggleCheckBox.Name = "themeToggleCheckBox";
            this.themeToggleCheckBox.Size = new System.Drawing.Size(98, 17);
            this.themeToggleCheckBox.TabIndex = 1;
            this.themeToggleCheckBox.Text = "Use dark mode";
            this.themeToggleCheckBox.UseVisualStyleBackColor = true;
            // 
            // logGroupBox
            // 
            this.logGroupBox.AutoSize = true;
            this.logGroupBox.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.logGroupBox.Controls.Add(this.logLayout);
            this.logGroupBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.logGroupBox.Location = new System.Drawing.Point(3, 459);
            this.logGroupBox.Margin = new System.Windows.Forms.Padding(3, 0, 3, 8);
            this.logGroupBox.Name = "logGroupBox";
            this.logGroupBox.Padding = new System.Windows.Forms.Padding(8, 8, 8, 8);
            this.logGroupBox.Size = new System.Drawing.Size(587, 138);
            this.logGroupBox.TabIndex = 4;
            this.logGroupBox.TabStop = false;
            this.logGroupBox.Text = "Authentication activity";
            // 
            // logLayout
            // 
            this.logLayout.AutoSize = true;
            this.logLayout.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.logLayout.ColumnCount = 1;
            this.logLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.logLayout.Controls.Add(this.authLogLabel, 0, 0);
            this.logLayout.Controls.Add(this.authLogTextBox, 0, 1);
            this.logLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.logLayout.Location = new System.Drawing.Point(8, 21);
            this.logLayout.Margin = new System.Windows.Forms.Padding(0);
            this.logLayout.Name = "logLayout";
            this.logLayout.RowCount = 2;
            this.logLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.logLayout.RowStyles.Add(new System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Absolute, 90F));
            this.logLayout.Size = new System.Drawing.Size(571, 109);
            this.logLayout.TabIndex = 0;
            // 
            // authLogLabel
            // 
            this.authLogLabel.AutoSize = true;
            this.authLogLabel.Location = new System.Drawing.Point(3, 0);
            this.authLogLabel.Margin = new System.Windows.Forms.Padding(3, 0, 3, 6);
            this.authLogLabel.Name = "authLogLabel";
            this.authLogLabel.Size = new System.Drawing.Size(65, 13);
            this.authLogLabel.TabIndex = 0;
            this.authLogLabel.Text = "Auth activity";
            // 
            // authLogTextBox
            // 
            this.authLogTextBox.BackColor = System.Drawing.SystemColors.Window;
            this.authLogTextBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.authLogTextBox.Location = new System.Drawing.Point(3, 22);
            this.authLogTextBox.Multiline = true;
            this.authLogTextBox.Name = "authLogTextBox";
            this.authLogTextBox.ReadOnly = true;
            this.authLogTextBox.ScrollBars = System.Windows.Forms.ScrollBars.Vertical;
            this.authLogTextBox.Size = new System.Drawing.Size(565, 84);
            this.authLogTextBox.TabIndex = 1;
            // 
            // settingsButtonRow
            // 
            this.settingsButtonRow.AutoSize = true;
            this.settingsButtonRow.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.settingsButtonRow.ColumnCount = 1;
            this.settingsButtonRow.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.settingsButtonRow.Controls.Add(this.settingsButtonPanel, 0, 0);
            this.settingsButtonRow.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsButtonRow.Location = new System.Drawing.Point(3, 608);
            this.settingsButtonRow.Name = "settingsButtonRow";
            this.settingsButtonRow.RowCount = 1;
            this.settingsButtonRow.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsButtonRow.Size = new System.Drawing.Size(587, 47);
            this.settingsButtonRow.TabIndex = 5;
            // 
            // settingsButtonPanel
            // 
            this.settingsButtonPanel.AutoSize = true;
            this.settingsButtonPanel.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.settingsButtonPanel.Controls.Add(this.saveSettingsButton);
            this.settingsButtonPanel.Controls.Add(this.cancelSettingsButton);
            this.settingsButtonPanel.Controls.Add(this.resetSettingsButton);
            this.settingsButtonPanel.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsButtonPanel.FlowDirection = System.Windows.Forms.FlowDirection.RightToLeft;
            this.settingsButtonPanel.Location = new System.Drawing.Point(0, 0);
            this.settingsButtonPanel.Margin = new System.Windows.Forms.Padding(0);
            this.settingsButtonPanel.Name = "settingsButtonPanel";
            this.settingsButtonPanel.Padding = new System.Windows.Forms.Padding(0, 6, 0, 6);
            this.settingsButtonPanel.Size = new System.Drawing.Size(587, 47);
            this.settingsButtonPanel.TabIndex = 0;
            this.settingsButtonPanel.WrapContents = false;
            // 
            // saveSettingsButton
            // 
            this.saveSettingsButton.AutoSize = true;
            this.saveSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.saveSettingsButton.BackColor = System.Drawing.SystemColors.Highlight;
            this.saveSettingsButton.FlatAppearance.BorderSize = 0;
            this.saveSettingsButton.FlatStyle = System.Windows.Forms.FlatStyle.Flat;
            this.saveSettingsButton.ForeColor = System.Drawing.Color.White;
            this.saveSettingsButton.Location = new System.Drawing.Point(549, 12);
            this.saveSettingsButton.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.saveSettingsButton.Name = "saveSettingsButton";
            this.saveSettingsButton.Size = new System.Drawing.Size(32, 23);
            this.saveSettingsButton.TabIndex = 0;
            this.saveSettingsButton.Text = "OK";
            this.saveSettingsButton.UseVisualStyleBackColor = false;
            // 
            // cancelSettingsButton
            // 
            this.cancelSettingsButton.AutoSize = true;
            this.cancelSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.cancelSettingsButton.Location = new System.Drawing.Point(487, 12);
            this.cancelSettingsButton.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.cancelSettingsButton.Name = "cancelSettingsButton";
            this.cancelSettingsButton.Size = new System.Drawing.Size(50, 23);
            this.cancelSettingsButton.TabIndex = 1;
            this.cancelSettingsButton.Text = "Cancel";
            this.cancelSettingsButton.UseVisualStyleBackColor = true;
            // 
            // resetSettingsButton
            // 
            this.resetSettingsButton.AutoSize = true;
            this.resetSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.resetSettingsButton.Location = new System.Drawing.Point(430, 12);
            this.resetSettingsButton.Margin = new System.Windows.Forms.Padding(6, 6, 6, 6);
            this.resetSettingsButton.Name = "resetSettingsButton";
            this.resetSettingsButton.Size = new System.Drawing.Size(45, 23);
            this.resetSettingsButton.TabIndex = 2;
            this.resetSettingsButton.Text = "Reset";
            this.resetSettingsButton.UseVisualStyleBackColor = true;
            // 
            // SettingsForm
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(96F, 96F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Dpi;
            this.ClientSize = new System.Drawing.Size(609, 674);
            this.Controls.Add(this.settingsLayout);
            this.MinimumSize = new System.Drawing.Size(625, 713);
            this.Name = "SettingsForm";
            this.Padding = new System.Windows.Forms.Padding(8, 8, 8, 8);
            this.Text = "Settings";
            this.settingsLayout.ResumeLayout(false);
            this.settingsLayout.PerformLayout();
            this.cloudGroupBox.ResumeLayout(false);
            this.cloudGroupBox.PerformLayout();
            this.cloudLayout.ResumeLayout(false);
            this.cloudLayout.PerformLayout();
            this.dataGroupBox.ResumeLayout(false);
            this.dataGroupBox.PerformLayout();
            this.dataLayout.ResumeLayout(false);
            this.dataLayout.PerformLayout();
            ((System.ComponentModel.ISupportInitialize)(this.pollIntervalNumeric)).EndInit();
            this.appGroupBox.ResumeLayout(false);
            this.appGroupBox.PerformLayout();
            this.appLayout.ResumeLayout(false);
            this.appLayout.PerformLayout();
            this.logGroupBox.ResumeLayout(false);
            this.logGroupBox.PerformLayout();
            this.logLayout.ResumeLayout(false);
            this.logLayout.PerformLayout();
            this.settingsButtonRow.ResumeLayout(false);
            this.settingsButtonRow.PerformLayout();
            this.settingsButtonPanel.ResumeLayout(false);
            this.settingsButtonPanel.PerformLayout();
            this.ResumeLayout(false);

    }

    #endregion

    internal TableLayoutPanel settingsLayout;
    internal GroupBox cloudGroupBox;
    internal TableLayoutPanel cloudLayout;
    internal Label usernameLabel;
    internal TextBox usernameTextBox;
    internal Label passwordLabel;
    internal TextBox passwordTextBox;
    internal Label credentialsHintLabel;
    internal Button testConnectionButton;
    internal GroupBox dataGroupBox;
    internal TableLayoutPanel dataLayout;
    internal NumericUpDown pollIntervalNumeric;
    internal Label plantLabel;
    internal ListBox plantListBox;
    internal Label selectedPlantLabel;
    internal Label pollIntervalLabel;
    internal GroupBox appGroupBox;
    internal TableLayoutPanel appLayout;
    internal CheckBox startOnBootCheckBox;
    internal CheckBox themeToggleCheckBox;
    internal GroupBox logGroupBox;
    internal TableLayoutPanel logLayout;
    internal Label authLogLabel;
    internal TextBox authLogTextBox;
    internal TableLayoutPanel settingsButtonRow;
    internal FlowLayoutPanel settingsButtonPanel;
    internal Button saveSettingsButton;
    internal Button cancelSettingsButton;
    internal Button resetSettingsButton;
}
