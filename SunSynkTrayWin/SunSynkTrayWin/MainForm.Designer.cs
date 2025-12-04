namespace SunSynkTrayWin;

partial class MainForm
{
    /// <summary>
    ///  Required designer variable.
    /// </summary>
    private System.ComponentModel.IContainer components = null;

    /// <summary>
    ///  Clean up any resources being used.
    /// </summary>
    /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
    protected override void Dispose(bool disposing)
    {
        if (disposing && (components != null))
        {
            components.Dispose();
        }
        base.Dispose(disposing);
    }

    #region Windows Form Designer generated code

    /// <summary>
    ///  Required method for Designer support - do not modify
    ///  the contents of this method with the code editor.
    /// </summary>
    private void InitializeComponent()
    {
            this.components = new System.ComponentModel.Container();
            System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(MainForm));
            this.trayIcon = new System.Windows.Forms.NotifyIcon(this.components);
            this.trayMenu = new System.Windows.Forms.ContextMenuStrip(this.components);
            this.openMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.refreshMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.pauseResumeMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.traySeparator = new System.Windows.Forms.ToolStripSeparator();
            this.exitMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.mainMenu = new System.Windows.Forms.MenuStrip();
            this.fileMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.settingsMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.aboutMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.quitMenuItem = new System.Windows.Forms.ToolStripMenuItem();
            this.liveLayout = new System.Windows.Forms.TableLayoutPanel();
            this.liveTopBarLayout = new System.Windows.Forms.TableLayoutPanel();
            this.statusSummaryControl = new SunSynkTrayWin.Controls.StatusSummaryControl();
            this.flowView = new SunSynkTrayWin.Controls.PowerFlowViewControl();
            this.dayChart = new SunSynkTrayWin.Controls.DayChartControl();
            this.liveActionsPanel = new System.Windows.Forms.FlowLayoutPanel();
            this.refreshNowButton = new System.Windows.Forms.Button();
            this.pauseResumeButton = new System.Windows.Forms.Button();
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
            this.trayMenu.SuspendLayout();
            this.mainMenu.SuspendLayout();
            this.liveLayout.SuspendLayout();
            this.liveTopBarLayout.SuspendLayout();
            this.liveActionsPanel.SuspendLayout();
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
            // trayIcon
            // 
            this.trayIcon.ContextMenuStrip = this.trayMenu;
            this.trayIcon.Icon = ((System.Drawing.Icon)(resources.GetObject("trayIcon.Icon")));
            this.trayIcon.Text = "SunSynk Tray";
            this.trayIcon.Visible = true;
            this.trayIcon.DoubleClick += new System.EventHandler(this.TrayIcon_DoubleClick);
            this.trayIcon.MouseDoubleClick += new System.Windows.Forms.MouseEventHandler(this.trayIcon_MouseDoubleClick);
            // 
            // trayMenu
            // 
            this.trayMenu.ImageScalingSize = new System.Drawing.Size(32, 32);
            this.trayMenu.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.openMenuItem,
            this.refreshMenuItem,
            this.pauseResumeMenuItem,
            this.traySeparator,
            this.exitMenuItem});
            this.trayMenu.Name = "trayMenu";
            this.trayMenu.Size = new System.Drawing.Size(231, 162);
            this.trayMenu.Opening += new System.ComponentModel.CancelEventHandler(this.trayMenu_Opening);
            // 
            // openMenuItem
            // 
            this.openMenuItem.Name = "openMenuItem";
            this.openMenuItem.Size = new System.Drawing.Size(230, 38);
            this.openMenuItem.Text = "Open";
            this.openMenuItem.Click += new System.EventHandler(this.OpenMenuItem_Click);
            // 
            // refreshMenuItem
            // 
            this.refreshMenuItem.Name = "refreshMenuItem";
            this.refreshMenuItem.Size = new System.Drawing.Size(230, 38);
            this.refreshMenuItem.Text = "Refresh now";
            this.refreshMenuItem.Click += new System.EventHandler(this.RefreshMenuItem_Click);
            // 
            // pauseResumeMenuItem
            // 
            this.pauseResumeMenuItem.Name = "pauseResumeMenuItem";
            this.pauseResumeMenuItem.Size = new System.Drawing.Size(230, 38);
            this.pauseResumeMenuItem.Text = "Pause polling";
            this.pauseResumeMenuItem.Click += new System.EventHandler(this.PauseResumeMenuItem_Click);
            // 
            // traySeparator
            // 
            this.traySeparator.Name = "traySeparator";
            this.traySeparator.Size = new System.Drawing.Size(227, 6);
            // 
            // exitMenuItem
            // 
            this.exitMenuItem.Name = "exitMenuItem";
            this.exitMenuItem.Size = new System.Drawing.Size(230, 38);
            this.exitMenuItem.Text = "Quit";
            this.exitMenuItem.Click += new System.EventHandler(this.ExitMenuItem_Click);
            // 
            // mainMenu
            // 
            this.mainMenu.GripMargin = new System.Windows.Forms.Padding(2, 2, 0, 2);
            this.mainMenu.ImageScalingSize = new System.Drawing.Size(24, 24);
            this.mainMenu.Items.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.fileMenuItem});
            this.mainMenu.Location = new System.Drawing.Point(0, 0);
            this.mainMenu.Name = "mainMenu";
            this.mainMenu.Padding = new System.Windows.Forms.Padding(20, 6, 0, 6);
            this.mainMenu.Size = new System.Drawing.Size(1400, 48);
            this.mainMenu.TabIndex = 1;
            this.mainMenu.Text = "mainMenu";
            // 
            // fileMenuItem
            // 
            this.fileMenuItem.DropDownItems.AddRange(new System.Windows.Forms.ToolStripItem[] {
            this.settingsMenuItem,
            this.aboutMenuItem,
            this.quitMenuItem});
            this.fileMenuItem.Name = "fileMenuItem";
            this.fileMenuItem.Size = new System.Drawing.Size(71, 36);
            this.fileMenuItem.Text = "File";
            // 
            // settingsMenuItem
            // 
            this.settingsMenuItem.Name = "settingsMenuItem";
            this.settingsMenuItem.Size = new System.Drawing.Size(233, 44);
            this.settingsMenuItem.Text = "Settings";
            this.settingsMenuItem.Click += new System.EventHandler(this.SettingsMenuItem_Click);
            // 
            // aboutMenuItem
            // 
            this.aboutMenuItem.Name = "aboutMenuItem";
            this.aboutMenuItem.Size = new System.Drawing.Size(233, 44);
            this.aboutMenuItem.Text = "About";
            this.aboutMenuItem.Click += new System.EventHandler(this.AboutMenuItem_Click);
            // 
            // quitMenuItem
            // 
            this.quitMenuItem.Name = "quitMenuItem";
            this.quitMenuItem.Size = new System.Drawing.Size(233, 44);
            this.quitMenuItem.Text = "Quit";
            this.quitMenuItem.Click += new System.EventHandler(this.ExitMenuItem_Click);
            // 
            // liveLayout
            // 
            this.liveLayout.ColumnCount = 1;
            this.liveLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.liveLayout.Controls.Add(this.liveTopBarLayout, 0, 0);
            this.liveLayout.Controls.Add(this.flowView, 0, 1);
            this.liveLayout.Controls.Add(this.dayChart, 0, 2);
            this.liveLayout.Controls.Add(this.liveActionsPanel, 0, 3);
            this.liveLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.liveLayout.Location = new System.Drawing.Point(0, 48);
            this.liveLayout.Margin = new System.Windows.Forms.Padding(0);
            this.liveLayout.Name = "liveLayout";
            this.liveLayout.RowCount = 4;
            this.liveLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.liveLayout.RowStyles.Add(new System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Percent, 40F));
            this.liveLayout.RowStyles.Add(new System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Percent, 60F));
            this.liveLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.liveLayout.Size = new System.Drawing.Size(1400, 1226);
            this.liveLayout.TabIndex = 0;
            // 
            // liveTopBarLayout
            // 
            this.liveTopBarLayout.AutoSize = true;
            this.liveTopBarLayout.ColumnCount = 1;
            this.liveTopBarLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.liveTopBarLayout.Controls.Add(this.statusSummaryControl, 0, 0);
            this.liveTopBarLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.liveTopBarLayout.Location = new System.Drawing.Point(0, 0);
            this.liveTopBarLayout.Margin = new System.Windows.Forms.Padding(0);
            this.liveTopBarLayout.Name = "liveTopBarLayout";
            this.liveTopBarLayout.RowCount = 1;
            this.liveTopBarLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.liveTopBarLayout.Size = new System.Drawing.Size(1400, 176);
            this.liveTopBarLayout.TabIndex = 7;
            // 
            // statusSummaryControl
            // 
            this.statusSummaryControl.AutoSize = true;
            this.statusSummaryControl.Dock = System.Windows.Forms.DockStyle.Fill;
            this.statusSummaryControl.Location = new System.Drawing.Point(12, 12);
            this.statusSummaryControl.Margin = new System.Windows.Forms.Padding(12, 12, 12, 12);
            this.statusSummaryControl.Name = "statusSummaryControl";
            this.statusSummaryControl.Size = new System.Drawing.Size(1376, 152);
            this.statusSummaryControl.TabIndex = 8;
            // 
            // flowView
            // 
            this.flowView.BackColor = System.Drawing.SystemColors.Window;
            this.flowView.Dock = System.Windows.Forms.DockStyle.Fill;
            this.flowView.Location = new System.Drawing.Point(0, 176);
            this.flowView.Margin = new System.Windows.Forms.Padding(0);
            this.flowView.MinimumSize = new System.Drawing.Size(0, 440);
            this.flowView.Name = "flowView";
            this.flowView.Padding = new System.Windows.Forms.Padding(48, 48, 48, 48);
            this.flowView.Size = new System.Drawing.Size(1400, 440);
            this.flowView.TabIndex = 7;
            this.flowView.Load += new System.EventHandler(this.flowView_Load);
            // 
            // dayChart
            // 
            this.dayChart.BackColor = System.Drawing.SystemColors.Window;
            this.dayChart.Dock = System.Windows.Forms.DockStyle.Fill;
            this.dayChart.Location = new System.Drawing.Point(0, 559);
            this.dayChart.Margin = new System.Windows.Forms.Padding(0);
            this.dayChart.MinimumSize = new System.Drawing.Size(0, 280);
            this.dayChart.Name = "dayChart";
            this.dayChart.Size = new System.Drawing.Size(1400, 575);
            this.dayChart.TabIndex = 4;
            // 
            // liveActionsPanel
            // 
            this.liveActionsPanel.AutoSize = true;
            this.liveActionsPanel.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.liveActionsPanel.Controls.Add(this.refreshNowButton);
            this.liveActionsPanel.Controls.Add(this.pauseResumeButton);
            this.liveActionsPanel.Dock = System.Windows.Forms.DockStyle.Fill;
            this.liveActionsPanel.Location = new System.Drawing.Point(0, 1134);
            this.liveActionsPanel.Margin = new System.Windows.Forms.Padding(0);
            this.liveActionsPanel.Name = "liveActionsPanel";
            this.liveActionsPanel.Padding = new System.Windows.Forms.Padding(12, 12, 12, 20);
            this.liveActionsPanel.Size = new System.Drawing.Size(1400, 92);
            this.liveActionsPanel.TabIndex = 5;
            // 
            // refreshNowButton
            // 
            this.refreshNowButton.AutoSize = true;
            this.refreshNowButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.refreshNowButton.Location = new System.Drawing.Point(24, 24);
            this.refreshNowButton.Margin = new System.Windows.Forms.Padding(12, 12, 12, 12);
            this.refreshNowButton.Name = "refreshNowButton";
            this.refreshNowButton.Size = new System.Drawing.Size(142, 35);
            this.refreshNowButton.TabIndex = 0;
            this.refreshNowButton.Text = "Refresh now";
            this.refreshNowButton.UseVisualStyleBackColor = true;
            this.refreshNowButton.Click += new System.EventHandler(this.RefreshNowButton_Click);
            // 
            // pauseResumeButton
            // 
            this.pauseResumeButton.AutoSize = true;
            this.pauseResumeButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.pauseResumeButton.Location = new System.Drawing.Point(190, 24);
            this.pauseResumeButton.Margin = new System.Windows.Forms.Padding(12, 12, 12, 12);
            this.pauseResumeButton.Name = "pauseResumeButton";
            this.pauseResumeButton.Size = new System.Drawing.Size(152, 35);
            this.pauseResumeButton.TabIndex = 1;
            this.pauseResumeButton.Text = "Pause polling";
            this.pauseResumeButton.UseVisualStyleBackColor = true;
            this.pauseResumeButton.Click += new System.EventHandler(this.PauseResumeButton_Click);
            // 
            // settingsLayout
            // 
            this.settingsLayout.AutoSize = true;
            this.settingsLayout.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.settingsLayout.ColumnCount = 1;
            this.settingsLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.settingsLayout.Controls.Add(this.cloudGroupBox, 0, 0);
            this.settingsLayout.Controls.Add(this.dataGroupBox, 0, 1);
            this.settingsLayout.Controls.Add(this.appGroupBox, 0, 2);
            this.settingsLayout.Controls.Add(this.logGroupBox, 0, 3);
            this.settingsLayout.Controls.Add(this.settingsButtonRow, 0, 4);
            this.settingsLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsLayout.Location = new System.Drawing.Point(10, 10);
            this.settingsLayout.Margin = new System.Windows.Forms.Padding(4);
            this.settingsLayout.Name = "settingsLayout";
            this.settingsLayout.RowCount = 6;
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsLayout.Size = new System.Drawing.Size(1617, 1397);
            this.settingsLayout.TabIndex = 0;
            // 
            // cloudGroupBox
            // 
            this.cloudGroupBox.AutoSize = true;
            this.cloudGroupBox.Controls.Add(this.cloudLayout);
            this.cloudGroupBox.Dock = System.Windows.Forms.DockStyle.Top;
            this.cloudGroupBox.Location = new System.Drawing.Point(6, 6);
            this.cloudGroupBox.Margin = new System.Windows.Forms.Padding(6);
            this.cloudGroupBox.Name = "cloudGroupBox";
            this.cloudGroupBox.Padding = new System.Windows.Forms.Padding(12, 14, 12, 14);
            this.cloudGroupBox.Size = new System.Drawing.Size(1605, 224);
            this.cloudGroupBox.TabIndex = 1;
            this.cloudGroupBox.TabStop = false;
            this.cloudGroupBox.Text = "SunSynk cloud access";
            // 
            // cloudLayout
            // 
            this.cloudLayout.AutoSize = true;
            this.cloudLayout.ColumnCount = 2;
            this.cloudLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 35F));
            this.cloudLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 65F));
            this.cloudLayout.Controls.Add(this.usernameLabel, 0, 0);
            this.cloudLayout.Controls.Add(this.usernameTextBox, 1, 0);
            this.cloudLayout.Controls.Add(this.passwordLabel, 0, 1);
            this.cloudLayout.Controls.Add(this.passwordTextBox, 1, 1);
            this.cloudLayout.Controls.Add(this.credentialsHintLabel, 0, 2);
            this.cloudLayout.Controls.Add(this.testConnectionButton, 1, 3);
            this.cloudLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.cloudLayout.Location = new System.Drawing.Point(12, 38);
            this.cloudLayout.Margin = new System.Windows.Forms.Padding(4);
            this.cloudLayout.Name = "cloudLayout";
            this.cloudLayout.RowCount = 4;
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.cloudLayout.Size = new System.Drawing.Size(1581, 172);
            this.cloudLayout.TabIndex = 0;
            // 
            // usernameLabel
            // 
            this.usernameLabel.AutoSize = true;
            this.usernameLabel.Location = new System.Drawing.Point(6, 0);
            this.usernameLabel.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.usernameLabel.Name = "usernameLabel";
            this.usernameLabel.Size = new System.Drawing.Size(110, 25);
            this.usernameLabel.TabIndex = 1;
            this.usernameLabel.Text = "Username";
            // 
            // usernameTextBox
            // 
            this.usernameTextBox.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.usernameTextBox.Location = new System.Drawing.Point(559, 6);
            this.usernameTextBox.Margin = new System.Windows.Forms.Padding(6);
            this.usernameTextBox.Name = "usernameTextBox";
            this.usernameTextBox.Size = new System.Drawing.Size(1016, 31);
            this.usernameTextBox.TabIndex = 2;
            // 
            // passwordLabel
            // 
            this.passwordLabel.AutoSize = true;
            this.passwordLabel.Location = new System.Drawing.Point(6, 55);
            this.passwordLabel.Margin = new System.Windows.Forms.Padding(6, 12, 6, 0);
            this.passwordLabel.Name = "passwordLabel";
            this.passwordLabel.Size = new System.Drawing.Size(106, 25);
            this.passwordLabel.TabIndex = 3;
            this.passwordLabel.Text = "Password";
            // 
            // passwordTextBox
            // 
            this.passwordTextBox.Anchor = ((System.Windows.Forms.AnchorStyles)(((System.Windows.Forms.AnchorStyles.Top | System.Windows.Forms.AnchorStyles.Left) 
            | System.Windows.Forms.AnchorStyles.Right)));
            this.passwordTextBox.Location = new System.Drawing.Point(559, 49);
            this.passwordTextBox.Margin = new System.Windows.Forms.Padding(6);
            this.passwordTextBox.Name = "passwordTextBox";
            this.passwordTextBox.Size = new System.Drawing.Size(1016, 31);
            this.passwordTextBox.TabIndex = 4;
            this.passwordTextBox.UseSystemPasswordChar = true;
            // 
            // credentialsHintLabel
            // 
            this.credentialsHintLabel.AutoSize = true;
            this.credentialsHintLabel.ForeColor = System.Drawing.SystemColors.GrayText;
            this.credentialsHintLabel.Location = new System.Drawing.Point(6, 94);
            this.credentialsHintLabel.Margin = new System.Windows.Forms.Padding(6, 8, 6, 0);
            this.credentialsHintLabel.MaximumSize = new System.Drawing.Size(700, 0);
            this.credentialsHintLabel.Name = "credentialsHintLabel";
            this.credentialsHintLabel.Size = new System.Drawing.Size(338, 25);
            this.credentialsHintLabel.TabIndex = 19;
            this.credentialsHintLabel.Text = "Use your SunSynk portal account.";
            // 
            // testConnectionButton
            // 
            this.testConnectionButton.Anchor = System.Windows.Forms.AnchorStyles.Left;
            this.testConnectionButton.AutoSize = true;
            this.testConnectionButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.testConnectionButton.Location = new System.Drawing.Point(559, 131);
            this.testConnectionButton.Margin = new System.Windows.Forms.Padding(6, 12, 6, 6);
            this.testConnectionButton.Name = "testConnectionButton";
            this.testConnectionButton.Size = new System.Drawing.Size(120, 35);
            this.testConnectionButton.TabIndex = 5;
            this.testConnectionButton.Text = "Get plants";
            this.testConnectionButton.UseVisualStyleBackColor = true;
            // 
            // dataGroupBox
            // 
            this.dataGroupBox.AutoSize = true;
            this.dataGroupBox.Controls.Add(this.dataLayout);
            this.dataGroupBox.Dock = System.Windows.Forms.DockStyle.Top;
            this.dataGroupBox.Location = new System.Drawing.Point(6, 242);
            this.dataGroupBox.Margin = new System.Windows.Forms.Padding(6);
            this.dataGroupBox.Name = "dataGroupBox";
            this.dataGroupBox.Padding = new System.Windows.Forms.Padding(12, 14, 12, 14);
            this.dataGroupBox.Size = new System.Drawing.Size(1605, 350);
            this.dataGroupBox.TabIndex = 2;
            this.dataGroupBox.TabStop = false;
            this.dataGroupBox.Text = "Sync and sites";
            // 
            // dataLayout
            // 
            this.dataLayout.AutoSize = true;
            this.dataLayout.ColumnCount = 2;
            this.dataLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 35F));
            this.dataLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 65F));
            this.dataLayout.Controls.Add(this.pollIntervalLabel, 0, 0);
            this.dataLayout.Controls.Add(this.pollIntervalNumeric, 1, 0);
            this.dataLayout.Controls.Add(this.plantLabel, 0, 1);
            this.dataLayout.Controls.Add(this.plantListBox, 1, 1);
            this.dataLayout.Controls.Add(this.selectedPlantLabel, 1, 2);
            this.dataLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.dataLayout.Location = new System.Drawing.Point(12, 38);
            this.dataLayout.Margin = new System.Windows.Forms.Padding(4);
            this.dataLayout.Name = "dataLayout";
            this.dataLayout.RowCount = 3;
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.dataLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.dataLayout.Size = new System.Drawing.Size(1581, 298);
            this.dataLayout.TabIndex = 0;
            // 
            // pollIntervalLabel
            // 
            this.pollIntervalLabel.AutoSize = true;
            this.pollIntervalLabel.Location = new System.Drawing.Point(6, 0);
            this.pollIntervalLabel.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.pollIntervalLabel.Name = "pollIntervalLabel";
            this.pollIntervalLabel.Size = new System.Drawing.Size(124, 25);
            this.pollIntervalLabel.TabIndex = 6;
            this.pollIntervalLabel.Text = "Poll interval";
            // 
            // pollIntervalNumeric
            // 
            this.pollIntervalNumeric.Anchor = ((System.Windows.Forms.AnchorStyles)((System.Windows.Forms.AnchorStyles.Left | System.Windows.Forms.AnchorStyles.Right)));
            this.pollIntervalNumeric.Location = new System.Drawing.Point(559, 0);
            this.pollIntervalNumeric.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
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
            this.pollIntervalNumeric.Size = new System.Drawing.Size(1016, 31);
            this.pollIntervalNumeric.TabIndex = 7;
            this.pollIntervalNumeric.Value = new decimal(new int[] {
            60,
            0,
            0,
            0});
            // 
            // plantLabel
            // 
            this.plantLabel.AutoSize = true;
            this.plantLabel.Location = new System.Drawing.Point(6, 31);
            this.plantLabel.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.plantLabel.Name = "plantLabel";
            this.plantLabel.Size = new System.Drawing.Size(151, 25);
            this.plantLabel.TabIndex = 9;
            this.plantLabel.Text = "Available sites";
            // 
            // plantListBox
            // 
            this.plantListBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.plantListBox.FormattingEnabled = true;
            this.plantListBox.IntegralHeight = false;
            this.plantListBox.ItemHeight = 25;
            this.plantListBox.Location = new System.Drawing.Point(559, 37);
            this.plantListBox.Margin = new System.Windows.Forms.Padding(6);
            this.plantListBox.Name = "plantListBox";
            this.plantListBox.Size = new System.Drawing.Size(1016, 230);
            this.plantListBox.TabIndex = 10;
            // 
            // selectedPlantLabel
            // 
            this.selectedPlantLabel.AutoSize = true;
            this.selectedPlantLabel.Location = new System.Drawing.Point(559, 273);
            this.selectedPlantLabel.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.selectedPlantLabel.Name = "selectedPlantLabel";
            this.selectedPlantLabel.Size = new System.Drawing.Size(223, 25);
            this.selectedPlantLabel.TabIndex = 11;
            this.selectedPlantLabel.Text = "Selected plant: (none)";
            // 
            // appGroupBox
            // 
            this.appGroupBox.AutoSize = true;
            this.appGroupBox.Controls.Add(this.appLayout);
            this.appGroupBox.Dock = System.Windows.Forms.DockStyle.Top;
            this.appGroupBox.Location = new System.Drawing.Point(6, 604);
            this.appGroupBox.Margin = new System.Windows.Forms.Padding(6);
            this.appGroupBox.Name = "appGroupBox";
            this.appGroupBox.Padding = new System.Windows.Forms.Padding(12, 14, 12, 14);
            this.appGroupBox.Size = new System.Drawing.Size(1605, 134);
            this.appGroupBox.TabIndex = 3;
            this.appGroupBox.TabStop = false;
            this.appGroupBox.Text = "App and OS";
            // 
            // appLayout
            // 
            this.appLayout.AutoSize = true;
            this.appLayout.ColumnCount = 1;
            this.appLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.appLayout.Controls.Add(this.startOnBootCheckBox, 0, 0);
            this.appLayout.Controls.Add(this.themeToggleCheckBox, 0, 1);
            this.appLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.appLayout.Location = new System.Drawing.Point(12, 38);
            this.appLayout.Margin = new System.Windows.Forms.Padding(4);
            this.appLayout.Name = "appLayout";
            this.appLayout.RowCount = 2;
            this.appLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.appLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.appLayout.Size = new System.Drawing.Size(1581, 82);
            this.appLayout.TabIndex = 0;
            // 
            // startOnBootCheckBox
            // 
            this.startOnBootCheckBox.AutoSize = true;
            this.startOnBootCheckBox.Location = new System.Drawing.Point(6, 6);
            this.startOnBootCheckBox.Margin = new System.Windows.Forms.Padding(6);
            this.startOnBootCheckBox.Name = "startOnBootCheckBox";
            this.startOnBootCheckBox.Size = new System.Drawing.Size(319, 29);
            this.startOnBootCheckBox.TabIndex = 8;
            this.startOnBootCheckBox.Text = "Start when Windows signs in";
            this.startOnBootCheckBox.UseVisualStyleBackColor = true;
            // 
            // themeToggleCheckBox
            // 
            this.themeToggleCheckBox.AutoSize = true;
            this.themeToggleCheckBox.Location = new System.Drawing.Point(6, 47);
            this.themeToggleCheckBox.Margin = new System.Windows.Forms.Padding(6);
            this.themeToggleCheckBox.Name = "themeToggleCheckBox";
            this.themeToggleCheckBox.Size = new System.Drawing.Size(189, 29);
            this.themeToggleCheckBox.TabIndex = 18;
            this.themeToggleCheckBox.Text = "Use dark mode";
            this.themeToggleCheckBox.UseVisualStyleBackColor = true;
            // 
            // logGroupBox
            // 
            this.logGroupBox.AutoSize = true;
            this.logGroupBox.Controls.Add(this.logLayout);
            this.logGroupBox.Dock = System.Windows.Forms.DockStyle.Top;
            this.logGroupBox.Location = new System.Drawing.Point(6, 750);
            this.logGroupBox.Margin = new System.Windows.Forms.Padding(6);
            this.logGroupBox.Name = "logGroupBox";
            this.logGroupBox.Padding = new System.Windows.Forms.Padding(12, 14, 12, 14);
            this.logGroupBox.Size = new System.Drawing.Size(1605, 137);
            this.logGroupBox.TabIndex = 4;
            this.logGroupBox.TabStop = false;
            this.logGroupBox.Text = "Authentication activity";
            // 
            // logLayout
            // 
            this.logLayout.AutoSize = true;
            this.logLayout.ColumnCount = 1;
            this.logLayout.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.logLayout.Controls.Add(this.authLogLabel, 0, 0);
            this.logLayout.Controls.Add(this.authLogTextBox, 0, 1);
            this.logLayout.Dock = System.Windows.Forms.DockStyle.Fill;
            this.logLayout.Location = new System.Drawing.Point(12, 38);
            this.logLayout.Margin = new System.Windows.Forms.Padding(4);
            this.logLayout.Name = "logLayout";
            this.logLayout.RowCount = 2;
            this.logLayout.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.logLayout.RowStyles.Add(new System.Windows.Forms.RowStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.logLayout.Size = new System.Drawing.Size(1581, 85);
            this.logLayout.TabIndex = 0;
            // 
            // authLogLabel
            // 
            this.authLogLabel.AutoSize = true;
            this.authLogLabel.Location = new System.Drawing.Point(6, 0);
            this.authLogLabel.Margin = new System.Windows.Forms.Padding(6, 0, 6, 0);
            this.authLogLabel.Name = "authLogLabel";
            this.authLogLabel.Size = new System.Drawing.Size(129, 25);
            this.authLogLabel.TabIndex = 16;
            this.authLogLabel.Text = "Auth activity";
            // 
            // authLogTextBox
            // 
            this.authLogTextBox.BackColor = System.Drawing.SystemColors.Window;
            this.authLogTextBox.Dock = System.Windows.Forms.DockStyle.Fill;
            this.authLogTextBox.Location = new System.Drawing.Point(6, 31);
            this.authLogTextBox.Margin = new System.Windows.Forms.Padding(6);
            this.authLogTextBox.Multiline = true;
            this.authLogTextBox.Name = "authLogTextBox";
            this.authLogTextBox.ReadOnly = true;
            this.authLogTextBox.ScrollBars = System.Windows.Forms.ScrollBars.Vertical;
            this.authLogTextBox.Size = new System.Drawing.Size(1569, 48);
            this.authLogTextBox.TabIndex = 17;
            // 
            // settingsButtonRow
            // 
            this.settingsButtonRow.AutoSize = true;
            this.settingsButtonRow.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.settingsButtonRow.ColumnCount = 2;
            this.settingsButtonRow.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle(System.Windows.Forms.SizeType.Percent, 100F));
            this.settingsButtonRow.ColumnStyles.Add(new System.Windows.Forms.ColumnStyle());
            this.settingsButtonRow.Controls.Add(this.settingsButtonPanel, 1, 0);
            this.settingsButtonRow.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsButtonRow.Location = new System.Drawing.Point(6, 905);
            this.settingsButtonRow.Margin = new System.Windows.Forms.Padding(6, 12, 6, 6);
            this.settingsButtonRow.Name = "settingsButtonRow";
            this.settingsButtonRow.RowCount = 1;
            this.settingsButtonRow.RowStyles.Add(new System.Windows.Forms.RowStyle());
            this.settingsButtonRow.Size = new System.Drawing.Size(1605, 53);
            this.settingsButtonRow.TabIndex = 19;
            // 
            // settingsButtonPanel
            // 
            this.settingsButtonPanel.AutoSize = true;
            this.settingsButtonPanel.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.settingsButtonPanel.Controls.Add(this.saveSettingsButton);
            this.settingsButtonPanel.Controls.Add(this.cancelSettingsButton);
            this.settingsButtonPanel.Controls.Add(this.resetSettingsButton);
            this.settingsButtonPanel.Dock = System.Windows.Forms.DockStyle.Fill;
            this.settingsButtonPanel.Location = new System.Drawing.Point(1350, 0);
            this.settingsButtonPanel.Margin = new System.Windows.Forms.Padding(0);
            this.settingsButtonPanel.Name = "settingsButtonPanel";
            this.settingsButtonPanel.Padding = new System.Windows.Forms.Padding(0, 6, 0, 0);
            this.settingsButtonPanel.Size = new System.Drawing.Size(255, 53);
            this.settingsButtonPanel.TabIndex = 12;
            this.settingsButtonPanel.WrapContents = false;
            // 
            // saveSettingsButton
            // 
            this.saveSettingsButton.AutoSize = true;
            this.saveSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.saveSettingsButton.BackColor = System.Drawing.Color.FromArgb(((int)(((byte)(58)))), ((int)(((byte)(134)))), ((int)(((byte)(255)))));
            this.saveSettingsButton.FlatAppearance.BorderSize = 0;
            this.saveSettingsButton.FlatStyle = System.Windows.Forms.FlatStyle.Flat;
            this.saveSettingsButton.ForeColor = System.Drawing.Color.White;
            this.saveSettingsButton.Location = new System.Drawing.Point(6, 12);
            this.saveSettingsButton.Margin = new System.Windows.Forms.Padding(6);
            this.saveSettingsButton.Name = "saveSettingsButton";
            this.saveSettingsButton.Size = new System.Drawing.Size(52, 35);
            this.saveSettingsButton.TabIndex = 0;
            this.saveSettingsButton.Text = "OK";
            this.saveSettingsButton.UseVisualStyleBackColor = false;
            // 
            // cancelSettingsButton
            // 
            this.cancelSettingsButton.AutoSize = true;
            this.cancelSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.cancelSettingsButton.Location = new System.Drawing.Point(70, 12);
            this.cancelSettingsButton.Margin = new System.Windows.Forms.Padding(6);
            this.cancelSettingsButton.Name = "cancelSettingsButton";
            this.cancelSettingsButton.Size = new System.Drawing.Size(89, 35);
            this.cancelSettingsButton.TabIndex = 2;
            this.cancelSettingsButton.Text = "Cancel";
            this.cancelSettingsButton.UseVisualStyleBackColor = true;
            // 
            // resetSettingsButton
            // 
            this.resetSettingsButton.AutoSize = true;
            this.resetSettingsButton.AutoSizeMode = System.Windows.Forms.AutoSizeMode.GrowAndShrink;
            this.resetSettingsButton.Location = new System.Drawing.Point(171, 12);
            this.resetSettingsButton.Margin = new System.Windows.Forms.Padding(6);
            this.resetSettingsButton.Name = "resetSettingsButton";
            this.resetSettingsButton.Size = new System.Drawing.Size(78, 35);
            this.resetSettingsButton.TabIndex = 1;
            this.resetSettingsButton.Text = "Reset";
            this.resetSettingsButton.UseVisualStyleBackColor = true;
            // 
            // MainForm
            // 
            this.AutoScaleDimensions = new System.Drawing.SizeF(192F, 192F);
            this.AutoScaleMode = System.Windows.Forms.AutoScaleMode.Dpi;
            this.ClientSize = new System.Drawing.Size(1400, 1274);
            this.Controls.Add(this.liveLayout);
            this.Controls.Add(this.mainMenu);
            this.Icon = ((System.Drawing.Icon)(resources.GetObject("$this.Icon")));
            this.MainMenuStrip = this.mainMenu;
            this.Margin = new System.Windows.Forms.Padding(12, 12, 12, 12);
            this.MinimumSize = new System.Drawing.Size(1406, 1281);
            this.Name = "MainForm";
            this.StartPosition = System.Windows.Forms.FormStartPosition.CenterScreen;
            this.Text = "SunSynk Tray";
            this.trayMenu.ResumeLayout(false);
            this.mainMenu.ResumeLayout(false);
            this.mainMenu.PerformLayout();
            this.liveLayout.ResumeLayout(false);
            this.liveLayout.PerformLayout();
            this.liveTopBarLayout.ResumeLayout(false);
            this.liveTopBarLayout.PerformLayout();
            this.liveActionsPanel.ResumeLayout(false);
            this.liveActionsPanel.PerformLayout();
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
            this.PerformLayout();

    }

    #endregion

    private NotifyIcon trayIcon;
    private ContextMenuStrip trayMenu;
    private ToolStripMenuItem openMenuItem;
    private ToolStripMenuItem refreshMenuItem;
    private ToolStripMenuItem pauseResumeMenuItem;
    private ToolStripSeparator traySeparator;
    private ToolStripMenuItem exitMenuItem;
    private MenuStrip mainMenu;
    private ToolStripMenuItem fileMenuItem;
    private ToolStripMenuItem settingsMenuItem;
    private ToolStripMenuItem quitMenuItem;
    private ToolStripMenuItem aboutMenuItem;
    private TableLayoutPanel liveLayout;
    private TableLayoutPanel liveTopBarLayout;
    private Controls.StatusSummaryControl statusSummaryControl;
    private Controls.PowerFlowViewControl flowView;
    private Controls.DayChartControl dayChart;
    private FlowLayoutPanel liveActionsPanel;
    private Button refreshNowButton;
    private Button pauseResumeButton;
    private TableLayoutPanel settingsLayout;
    private TableLayoutPanel settingsButtonRow;
    private Label usernameLabel;
    private TextBox usernameTextBox;
    private Label passwordLabel;
    private TextBox passwordTextBox;
    private Label pollIntervalLabel;
    private NumericUpDown pollIntervalNumeric;
    private CheckBox startOnBootCheckBox;
    private Label plantLabel;
    private ListBox plantListBox;
    private Label selectedPlantLabel;
    private FlowLayoutPanel settingsButtonPanel;
    private Button testConnectionButton;
    private Button saveSettingsButton;
    private Button resetSettingsButton;
    private Button cancelSettingsButton;
    private Label authLogLabel;
    private TextBox authLogTextBox;
    private CheckBox themeToggleCheckBox;
    private GroupBox cloudGroupBox;
    private TableLayoutPanel cloudLayout;
    private Label credentialsHintLabel;
    private GroupBox dataGroupBox;
    private TableLayoutPanel dataLayout;
    private GroupBox appGroupBox;
    private TableLayoutPanel appLayout;
    private GroupBox logGroupBox;
    private TableLayoutPanel logLayout;
}
