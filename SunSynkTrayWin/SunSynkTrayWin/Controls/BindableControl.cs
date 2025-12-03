using System.ComponentModel;
using System.Windows.Forms;
using SunSynkTrayWin.ViewModels;

namespace SunSynkTrayWin.Controls;

/// <summary>
/// Generic base for controls that bind to a view model and update on property changes.
/// </summary>
public abstract class BindableControl<TViewModel> : UserControl where TViewModel : INotifyPropertyChanged
{
    private TViewModel? _viewModel;

    protected TViewModel? ViewModel => _viewModel;

    public void Bind(TViewModel viewModel)
    {
        if (_viewModel != null)
        {
            _viewModel.PropertyChanged -= ViewModelOnPropertyChanged;
        }

        _viewModel = viewModel;
        _viewModel.PropertyChanged += ViewModelOnPropertyChanged;
        ApplyModelToUi();
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing && _viewModel != null)
        {
            _viewModel.PropertyChanged -= ViewModelOnPropertyChanged;
        }

        base.Dispose(disposing);
    }

    private void ViewModelOnPropertyChanged(object? sender, PropertyChangedEventArgs e)
    {
        this.InvokeIfRequired(() => OnViewModelPropertyChanged(e));
    }

    protected abstract void OnViewModelPropertyChanged(PropertyChangedEventArgs e);

    protected abstract void ApplyModelToUi();
}
