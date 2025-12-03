using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace SunSynkTrayWin.Storage;

/// <summary>
/// Minimal secure store for credentials and selection using DPAPI. Encrypts username/password
/// per current user and persists to a JSON file under %AppData%/SunSynkTray/settings.json.
/// </summary>
public class SecureSettingsStore
{
    private readonly string _settingsPath;
    private readonly byte[] _entropy = Encoding.UTF8.GetBytes("SunSynkTray_v1");

    public SecureSettingsStore()
    {
        var appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
        var dir = Path.Combine(appData, "SunSynkTray");
        Directory.CreateDirectory(dir);
        _settingsPath = Path.Combine(dir, "settings.json");
    }

    public SettingsData GetDefaultSettings() => SettingsData.Default;

    public void SaveSettings(SettingsData settings)
    {
        var effective = settings.Normalize();
        var payload = new SettingsPayload
        {
            Username = ProtectToBase64(effective.Username ?? string.Empty),
            Password = ProtectToBase64(effective.Password ?? string.Empty),
            PlantId = effective.PlantId,
            PlantName = effective.PlantName,
            PollIntervalSeconds = effective.PollIntervalSeconds,
            StartOnBoot = effective.StartOnBoot,
            UseDarkMode = effective.UseDarkMode
        };

        var json = JsonSerializer.Serialize(payload);
        File.WriteAllText(_settingsPath, json);
    }

    public SettingsData LoadSettings()
    {
        if (!File.Exists(_settingsPath))
        {
            return GetDefaultSettings();
        }

        try
        {
            var json = File.ReadAllText(_settingsPath);
            var payload = JsonSerializer.Deserialize<SettingsPayload>(json);
            if (payload is null)
            {
                return GetDefaultSettings();
            }

            var username = UnprotectFromBase64(payload.Username);
            var password = UnprotectFromBase64(payload.Password);
            return new SettingsData
            {
                Username = username,
                Password = password,
                PlantId = payload.PlantId,
                PlantName = payload.PlantName,
                PollIntervalSeconds = payload.PollIntervalSeconds ?? 0,
                StartOnBoot = payload.StartOnBoot.GetValueOrDefault(),
                UseDarkMode = payload.UseDarkMode.GetValueOrDefault(SettingsData.Default.UseDarkMode)
            }.Normalize();
        }
        catch
        {
            // Corrupted or unreadable file; ignore and return empty.
            return GetDefaultSettings();
        }
    }

    private string ProtectToBase64(string value)
    {
        var bytes = Encoding.UTF8.GetBytes(value ?? string.Empty);
        var protectedBytes = ProtectedData.Protect(bytes, _entropy, DataProtectionScope.CurrentUser);
        return Convert.ToBase64String(protectedBytes);
    }

    private string UnprotectFromBase64(string? value)
    {
        if (string.IsNullOrWhiteSpace(value))
        {
            return string.Empty;
        }

        var protectedBytes = Convert.FromBase64String(value);
        var unprotected = ProtectedData.Unprotect(protectedBytes, _entropy, DataProtectionScope.CurrentUser);
        return Encoding.UTF8.GetString(unprotected);
    }

    private class SettingsPayload
    {
        public string? Username { get; set; }
        public string? Password { get; set; }
        public int? PlantId { get; set; }
        public string? PlantName { get; set; }
        public int? PollIntervalSeconds { get; set; }
        public bool? StartOnBoot { get; set; }
        public bool? UseDarkMode { get; set; }
    }
}

public class SettingsData
{
    public string? Username { get; set; }
    public string? Password { get; set; }
    public int? PlantId { get; set; }
    public string? PlantName { get; set; }
    public int PollIntervalSeconds { get; set; } = 60;
    public bool StartOnBoot { get; set; }
    public bool UseDarkMode { get; set; } = true;

    public static SettingsData Default => new()
    {
        Username = string.Empty,
        Password = string.Empty,
        PollIntervalSeconds = 60,
        StartOnBoot = false,
        UseDarkMode = true
    };

    public SettingsData Normalize(SettingsData? defaults = null)
    {
        var d = defaults ?? Default;
        return new SettingsData
        {
            Username = Username ?? d.Username,
            Password = Password ?? d.Password,
            PlantId = PlantId,
            PlantName = PlantName,
            PollIntervalSeconds = PollIntervalSeconds > 0 ? PollIntervalSeconds : d.PollIntervalSeconds,
            StartOnBoot = StartOnBoot,
            UseDarkMode = UseDarkMode
        };
    }
}
