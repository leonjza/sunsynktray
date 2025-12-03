using System.Collections.Generic;
using System.ComponentModel;
using System.Runtime.CompilerServices;

namespace SunSynkTrayWin.ViewModels;

/// <summary>
/// Simple base for INotifyPropertyChanged consumers with optional extension hook.
/// </summary>
public abstract class ObservableObject : INotifyPropertyChanged
{
    public event PropertyChangedEventHandler? PropertyChanged;

    protected bool SetProperty<T>(ref T storage, T value, [CallerMemberName] string? propertyName = null)
    {
        if (EqualityComparer<T>.Default.Equals(storage, value))
        {
            return false;
        }

        storage = value;
        OnPropertyChanged(propertyName);
        OnPropertyChangedExtended(propertyName);
        return true;
    }

    protected void OnPropertyChanged(string? propertyName)
    {
        if (string.IsNullOrWhiteSpace(propertyName))
        {
            return;
        }

        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }

    /// <summary>
    /// Allows derived classes to trigger additional notifications for dependent properties.
    /// </summary>
    /// <param name="propertyName">Name of the property that changed.</param>
    protected virtual void OnPropertyChangedExtended(string? propertyName)
    {
    }
}
