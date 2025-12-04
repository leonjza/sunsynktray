using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Windows.Forms;
using SunSynkTrayWin.Api;
using SunSynkTrayWin.Theme;
using SunSynkTrayWin;

namespace SunSynkTrayWin.Controls;

public class PowerFlowViewControl : UserControl
{
    private readonly Font _titleFont;
    private readonly Font _valueFont;
    private readonly StringFormat _centerFormat;
    private readonly System.Windows.Forms.Timer _animationTimer;
    private float _animationOffset;
    private PowerFlowData? _flow;
    private readonly ThemePaletteHost _paletteHost = new(ThemeManager.LightPalette);
    private float _scale = 1f;

    public PowerFlowViewControl()
    {
        DoubleBuffered = true;
        ResizeRedraw = true;
        BackColor = _paletteHost.Palette.Window;
        AutoScaleMode = AutoScaleMode.Dpi;
        Padding = new Padding(16);
        MinimumSize = new Size(0, 260);

        _titleFont = new Font(Font.FontFamily, 9F, FontStyle.Regular);
        _valueFont = new Font(Font.FontFamily, 11F, FontStyle.Bold);
        _centerFormat = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center };

        _animationTimer = new System.Windows.Forms.Timer { Interval = 33 };
        _animationTimer.Tick += (_, _) =>
        {
            _animationOffset += 2.5f;
            Invalidate();
        };
        _animationTimer.Start();
    }

    public void SetFlow(PowerFlowData flow)
    {
        _flow = flow;
        Invalidate();
    }

    public void SetPalette(ThemePalette palette)
    {
        _paletteHost.Apply(this, palette);
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            _titleFont.Dispose();
            _valueFont.Dispose();
            _centerFormat.Dispose();
            _animationTimer.Dispose();
        }

        base.Dispose(disposing);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        base.OnPaint(e);
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        var palette = _paletteHost.Palette;

        if (_flow == null)
        {
            g.Clear(palette.Window);
            DrawPlaceholder(g);
            return;
        }

        g.Clear(palette.Window);
        _scale = GetScale(e.Graphics);
        var scale = _scale;
        var bounds = ClientRectangle;
        var padding = (int)Math.Round(15 * scale);
        if (Padding.All != padding)
        {
            Padding = new Padding(padding);
        }

        bounds.Inflate(-Padding.Horizontal / 2, -Padding.Vertical / 2);

        var nodeWidth = Math.Min((int)(200 * scale), Math.Max((int)(118 * scale), bounds.Width / 5));
        var nodeHeight = (int)(70 * scale);
        var hSpacing = nodeWidth + (int)(82 * scale);
        var vSpacing = nodeHeight + (int)(32 * scale);
        var center = new Point(bounds.Left + bounds.Width / 2, bounds.Top + bounds.Height / 2);

        var inverterRect = CenterRect(center, nodeWidth, nodeHeight);
        var pvRect = CenterRect(new Point(center.X - hSpacing, center.Y - vSpacing / 2), nodeWidth, nodeHeight);
        var batteryRect = CenterRect(new Point(center.X - hSpacing, center.Y + vSpacing / 2), nodeWidth, nodeHeight);
        var gridRect = CenterRect(new Point(center.X + hSpacing, center.Y - vSpacing / 2), nodeWidth, nodeHeight);
        var loadRect = CenterRect(new Point(center.X + hSpacing, center.Y + vSpacing / 2), nodeWidth, nodeHeight);

        DrawNodes(g, pvRect, batteryRect, gridRect, loadRect, inverterRect);
        DrawConnections(g, pvRect, batteryRect, gridRect, loadRect, inverterRect);
    }

    private void DrawPlaceholder(Graphics g)
    {
        var text = "Flow data will appear here once available.";
        var palette = _paletteHost.Palette;
        using var brush = new SolidBrush(palette.SubtleText);
        g.DrawString(text, _titleFont, brush, ClientRectangle, _centerFormat);
    }

    private void DrawNodes(Graphics g, Rectangle pvRect, Rectangle batteryRect, Rectangle gridRect, Rectangle loadRect, Rectangle inverterRect)
    {
        DrawNode(g, pvRect, "PV", FormatPower(_flow!.PvPower));

        var socText = _flow!.Soc.HasValue ? $"{_flow.Soc:0}%" : "n/a";
        var socColor = _flow.Soc.HasValue ? StatusColorHelper.GetSocColor(_flow.Soc.Value) : (Color?)null;
        DrawBatteryNode(g, batteryRect, FormatPower(_flow.BattPower), socText, socColor);

        var gridValue = GetGridLabel(_flow);
        DrawNode(g, gridRect, "Grid", gridValue);

        DrawNode(g, loadRect, "Load", FormatPower(_flow.LoadOrEpsPower));

        DrawNode(g, inverterRect, "Inverter", string.Empty, isPrimary: true);
    }

    private void DrawConnections(Graphics g, Rectangle pvRect, Rectangle batteryRect, Rectangle gridRect, Rectangle loadRect, Rectangle inverterRect)
    {
        const int idleThreshold = 10;
        var palette = _paletteHost.Palette;
        var accent = palette.Accent;
        var passive = palette.Muted;

        var pvPoints = ResolveDirection(
            OffsetToward(MidRight(pvRect), MidLeft(inverterRect), 12),
            OffsetToward(MidLeft(inverterRect), MidRight(pvRect), 12),
            _flow!.PvTo,
            false);
        var loadPoints = ResolveDirection(
            OffsetToward(MidRight(inverterRect), MidLeft(loadRect), 12),
            OffsetToward(MidLeft(loadRect), MidRight(inverterRect), 12),
            _flow.ToLoad,
            false);
        // Battery direction is determined strictly by API flags.
        var batteryToInverter = _flow.BatTo;
        var inverterToBattery = _flow.ToBat;
        var batteryPoints = ResolveDirection(
            OffsetToward(MidRight(batteryRect), MidLeft(inverterRect), 12), // forward = battery -> inverter
            OffsetToward(MidLeft(inverterRect), MidRight(batteryRect), 12), // reverse = inverter -> battery
            batteryToInverter,
            inverterToBattery);
        var gridPoints = ResolveDirection(
            OffsetToward(MidRight(inverterRect), MidLeft(gridRect), 12),   // forward = inverter -> grid
            OffsetToward(MidLeft(gridRect), MidRight(inverterRect), 12),   // reverse = grid -> inverter
            _flow.ToGrid,
            _flow.GridTo);

        var connectors = new[]
        {
            BuildConnector(pvPoints.Start, pvPoints.End, hasDirection: pvPoints.HasDirection, _flow.PvPower, requireDirectionFlag: true, accent, passive, idleThreshold),
            BuildConnector(loadPoints.Start, loadPoints.End, hasDirection: loadPoints.HasDirection, _flow.LoadOrEpsPower, requireDirectionFlag: false, accent, passive, idleThreshold),
            BuildConnector(batteryPoints.Start, batteryPoints.End, hasDirection: batteryPoints.HasDirection, _flow.BattPower, requireDirectionFlag: true, accent, passive, idleThreshold),
            BuildConnector(gridPoints.Start, gridPoints.End, hasDirection: gridPoints.HasDirection, _flow.GridOrMeterPower, requireDirectionFlag: true, accent, passive, idleThreshold)
        };

        foreach (var connector in connectors)
        {
            DrawConnector(g, connector.Start, connector.End, connector.Color, connector.Active);
        }

        DrawFlowDots(g, connectors);
    }

    private void DrawNode(Graphics g, Rectangle rect, string title, string value, bool isPrimary = false, Color? valueColor = null)
    {
        var palette = _paletteHost.Palette;
        using var bg = new SolidBrush(palette.Surface);
        using var border = new Pen(palette.Border, 1.5f);
        DrawRoundedRect(g, rect, 12, bg, border);

        var titleRect = new Rectangle(rect.X + 8, rect.Y + 8, rect.Width - 16, _titleFont.Height + 6);
        var valueRect = new Rectangle(rect.X + 8, rect.Y + rect.Height / 2 - 4, rect.Width - 16, rect.Height / 2);

        using var titleBrush = new SolidBrush(palette.SubtleText);
        using var valueBrush = new SolidBrush(valueColor ?? palette.Text);

        if (isPrimary)
        {
            // Center the title in the whole node and bump the weight.
            using var primaryFont = new Font(_valueFont.FontFamily, 12.5f, FontStyle.Bold);
            g.DrawString(title, primaryFont, valueBrush, rect, _centerFormat);
        }
        else
        {
            g.DrawString(title, _titleFont, titleBrush, titleRect, _centerFormat);
            g.DrawString(value, _valueFont, valueBrush, valueRect, _centerFormat);
        }
    }

    private void DrawBatteryNode(Graphics g, Rectangle rect, string wattText, string socText, Color? socColor)
    {
        var palette = _paletteHost.Palette;
        using var bg = new SolidBrush(palette.Surface);
        using var border = new Pen(palette.Border, 1.5f);
        DrawRoundedRect(g, rect, 12, bg, border);

        var titleRect = new Rectangle(rect.X + 8, rect.Y + 8, rect.Width - 16, _titleFont.Height + 6);
        var valueRect = new Rectangle(rect.X + 8, rect.Y + rect.Height / 2 - 6, rect.Width - 16, _valueFont.Height + 14);

        using var titleBrush = new SolidBrush(palette.SubtleText);
        using var wattBrush = new SolidBrush(palette.Text);
        using var socBrush = new SolidBrush(socColor ?? palette.Text);

        g.DrawString("Battery", _titleFont, titleBrush, titleRect, _centerFormat);

        using var socFont = new Font(_valueFont, FontStyle.Bold);
        var socSize = g.MeasureString(socText, socFont);
        var wattSize = g.MeasureString(wattText, _valueFont);
        const float gap = 6f;
        var totalWidth = wattSize.Width + gap + socSize.Width;
        var startX = valueRect.X + (valueRect.Width - totalWidth) / 2f;
        var centerY = valueRect.Y + valueRect.Height / 2f;

        var wattPos = new PointF(startX, centerY - _valueFont.Height / 2f);
        g.DrawString(wattText, _valueFont, wattBrush, wattPos);

        var socPos = new PointF(startX + wattSize.Width + gap, centerY - socFont.Height / 2f);
        g.DrawString(socText, socFont, socBrush, socPos);
    }

    private void DrawConnector(Graphics g, Point start, Point end, Color color, bool active)
    {
        var penWidth = (active ? 3.3f : 2f) * _scale;
        using var pen = new Pen(color, penWidth) { StartCap = LineCap.Round, EndCap = LineCap.Round };
        g.DrawLine(pen, start, end);
    }

    private void DrawFlowDots(Graphics g, ReadOnlySpan<Connector> connectors)
    {
        var spacing = 28f * _scale;
        var radius = 3.2f * _scale;
        foreach (var connector in connectors)
        {
            if (!connector.HasDirection)
            {
                continue;
            }

            var dx = connector.End.X - connector.Start.X;
            var dy = connector.End.Y - connector.Start.Y;
            var length = (float)Math.Sqrt(dx * dx + dy * dy);
            if (length < spacing)
            {
                continue;
            }

            var ux = dx / length;
            var uy = dy / length;
            var offset = _animationOffset % spacing;

            var palette = _paletteHost.Palette;
            var dotColor = connector.Active ? connector.Color : palette.Muted;
            using var brush = new SolidBrush(dotColor);
            for (var d = offset; d < length; d += spacing)
            {
                var cx = connector.Start.X + ux * d;
                var cy = connector.Start.Y + uy * d;
                g.FillEllipse(brush, cx - radius, cy - radius, radius * 2, radius * 2);
            }
        }
    }

    private static void DrawRoundedRect(Graphics g, Rectangle rect, int radius, Brush fill, Pen stroke)
    {
        using var path = RoundedRect(rect, radius);
        g.FillPath(fill, path);
        g.DrawPath(stroke, path);
    }

    private static GraphicsPath RoundedRect(Rectangle bounds, int radius)
    {
        if (radius <= 0)
        {
            var pathRect = new GraphicsPath();
            pathRect.AddRectangle(bounds);
            pathRect.CloseFigure();
            return pathRect;
        }

        var diameter = radius * 2;
        var arc = new Rectangle(bounds.Location, new Size(diameter, diameter));
        var path = new GraphicsPath();

        path.AddArc(arc, 180, 90);
        arc.X = bounds.Right - diameter;
        path.AddArc(arc, 270, 90);
        arc.Y = bounds.Bottom - diameter;
        path.AddArc(arc, 0, 90);
        arc.X = bounds.Left;
        path.AddArc(arc, 90, 90);
        path.CloseFigure();
        return path;
    }

    private static Rectangle CenterRect(Point center, int width, int height)
    {
        return new Rectangle(center.X - width / 2, center.Y - height / 2, width, height);
    }

    private static Point MidLeft(Rectangle rect) => new(rect.Left, rect.Top + rect.Height / 2);
    private static Point MidRight(Rectangle rect) => new(rect.Right, rect.Top + rect.Height / 2);

    private static Point OffsetToward(Point from, Point to, int distance)
    {
        var dx = to.X - from.X;
        var dy = to.Y - from.Y;
        var length = Math.Sqrt(dx * dx + dy * dy);
        if (length <= 0.0001)
        {
            return from;
        }

        var ux = dx / length;
        var uy = dy / length;
        return new Point((int)Math.Round(from.X + ux * distance), (int)Math.Round(from.Y + uy * distance));
    }

    private static string FormatPower(int power)
    {
        return $"{power} W";
    }

    private static string GetGridLabel(PowerFlowData flow)
    {
        const int idleThreshold = 10;
        if (Math.Abs(flow.GridOrMeterPower) <= idleThreshold && !flow.ToGrid && !flow.GridTo)
        {
            return "idle";
        }

        return FormatPower(flow.GridOrMeterPower);
    }

    private static (Point Start, Point End, bool HasDirection) ResolveDirection(
        Point forwardStart,
        Point forwardEnd,
        bool forwardFlag,
        bool reverseFlag)
    {
        if (forwardFlag == reverseFlag)
        {
            // Ambiguous or no direction; keep the line but mark it directionless (no dots).
            return (forwardStart, forwardEnd, false);
        }

        if (forwardFlag && !reverseFlag)
        {
            return (forwardStart, forwardEnd, true);
        }

        return (forwardEnd, forwardStart, true);
    }

    private Connector BuildConnector(
        Point start,
        Point end,
        bool hasDirection,
        int magnitude,
        bool requireDirectionFlag,
        Color accent,
        Color passive,
        int idleThreshold)
    {
        var active = (!requireDirectionFlag || hasDirection) && Math.Abs(magnitude) > idleThreshold;
        var color = active ? accent : passive;
        return new Connector(start, end, color, active)
        {
            HasDirection = hasDirection
        };
    }

    private readonly record struct Connector(Point Start, Point End, Color Color, bool Active)
    {
        public bool HasDirection { get; init; }
    }

    private static float GetScale(Graphics g)
    {
        var dpiScale = g.DpiX / 96f;
        if (dpiScale <= 1f)
        {
            return Clamp(dpiScale * 0.95f, 0.85f, 1.0f);
        }

        var boosted = dpiScale * 1.35f; // modest boost on HiDPI
        return Clamp(boosted, 1.15f, 2.0f);
    }

    private static float Clamp(float value, float min, float max)
    {
        if (value < min) return min;
        if (value > max) return max;
        return value;
    }
}
