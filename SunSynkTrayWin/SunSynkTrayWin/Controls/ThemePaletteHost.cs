using System.Windows.Forms;
using SunSynkTrayWin.Theme;

namespace SunSynkTrayWin.Controls;

/// <summary>
/// Small helper to store and apply the active theme palette for controls.
/// </summary>
internal sealed class ThemePaletteHost
{
    public ThemePaletteHost(ThemePalette palette) => Palette = palette;

    public ThemePalette Palette { get; private set; }

    public void Apply(Control control, ThemePalette palette)
    {
        Palette = palette;
        control.BackColor = palette.Window;
        control.Invalidate();
    }
}
