using System;
using System.Threading.Tasks;
using SunSynkTrayWin.ViewModels;

namespace SunSynkTrayWin;

internal static class StatusOperationRunner
{
    public static async Task RunWithStatusAsync(
        StatusViewModel status,
        string context,
        string workingStatus,
        string successStatus,
        Func<Task> action,
        Action? onSuccess = null,
        Action<Exception>? onError = null,
        Action<string>? log = null)
    {
        status.ConnectionStatus = workingStatus;
        status.LastUpdated = $"{context} requested at {DateTime.Now:T}";
        log?.Invoke($"{context}...");

        try
        {
            await action().ConfigureAwait(false);
            status.ConnectionStatus = successStatus;
            status.LastUpdated = $"{context} completed at {DateTime.Now:T}";
            onSuccess?.Invoke();
        }
        catch (Exception ex)
        {
            status.ConnectionStatus = "Error";
            status.LastUpdated = $"{context} failed: {ex.Message}";
            log?.Invoke($"{context} failed: {ex.Message}");
            onError?.Invoke(ex);
            throw;
        }
    }
}
