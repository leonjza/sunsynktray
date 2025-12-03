using System.Drawing.Drawing2D;
using SunSynkTrayWin.Theme;

namespace SunSynkTrayWin.Controls;

public class DayChartControl : Panel
{
    public record ChartPoint(double X, double Y, string Label);
    public record ChartSeries(string Name, IReadOnlyList<ChartPoint> Points);

    private readonly List<ChartSeries> _series = new();
    private readonly Color[] _palette =
    [
        Color.SeaGreen,
        Color.SteelBlue,
        Color.OrangeRed,
        Color.MediumPurple,
        Color.Goldenrod
    ];
    private readonly ThemePaletteHost _paletteHost = new(ThemeManager.LightPalette);
    private ChartPoint? _hoverPoint;
    private ChartSeries? _hoverSeries;
    private PointF _hoverScreen;

    public DayChartControl()
    {
        DoubleBuffered = true;
        BackColor = SystemColors.Window;
        MouseMove += DayChartControl_MouseMove;
        MouseLeave += (_, _) => { _hoverPoint = null; Invalidate(); };
    }

    public void SetSeries(IEnumerable<ChartSeries> series)
    {
        _series.Clear();
        _series.AddRange(series);
        Invalidate();
    }

    public void SetPalette(ThemePalette palette)
    {
        _paletteHost.Apply(this, palette);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        base.OnPaint(e);
        e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
        var theme = _paletteHost.Palette;
        e.Graphics.Clear(theme.Window);

        var client = ClientRectangle;
        if (client.Width <= 0 || client.Height <= 0)
        {
            return;
        }

        const int leftMargin = 80;
        const int bottomMargin = 32;
        const int topMargin = 50;

        var allPoints = _series.SelectMany(s => s.Points).ToList();
        if (allPoints.Count == 0)
        {
            return;
        }

        var minX = allPoints.Min(p => p.X);
        var maxX = allPoints.Max(p => p.X);

        var socSeries = _series.Where(IsSocSeries).ToList();
        var primarySeries = _series.Where(s => !IsSocSeries(s)).ToList();

        double primaryMinY, primaryMaxY;
        if (primarySeries.SelectMany(s => s.Points).Any())
        {
            primaryMinY = primarySeries.SelectMany(s => s.Points).Min(p => p.Y);
            primaryMaxY = primarySeries.SelectMany(s => s.Points).Max(p => p.Y);
        }
        else if (socSeries.SelectMany(s => s.Points).Any())
        {
            // If only SOC is present, use 0-100 as the primary axis.
            primaryMinY = 0;
            primaryMaxY = 100;
        }
        else
        {
            primaryMinY = 0;
            primaryMaxY = 1;
        }

        // SOC is always displayed on a 0-100 scale.
        const double socMinY = 0;
        const double socMaxY = 100;

        var legendWidth = 0;
        var legendHeight = 0;
        if (_series.Count > 0)
        {
            const int swatch = 12;
            const int spacing = 10;
            var maxWidth = 0f;
            for (var i = 0; i < _series.Count; i++)
            {
                var textSize = e.Graphics.MeasureString(_series[i].Name, Font);
                var rowWidth = swatch + 8 + textSize.Width;
                maxWidth = Math.Max(maxWidth, rowWidth);
            }
            var rowHeight = Math.Max(swatch, Font.Height) + 6;
            legendWidth = (int)Math.Ceiling((double)maxWidth) + spacing;
            legendHeight = _series.Count * rowHeight + spacing;
        }

        const int legendPadding = 8;
        // Reserve space for right-axis labels (SOC) plus legend.
        var socLabelWidth = 0f;
        var hasSoc = socSeries.SelectMany(s => s.Points).Any();
        if (hasSoc)
        {
            for (var i = 0; i <= 4; i++)
            {
                var yVal = socMaxY - (socMaxY - socMinY) * (i / 4d);
                var label = yVal.ToString("0");
                var size = e.Graphics.MeasureString(label, Font);
                socLabelWidth = Math.Max(socLabelWidth, size.Width);
            }
        }
        // Right margin: SOC labels + small gutter + legend (if any).
        var rightMargin = (int)Math.Ceiling(socLabelWidth) + legendPadding + (legendWidth > 0 ? legendWidth + legendPadding : 12);
        // Bottom margin: measure longest X tick to reserve space for labels.
        var maxXLabelHeight = 0f;
        {
            var testLabel = "00:00";
            var size = e.Graphics.MeasureString(testLabel, Font);
            maxXLabelHeight = size.Height;
        }
        var bottom = Math.Max(bottomMargin, (int)Math.Ceiling(maxXLabelHeight) + 12);

        var plotWidth = Math.Max(10, client.Width - leftMargin - rightMargin);
        var plotHeight = Math.Max(10, client.Height - topMargin - bottom);
        var rect = new Rectangle(client.Left + leftMargin, client.Top + topMargin, plotWidth, plotHeight);
        if (_series.Count == 0)
        {
            return;
        }

        // Draw legend background to avoid overlap with plot lines.
        if (legendWidth > 0)
        {
            var legendXStart = rect.Right + (int)Math.Ceiling(socLabelWidth) + legendPadding;
            var legendRect = new Rectangle(legendXStart, rect.Top + 6, legendWidth + legendPadding, Math.Max(legendHeight, 40));
            using var legendBg = new SolidBrush(theme.Window);
            e.Graphics.FillRectangle(legendBg, legendRect);
        }

        if (maxX - minX < 0.001)
        {
            maxX = minX + 1;
        }

        // Axes
        using var axisPen = new Pen(theme.Border, 1);
        e.Graphics.DrawLine(axisPen, rect.Left, rect.Bottom, rect.Right, rect.Bottom);
        e.Graphics.DrawLine(axisPen, rect.Left, rect.Top, rect.Left, rect.Bottom);
        var hasPrimary = primarySeries.SelectMany(s => s.Points).Any();
        var leftAxisMin = hasPrimary ? primaryMinY : socMinY;
        var leftAxisMax = hasPrimary ? primaryMaxY : socMaxY;

        // Y ticks
        var yTicks = 4;
        for (var i = 0; i <= yTicks; i++)
        {
            var yFrac = i / (double)yTicks;
            var yVal = leftAxisMax - (leftAxisMax - leftAxisMin) * yFrac;
            var yPos = rect.Top + rect.Height * yFrac;
            e.Graphics.DrawLine(axisPen, rect.Left - 5, (float)yPos, rect.Left, (float)yPos);
            var label = yVal.ToString("0");
            var size = e.Graphics.MeasureString(label, Font);
            using var tickBrush = new SolidBrush(theme.SubtleText);
            e.Graphics.DrawString(label, Font, tickBrush, rect.Left - leftMargin + 5, (float)yPos - size.Height / 2);
        }

        // Right axis for SOC when both are present.
        if (hasSoc && hasPrimary)
        {
            e.Graphics.DrawLine(axisPen, rect.Right, rect.Top, rect.Right, rect.Bottom);
            for (var i = 0; i <= yTicks; i++)
            {
                var yFrac = i / (double)yTicks;
                var yVal = socMaxY - (socMaxY - socMinY) * yFrac;
                var yPos = rect.Top + rect.Height * yFrac;
                e.Graphics.DrawLine(axisPen, rect.Right, (float)yPos, rect.Right + 5, (float)yPos);
                var label = yVal.ToString("0");
                var size = e.Graphics.MeasureString(label, Font);
                using var tickBrush = new SolidBrush(theme.SubtleText);
                var labelX = rect.Right + 8;
                e.Graphics.DrawString(label, Font, tickBrush, labelX, (float)yPos - size.Height / 2);
            }
        }

        // X ticks (hours)
        var hourStep = 120d; // minutes
        for (var xVal = Math.Ceiling(minX / hourStep) * hourStep; xVal <= maxX; xVal += hourStep)
        {
            var xFrac = (xVal - minX) / (maxX - minX);
            var xPos = rect.Left + rect.Width * xFrac;
            e.Graphics.DrawLine(axisPen, (float)xPos, rect.Bottom, (float)xPos, rect.Bottom + 5);
            var ts = TimeSpan.FromMinutes(xVal);
            var label = ts.ToString(@"hh\:mm");
            var size = e.Graphics.MeasureString(label, Font);
            using var tickBrush = new SolidBrush(theme.SubtleText);
            e.Graphics.DrawString(label, Font, tickBrush, (float)xPos - size.Width / 2, rect.Bottom + 8);
        }

        // Series lines
        _hoverPoint = null;
        _hoverSeries = null;
        for (var sIndex = 0; sIndex < _series.Count; sIndex++)
        {
            var series = _series[sIndex];
            var color = _palette[sIndex % _palette.Length];
            using var pen = new Pen(color, 2);

            var points = series.Points
                .OrderBy(p => p.X)
                .Select(p =>
                {
                    double minY, maxY;
                    if (IsSocSeries(series))
                    {
                        minY = socMinY;
                        maxY = socMaxY;
                    }
                    else
                    {
                        minY = primaryMinY;
                        maxY = primaryMaxY;
                    }

                    if (Math.Abs(maxY - minY) < 0.001)
                    {
                        maxY = minY + 1;
                    }

                    return new PointF(
                        rect.Left + (float)((p.X - minX) / (maxX - minX) * rect.Width),
                        rect.Bottom - (float)((p.Y - minY) / (maxY - minY) * rect.Height));
                })
                .ToArray();

            if (points.Length >= 2)
            {
                e.Graphics.DrawLines(pen, points);
            }
            else if (points.Length == 1)
            {
                e.Graphics.FillEllipse(new SolidBrush(color), points[0].X - 2, points[0].Y - 2, 4, 4);
            }

            // Hover detection
            for (var i = 0; i < points.Length; i++)
            {
                var pt = points[i];
                if (Distance(pt, _hoverScreen) < 6)
                {
                    _hoverPoint = series.Points.OrderBy(p => p.X).ElementAt(i);
                    _hoverSeries = series;
                    _hoverScreen = pt;
                }
            }

            // Legend
            var legendX = rect.Right + (int)Math.Ceiling(socLabelWidth) + legendPadding + 8;
            var textSize = e.Graphics.MeasureString(series.Name, Font);
            const int swatch = 12;
            var rowHeight = (int)Math.Ceiling(Math.Max(swatch, textSize.Height)) + 6;
            var legendY = rect.Top + 10 + sIndex * rowHeight;
            var swatchY = legendY + (rowHeight - swatch) / 2;
            var textY = legendY + (rowHeight - textSize.Height) / 2;
            using var brush = new SolidBrush(color);
            e.Graphics.FillRectangle(brush, legendX, swatchY, swatch, swatch);
            using var textBrush = new SolidBrush(theme.Text);
            e.Graphics.DrawString(series.Name, Font, textBrush, legendX + swatch + 8, textY);
        }

        // Tooltip
        if (_hoverPoint != null && _hoverSeries != null)
        {
            var text = $"{_hoverSeries.Name}: {_hoverPoint.Y:0} @ {_hoverPoint.Label}";
            var size = e.Graphics.MeasureString(text, Font);
            var tooltipRect = new RectangleF(_hoverScreen.X + 8, _hoverScreen.Y - size.Height - 8, size.Width + 8, size.Height + 4);
            using var bg = new SolidBrush(Color.FromArgb(230, theme.Surface));
            using var border = new Pen(theme.Border);
            e.Graphics.FillRectangle(bg, tooltipRect);
            e.Graphics.DrawRectangle(border, tooltipRect.X, tooltipRect.Y, tooltipRect.Width, tooltipRect.Height);
            using var textBrush = new SolidBrush(theme.Text);
            e.Graphics.DrawString(text, Font, textBrush, tooltipRect.X + 4, tooltipRect.Y + 2);
        }
    }

    private void DayChartControl_MouseMove(object? sender, MouseEventArgs e)
    {
        _hoverScreen = e.Location;
        Invalidate();
    }

    protected override void OnResize(EventArgs eventargs)
    {
        base.OnResize(eventargs);
        Invalidate();
    }

    private static bool IsSocSeries(ChartSeries series) =>
        series.Name?.IndexOf("soc", StringComparison.OrdinalIgnoreCase) >= 0;

    private static double Distance(PointF a, PointF b)
    {
        var dx = a.X - b.X;
        var dy = a.Y - b.Y;
        return Math.Sqrt(dx * dx + dy * dy);
    }
}
