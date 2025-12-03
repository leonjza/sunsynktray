using System;
using System.Windows.Forms;

namespace SunSynkTrayWin;

internal static class ControlExtensions
{
    public static void InvokeIfRequired(this Control control, Action action)
    {
        if (control.IsDisposed)
        {
            return;
        }

        if (control.InvokeRequired)
        {
            control.BeginInvoke(action);
        }
        else
        {
            action();
        }
    }
}
