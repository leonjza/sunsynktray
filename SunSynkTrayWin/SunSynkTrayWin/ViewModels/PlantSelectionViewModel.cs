namespace SunSynkTrayWin.ViewModels;

public sealed class PlantSelectionViewModel : ObservableObject
{
    private int? _id;
    private string? _name;
    private bool _isAvailable;

    public int? Id
    {
        get => _id;
        set => SetProperty(ref _id, value);
    }

    public string? Name
    {
        get => _name;
        set => SetProperty(ref _name, value);
    }

    public bool IsAvailable
    {
        get => _isAvailable;
        set => SetProperty(ref _isAvailable, value);
    }

    public string DisplayLabel
    {
        get
        {
            if (!_id.HasValue)
            {
                return "Selected plant: (none)";
            }

            if (!_isAvailable)
            {
                return $"Saved plant ID {_id} not in list";
            }

            var displayName = _name ?? $"Plant {_id}";
            return $"Selected plant: {displayName} (ID: {_id})";
        }
    }

    protected override void OnPropertyChangedExtended(string? propertyName)
    {
        if (propertyName is nameof(Id) or nameof(Name) or nameof(IsAvailable))
        {
            OnPropertyChanged(nameof(DisplayLabel));
        }
    }
}
