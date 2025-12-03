using System.Drawing;

namespace SunSynkTrayWin;

internal static class StatusColorHelper
{
    public static Color GetSocColor(double soc)
    {
        var clamped = Math.Max(0, Math.Min(100, soc));
        if (clamped >= 80) return Color.FromArgb(34, 197, 94);      // green
        if (clamped >= 60) return Color.FromArgb(132, 204, 22);     // yellow-green
        if (clamped >= 40) return Color.FromArgb(234, 179, 8);      // amber
        return Color.FromArgb(239, 68, 68);                         // red
    }
}
