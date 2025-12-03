using System.Runtime.InteropServices;
using System.Drawing;

namespace SunSynkTrayWin.Theme;

public enum AppTheme
{
    Light,
    Dark
}

public sealed class ThemePalette
{
    public ThemePalette(Color window, Color surface, Color text, Color subtleText, Color accent, Color border, Color muted, Color inputBackground)
    {
        Window = window;
        Surface = surface;
        Text = text;
        SubtleText = subtleText;
        Accent = accent;
        Border = border;
        Muted = muted;
        InputBackground = inputBackground;
    }

    public Color Window { get; }
    public Color Surface { get; }
    public Color Text { get; }
    public Color SubtleText { get; }
    public Color Accent { get; }
    public Color Border { get; }
    public Color Muted { get; }
    public Color InputBackground { get; }
}

public sealed class ThemeManager
{
    public static ThemePalette LightPalette { get; } = new(
        window: Color.White,
        surface: Color.FromArgb(247, 249, 252),
        text: Color.FromArgb(34, 39, 46),
        subtleText: Color.FromArgb(110, 118, 129),
        accent: Color.FromArgb(58, 134, 255),
        border: Color.FromArgb(208, 216, 224),
        muted: Color.FromArgb(190, 196, 205),
        inputBackground: Color.White);

    public static ThemePalette DarkPalette { get; } = new(
        window: Color.FromArgb(30, 32, 36),
        surface: Color.FromArgb(42, 45, 50),
        text: Color.WhiteSmoke,
        subtleText: Color.FromArgb(180, 185, 190),
        accent: Color.FromArgb(86, 156, 255),
        border: Color.FromArgb(70, 74, 82),
        muted: Color.FromArgb(95, 100, 110),
        inputBackground: Color.FromArgb(34, 36, 40));

    private readonly ThemePalette _light = LightPalette;
    private readonly ThemePalette _dark = DarkPalette;

    public ThemePalette GetPalette(AppTheme theme) => theme == AppTheme.Dark ? _dark : _light;

    public ToolStripProfessionalRenderer CreateRenderer(ThemePalette palette) => new(new PaletteColorTable(palette));

    public static void ApplyWindowTheme(IntPtr handle, AppTheme theme)
    {
        if (handle == IntPtr.Zero)
        {
            return;
        }

        var build = Environment.OSVersion.Version.Build;
        var attribute = build >= 22000 ? 20 : 19; // DWMWA_USE_IMMERSIVE_DARK_MODE
        var useDark = theme == AppTheme.Dark ? 1 : 0;
        DwmSetWindowAttribute(handle, attribute, ref useDark, sizeof(int));
    }

    private sealed class PaletteColorTable : ProfessionalColorTable
    {
        private readonly ThemePalette _palette;

        public PaletteColorTable(ThemePalette palette) => _palette = palette;

        public override Color ToolStripDropDownBackground => _palette.Surface;
        public override Color ImageMarginGradientBegin => _palette.Surface;
        public override Color ImageMarginGradientMiddle => _palette.Surface;
        public override Color ImageMarginGradientEnd => _palette.Surface;
        public override Color MenuBorder => _palette.Border;
        public override Color MenuItemBorder => _palette.Border;
        public override Color MenuItemSelected => _palette.Border;
        public override Color MenuItemSelectedGradientBegin => _palette.Border;
        public override Color MenuItemSelectedGradientEnd => _palette.Border;
        public override Color MenuItemPressedGradientBegin => _palette.Border;
        public override Color MenuItemPressedGradientEnd => _palette.Border;
        public override Color SeparatorDark => _palette.Border;
        public override Color SeparatorLight => _palette.Border;
    }

    [DllImport("dwmapi.dll")]
    private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int attrValue, int attrSize);
}
