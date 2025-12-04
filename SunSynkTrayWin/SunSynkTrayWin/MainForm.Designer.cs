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
        components = new System.ComponentModel.Container();
        System.ComponentModel.ComponentResourceManager resources = new System.ComponentModel.ComponentResourceManager(typeof(MainForm));
        trayIcon = new NotifyIcon(components);
        trayMenu = new ContextMenuStrip(components);
        openMenuItem = new ToolStripMenuItem();
        refreshMenuItem = new ToolStripMenuItem();
        pauseResumeMenuItem = new ToolStripMenuItem();
        traySeparator = new ToolStripSeparator();
        exitMenuItem = new ToolStripMenuItem();
        mainMenu = new MenuStrip();
        fileMenuItem = new ToolStripMenuItem();
        settingsMenuItem = new ToolStripMenuItem();
        quitMenuItem = new ToolStripMenuItem();
        aboutMenuItem = new ToolStripMenuItem();
        liveLayout = new TableLayoutPanel();
        liveTopBarLayout = new TableLayoutPanel();
        statusSummaryControl = new SunSynkTrayWin.Controls.StatusSummaryControl();
        flowView = new SunSynkTrayWin.Controls.PowerFlowViewControl();
        dayChart = new SunSynkTrayWin.Controls.DayChartControl();
        liveActionsPanel = new FlowLayoutPanel();
        refreshNowButton = new Button();
        pauseResumeButton = new Button();
        settingsLayout = new TableLayoutPanel();
        readinessPanel = new FlowLayoutPanel();
        readinessDot = new Panel();
        readinessStatusLabel = new Label();
        usernameLabel = new Label();
        usernameTextBox = new TextBox();
        passwordLabel = new Label();
        passwordTextBox = new TextBox();
        testConnectionButton = new Button();
        pollIntervalLabel = new Label();
        pollIntervalNumeric = new NumericUpDown();
        startOnBootCheckBox = new CheckBox();
        plantLabel = new Label();
        plantListBox = new ListBox();
        selectedPlantLabel = new Label();
        settingsButtonPanel = new FlowLayoutPanel();
        saveSettingsButton = new Button();
        cancelSettingsButton = new Button();
        resetSettingsButton = new Button();
        authLogLabel = new Label();
        authLogTextBox = new TextBox();
        credentialsHintLabel = new Label();
        themeToggleCheckBox = new CheckBox();
        trayMenu.SuspendLayout();
        mainMenu.SuspendLayout();
        liveLayout.SuspendLayout();
        liveTopBarLayout.SuspendLayout();
        liveActionsPanel.SuspendLayout();
        settingsLayout.SuspendLayout();
        readinessPanel.SuspendLayout();
        ((System.ComponentModel.ISupportInitialize)pollIntervalNumeric).BeginInit();
        settingsButtonPanel.SuspendLayout();
        SuspendLayout();
        // 
        // trayIcon
        // 
        trayIcon.ContextMenuStrip = trayMenu;
        trayIcon.Icon = null;
        trayIcon.Text = "SunSynk Tray";
        trayIcon.Visible = true;
        trayIcon.DoubleClick += TrayIcon_DoubleClick;
        // 
        // trayMenu
        // 
        trayMenu.ImageScalingSize = new Size(32, 32);
        trayMenu.Items.AddRange(new ToolStripItem[] { openMenuItem, refreshMenuItem, pauseResumeMenuItem, traySeparator, exitMenuItem });
        trayMenu.Name = "trayMenu";
        trayMenu.Size = new Size(231, 162);
        // 
        // openMenuItem
        // 
        openMenuItem.Name = "openMenuItem";
        openMenuItem.Size = new Size(230, 38);
        openMenuItem.Text = "Open";
        openMenuItem.Click += OpenMenuItem_Click;
        // 
        // refreshMenuItem
        // 
        refreshMenuItem.Name = "refreshMenuItem";
        refreshMenuItem.Size = new Size(230, 38);
        refreshMenuItem.Text = "Refresh now";
        refreshMenuItem.Click += RefreshMenuItem_Click;
        // 
        // pauseResumeMenuItem
        // 
        pauseResumeMenuItem.Name = "pauseResumeMenuItem";
        pauseResumeMenuItem.Size = new Size(230, 38);
        pauseResumeMenuItem.Text = "Pause polling";
        pauseResumeMenuItem.Click += PauseResumeMenuItem_Click;
        // 
        // traySeparator
        // 
        traySeparator.Name = "traySeparator";
        traySeparator.Size = new Size(227, 6);
        // 
        // exitMenuItem
        // 
        exitMenuItem.Name = "exitMenuItem";
        exitMenuItem.Size = new Size(230, 38);
        exitMenuItem.Text = "Quit";
        exitMenuItem.Click += ExitMenuItem_Click;
        // 
        // mainMenu
        // 
        mainMenu.ImageScalingSize = new Size(24, 24);
        mainMenu.Items.AddRange(new ToolStripItem[] { fileMenuItem });
        mainMenu.Location = new Point(0, 0);
        mainMenu.Name = "mainMenu";
        mainMenu.Padding = new Padding(10, 3, 0, 3);
        mainMenu.Size = new Size(1671, 42);
        mainMenu.TabIndex = 1;
        mainMenu.Text = "mainMenu";
        // 
        // fileMenuItem
        // 
        fileMenuItem.DropDownItems.AddRange(new ToolStripItem[] { settingsMenuItem, aboutMenuItem, quitMenuItem });
        fileMenuItem.Name = "fileMenuItem";
        fileMenuItem.Size = new Size(71, 36);
        fileMenuItem.Text = "File";
        // 
        // settingsMenuItem
        // 
        settingsMenuItem.Name = "settingsMenuItem";
        settingsMenuItem.Size = new Size(233, 44);
        settingsMenuItem.Text = "Settings";
        settingsMenuItem.Click += SettingsMenuItem_Click;
        // 
        // quitMenuItem
        // 
        quitMenuItem.Name = "quitMenuItem";
        quitMenuItem.Size = new Size(233, 44);
        quitMenuItem.Text = "Quit";
        quitMenuItem.Click += ExitMenuItem_Click;
        // 
        // aboutMenuItem
        // 
        aboutMenuItem.Name = "aboutMenuItem";
        aboutMenuItem.Size = new Size(233, 44);
        aboutMenuItem.Text = "About";
        aboutMenuItem.Click += AboutMenuItem_Click;
        // 
        // liveLayout
        // 
        liveLayout.ColumnCount = 1;
        liveLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        liveLayout.Controls.Add(liveTopBarLayout, 0, 0);
        liveLayout.Controls.Add(flowView, 0, 1);
        liveLayout.Controls.Add(dayChart, 0, 2);
        liveLayout.Controls.Add(liveActionsPanel, 0, 3);
        liveLayout.Dock = DockStyle.Fill;
        liveLayout.Location = new Point(0, 42);
        liveLayout.Margin = new Padding(0);
        liveLayout.Name = "liveLayout";
        liveLayout.RowCount = 4;
        liveLayout.RowStyles.Add(new RowStyle());
        liveLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 40F));
        liveLayout.RowStyles.Add(new RowStyle(SizeType.Percent, 60F));
        liveLayout.RowStyles.Add(new RowStyle());
        liveLayout.Size = new Size(1671, 1172);
        liveLayout.TabIndex = 0;
        // 
        // liveLayout
        // 
        // liveTopBarLayout
        // 
        liveTopBarLayout.AutoSize = true;
        liveTopBarLayout.ColumnCount = 1;
        liveTopBarLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        liveTopBarLayout.Controls.Add(statusSummaryControl, 0, 0);
        liveTopBarLayout.Dock = DockStyle.Fill;
        liveTopBarLayout.Location = new Point(0, 0);
        liveTopBarLayout.Margin = new Padding(0);
        liveTopBarLayout.Name = "liveTopBarLayout";
        liveTopBarLayout.RowCount = 1;
        liveTopBarLayout.RowStyles.Add(new RowStyle());
        liveTopBarLayout.Size = new Size(1671, 140);
        liveTopBarLayout.TabIndex = 7;
        // 
        // statusSummaryControl
        // 
        statusSummaryControl.AutoSize = true;
        statusSummaryControl.Dock = DockStyle.Fill;
        statusSummaryControl.Location = new Point(6, 6);
        statusSummaryControl.Margin = new Padding(6);
        statusSummaryControl.Name = "statusSummaryControl";
        statusSummaryControl.Size = new Size(1659, 128);
        statusSummaryControl.TabIndex = 8;
        // 
        // flowView
        // 
        flowView.BackColor = SystemColors.Window;
        flowView.Dock = DockStyle.Fill;
        flowView.Location = new Point(0, 140);
        flowView.Margin = new Padding(0);
        flowView.MinimumSize = new Size(0, 220);
        flowView.Name = "flowView";
        flowView.Padding = new Padding(24);
        flowView.Size = new Size(1663, 380);
        flowView.TabIndex = 7;
        // 
        // dayChart
        // 
        dayChart.BackColor = SystemColors.Window;
        dayChart.Dock = DockStyle.Fill;
        dayChart.Location = new Point(0, 520);
        dayChart.Margin = new Padding(0);
        dayChart.MinimumSize = new Size(0, 140);
        dayChart.Name = "dayChart";
        dayChart.Size = new Size(1663, 573);
        dayChart.TabIndex = 4;
        // 
        // liveActionsPanel
        // 
        liveActionsPanel.AutoSize = true;
        liveActionsPanel.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        liveActionsPanel.Controls.Add(refreshNowButton);
        liveActionsPanel.Controls.Add(pauseResumeButton);
        liveActionsPanel.Dock = DockStyle.Fill;
        liveActionsPanel.Location = new Point(0, 1093);
        liveActionsPanel.Margin = new Padding(0);
        liveActionsPanel.Name = "liveActionsPanel";
        liveActionsPanel.Padding = new Padding(6, 6, 6, 10);
        liveActionsPanel.Size = new Size(1663, 70);
        liveActionsPanel.TabIndex = 5;
        // 
        // refreshNowButton
        // 
        refreshNowButton.AutoSize = true;
        refreshNowButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        refreshNowButton.Location = new Point(12, 12);
        refreshNowButton.Margin = new Padding(6);
        refreshNowButton.Name = "refreshNowButton";
        refreshNowButton.TabIndex = 0;
        refreshNowButton.Text = "Refresh now";
        refreshNowButton.UseVisualStyleBackColor = true;
        refreshNowButton.Click += RefreshNowButton_Click;
        // 
        // pauseResumeButton
        // 
        pauseResumeButton.AutoSize = true;
        pauseResumeButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        pauseResumeButton.Location = new Point(194, 12);
        pauseResumeButton.Margin = new Padding(6);
        pauseResumeButton.Name = "pauseResumeButton";
        pauseResumeButton.TabIndex = 1;
        pauseResumeButton.Text = "Pause polling";
        pauseResumeButton.UseVisualStyleBackColor = true;
        pauseResumeButton.Click += PauseResumeButton_Click;
        // 
        // settingsLayout
        // 
        settingsLayout.AutoSize = true;
        settingsLayout.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        settingsLayout.ColumnCount = 2;
        settingsLayout.ColumnStyles.Add(new ColumnStyle());
        settingsLayout.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100F));
        settingsLayout.Controls.Add(readinessPanel, 1, 0);
        settingsLayout.Controls.Add(usernameLabel, 0, 1);
        settingsLayout.Controls.Add(usernameTextBox, 1, 1);
        settingsLayout.Controls.Add(passwordLabel, 0, 2);
        settingsLayout.Controls.Add(passwordTextBox, 1, 2);
        settingsLayout.Controls.Add(testConnectionButton, 1, 3);
        settingsLayout.Controls.Add(pollIntervalLabel, 0, 4);
        settingsLayout.Controls.Add(pollIntervalNumeric, 1, 4);
        settingsLayout.Controls.Add(startOnBootCheckBox, 1, 5);
        settingsLayout.Controls.Add(themeToggleCheckBox, 1, 6);
        settingsLayout.Controls.Add(plantLabel, 0, 7);
        settingsLayout.Controls.Add(plantListBox, 1, 7);
        settingsLayout.Controls.Add(selectedPlantLabel, 1, 8);
        settingsLayout.Controls.Add(settingsButtonPanel, 1, 9);
        settingsLayout.Controls.Add(authLogLabel, 0, 10);
        settingsLayout.Controls.Add(authLogTextBox, 1, 10);
        settingsLayout.Dock = DockStyle.Fill;
        settingsLayout.Location = new Point(10, 10);
        settingsLayout.Margin = new Padding(4);
        settingsLayout.Name = "settingsLayout";
        settingsLayout.RowCount = 11;
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle());
        settingsLayout.RowStyles.Add(new RowStyle(SizeType.AutoSize));
        settingsLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 256F));
        settingsLayout.RowStyles.Add(new RowStyle()); // selected plant label
        settingsLayout.RowStyles.Add(new RowStyle(SizeType.AutoSize)); // buttons
        settingsLayout.RowStyles.Add(new RowStyle(SizeType.Absolute, 110F)); // auth log (approx 5 rows)
        settingsLayout.Size = new Size(1617, 1397);
        settingsLayout.TabIndex = 0;
        // 
        // readinessPanel
        // 
        readinessPanel.Anchor = AnchorStyles.Top | AnchorStyles.Right;
        readinessPanel.AutoSize = true;
        readinessPanel.Controls.Add(readinessDot);
        readinessPanel.Controls.Add(readinessStatusLabel);
        readinessPanel.FlowDirection = FlowDirection.RightToLeft;
        readinessPanel.Location = new Point(1350, 6);
        readinessPanel.Margin = new Padding(6);
        readinessPanel.Name = "readinessPanel";
        readinessPanel.Size = new Size(261, 40);
        readinessPanel.TabIndex = 0;
        readinessPanel.WrapContents = false;
        readinessPanel.Visible = false;
        // 
        // readinessDot
        // 
        readinessDot.BackColor = Color.OrangeRed;
        readinessDot.BorderStyle = BorderStyle.FixedSingle;
        readinessDot.Location = new Point(226, 6);
        readinessDot.Margin = new Padding(6, 6, 11, 6);
        readinessDot.Name = "readinessDot";
        readinessDot.Size = new Size(24, 28);
        readinessDot.TabIndex = 0;
        // 
        // readinessStatusLabel
        // 
        readinessStatusLabel.AutoSize = true;
        readinessStatusLabel.Location = new Point(6, 4);
        readinessStatusLabel.Margin = new Padding(6, 4, 6, 0);
        readinessStatusLabel.Name = "readinessStatusLabel";
        readinessStatusLabel.Size = new Size(208, 32);
        readinessStatusLabel.TabIndex = 1;
        readinessStatusLabel.Text = "Not authenticated";
        // 
        // usernameLabel
        // 
        usernameLabel.AutoSize = true;
        usernameLabel.Location = new Point(6, 52);
        usernameLabel.Margin = new Padding(6, 0, 6, 0);
        usernameLabel.Name = "usernameLabel";
        usernameLabel.Size = new Size(121, 32);
        usernameLabel.TabIndex = 1;
        usernameLabel.Text = "Username";
        // 
        // usernameTextBox
        // 
        usernameTextBox.Anchor = AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right;
        usernameTextBox.Location = new Point(182, 58);
        usernameTextBox.Margin = new Padding(6);
        usernameTextBox.Name = "usernameTextBox";
        usernameTextBox.Size = new Size(1429, 39);
        usernameTextBox.TabIndex = 2;
        // 
        // passwordLabel
        // 
        passwordLabel.AutoSize = true;
        passwordLabel.Location = new Point(6, 103);
        passwordLabel.Margin = new Padding(6, 0, 6, 0);
        passwordLabel.Name = "passwordLabel";
        passwordLabel.Size = new Size(111, 32);
        passwordLabel.TabIndex = 3;
        passwordLabel.Text = "Password";
        // 
        // passwordTextBox
        // 
        passwordTextBox.Anchor = AnchorStyles.Top | AnchorStyles.Left | AnchorStyles.Right;
        passwordTextBox.Location = new Point(182, 109);
        passwordTextBox.Margin = new Padding(6);
        passwordTextBox.Name = "passwordTextBox";
        passwordTextBox.Size = new Size(1429, 39);
        passwordTextBox.TabIndex = 4;
        passwordTextBox.UseSystemPasswordChar = true;
        // 
        // testConnectionButton
        // 
        testConnectionButton.AutoSize = true;
        testConnectionButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        testConnectionButton.Location = new Point(182, 160);
        testConnectionButton.Margin = new Padding(6, 6, 6, 21);
        testConnectionButton.Name = "testConnectionButton";
        testConnectionButton.TabIndex = 5;
        testConnectionButton.Text = "Get plants";
        testConnectionButton.UseVisualStyleBackColor = true;
        testConnectionButton.Click += TestConnectionButton_Click;
        // 
        // pollIntervalLabel
        // 
        pollIntervalLabel.AutoSize = true;
        pollIntervalLabel.Location = new Point(6, 231);
        pollIntervalLabel.Margin = new Padding(6, 0, 6, 0);
        pollIntervalLabel.Name = "pollIntervalLabel";
        pollIntervalLabel.Size = new Size(138, 32);
        pollIntervalLabel.TabIndex = 6;
        pollIntervalLabel.Text = "Poll interval";
        // 
        // pollIntervalNumeric
        // 
        pollIntervalNumeric.Anchor = AnchorStyles.Left | AnchorStyles.Right;
        pollIntervalNumeric.Location = new Point(182, 231);
        pollIntervalNumeric.Margin = new Padding(6, 0, 6, 0);
        pollIntervalNumeric.Maximum = new decimal(new int[] { 900, 0, 0, 0 });
        pollIntervalNumeric.Minimum = new decimal(new int[] { 30, 0, 0, 0 });
        pollIntervalNumeric.Name = "pollIntervalNumeric";
        pollIntervalNumeric.Size = new Size(1429, 39);
        pollIntervalNumeric.TabIndex = 7;
        pollIntervalNumeric.Value = new decimal(new int[] { 60, 0, 0, 0 });
        // 
        // startOnBootCheckBox
        // 
        startOnBootCheckBox.AutoSize = true;
        startOnBootCheckBox.Location = new Point(182, 276);
        startOnBootCheckBox.Margin = new Padding(6);
        startOnBootCheckBox.Name = "startOnBootCheckBox";
        startOnBootCheckBox.Size = new Size(351, 36);
        startOnBootCheckBox.TabIndex = 8;
        startOnBootCheckBox.Text = "Start when Windows signs in";
        startOnBootCheckBox.UseVisualStyleBackColor = true;
        // 
        // themeToggleCheckBox
        // 
        themeToggleCheckBox.AutoSize = true;
        themeToggleCheckBox.Location = new Point(182, 336);
        themeToggleCheckBox.Margin = new Padding(6);
        themeToggleCheckBox.Name = "themeToggleCheckBox";
        themeToggleCheckBox.Size = new Size(175, 36);
        themeToggleCheckBox.TabIndex = 18;
        themeToggleCheckBox.Text = "Use dark mode";
        themeToggleCheckBox.UseVisualStyleBackColor = true;
        // 
        // plantLabel
        // 
        plantLabel.AutoSize = true;
        plantLabel.Location = new Point(6, 388);
        plantLabel.Margin = new Padding(6, 0, 6, 0);
        plantLabel.Name = "plantLabel";
        plantLabel.Size = new Size(164, 32);
        plantLabel.TabIndex = 9;
        plantLabel.Text = "Available sites";
        // 
        // plantListBox
        // 
        plantListBox.Dock = DockStyle.Fill;
        plantListBox.FormattingEnabled = true;
        plantListBox.IntegralHeight = false;
        plantListBox.ItemHeight = 32;
        plantListBox.Location = new Point(182, 394);
        plantListBox.Margin = new Padding(6);
        plantListBox.Name = "plantListBox";
        plantListBox.Size = new Size(1429, 244);
        plantListBox.TabIndex = 10;
        plantListBox.SelectedIndexChanged += PlantListBox_SelectedIndexChanged;
        plantListBox.Format += PlantListBox_Format;
        // 
        // selectedPlantLabel
        // 
        selectedPlantLabel.AutoSize = true;
        selectedPlantLabel.Location = new Point(182, 608);
        selectedPlantLabel.Margin = new Padding(6, 0, 6, 0);
        selectedPlantLabel.Name = "selectedPlantLabel";
        selectedPlantLabel.Size = new Size(247, 32);
        selectedPlantLabel.TabIndex = 11;
        selectedPlantLabel.Text = "Selected plant: (none)";
        // settingsButtonPanel
        // 
        settingsButtonPanel.AutoSize = true;
        settingsButtonPanel.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        settingsButtonPanel.Controls.Add(saveSettingsButton);
        settingsButtonPanel.Controls.Add(cancelSettingsButton);
        settingsButtonPanel.Controls.Add(resetSettingsButton);
        settingsButtonPanel.Dock = DockStyle.Left;
        settingsButtonPanel.Location = new Point(182, 642);
        settingsButtonPanel.Margin = new Padding(6);
        settingsButtonPanel.Name = "settingsButtonPanel";
        settingsButtonPanel.Size = new Size(300, 62);
        settingsButtonPanel.TabIndex = 12;
        // 
        // saveSettingsButton
        // 
        saveSettingsButton.AutoSize = true;
        saveSettingsButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        saveSettingsButton.BackColor = Color.FromArgb(58, 134, 255);
        saveSettingsButton.FlatAppearance.BorderSize = 0;
        saveSettingsButton.FlatStyle = FlatStyle.Flat;
        saveSettingsButton.ForeColor = Color.White;
        saveSettingsButton.Location = new Point(6, 6);
        saveSettingsButton.Margin = new Padding(6);
        saveSettingsButton.Name = "saveSettingsButton";
        saveSettingsButton.TabIndex = 0;
        saveSettingsButton.Text = "OK";
        saveSettingsButton.UseVisualStyleBackColor = false;
        saveSettingsButton.Click += SaveSettingsButton_Click;
        // 
        // cancelSettingsButton
        // 
        cancelSettingsButton.AutoSize = true;
        cancelSettingsButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        cancelSettingsButton.Location = new Point(155, 6);
        cancelSettingsButton.Margin = new Padding(6);
        cancelSettingsButton.Name = "cancelSettingsButton";
        cancelSettingsButton.TabIndex = 2;
        cancelSettingsButton.Text = "Cancel";
        cancelSettingsButton.UseVisualStyleBackColor = true;
        cancelSettingsButton.Click += CancelSettingsButton_Click;
        // 
        // resetSettingsButton
        // 
        resetSettingsButton.AutoSize = true;
        resetSettingsButton.AutoSizeMode = AutoSizeMode.GrowAndShrink;
        resetSettingsButton.Location = new Point(329, 6);
        resetSettingsButton.Margin = new Padding(6);
        resetSettingsButton.Name = "resetSettingsButton";
        resetSettingsButton.TabIndex = 1;
        resetSettingsButton.Text = "Reset";
        resetSettingsButton.UseVisualStyleBackColor = true;
        resetSettingsButton.Click += ResetSettingsButton_Click;
        // 
        // authLogLabel
        // 
        authLogLabel.AutoSize = true;
        authLogLabel.Location = new Point(6, 628);
        authLogLabel.Margin = new Padding(6, 0, 6, 0);
        authLogLabel.Name = "authLogLabel";
        authLogLabel.Size = new Size(147, 32);
        authLogLabel.TabIndex = 16;
        authLogLabel.Text = "Auth activity";
        // 
        // authLogTextBox
        // 
        authLogTextBox.BackColor = SystemColors.Window;
        authLogTextBox.Dock = DockStyle.Top;
        authLogTextBox.Location = new Point(182, 628);
        authLogTextBox.Margin = new Padding(6, 6, 6, 12);
        authLogTextBox.Multiline = true;
        authLogTextBox.Name = "authLogTextBox";
        authLogTextBox.ReadOnly = true;
        authLogTextBox.ScrollBars = ScrollBars.Vertical;
        authLogTextBox.Size = new Size(1429, 90);
        authLogTextBox.TabIndex = 17;
        // 
        // credentialsHintLabel
        // 
        credentialsHintLabel.Location = new Point(0, 0);
        credentialsHintLabel.Name = "credentialsHintLabel";
        credentialsHintLabel.Size = new Size(100, 23);
        credentialsHintLabel.TabIndex = 0;
        // 
        // MainForm
        // 
        AutoScaleDimensions = new SizeF(96F, 96F);
        AutoScaleMode = AutoScaleMode.Dpi;
        ClientSize = new Size(700, 680);
        Controls.Add(liveLayout);
        Controls.Add(mainMenu);
        Icon = new Icon(Path.Combine(AppContext.BaseDirectory, "sunsynk.ico"));
        MainMenuStrip = mainMenu;
        Margin = new Padding(6);
        MinimumSize = new Size(700, 680);
        Name = "MainForm";
        StartPosition = FormStartPosition.CenterScreen;
        Text = "SunSynk Tray";
        trayMenu.ResumeLayout(false);
        mainMenu.ResumeLayout(false);
        mainMenu.PerformLayout();
        liveLayout.ResumeLayout(false);
        liveLayout.PerformLayout();
        liveTopBarLayout.ResumeLayout(false);
        liveTopBarLayout.PerformLayout();
        liveActionsPanel.ResumeLayout(false);
        settingsLayout.ResumeLayout(false);
        settingsLayout.PerformLayout();
        readinessPanel.ResumeLayout(false);
        readinessPanel.PerformLayout();
        ((System.ComponentModel.ISupportInitialize)pollIntervalNumeric).EndInit();
        settingsButtonPanel.ResumeLayout(false);
        settingsButtonPanel.PerformLayout();
        ResumeLayout(false);
        PerformLayout();
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
    private FlowLayoutPanel readinessPanel;
    private Panel readinessDot;
    private Label readinessStatusLabel;
    private FlowLayoutPanel settingsButtonPanel;
    private Button testConnectionButton;
    private Button saveSettingsButton;
    private Button resetSettingsButton;
    private Button cancelSettingsButton;
    private Label credentialsHintLabel;
    private Label authLogLabel;
    private TextBox authLogTextBox;
    private CheckBox themeToggleCheckBox;
}
